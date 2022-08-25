#!/bin/bash

set -x -e

RUST_MIN_VERSION=1.48.0

# This is just to make sure we've got the compiler.
rustc +$RUST_MIN_VERSION --version

rm -rf ./components/wasm-opt-sys/binaryen
rm -rf ./components/wasm-opt-cxx-sys/binaryen

cp -r ./binaryen ./components/wasm-opt-sys/
cp -r ./binaryen ./components/wasm-opt-cxx-sys/

# Make sure we don't publish this recursive submodule.
# Not needed by our build.
rm -rf ./components/wasm-opt-sys/binaryen/third_party/googletest
rm -rf ./components/wasm-opt-cxx-sys/binaryen/third_party/googletest

# cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt-sys/Cargo.toml --dry-run
# cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt-cxx-sys/Cargo.toml --dry-run
# cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt/Cargo.toml --dry-run

cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt-sys/Cargo.toml
cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt-cxx-sys/Cargo.toml
cargo +$RUST_MIN_VERSION publish --manifest-path ./components/wasm-opt/Cargo.toml