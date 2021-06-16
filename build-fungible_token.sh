#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package fungible_token && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/fungible_token.wasm -o target/wasm32-unknown-unknown/debug/fungible_token-opt.wasm
