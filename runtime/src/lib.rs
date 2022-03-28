use std::sync::Arc;

use module_store::ModuleStore;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use wasmer::Store;

pub mod module_store;
pub mod routes;

#[derive(Clone)]
pub struct ServerState {
    pub module_store: Arc<Mutex<ModuleStore>>,
    pub wasm_store: Arc<Store>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamedFunction {
    pub name: String,
    pub data_base64: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmFunction {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteFunction {
    pub name: String,
    pub function: WasmFunction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    RegisterFunction(NamedFunction),
    ExecuteFunction(ExecuteFunction),
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
