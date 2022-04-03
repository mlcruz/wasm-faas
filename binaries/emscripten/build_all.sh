#! /bin/bash
source emsdk/emsdk_env.sh
emcc ./cpp/sum/sum.c -O3 -o compiled/sum.wasm --no-entry 
emcc ./cpp/div/div.c -O3 -o compiled/div.wasm --no-entry 
emcc ./cpp/import/import.c -O3 -o compiled/import.wasm --no-entry  -s ERROR_ON_UNDEFINED_SYMBOLS=0