#! /bin/bash
set -eux

# Script to build rust code.
# Requirements: cargo.
# If OPTIMIZE is set, additionally wasm-opt (from binaryen), wasm-snip (from rustwasm)

RUSTC_FLAGS=""
TARGET_DIR=debug
if [ -n "${OPTIMIZE+x}" ]; then
  RUSTC_FLAGS="--release"
  TARGET_DIR=release
fi

(
  cd crates/cli
  cargo rustc --target wasm32-wasi $RUSTC_FLAGS
)

(
  cd crates/graphql-loader
  cargo rustc --target wasm32-unknown-unknown $RUSTC_FLAGS
)

if [ -n "${OPTIMIZE+x}" ]; then
  (
    cd target/wasm32-wasi/$TARGET_DIR
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
    cd target/wasm32-wasi/$TARGET_DIR
    cp nitrogql-cli.wasm nitrogql-cli.opt.wasm
  )

  (
    cd target/wasm32-unknown-unknown/$TARGET_DIR
    cp graphql-loader.wasm graphql-loader.opt.wasm
  )
fi