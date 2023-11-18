use std::{
    fs,
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    process,
};

use anyhow::Result;
use builtins::nitrogql_builtins;
use clap::Parser;
use context::OutputFormat;
use file_store::FileStore;
use globmatch::wrappers::{build_matchers, match_paths};
use graphql_builtins::generate_builtins;
use graphql_type_system::Schema;
use itertools::Itertools;
use log::info;
use nitrogql_ast::{
    operation::OperationDocument, set_current_file_of_pos,
    type_system::TypeSystemOrExtensionDocument,
};
use nitrogql_introspection::schema_from_introspection_json;
use nitrogql_plugin::{Plugin, PluginSchemaExtensions};
use nitrogql_semantics::{resolve_operation_extensions, OperationExtension};
use nitrogql_utils::{get_cwd, normalize_path};
use output::CliOutput;
use plugin_host::PluginHost;
use schema_loader::LoadedSchema;

use crate::{
    context::CliContext,
    error::CliError,
    file_store::FileKind,
    load_plugins::load_plugins,
    schema_loader::{load_schema_js, schema_kind_by_path, LoadSchemaJsResult, SchemaFileKind},
};
use nitrogql_config_file::load_config;

use nitrogql_error::{print_positioned_error, PositionedError};
use nitrogql_parser::{parse_operation_document, parse_type_system_document};

use self::{check::run_check, context::CliConfig, generate::run_generate};

mod builtins;
mod check;
mod context;
mod error;
mod file_store;
mod generate;
mod load_plugins;
mod output;
mod plugin_host;
mod schema_loader;

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
    /// Output format of CLI.
    #[arg(long, default_value = "human")]
    output_format: OutputFormat,
    commands: Vec<String>,
}

fn main() {
    let exit_code = run_cli(std::env::args());
    process::exit(exit_code.try_into().unwrap());
}

/// Run as CLI. Returns 0 if successful
pub fn run_cli(args: impl IntoIterator<Item = String>) -> usize {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .env()
        .init()
        .unwrap();
    let mut output = CliOutput::new();
    let file_store = Box::leak(Box::new(FileStore::new()));
    let args = Args::parse_from(args);
    let output_format = args.output_format;
    let res = run_cli_impl(args, file_store, &mut output);
    let code = match res {
        Ok(()) => 0,
        Err(err) => {
            let message = err
                .inner
                .into_iter()
                .map(|e| {
                    if e.has_position() {
                        print_positioned_error(&e, file_store)
                    } else {
                        format!("{}", e.into_inner())
                    }
                })
                .join("\n");
            output.command_error(err.command, message);
            1
        }
    };

    match output_format {
        OutputFormat::Human => {
            output.human_output(file_store);
        }
        OutputFormat::Json => {
            output.json_output(file_store);
        }
        OutputFormat::Rdjson => {
            output.rdjson_output(file_store);
        }
    }

    code
}

