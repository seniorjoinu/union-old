#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package claim_token && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/claim_token.wasm -o target/wasm32-unknown-unknown/debug/claim_token-opt.wasm
