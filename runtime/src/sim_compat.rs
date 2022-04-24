use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crossbeam::sync::ShardedLock;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::Rng;
use wasmer::{Instance, Store};

#[repr(C)]
#[derive(Debug)]
pub enum ArgType {
    /// Signed 32 bit integer.
    I32,
    /// Signed 64 bit integer.
    I64,
    /// Floating point 32 bit integer.
    F32,
    /// Floating point 64 bit integer.
    F64,
    /// A 128 bit number.
    V128,
    /// A reference to opaque data in the Wasm instance.
    ExternRef, /* = 128 */
    /// A reference to a Wasm function.
    FuncRef,
}

impl From<ArgType> for wasmer::Type {
    fn from(arg: ArgType) -> Self {
        match arg {
            ArgType::I32 => wasmer::Type::I32,
            ArgType::I64 => wasmer::Type::I64,
            ArgType::F32 => wasmer::Type::F32,
            ArgType::F64 => wasmer::Type::F64,
            ArgType::V128 => wasmer::Type::V128,
            ArgType::ExternRef => wasmer::Type::ExternRef,
            ArgType::FuncRef => wasmer::Type::FuncRef,
        }
    }
}

use crate::{
    compile_wasm, module_store::ModuleStore,
    server::routes::register_function::RegisterModulePayload,
};

#[repr(C)]
pub enum StaticModuleList {
    WasmDiv,
    WasmSum,
}

#[repr(C)]
#[derive(Debug)]
pub struct WasmArg {
    pub value: *const c_char,
    pub arg_type: ArgType,
}

#[repr(C)]
#[derive(Debug)]
pub struct WasmFunction {
    pub name: *const c_char,
    pub args: [WasmArg; 2],
}

impl From<WasmFunction> for crate::runtime::execute_module::WasmFunction {
    fn from(function_compat: WasmFunction) -> Self {
        let name = unsafe { CStr::from_ptr(function_compat.name) }
            .to_str()
            .unwrap()
            .to_owned();

        let args = function_compat
            .args
            .map(|arg| {
                let value = unsafe { CStr::from_ptr(arg.value) }
                    .to_str()
                    .unwrap()
                    .to_owned();

                crate::runtime::execute_module::WasmArg {
                    value,
                    arg_type: arg.arg_type.into(),
                }
            })
            .to_vec();

        Self { name, args }
    }
}

