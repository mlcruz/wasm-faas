use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{compile_wasm, ServerState};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterModulePayload {
    pub name: String,
    pub data_base64: String,
    pub wasi: bool,
}

pub async fn register_function_handler(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<RegisterModulePayload>,
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

    module_store
        .add(payload.name, module, payload.wasi)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)))?;
    Ok("OK")
}
