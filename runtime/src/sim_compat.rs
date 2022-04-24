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
    compile_wasm, module_store::ModuleStore, runtime::execute_module::ExecuteModuleRequest,
    server::routes::register_function::RegisterModulePayload,
};

#[repr(C)]
pub enum ModuleList {
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

impl ModuleList {
    fn build_registration_payload(&self) -> RegisterModulePayload {
        let data = match self {
            ModuleList::WasmDiv => WASM_DIV,
            ModuleList::WasmSum => WASM_SUM,
        };

        let base64 = base64::encode(data);

        RegisterModulePayload {
            name: "sum".into(),
            data_base64: base64,
            wasi: false,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ModuleList::WasmDiv => "div",
            ModuleList::WasmSum => "sum",
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
pub extern "C" fn register_module(runtime_id: u64, module: ModuleList) -> *const c_char {
    // println!("registering {}", runtime_id);

    let module_payload = module.build_registration_payload();
    let module_name_compat = module.name().as_bytes().to_vec();
    let module_name_compat: CString = CString::new(module_name_compat).unwrap();

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

    module_name_compat.into_raw()
}

#[no_mangle]
pub extern "C" fn free_ffi_string(data: *mut c_char) {
    unsafe { CString::from_raw(data) };
}

#[no_mangle]
pub extern "C" fn execute_module(
    runtime_id: u64,
    module: ModuleList,
    function: WasmFunction,
) -> i32 {
    // println!("executing {:#?}", function);

    let module_name = module.name().to_string();

    let runtime_lock = SHARED_RUNTIMES
        .read()
        .expect("failed to get runtime read lock");

    let runtime = runtime_lock
        .get(&runtime_id)
        .expect("failed to get runtime id");

    let lock = runtime.lock();

    let module = lock.module_store.get(&module_name).expect("missing module");

    let execute_module_request = ExecuteModuleRequest {
        module_name,
        function: function.into(),
    };

    let instance = Instance::new(&module.module, &module.imports).unwrap();

    let wasm_function = instance
        .exports
        .get_function(&execute_module_request.function.name)
        .unwrap();

    let args = &execute_module_request
        .function
        .args
        .into_iter()
        .map(parse_arg)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let fn_result = wasm_function.call(args).unwrap();

    let result = fn_result[0].i32().unwrap();

    result
}

fn parse_arg(arg: crate::runtime::execute_module::WasmArg) -> anyhow::Result<wasmer::Value> {
    Ok(match arg.arg_type {
        wasmer::ValType::I32 => wasmer::Value::I32(arg.value.parse()?),
        wasmer::ValType::I64 => wasmer::Value::I64(arg.value.parse()?),
        wasmer::ValType::F32 => wasmer::Value::F32(arg.value.parse()?),
        wasmer::ValType::F64 => wasmer::Value::F64(arg.value.parse()?),
        wasmer::ValType::V128 => todo!(),
        wasmer::ValType::ExternRef => todo!(),
        wasmer::ValType::FuncRef => todo!(),
    })
}
