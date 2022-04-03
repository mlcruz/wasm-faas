use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use chrono::NaiveDateTime;
use module_store::ModuleStore;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use wasmer::{Module, Store};

pub mod module_store;
pub mod runtime;
pub mod server;

#[derive(Clone)]
pub struct ServerState {
    pub module_store: Arc<Mutex<ModuleStore>>,
    pub wasm_store: Arc<Store>,
    pub known_nodes: Arc<Mutex<HashMap<SocketAddr, NaiveDateTime>>>,
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

pub fn compile_wasm(store: &Store, data: &[u8]) -> Result<Module, wasmer::CompileError> {
    Module::new(store, data)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
