use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    sync::Arc,
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wasmer_runtime::{compile, imports, DynFunc};
use wasmfass::{kernel_store::KernelStore, Command, NamedFunction};

fn main() {
    let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), 9999);
    let sock = std::net::TcpListener::bind(addr).unwrap();

    let kernel_store = Arc::new(Mutex::new(KernelStore::default()));

    sock.incoming().for_each(|s| {
        let s = s.unwrap();
        let kernel_store = kernel_store.clone();
        std::thread::spawn(move || handle_stream(s, kernel_store));
    });
}

fn handle_stream(
    mut stream: TcpStream,
    kernel_store: Arc<Mutex<KernelStore>>,
) -> anyhow::Result<()> {
    let mut stream_reader = BufReader::new(stream.try_clone()?);

    let mut data = String::new();
    while let Ok(_bytes_read) = stream_reader.read_line(&mut data) {
        let command: Command = serde_json::from_str(&data)?;
        match command {
            Command::RegisterFunction(named_function) => {
                println!("registering {}", named_function.name);
                register_function(named_function, &kernel_store)?;
                stream.write("OK\n".as_bytes())?;
            }
            Command::ExecuteFunction(function) => {
                let ks = kernel_store.lock();
                let module = ks.get(&function.name);

                if let Some(module) = module {
                    let imports = imports! {};
                    let instanace = module.instantiate(&imports).map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                    })?;

                    let wasm_function = instanace
                        .exports
                        .get::<DynFunc>(&function.function.name)
                        .map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                    })?;

                    let result = wasm_function.call(&function.function.args).map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                    })?;

                    let serialized = serde_json::to_string(&result)?;

                    stream.write_all(serialized.as_bytes())?;
                }
            }
        }
    }

    stream.shutdown(std::net::Shutdown::Both)?;
    Ok(())
}

fn register_function(
    named_function: NamedFunction,
    kernel_store: &Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, KernelStore>>,
) -> Result<(), anyhow::Error> {
    let data = base64::decode(named_function.data_base64)?;
    let name = named_function.name;
    let module = compile_wasm(&data)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    let mut kernel_store = kernel_store.lock();
    kernel_store.add(name, module);
    Ok(())
}

fn compile_wasm(
    data: &[u8],
) -> Result<wasmer_runtime::Module, wasmer_runtime::error::CompileError> {
    compile(&data)
}
