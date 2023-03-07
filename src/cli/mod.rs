use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use globmatch::wrappers::{build_matchers, match_paths};
use log::{debug, error};
use thiserror::Error;

use crate::{
    cli::{context::CliContext, error::CliError},
    error::print_positioned_error,
    graphql_parser::{
        ast::{base::set_current_file_of_pos, OperationDocument, TypeSystemOrExtensionDocument},
        parser::{parse_operation_document, parse_type_system_document},
    },
};

use self::{check::run_check, context::CliConfig, generate::run_generate};

mod check;
mod context;
mod error;
mod generate;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    /// Path to schema document(s).
    schema: Vec<String>,
    #[arg(long)]
    /// Path to operation document(s).
    operation: Vec<String>,
    #[arg(long)]
    /// Path to save schema type definition file.
    schema_output: Option<PathBuf>,
    commands: Vec<String>,
}

/// Run as CLI. Returns 0 if successful
pub fn run_cli(args: impl IntoIterator<Item = String>) -> usize {
    pretty_env_logger::init();
    match run_cli_impl(args) {
        Ok(()) => 0,
        Err(err) => {
            error!("{err}");
            1
        }
    }
}

fn run_cli_impl(args: impl IntoIterator<Item = String>) -> Result<()> {
    let args = Args::parse_from(args);
    if args.commands.is_empty() {
        return Err(CliError::NoCommandSpecified.into());
    }
    if args.schema.is_empty() {
        return Err(CliError::NoSchemaSpecified.into());
    }
    let schema_files = load_glob_files(&args.schema)?;
    let mut file_by_index = schema_files
        .iter()
        .map(|(path, src)| (path.clone(), src.as_str()))
        .collect::<Vec<_>>();
    let schema_docs = schema_files
        .iter()
        .enumerate()
        .map(
            |(file_idx, (path, buf))| -> Result<TypeSystemOrExtensionDocument> {
                debug!("parsing(schema) {}", path.to_string_lossy());
                set_current_file_of_pos(file_idx);
                let doc = parse_type_system_document(&buf)?;
                Ok(doc)
            },
        )
        .collect::<Result<Vec<_>>>();
    let schema_docs = schema_docs?;
    let merged_schema_doc = TypeSystemOrExtensionDocument::merge(schema_docs);

    let operation_files = load_glob_files(&args.operation)?;
    let next_file_index = file_by_index.len();
    file_by_index.extend(
        operation_files
            .iter()
            .map(|(path, src)| (path.clone(), src.as_str())),
    );

    let operation_docs = operation_files
        .iter()
        .enumerate()
        .map(
            |(file_idx, (path, buf))| -> Result<(PathBuf, OperationDocument)> {
                let file_idx = next_file_index + file_idx;
                debug!("parsing(operation) {}", path.to_string_lossy());
                set_current_file_of_pos(file_idx);

                let doc = parse_operation_document(&buf)?;
                Ok((path.clone(), doc))
            },
        )
        .collect::<Result<Vec<_>>>();
    let operation_docs = operation_docs?;

    let mut context = CliContext::SchemaUnresolved {
        config: CliConfig {
            schema_output: args.schema_output,
        },
        schema: merged_schema_doc,
        operations: operation_docs,
        file_by_index,
    };

    for command in args.commands.iter() {
        let file_source_by_index = context.file_by_index();
        context = run_command(command, context).map_err(|err| CommandError {
            message: print_positioned_error(&err, &file_source_by_index),
        })?;
    }

    Ok(())
}

#[derive(Error, Debug)]
#[error("Error running command:\n{message}")]
struct CommandError {
    message: String,
}

fn run_command<'a>(command: &str, context: CliContext<'a>) -> crate::error::Result<CliContext<'a>> {
    match command {
        "check" => run_check(context),
        "generate" => run_generate(context),
        command => Err(CliError::UnknownCommand(command.to_owned()).into()),
    }
}

fn load_glob_files<'a, S: AsRef<str> + 'a>(
    globs: impl IntoIterator<Item = &'a S>,
) -> Result<Vec<(PathBuf, String)>> {
    let path_strs: Vec<&str> = globs.into_iter().map(|s| s.as_ref()).collect();
    if path_strs.is_empty() {
        return Ok(vec![]);
    }

    let root = std::env::current_dir()?;
    let schema_matchers =
        build_matchers(&path_strs, &root).map_err(|err| CliError::GlobError(err))?;

    let (paths, _) = match_paths(schema_matchers, None, None);
    let results = paths
        .into_iter()
        .map(|path| {
            debug!("loading {}", path.to_string_lossy());
            fs::read_to_string(&path).map(|res| (path, res))
        })
        .collect::<std::io::Result<_>>();

    results.map_err(|err| err.into())
}
