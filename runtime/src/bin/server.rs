use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{extract::Extension, routing::post, Router};
use tokio::sync::Mutex;
use wasmer::Store;
use wasmfaas::{
    module_store::ModuleStore,
    server::routes::{
        execute_function::execute_function_handler, register_function::register_function_handler,
    },
    ServerState,
};

#[tokio::main]
async fn main() {
    // run our app with hyper
    // `axum::Server` is a re-exporst of `hyper::Server`
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
        .route("/register", post(register_function_handler))
        .route("/exec", post(execute_function_handler))
        .layer(Extension(server_state));

    println!("Running at {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
        .await
        .unwrap();
}
