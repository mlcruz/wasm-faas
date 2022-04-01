How to compile stuff:

TL;DR

1. Initialize emscripten by running initialize.sh
2. Add emcc and other to your path (as per initialize.sh output) or run `source emsdk/emsdk_env.sh`
3. compile stuff using emcc as a replacement for gcc or em++ as a drop-in replacement for g++ using the `-s STANDALONE_WASM` flag or `-o output.wasm`

if you have any further questions, check the documentation at https://emscripten.org/docs/compiling/index.html
and https://github.com/emscripten-core/emscripten/wiki/WebAssembly-Standalone
