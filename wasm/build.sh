#!/usr/bin/env sh
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown --no-default-features --features wasm --release
wasm-bindgen --out-dir pkg --web ../target/wasm32-unknown-unknown/release/hectic-rs.wasm
python -m http.server 8000
