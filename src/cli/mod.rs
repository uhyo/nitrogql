use std::{
    fs,
    io::{stderr, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use globmatch::wrappers::{build_matchers, match_paths};

use crate::{
    cli::{context::CliContext, error::CliError},
    graphql_parser::{ast::TypeSystemOrExtensionDocument, parser::parse_type_system_document},
};

use self::check::run_check;

mod check;
mod context;
mod error;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    /// Path to schema document(s).
    schema: Vec<String>,
    #[arg(long)]
    /// Path to operation document(s).
    operation: Vec<String>,
    commands: Vec<String>,
}

/// Run as CLI. Returns 0 if successful
pub fn run_cli(args: impl IntoIterator<Item = String>) -> usize {
    match run_cli_impl(args) {
        Ok(()) => 0,
        Err(err) => {
            writeln!(stderr(), "{err}").unwrap();
            1
        }
    }
}

fn run_cli_impl(args: impl IntoIterator<Item = String>) -> Result<()> {
    let args = Args::parse_from(args);
    println!("Hello, {args:?}");
    if args.commands.is_empty() {
        return Err(CliError::NoCommandSpecified.into());
    }
    if args.schema.is_empty() {
        return Err(CliError::NoSchemaSpecified.into());
    }
    let schema_files = load_schema_files(&args)?;
    let schema_docs = schema_files
        .iter()
        .map(|(_, buf)| -> Result<TypeSystemOrExtensionDocument> {
            let doc = parse_type_system_document(&buf)?;
            Ok(doc)
        })
        .collect::<Result<Vec<_>>>();
    let schema_docs = schema_docs?;
    let merged_schema_doc = TypeSystemOrExtensionDocument::merge(schema_docs);

    let mut context = CliContext::SchemaUnresolved {
        schema: merged_schema_doc,
    };

    for command in args.commands.iter() {
        context = run_command(command, context)?;
    }

    Ok(())
}

fn run_command<'a>(command: &str, context: CliContext<'a>) -> Result<CliContext<'a>> {
    match command {
        "check" => run_check(context),
        command => Err(CliError::UnknownCommand(command.to_owned()).into()),
    }
}

fn load_schema_files(args: &Args) -> Result<Vec<(PathBuf, String)>> {
    let path_strs: Vec<&str> = args.schema.iter().map(|s| s.as_str()).collect();
    let root = std::env::current_dir()?;
    let schema_matchers =
        build_matchers(&path_strs, &root).map_err(|err| CliError::GlobError(err))?;

    let (paths, _) = match_paths(schema_matchers, None, None);
    let results = paths
        .into_iter()
        .map(|path| fs::read_to_string(&path).map(|res| (path, res)))
        .collect::<std::io::Result<_>>();

    results.map_err(|err| err.into())
}
