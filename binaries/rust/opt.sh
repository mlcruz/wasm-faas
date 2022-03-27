cargo build --release --all --target wasm32-wasi
wasm-opt -c -o ./bin/hello_world.wasm -Os target/wasm32-wasi/release/hello_world.wasm