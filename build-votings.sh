#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package votings && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/debug/votings.wasm -o ./target/wasm32-unknown-unknown/release/votings-opt.wasm
