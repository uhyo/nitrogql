# nitrogql

### build

```sh
cargo rustc --target wasm32-wasi --release -- -Z wasi-exec-model=reactor
```

### Example command

```sh
cargo run --bin nitrogql-cli -- generate -c sample_gql/graphql.config.yaml
```