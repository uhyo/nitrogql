use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::Result;
use clap::Parser;
use globmatch::wrappers::{build_matchers, match_paths};
use log::{debug, error, trace};
use nitrogql_ast::{
    operation::OperationDocument, set_current_file_of_pos,
    type_system::TypeSystemOrExtensionDocument,
};
use nitrogql_utils::get_cwd;
use thiserror::Error;

use crate::{context::CliContext, error::CliError};
use nitrogql_config_file::load_config;

use nitrogql_error::print_positioned_error;
use nitrogql_parser::{parse_operation_document, parse_type_system_document};

use self::{check::run_check, context::CliConfig, generate::run_generate};

mod check;
mod context;
mod error;
mod generate;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, short = 'c')]
    /// Path to config file.
    config_file: Option<PathBuf>,
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

fn main() {
    let exit_code = run_cli(std::env::args());
    process::exit(exit_code.try_into().unwrap());
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
    let cwd = get_cwd()?;
    let config_file = load_config(&cwd, args.config_file.as_ref().map(|p| p.as_path()))?;
    let (root_dir, mut config) = if let Some((config_path, config_file)) = config_file {
        debug!("Loaded config file from {}", config_path.display());
        (
            config_path
                .parent()
                .map(|path| path.to_owned())
                .unwrap_or(PathBuf::new()),
            config_file,
        )
    } else {
        (get_cwd()?, Default::default())
    };
    // Override config with args
    if !args.schema.is_empty() {
        config.schema = args.schema;
    }
    if !args.operation.is_empty() {
        config.operations = args.operation;
    }
    if let Some(path) = args.schema_output {
        config.generate.schema_output = Some(path);
    }
    debug!("Loaded config {config:?}");

    let config = CliConfig { root_dir, config };

    if config.config.schema.is_empty() {
        return Err(CliError::NoSchemaSpecified.into());
    }

    let schema_files = load_glob_files(&config.root_dir, &config.config.schema)?;
    let file_by_index = schema_files
        .iter()
        .map(|(path, src)| (path.clone(), src.as_str()))
        .collect::<Vec<_>>();
    let schema_docs = schema_files
        .iter()
        .enumerate()
        .map(
            |(file_idx, (path, buf))| -> Result<TypeSystemOrExtensionDocument> {
                debug!("parsing(schema) {} {}", path.to_string_lossy(), file_idx);
                set_current_file_of_pos(file_idx);
                let doc = parse_type_system_document(&buf)?;
                Ok(doc)
            },
        )
        .collect::<Result<Vec<_>>>();
    let schema_docs = schema_docs?;
    let merged_schema_doc = TypeSystemOrExtensionDocument::merge(schema_docs);

    let operation_files = load_glob_files(&config.root_dir, &config.config.operations)?;
    let op_file_index = file_by_index.len();

    let operation_docs = operation_files
        .iter()
        .map(
            |(path, buf)| -> Result<(PathBuf, OperationDocument, Vec<(PathBuf, &str)>)> {
                debug!("parsing(operation) {}", path.to_string_lossy());
                set_current_file_of_pos(op_file_index);

                let doc = parse_operation_document(&buf)?;
                let file_by_idx_op = file_by_index
                    .iter()
                    .map(|(path, buf)| (path.clone(), *buf))
                    .chain(vec![(path.clone(), buf.as_str())])
                    .collect::<Vec<_>>();
                Ok((path.clone(), doc, file_by_idx_op))
            },
        )
        .collect::<Result<Vec<_>>>();
    let operation_docs = operation_docs?;

    let mut context = CliContext::SchemaUnresolved {
        config,
        schema: merged_schema_doc,
        operations: operation_docs,
        file_by_index,
    };

    for command in args.commands.iter() {
        let file_source_by_index = context.file_by_index();
        context = run_command(command, context).map_err(|err| {
            if err.has_position() {
                CommandError::Message {
                    message: print_positioned_error(&err, &file_source_by_index),
                }
            } else {
                CommandError::Other(err.into_inner())
            }
        })?;
    }

    Ok(())
}

#[derive(Error, Debug)]
enum CommandError {
    #[error("Error running command:\n{message}")]
    Message { message: String },
    #[error("Error running command:\n{0}")]
    Other(#[from] anyhow::Error),
}

fn run_command<'a>(
    command: &str,
    context: CliContext<'a>,
) -> nitrogql_error::Result<CliContext<'a>> {
    match command {
        "check" => run_check(context),
        "generate" => run_generate(context),
        command => Err(CliError::UnknownCommand(command.to_owned()).into()),
    }
}

fn load_glob_files<'a, S: AsRef<str> + 'a>(
    root: &Path,
    globs: impl IntoIterator<Item = &'a S>,
) -> Result<Vec<(PathBuf, String)>> {
    let path_strs: Vec<&str> = globs.into_iter().map(|s| s.as_ref()).collect();
    if path_strs.is_empty() {
        return Ok(vec![]);
    }

    trace!("load_glob_files");
    let schema_matchers =
        build_matchers(&path_strs, &root).map_err(|err| CliError::GlobError(err))?;
    let (paths, _) = match_paths(schema_matchers, None, None);
    trace!("match_paths {paths:?}");
    let results = paths
        .into_iter()
        .map(|path| {
            debug!("loading {}", path.to_string_lossy());
            fs::read_to_string(&path).map(|res| (path, res))
        })
        .collect::<std::io::Result<_>>();

    results.map_err(|err| err.into())
}
