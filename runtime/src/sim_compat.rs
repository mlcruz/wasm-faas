use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::Rng;

use crate::module_store::ModuleStore;

#[derive(Default)]
pub struct WasmRuntime {
    module_store: ModuleStore,
}

#[no_mangle]
static SHARED_RUNTIMES: Lazy<parking_lot::Mutex<HashMap<u64, WasmRuntime>>> =
    Lazy::new(|| parking_lot::Mutex::default());

#[no_mangle]
pub extern "C" fn initialize_runtime() -> u64 {
    let mut rng = rand::thread_rng();
    let rand: u64 = rng.gen();

    SHARED_RUNTIMES.lock().insert(rand, WasmRuntime::default());

    rand
}

#[no_mangle]
pub extern "C" fn register_module(runtime_id: u64) {
    println!("registering {}", runtime_id);
}
