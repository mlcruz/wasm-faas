use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
};

use wasmer_runtime::{imports, instantiate};

struct NamedFunction {
    name: String,
    data: Vec<u8>,
}

enum Command {
    RegisterFunction(NamedFunction),
}

struct Package {}

fn main() {
    let sock = std::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4("0.0.0.0".parse().unwrap()),
        9999,
    ))
    .unwrap();

    sock.incoming().for_each(|s| {
        let s = s.unwrap();
        println!("accepted");
    });
}

fn instantiate_binary_blob(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let import_object = imports! {};
    let mut instance = instantiate(data, &import_object)?;
    instance.exports();

    Ok(())
}