static WASM_SUM: &[u8] = include_bytes!(r#"../../binaries/compiled/sum.wasm"#);
static WASM_DIV: &[u8] = include_bytes!(r#"../../binaries/compiled/div.wasm"#);

impl StaticModuleList {
    fn data_base64(&self) -> String {
        let data = match self {
            StaticModuleList::WasmDiv => WASM_DIV,
            StaticModuleList::WasmSum => WASM_SUM,
        };
        base64::encode(data)
    }

    fn name(&self) -> &'static str {
        match self {
            StaticModuleList::WasmDiv => "div",
            StaticModuleList::WasmSum => "sum",
        }
    }
}

#[derive(Default)]
pub struct WasmRuntime {
    module_store: ModuleStore,
}

#[no_mangle]
static SHARED_RUNTIMES: Lazy<ShardedLock<HashMap<u64, Mutex<WasmRuntime>>>> =
    Lazy::new(|| ShardedLock::default());

static STORE: Lazy<Store> = Lazy::new(|| Store::default());

#[no_mangle]
pub extern "C" fn initialize_runtime() -> u64 {
    let mut rng = rand::thread_rng();
    let rand: u64 = rng.gen();

    SHARED_RUNTIMES
        .write()
        .expect("failed to get runtimes write lock")
        .insert(rand, Mutex::default());

    rand
}

#[no_mangle]
pub extern "C" fn get_static_module_data(module: StaticModuleList) -> *mut c_char {
    let base_64 = module.data_base64().as_bytes().to_vec();
    let base64_compat: CString = CString::new(base_64).unwrap();

    base64_compat.into_raw()
}

#[no_mangle]
pub extern "C" fn get_runtime_module_base64_data(
    runtime_id: u64,
    module: StaticModuleList,
) -> *const c_char {
    // println!("registering {}", runtime_id);

    let runtime_lock = SHARED_RUNTIMES
        .read()
        .expect("failed to get runtime read lock");

    let runtime = runtime_lock
        .get(&runtime_id)
        .expect("failed to get runtime id");

    let lock = runtime.lock();

    let contains_module = lock.module_store.contains_key(&module.name());

    if !contains_module {
        panic!("Missing module base64");
    }

    let base_64 = module.data_base64().as_bytes().to_vec();
    let base64_compat: CString = CString::new(base_64).unwrap();

    base64_compat.into_raw()
}

#[no_mangle]
pub extern "C" fn register_module(
    runtime_id: u64,
    module_name: *const c_char,
    module_data_base_64: *const c_char,
) -> *const c_char {
    // println!("registering {}", runtime_id);

    let module_name_str = unsafe { CStr::from_ptr(module_name) }
        .to_str()
        .expect("invalid string");

    let module_name_data = unsafe { CStr::from_ptr(module_data_base_64) }
        .to_str()
        .expect("invalid data base 64");

    let module_payload = RegisterModulePayload {
        data_base64: module_name_data.to_string(),
        name: module_name_str.to_string(),
        wasi: false,
    };

    let data = base64::decode(module_payload.data_base64).expect("failed to decode module");

    let module = compile_wasm(&STORE, &data).expect("failed to compile module wasm");

    let runtime_lock = SHARED_RUNTIMES
        .read()
        .expect("failed to get runtime read lock");

    let runtime = runtime_lock
        .get(&runtime_id)
        .expect("failed to get runtime id");

    let mut lock = runtime.lock();

    lock.module_store
        .add(module_payload.name, module, module_payload.wasi)
        .expect("failed to add module to store");

    module_name
}

#[no_mangle]
pub extern "C" fn free_ffi_string(data: *mut c_char) {
    unsafe { CString::from_raw(data) };
}

#[no_mangle]
pub extern "C" fn is_module_registered(runtime_id: u64, module: StaticModuleList) -> bool {
    // println!("executing {:#?}", function);

    let runtime_lock = SHARED_RUNTIMES
        .read()
        .expect("failed to get runtime read lock");

    let runtime = runtime_lock
        .get(&runtime_id)
        .expect("failed to get runtime id");

    let lock = runtime.lock();

    lock.module_store.contains_key(module.name())
}

#[no_mangle]
pub extern "C" fn execute_module(
    runtime_id: u64,
    module_name: *const c_char,
    function: WasmFunction,
) -> i32 {
    // println!("executing {:#?}", function);

    let module_name = unsafe { CStr::from_ptr(module_name) }
        .to_str()
        .expect("invalid string");

    let func_name = unsafe { CStr::from_ptr(function.name) }
        .to_str()
        .expect("invalid string");

    let runtime_lock = SHARED_RUNTIMES
        .read()
        .expect("failed to get runtime read lock");

    let runtime = runtime_lock
        .get(&runtime_id)
        .expect("failed to get runtime id");

    let lock = runtime.lock();

    let module = lock.module_store.get(&module_name).expect("missing module");

    let instance = Instance::new(&module.module, &module.imports).unwrap();

    let wasm_function = instance.exports.get_function(func_name).unwrap();

    let args = [
        parse_arg(&function.args[0]).unwrap(),
        parse_arg(&function.args[1]).unwrap(),
    ];

    let fn_result = wasm_function.call(&args).unwrap();

    let result = fn_result[0].i32().unwrap();

    result
}

fn parse_arg(arg: &WasmArg) -> anyhow::Result<wasmer::Value> {
    let value = unsafe { CStr::from_ptr(arg.value).to_str()? };

    Ok(match arg.arg_type {
        ArgType::I32 => wasmer::Value::I32(value.parse()?),
        ArgType::I64 => wasmer::Value::I64(value.parse()?),
        ArgType::F32 => wasmer::Value::F32(value.parse()?),
        ArgType::F64 => wasmer::Value::F64(value.parse()?),
        ArgType::V128 => todo!(),
        ArgType::ExternRef => todo!(),
        ArgType::FuncRef => todo!(),
    })
}
