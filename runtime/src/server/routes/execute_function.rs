use axum::{extract::Extension, http::StatusCode, Json};

use crate::{
    runtime::execute_module::{execute_function, ExecuteModuleRequest, WasmResult},
    ServerState,
};

pub async fn execute_function_handler(
    Extension(state): Extension<ServerState>,
    Json(payload): Json<ExecuteModuleRequest>,
) -> Result<Json<Vec<WasmResult>>, (StatusCode, String)> {
    println!("{:#?}", payload);
    let module_store = state.module_store.lock().await;
    let module_package = module_store
        .get(&payload.module_name)
        .ok_or((StatusCode::BAD_REQUEST, "module not found".to_owned()))?;

    let result = execute_function(module_package, payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, String::from(format!("{:?}", e))))?;

    let result = result
        .into_iter()
        .map(|v| WasmResult {
            result_type: v.ty(),
            result: v.to_string(),
        })
        .collect::<Vec<_>>();

    println!("{:#?}", result);

    Ok(result.into())
}
