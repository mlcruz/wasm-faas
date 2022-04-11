use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use chrono::NaiveDateTime;
use module_store::ModuleStore;
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
