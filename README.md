# nitrogql


## Memo

`cargo install cargo-wasi` to install `cargo-wasi`.

### Example command

```sh
cargo run -- --schema 'sample_gql/schema/*.graphql' --operation 'sample_gql/operations/*.graphql' --schema-output sample_gql/schema.d.ts generate
```