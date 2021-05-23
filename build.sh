#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package communitea
ic-cdk-optimizer ./target/wasm32-unknown-unknown/debug/communitea.wasm -o ./target/wasm32-unknown-unknown/release/communitea-opt.wasm
