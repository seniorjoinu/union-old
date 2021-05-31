#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package union_wallet && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/union_wallet.wasm -o target/wasm32-unknown-unknown/debug/union_wallet-opt.wasm
