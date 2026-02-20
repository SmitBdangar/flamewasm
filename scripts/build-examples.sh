#!/usr/bin/env bash
# Build all example Rust crates to .wasm (requires wasm32-wasi target)
set -euo pipefail

echo "Adding wasm32-wasip1 target if missing..."
rustup target add wasm32-wasip1 2>/dev/null || true

for example in examples/hello_world examples/wasi_fs examples/sandbox_demo; do
    if [ -f "$example/Cargo.toml" ]; then
        echo "Building $example ..."
        cargo build --target wasm32-wasip1 --release --manifest-path "$example/Cargo.toml"
    fi
done
echo "All examples built."
