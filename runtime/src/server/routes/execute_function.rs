use axum::{extract::Extension, http::StatusCode, Json};

use crate::{
    runtime::execute_module::{ExecuteModuleRequest, WasmResult},
    ServerState,
};

pub async fn execute_function_handler(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<ExecuteModuleRequest>,
) -> Result<Json<Vec<WasmResult>>, (StatusCode, String)> {
    // let result = execute_module(state.module_store.clone(), payload)
    //     .await
    //     .map_err(|e| (StatusCode::BAD_REQUEST, String::from(format!("{:?}", e))))?;

    // let result = result
    //     .into_iter()
    //     .map(|v| WasmResult {
    //         result_type: v.ty(),
    //         result: v.to_string(),
    //     })
    //     .collect::<Vec<_>>();

    // Ok(result.into())

    todo!()
}
