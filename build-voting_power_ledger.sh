#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package voting_power_ledger && \
 ic-cdk-optimizer target/wasm32-unknown-unknown/debug/voting_power_ledger.wasm -o target/wasm32-unknown-unknown/debug/voting_power_ledger-opt.wasm
