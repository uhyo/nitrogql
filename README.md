# nitrogql


## Memo

`cargo install cargo-wasi` to install `cargo-wasi`.

### build

```sh
cargo rustc --target wasm32-wasi --release -- -Z wasi-exec-model=reactor
```

### Example command

```sh
cargo run -- generate -c sample_gql/graphql.config.yaml
```