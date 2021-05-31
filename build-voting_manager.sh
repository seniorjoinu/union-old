#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package voting_manager && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/voting_manager.wasm -o target/wasm32-unknown-unknown/debug/voting_manager-opt.wasm
