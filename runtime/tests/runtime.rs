use wasmfass::{
    runtime::execute_module::ExecuteModuleRequest,
    server::routes::register_function::RegisterFunction,
};

#[test]
// don't forget to start the runtime before running those tests
fn register_function_test() {
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let function_data = std::fs::read("../binaries/compiled/sum.wasm").unwrap();
    tokio_rt.block_on(async move {
        let response = register_function("sum", function_data, false).await;

        let response = response.error_for_status().unwrap().text().await.unwrap();
        println!("{}", response);
    });
}

async fn register_function(name: &str, wasm: Vec<u8>, wasi: bool) -> reqwest::Response {
    let base_64 = base64::encode(wasm);
    let client = reqwest::Client::new();

    let body = RegisterFunction {
        data_base64: base_64,
        name: name.to_owned(),
        wasi,
    };

    let request = client
        .post("http://127.0.0.1:3000/register")
        .json(&body)
        .build()
        .unwrap();

    client.execute(request).await.unwrap()
}

#[test]
// don't forget to start the runtime before running those tests
fn execute_function_test() {
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let function_data = std::fs::read("../binaries/compiled/sum.wasm").unwrap();
    let execute_payload = std::fs::read_to_string("tests/data/sum_request.json").unwrap();
    println!("{:#?}", execute_payload);
    let execute_payload: ExecuteModuleRequest = serde_json::from_str(&execute_payload).unwrap();
    tokio_rt.block_on(async move {
        register_function("sum", function_data, false)
            .await
            .error_for_status()
            .unwrap();

        let result = execute_function(execute_payload)
            .await
            .error_for_status()
            .unwrap();

        println!("{}", result.text().await.unwrap())
    });
}

async fn execute_function(request: ExecuteModuleRequest) -> reqwest::Response {
    let client = reqwest::Client::new();

    let request = client
        .post("http://127.0.0.1:3000/exec")
        .json(&request)
        .build()
        .unwrap();

    client.execute(request).await.unwrap()
}
