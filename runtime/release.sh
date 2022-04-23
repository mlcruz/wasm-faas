#! /bin/bash
export LD_LIBRARY_PATH=$PWD/compat
cbindgen --config cbindgen.toml --output compat/libwasmfaas.h
cargo build --release 
cp target/release/libwasmfaas.so compat/libwasmfaas.so
pushd compat
g++ test.cpp -o test -Wall -I. -L. libwasmfaas.so
./test
popd 