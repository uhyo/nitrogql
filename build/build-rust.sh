#! /bin/bash
set -eux

# Script to build rust code.
# Requirements: cargo (nightly), wasm-opt (from binaryen)
(
  cd crates/cli
  cargo +nightly rustc --target wasm32-wasi --release -- -Z wasi-exec-model=reactor
)
(
  cd target/wasm32-wasi/release
  wasm-opt nitrogql-cli.wasm -Oz -o nitrogql-cli.opt.wasm
)

(
  cd crates/graphql-loader
  cargo +nightly rustc --target wasm32-unknown-unknown --release
)
(
  cd target/wasm32-unknown-unknown/release
  wasm-opt graphql-loader.wasm -Oz -o graphql-loader.opt.wasm
)