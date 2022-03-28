use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wasmer::{Module, Store};

use crate::ServerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFunction {
    name: String,
    data_base64: String,
}

pub async fn register_function_handler(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<RegisterFunction>,
) -> Result<&'static str, (StatusCode, String)> {
    let data = base64::decode(payload.data_base64).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Failed to decode base64"),
        )
    })?;

    let module = compile_wasm(&state.wasm_store, &data)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)))?;

    let mut module_store = state.module_store.lock().await;

    module_store.add(payload.name, module);
    Ok("OK")
}

fn compile_wasm(store: &Store, data: &[u8]) -> Result<Module, wasmer::CompileError> {
    Module::new(store, data)
}