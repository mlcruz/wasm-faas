use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wasmer::Instance;
use wasmer_wasi::{WasiEnv, WasiStateBuilder};

use crate::ServerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmArg {
    pub value: String,
    pub arg_type: wasmer::Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmFunction {
    pub name: String,
    pub args: Vec<wasmer::Type>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteModule {
    pub module_name: String,
    pub function: WasmFunction,
}

pub async fn execute_function(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<ExecuteModule>,
) -> Result<(), (StatusCode, String)> {
    let module_store = state.module_store.lock().await;

    let module = module_store
        .get(&payload.module_name)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, String::from("Module not found")))?;

    let wasi_state = WasiStateBuilder::default().build().unwrap();
    let mut wasi_env = WasiEnv::new(wasi_state);
    let imports = wasi_env.import_object(module).unwrap();
    let instance = Instance::new(&module, &imports).unwrap();
    let wasm_function = instance
        .exports
        .get_function(&payload.function.name)
        .map_err(|e| (StatusCode::BAD_REQUEST, String::from(format!("{:?}", e))))?;

    let data = wasm_function
        .call(&payload.function.args)
        .map_err(|e| (StatusCode::BAD_REQUEST, String::from(format!("{:?}", e))))?;

    todo!()
}

fn parse_arg(arg: WasmArg) -> anyhow::Result<wasmer::Value> {
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

// fn execute_function(
//     kernel_store: &Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, ModuleStore>>,
//     function: wasmfass::ExecuteFunction,
//     stream: &mut TcpStream,
// ) -> Result<(), anyhow::Error> {
//     let ks = kernel_store.lock();
//     let module = ks.get(&function.name);
//     println!("{}", module.is_some());

//     Ok(if let Some(module) = module {
//         let wasi_state = WasiStateBuilder::default().build()?;

//
//

//         //   stream.write("\n".as_bytes())?;
//     })
// }
