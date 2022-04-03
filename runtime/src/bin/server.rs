use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::Extension, routing::post, Router};
use tokio::sync::Mutex;
use wasmer::Store;
use wasmfass::{
    module_store::ModuleStore, server::routes::register_function::register_function_handler,
    ServerState,
};

#[tokio::main]
async fn main() {
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let module_store = Arc::new(Mutex::new(ModuleStore::default()));
    let wasm_store = Arc::new(Store::default());
    let known_nodes = Arc::new(Mutex::new(HashMap::default()));

    let server_state = ServerState {
        module_store,
        wasm_store,
        known_nodes,
    };

    let app = Router::new()
        .route("/", post(register_function_handler))
        .layer(Extension(server_state));

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
        .await
        .unwrap();
}
