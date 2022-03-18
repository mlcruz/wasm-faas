use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use wasmfass::{Command, NamedFunction};

#[test]
// don't forget to start the runtime before running those tests
fn register_function() {
    let function_data = std::fs::read("../binaries/bin/hello_world.wasm").unwrap();
    let base_64 = base64::encode(function_data);

    let register_function_cmd = Command::RegisterFunction(NamedFunction {
        data_base64: base_64,
        name: "hello_world".to_string(),
    });

    let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), 9999);

    let mut connection = std::net::TcpStream::connect(addr).unwrap();

    let serialized_cmd = serde_json::to_string(&register_function_cmd).unwrap();
    connection.write(serialized_cmd.as_bytes()).unwrap();
    connection.write("\n".as_bytes()).unwrap();
    connection.flush().unwrap();

    let mut response = String::new();

    let mut reader = BufReader::new(connection);
    reader.read_line(&mut response).unwrap();

    println!("{}", response);
    assert_eq!("OK\n", response);
}