fn run_cli_impl(
    args: Args,
    file_store: &mut FileStore,
    output: &mut CliOutput,
) -> Result<(), CommandError> {
    if args.commands.is_empty() {
        return Err(CliError::NoCommandSpecified.into());
    }
    let cwd = get_cwd()?;
    let config_file = load_config(&cwd, args.config_file.as_deref())?;
    let (root_dir, mut config) = if let Some((config_path, config_file)) = config_file {
        info!("Loaded config file from {}", config_path.display());
        (
            normalize_path(config_path.parent().unwrap_or(Path::new(""))),
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
    info!("Loaded config {config:?}");
    info!("root_dir {}", root_dir.display());

    let mut plugins = load_plugins(&config.plugins)?;

    if config.schema.is_empty() {
        return Err(CliError::NoSchemaSpecified.into());
    }

    let schema_files = load_glob_files(&root_dir, &config.schema)?;
    let (schema_docs, schema_errors): (Vec<_>, Vec<_>) = schema_files
        .into_iter()
        .map(|(path, buf)| match schema_kind_by_path(&path) {
            SchemaFileKind::GraphQL => {
                let file_idx = file_store.add_file(path, buf, FileKind::Schema);
                let (ref path, buf, _) = file_store.get_file(file_idx).unwrap();
                info!("parsing(schema) {} {}", path.to_string_lossy(), file_idx);
                set_current_file_of_pos(file_idx);
                let doc = parse_type_system_document(buf)?;
                Ok(LoadedSchema::GraphQL(doc))
            }
            SchemaFileKind::IntrospectionJson => {
                let file_idx = file_store.add_file(path, buf, FileKind::Schema);
                let (ref path, buf, _) = file_store.get_file(file_idx).unwrap();
                info!("parsing(introspection) {}", path.to_string_lossy());
                let doc = schema_from_introspection_json(buf)?;
                Ok(LoadedSchema::Introspection(doc))
            }
            SchemaFileKind::SchemaJavaScript => {
                info!("loading schema js {}", path.to_string_lossy());
                let LoadSchemaJsResult {
                    schema,
                    type_extensions,
                } = load_schema_js(&path)?;
                let file_idx = file_store.add_file(path, schema, FileKind::Schema);
                let (_, buf, _) = file_store.get_file(file_idx).unwrap();
                set_current_file_of_pos(file_idx);
                let doc = parse_type_system_document(buf)?;
                for p in plugins.iter_mut() {
                    p.load_schema_extensions(PluginSchemaExtensions {
                        type_extensions: &type_extensions,
                    });
                }
                Ok(LoadedSchema::GraphQL(doc))
            }
        })
        .partition_result();
    if !schema_errors.is_empty() {
        return Err(CommandError::merge(schema_errors));
    }

    let config = CliConfig {
        root_dir,
        config,
        plugins,
    };
    let merged_schema_doc = {
        let mut merged_schema_doc = resolve_loaded_schema(schema_docs)?;
        extend_loaded_schema(&mut merged_schema_doc, file_store, &config.plugins)?;
        merged_schema_doc
    };

    let operation_files = load_glob_files(&config.root_dir, &config.config.operations)?;

    let (operation_docs, operation_errors): (Vec<_>, Vec<_>) =
        operation_files
            .into_iter()
            .map(
                |(path, buf)| -> Result<
                    (PathBuf, OperationDocument, OperationExtension, usize),
                    CommandError,
                > {
                    info!("parsing(operation) {}", path.to_string_lossy());
                    let file_idx = file_store.add_file(path.clone(), buf, FileKind::Operation);
                    let (_, buf, _) = file_store.get_file(file_idx).unwrap();
                    set_current_file_of_pos(file_idx);

                    let doc = parse_operation_document(buf)?;
                    let (doc, extensions) = resolve_operation_extensions(doc)?;
                    Ok((path, doc, extensions, file_idx))
                },
            )
            .partition_result();
    if !operation_errors.is_empty() {
        return Err(CommandError::merge(operation_errors));
    }

    let mut context = CliContext::SchemaUnresolved {
        config,
        schema: merged_schema_doc,
        operations: operation_docs,
        file_store,
        output,
    };

    for command in args.commands.iter() {
        context = run_command(command, context)
            .map_err(|err| CommandError::new(vec![err], command.clone()))?;
    }

    Ok(())
}

struct CommandError {
    pub inner: Vec<PositionedError>,
    pub command: Option<String>,
}

impl CommandError {
    pub fn new(inner: Vec<PositionedError>, command: String) -> Self {
        CommandError {
            inner,
            command: Some(command),
        }
    }
    pub fn merge(errors: impl IntoIterator<Item = Self>) -> Self {
        let (inner, command) =
            errors
                .into_iter()
                .fold((vec![], None), |(mut inner, command), err| {
                    inner.extend(err.inner);
                    (inner, command.or(err.command))
                });
        CommandError { inner, command }
    }
}

impl<E: Into<PositionedError>> From<E> for CommandError {
    fn from(err: E) -> Self {
        CommandError {
            inner: vec![err.into()],
            command: None,
        }
    }
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

    info!("load_glob_files {} {}", root.display(), path_strs.join(" "));
    let schema_matchers = build_matchers(&path_strs, root).map_err(CliError::GlobError)?;
    let (paths, _) = match_paths(schema_matchers, None, None);
    info!("match_paths {paths:?}");
    let results = paths
        .into_iter()
        .map(|path| {
            info!("loading {}", path.to_string_lossy());
            fs::read_to_string(&path).map(|res| (path, res))
        })
        .collect::<std::io::Result<_>>();

    results.map_err(|err| err.into())
}

fn resolve_loaded_schema<'src>(
    schema_docs: Vec<LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>>,
) -> Result<LoadedSchema<TypeSystemOrExtensionDocument>, CliError> {
    let mut introsection: Option<Schema<_, _>> = None;
    let mut documents: Vec<TypeSystemOrExtensionDocument> = vec![];
    for doc in schema_docs {
        match doc {
            LoadedSchema::Introspection(doc) => {
                if introsection.is_some() {
                    return Err(CliError::IntrospectionOnce);
                }
                introsection = Some(doc);
            }
            LoadedSchema::GraphQL(doc) => documents.push(doc),
        }
    }
    if introsection.is_some() && !documents.is_empty() {
        return Err(CliError::MixGraphQLAndIntrospection);
    }
    match introsection {
        Some(doc) => Ok(LoadedSchema::Introspection(doc)),
        None => {
            let merged = TypeSystemOrExtensionDocument::merge(documents);
            Ok(LoadedSchema::GraphQL(merged))
        }
    }
}

/// Extend loaded schema with builtins and plugins.
fn extend_loaded_schema<'src>(
    schema: &mut LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
    file_store: &mut FileStore,
    plugins: &[Plugin<'src>],
) -> Result<(), CommandError> {
    let mut plugin_host = PluginHost::new(file_store);
    match schema {
        LoadedSchema::Introspection(_) => Ok(()),
        LoadedSchema::GraphQL(doc) => {
            doc.extend(generate_builtins());
            doc.extend(nitrogql_builtins());
            for plugin in plugins {
                if let Some(addition) = plugin.schema_addition(&mut plugin_host)? {
                    doc.extend(addition.definitions);
                }
            }
            Ok(())
        }
    }
}

/// Allocate a string buffer of given size.
///
/// # Safety
/// Caller should guarantee that the contents of returned buffer should be valid UTF-8 strings.
#[no_mangle]
pub extern "C" fn alloc_string(len_bytes: usize) -> *mut u8 {
    let str = String::with_capacity(len_bytes);
    let mut str = ManuallyDrop::new(str);
    str.as_mut_ptr()
}

/// Frees a string buffer returned by `alloc_string`.
///
/// # Safety
/// `free_string` should only be called with a pointer returned by `alloc_string` with the same value of `len_bytes` argument.
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut u8, len_bytes: usize) {
    let _ = unsafe { String::from_raw_parts(ptr, 0, len_bytes) };
}
