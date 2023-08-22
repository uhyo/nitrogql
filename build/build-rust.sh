#! /bin/bash
set -eux

# Script to build rust code.
# Requirements: cargo.
# If OPTIMIZE is set, additionally wasm-opt (from binaryen), wasm-snip (from rustwasm)
(
  cd crates/cli
  cargo rustc --target wasm32-wasi --release
)

(
  cd crates/graphql-loader
  cargo rustc --target wasm32-unknown-unknown --release
)

if [ -n "${OPTIMIZE+x}" ]; then
  (
    cd target/wasm32-wasi/release
    wasm-opt nitrogql-cli.wasm -Oz -o nitrogql-cli.opt.wasm.tmp
    wasm-snip nitrogql-cli.opt.wasm.tmp > nitrogql-cli.opt.wasm
  )

  (
    cd target/wasm32-unknown-unknown/release
    wasm-opt graphql-loader.wasm -Oz -o graphql-loader.opt.wasm.tmp
    wasm-snip graphql-loader.opt.wasm.tmp > graphql-loader.opt.wasm
  )
else
  (
    cd target/wasm32-wasi/release
    cp nitrogql-cli.wasm nitrogql-cli.opt.wasm
  )

  (
    cd target/wasm32-unknown-unknown/release
    cp graphql-loader.wasm graphql-loader.opt.wasm
  )
fi