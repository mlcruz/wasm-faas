use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Extension},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::ServerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterNode {
    name: String,
}

pub async fn register_node(
    Extension(state): Extension<ServerState>,
    Json(_payload): Json<RegisterNode>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<&'static str, (StatusCode, String)> {
    let mut known_nodes = state.known_nodes.lock().await;
    known_nodes.insert(addr, Utc::now().naive_utc());
    Ok("OK")
}
