use std::{
    io::{BufRead, BufReader, Write},
    net::{IpAddr, SocketAddr},
};

use wasmfass::{Command, ExecuteFunction, NamedFunction, WasmFunction};

#[test]
// don't forget to start the runtime before running those tests
fn register_function_test() {
    let function_data = std::fs::read("../binaries/bin/hello_world.wasm").unwrap();
    let response = register_function(function_data);

    println!("{}", response);
    assert_eq!("OK\n", response);
}

fn register_function(function_data: Vec<u8>) -> String {
    let base_64 = base64::encode(function_data);
    let register_function_cmd = Command::RegisterFunction(NamedFunction {
        data_base64: base_64,
        name: "hello_world".to_string(),
    });
    let mut serialized_cmd = base64::encode(
        serde_json::to_string(&register_function_cmd)
            .unwrap()
            .as_bytes(),
    );
    serialized_cmd.push('\n');
    let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), 9999);
    let mut connection = std::net::TcpStream::connect(addr).unwrap();
    connection.write(serialized_cmd.as_bytes()).unwrap();
    connection.write("\n".as_bytes()).unwrap();
    let mut response = String::new();
    let mut reader = BufReader::new(connection);
    reader.read_line(&mut response).unwrap();
    response
}

#[test]
// don't forget to start the runtime before running those tests
fn execute_function() {
    let function_data = std::fs::read("../binaries/bin/hello_world.wasm").unwrap();
    let response = register_function(function_data);
    println!("{}", response);
    assert_eq!("OK\n", response);

    let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), 9999);

    let mut connection = std::net::TcpStream::connect(addr).unwrap();

    let execute_function_cmd = Command::ExecuteFunction(ExecuteFunction {
        name: "hello_world".to_string(),
        function: WasmFunction {
            name: "main".to_string(),
            args: vec![],
        },
    });

    let serialized_cmd = base64::encode(
        serde_json::to_string(&execute_function_cmd)
            .unwrap()
            .as_bytes(),
    );
    connection.write(serialized_cmd.as_bytes()).unwrap();
    connection.write("\n".as_bytes()).unwrap();
    connection.flush().unwrap();

    let mut reader = BufReader::new(connection);

    let mut response = String::new();
    reader.read_line(&mut response).unwrap();

    assert_eq!("OK\n", response);
}
