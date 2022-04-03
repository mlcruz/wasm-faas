use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wasmer::{imports, ImportObject, Instance, Module};
use wasmer_wasi::{WasiEnv, WasiStateBuilder};

use crate::module_store::ModuleStore;

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
pub struct WasmImport {
    pub module_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteModuleRequest {
    pub module_name: String,
    pub imports: Vec<WasmImport>,
    pub function: WasmFunction,
    pub wasi: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmResult {
    result: String,
    result_type: wasmer::ValType,
}

async fn execute_function(
    module: &Module,
    payload: ExecuteModuleRequest,
    store: ModuleStore,
) -> anyhow::Result<Box<[wasmer::Value]>> {
    let module_imports = resolve_imports(module, &payload, store).await?;

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

async fn resolve_imports(
    module: &Module,
    payload: &ExecuteModuleRequest,
    store: ModuleStore,
) -> anyhow::Result<ImportObject> {
    let mut base_imports = if payload.wasi {
        let wasi_state = WasiStateBuilder::default().build()?;
        let mut wasi_env = WasiEnv::new(wasi_state);
        let imports = wasi_env.import_object(&module)?;
        imports
    } else {
        let imports = imports! {};
        imports
    };

    let empty_imports = imports! {};

    for meta in &payload.imports {
        if let Some(module) = store.get(&meta.module_name) {
            let instance = Instance::new(&module, &empty_imports)?;
            base_imports.register(&meta.module_name, instance.exports);
        };
    }

    return Ok(base_imports);
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

    use crate::{
        compile_wasm,
        module_store::ModuleStore,
        runtime::execute_module::{execute_function, ExecuteModuleRequest, WasmArg, WasmFunction},
    };

    use super::{resolve_imports, WasmImport};

    static WASM_SUM: &[u8] = include_bytes!(r#"../../../binaries/compiled/sum.wasm"#);
    static WASM_DIV: &[u8] = include_bytes!(r#"../../../binaries/compiled/div.wasm"#);
    static WASM_IMPORT: &[u8] = include_bytes!(r#"../../../binaries/compiled/import.wasm"#);

    #[test]
    fn test_execute_function() -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;

        let mut module_store = ModuleStore::default();
        let wasm_store = Store::default();
        let wasm_add_one = compile_wasm(&wasm_store, WASM_SUM)?;
        module_store.add("sum", wasm_add_one);

        let module_store = module_store;
        let payload = ExecuteModuleRequest {
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
            imports: vec![],
            wasi: false,
        };

        let module = module_store.get("sum").unwrap().clone();

        let json = serde_json::to_string_pretty(&payload)?;
        println!("{}", json);
        let result = runtime.block_on(execute_function(&module, payload, module_store))?;
        println!("{:#?}", result);
        std::fs::write("tests/data/sum_request.json", json)?;
        let result = &result[0].i32().unwrap();
        assert_eq!(*result, 20);
        Ok(())
    }

    #[test]
    fn test_resolve_imports() -> anyhow::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;
        let mut module_store = ModuleStore::default();
        let wasm_store = Store::default();
        let wasm_add_one = compile_wasm(&wasm_store, WASM_SUM)?;
        let wasm_div = compile_wasm(&wasm_store, WASM_DIV)?;
        let wasm_import = compile_wasm(&wasm_store, WASM_IMPORT)?;

        module_store.add("sum", wasm_add_one);
        module_store.add("div", wasm_div);
        module_store.add("import", wasm_import);

        let module_store = module_store;
        let payload = ExecuteModuleRequest {
            module_name: "import".into(),
            function: WasmFunction {
                name: "div_sum".into(),
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
            imports: vec![WasmImport {
                module_name: "sum".to_string(),
            }],
            wasi: false,
        };

        let module = module_store.get("import").unwrap().clone();

        let json = serde_json::to_string_pretty(&payload)?;
        println!("{}", json);
        let result = runtime.block_on(execute_function(&module, payload, module_store))?;
        println!("{:#?}", result);
        // std::fs::write("tests/data/sum_request.json", json)?;
        // let result = &result[0].i32().unwrap();
        // assert_eq!(*result, 20);
        Ok(())
    }
}
