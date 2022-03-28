use std::{
    io::{BufRead, BufReader, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    sync::Arc,
};

use parking_lot::Mutex;
use wasmer::{Instance, Module, Store};
use wasmer_wasi::{WasiEnv, WasiStateBuilder};
use wasmfass::{module_store::ModuleStore, Command, NamedFunction};

fn main() {
    let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), 9999);
    let sock = std::net::TcpListener::bind(addr).unwrap();

    let kernel_store = Arc::new(Mutex::new(ModuleStore::default()));
    let wasm_store = Arc::new(Store::default());

    sock.incoming().for_each(|s| {
        let s = s.unwrap();
        let kernel_store = kernel_store.clone();
        let wasm_store = wasm_store.clone();
        std::thread::spawn(move || handle_stream(s, kernel_store, wasm_store));
    });
}

fn handle_stream(stream: TcpStream, kernel_store: Arc<Mutex<ModuleStore>>, wasm_store: Arc<Store>) {
    handle_stream_inner(stream, kernel_store, wasm_store).unwrap();
}

fn handle_stream_inner(
    mut stream: TcpStream,
    kernel_store: Arc<Mutex<ModuleStore>>,
    wasm_store: Arc<Store>,
) -> anyhow::Result<()> {
    let mut stream_reader = BufReader::new(stream.try_clone()?);

    let mut data = String::new();
    while let Ok(_bytes_read) = stream_reader.read_line(&mut data) {
        data.pop();

        let base64 = base64::decode(&data)?;

        let command: Command = serde_json::from_str(std::str::from_utf8(&base64)?)?;
        data.clear();
        match command {
            Command::RegisterFunction(named_function) => {
                println!("registering {}", named_function.name);
                register_function(named_function, &kernel_store, &wasm_store)?;
                stream.write("OK\n".as_bytes())?;
            }
            Command::ExecuteFunction(function) => {
                println!("executing: {:#?}", function);
                execute_function(&kernel_store, function, &mut stream)?;
                println!("foobar");
            }
        }
    }

    stream.shutdown(std::net::Shutdown::Both).ok();
    Ok(())
}

fn execute_function(
    kernel_store: &Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, ModuleStore>>,
    function: wasmfass::ExecuteFunction,
    stream: &mut TcpStream,
) -> Result<(), anyhow::Error> {
    let ks = kernel_store.lock();
    let module = ks.get(&function.name);
    println!("{}", module.is_some());

    Ok(if let Some(module) = module {
        let wasi_state = WasiStateBuilder::default().build()?;
        let mut wasi_env = WasiEnv::new(wasi_state);

        let imports = wasi_env.import_object(module)?;

        let instance = Instance::new(&module, &imports)?;

        let wasm_function = instance.exports.get_function(&function.function.name)?;
        // let data = wasm_function.call(&[wasmer::Value::I32(0), wasmer::Value::I32(0)])?;

        //   stream.write("\n".as_bytes())?;
    })
}

fn register_function(
    named_function: NamedFunction,
    kernel_store: &Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, ModuleStore>>,
    wasm_store: &Store,
) -> Result<(), anyhow::Error> {
    let data = base64::decode(named_function.data_base64)?;
    let name = named_function.name;
    let module = compile_wasm(wasm_store, &data)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    let mut kernel_store = kernel_store.lock();
    kernel_store.add(name, module);
    Ok(())
}

fn compile_wasm(store: &Store, data: &[u8]) -> Result<Module, wasmer::CompileError> {
    Module::new(store, data)
}
