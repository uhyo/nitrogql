mod base64_vlq;
mod checker;
mod cli;
mod error;
mod extension_resolver;
mod graphql_builtins;
mod graphql_parser;
mod graphql_printer;
mod json_printer;
mod source_map_writer;
mod type_printer;
mod utils;

use cli::run_cli;

fn main() -> anyhow::Result<()> {
    run_cli(std::env::args());
    Ok(())
}
