use std::sync::Arc;

use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use wasmer::{imports, Instance, Value};
use wasmer_wasi::{WasiEnv, WasiStateBuilder};

use crate::{module_store::ModuleStore, ServerState};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmArg {
    pub value: String,
    pub arg_type: wasmer::Type,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmFunction {
    pub name: String,
    pub args: Vec<WasmArg>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteModule {
    pub module_name: String,
    pub function: WasmFunction,
    pub wasi: bool,
}

pub async fn execute_function(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<ExecuteModule>,
) -> Result<(), (StatusCode, String)> {
    execute_function_inner(state.module_store.clone(), payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, String::from(format!("{:?}", e))))?;

    todo!()
}

async fn execute_function_inner(
    module_store: Arc<Mutex<ModuleStore>>,
    payload: ExecuteModule,
) -> anyhow::Result<Box<[Value]>> {
    let module_store = module_store.lock().await;

    let module = module_store
        .get(&payload.module_name)
        .ok_or_else(|| anyhow::anyhow!("Module not found"))?;

    let module_imports = if payload.wasi {
        let wasi_state = WasiStateBuilder::default().build()?;
        let mut wasi_env = WasiEnv::new(wasi_state);
        let imports = wasi_env.import_object(module)?;
        imports
    } else {
        let imports = imports! {};
        imports
    };

    let instance = Instance::new(&module, &module_imports)?;

    let wasm_function = instance.exports.get_function(&payload.function.name)?;

    let args = &payload
        .function
        .args
        .into_iter()
        .map(parse_arg)
        .collect::<Result<Vec<_>, _>>()?;

    let fn_result = wasm_function.call(args)?;

    Ok(fn_result)
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;
    use wasmer::Store;

    use crate::{compile_wasm, module_store::ModuleStore};

    static WASM_SUM: &[u8] = include_bytes!(r#"../../../binaries/compiled/sum.wasm"#);
    use super::{execute_function_inner, ExecuteModule, WasmArg, WasmFunction};

    #[test]
    fn test_execute_function() -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;

        let mut module_store = ModuleStore::default();
        let wasm_store = Store::default();
        let wasm_add_one = compile_wasm(&wasm_store, WASM_SUM)?;
        module_store.add("sum", wasm_add_one);

        let module_store = Arc::new(Mutex::new(module_store));
        let payload = ExecuteModule {
            module_name: "sum".into(),
            function: WasmFunction {
                name: "sum".into(),
                args: vec![
                    WasmArg {
                        value: "10".into(),
                        arg_type: wasmer::ValType::I32,
                    },
                    WasmArg {
                        value: "10".into(),
                        arg_type: wasmer::ValType::I32,
                    },
                ],
            },
            wasi: false,
        };

        let json = serde_json::to_string_pretty(&payload)?;
        println!("{}", json);
        let result = runtime.block_on(execute_function_inner(module_store, payload))?;
        println!("{:#?}", result);
        std::fs::write("tests/data/sum_request.json", json)?;
        let result = &result[0].i32().unwrap();
        assert_eq!(*result, 20);
        Ok(())
    }
}
