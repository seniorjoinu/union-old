#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --package communitea
if [ $? -eq 0 ]
then
  ic-cdk-optimizer ./target/wasm32-unknown-unknown/debug/communitea.wasm -o ./target/wasm32-unknown-unknown/release/communitea-opt.wasm
else
  echo "Canister compilation failed"
