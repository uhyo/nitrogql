#! /bin/bash
set -eux

# Script to build rust code.
# Expected to be run after `build-rust.sh`
# Requirements: node

npm run build -w @nitrogql/core

mkdir -p packages/cli/wasm
cp target/wasm32-wasi/release/nitrogql-cli.opt.wasm packages/cli/wasm/nitrogql-cli.wasm

mkdir -p packages/graphql-loader/wasm
cp target/wasm32-unknown-unknown/release/graphql-loader.opt.wasm packages/graphql-loader/wasm/graphql-loader.wasm

mkdir -p packages/rollup-plugin/wasm
cp target/wasm32-unknown-unknown/release/graphql-loader.opt.wasm packages/rollup-plugin/wasm/graphql-loader.wasm

current_version=$(npm pkg get version --json)
npm pkg set version=${current_version} --json --workspaces
npm pkg set dependencies.@nitrogql/core=${current_version} --json -w @nitrogql/cli
npx prettier --write "./**/package.json"