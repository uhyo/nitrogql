#! /bin/bash
set -eux

# Script to build rust code.
# Requirements: cargo, wasm-opt (from binaryen), wasm-snip (from rustwasm)
(
  cd crates/cli
  cargo rustc --target wasm32-wasi --release
)
(
  cd target/wasm32-wasi/release
  wasm-opt nitrogql-cli.wasm -Oz -o nitrogql-cli.opt.wasm.tmp
  wasm-snip nitrogql-cli.opt.wasm.tmp > nitrogql-cli.opt.wasm
)

(
  cd crates/graphql-loader
  cargo rustc --target wasm32-unknown-unknown --release
)
(
  cd target/wasm32-unknown-unknown/release
  wasm-opt graphql-loader.wasm -Oz -o graphql-loader.opt.wasm.tmp
  wasm-snip graphql-loader.opt.wasm.tmp > graphql-loader.opt.wasm
)