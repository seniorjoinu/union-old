#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package voting_token && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/voting_token.wasm -o target/wasm32-unknown-unknown/debug/voting_token-opt.wasm
