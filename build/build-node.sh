#! /bin/bash
set -eux

# Script to build rust code.
# Expected to be run after `build-rust.sh`
# Requirements: node

TARGET_DIR=debug
if [ -n "${OPTIMIZE+x}" ]; then
  RUSTC_FLAGS="--release"
  TARGET_DIR=release
fi

npm run build -w @nitrogql/core -w @nitrogql/loader-core -w @nitrogql/wasi-preview1 -w @nitrogql/esbuild-register

mkdir -p packages/cli/wasm
cp target/wasm32-wasi/$TARGET_DIR/nitrogql-cli.opt.wasm packages/cli/wasm/nitrogql-cli.wasm

mkdir -p packages/loader-core/wasm
cp target/wasm32-unknown-unknown/$TARGET_DIR/graphql-loader.opt.wasm packages/loader-core/wasm/graphql-loader.wasm

current_version=$(npm pkg get version --json)
npm pkg set version=${current_version} --json --workspaces
npm pkg set dependencies.@nitrogql/esbuild-register=${current_version} --json -w @nitrogql/core
npm pkg set dependencies.@nitrogql/core=${current_version} --json -w @nitrogql/cli -w @nitrogql/rollup-plugin -w @nitrogql/graphql-loader -w @nitrogql/jest-transform
npm pkg set dependencies.@nitrogql/loader-core=${current_version} --json -w @nitrogql/rollup-plugin -w @nitrogql/graphql-loader -w @nitrogql/jest-transform
npm pkg set dependencies.@nitrogql/wasi-preview1=${current_version} --json -w @nitrogql/cli
npx prettier --write "./**/package.json"