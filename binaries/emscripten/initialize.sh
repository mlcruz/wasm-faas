#! /bin/bash

git submodule init
git submodule update
pushd emsdk
./emsdk install latest
./emsdk activate latest
popd