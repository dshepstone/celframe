#!/usr/bin/env bash
# Builds both prototypes into target/release/spike-app.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release -p spike-renderer --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/spike_renderer.wasm --target web --out-dir app/ui/pkg
cargo build --release -p spike-app
