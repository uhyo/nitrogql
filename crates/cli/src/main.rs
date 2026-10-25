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
use nitrogql_ast::type_system::TypeSystemOrExtensionDocument;
use nitrogql_introspection::schema_from_introspection_json;
use nitrogql_plugin::{Plugin, PluginSchemaExtensions};
use nitrogql_utils::{get_cwd, normalize_path};
use output::CliOutput;
use plugin_host::PluginHost;
use schema_loader::LoadedSchema;
use typed_arena::Arena;

use crate::{
    context::CliContext,
    error::CliError,
    file_store::FileKind,
    load_plugins::load_plugins,
    schema_loader::{LoadSchemaJsResult, SchemaFileKind, load_schema_js, schema_kind_by_path},
};
use nitrogql_config_file::load_config;

use nitrogql_error::{PositionedError, print_positioned_error};
use nitrogql_operation_set::OperationSet;
use nitrogql_parser::parse_type_system_document;

use self::{
    check::run_check, content_mapper::run_content_mapper, context::CliConfig,
    generate::run_generate,
};

mod builtins;
mod check;
mod content_mapper;
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
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) == Some("content-mapper") {
        nitrogql_async_runtime::spawn(run_content_mapper());
    } else {
        nitrogql_async_runtime::spawn(run_cli(args));
    }
    nitrogql_async_runtime::drive();
}

/// Run as CLI.
pub async fn run_cli(args: impl IntoIterator<Item = String>) {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .env()
        .init()
        .unwrap();
    let mut output = CliOutput::new();
    let sources = Arena::new();
    let mut file_store = FileStore::new(&sources);
    let args = Args::parse_from(args);
    let output_format = args.output_format;
    let res = run_cli_impl(args, &mut file_store, &mut output).await;
    let code = match res {
        Ok(()) => 0,
        Err(err) => {
            let message = err
                .inner
                .into_iter()
                .map(|e| print_positioned_error(&e, &file_store))
                .join("\n");
            output.command_error(err.command, message);
            1
        }
    };

    match output_format {
        OutputFormat::Human => {
            output.human_output(&file_store);
        }
        OutputFormat::Json => {
            output.json_output(&file_store);
        }
        OutputFormat::Rdjson => {
            output.rdjson_output(&file_store);
        }
    }

    process::exit(code);
}

async fn run_cli_impl(
    args: Args,
    file_store: &mut FileStore<'_>,
    output: &mut CliOutput,
) -> Result<(), CommandError> {
    if args.commands.is_empty() {
        return Err(CliError::NoCommandSpecified.into());
    }
    let cwd = get_cwd()?;
    let config_file = load_config(&cwd, args.config_file.as_deref()).await?;
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

    let merged_schema_doc =
        load_schema(&root_dir, &config.schema, file_store, &mut plugins).await?;

    let config = CliConfig {
        root_dir,
        config,
        plugins,
    };

    let operation_files = load_glob_files(&config.root_dir, &config.config.operations)?;

    let mut operations = OperationSet::new();
    for (path, buf) in operation_files {
        info!("parsing(operation) {}", path.to_string_lossy());
        let file_idx = file_store.add_file(path.clone(), buf, FileKind::Operation);
        let (_, buf, _) = file_store.get_file(file_idx).unwrap();
        operations.add_file(&path, buf, file_idx);
    }
    // Other errors are reported by the check command.
    let syntax_errors = operations.take_syntax_errors();
    if !syntax_errors.is_empty() {
        return Err(CommandError {
            inner: syntax_errors,
            command: None,
        });
    }

    let mut context = CliContext::SchemaUnresolved {
        config,
        schema: merged_schema_doc,
        operations,
        file_store: &*file_store,
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

/// Loads schema files matched by given globs.
/// Returned schema is merged and extended with builtins and plugins.
pub(crate) async fn load_schema<'src>(
    root_dir: &Path,
    schema_globs: &[String],
    file_store: &mut FileStore<'src>,
    plugins: &mut [Plugin],
) -> Result<LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>, CommandError> {
    let sources = read_schema_files(root_dir, schema_globs, plugins).await?;
    parse_schema_files(sources, file_store, plugins)
}

/// Content of a schema file, which is not parsed yet.
pub(crate) struct SchemaSource {
    path: PathBuf,
    content: String,
    /// Whether the content is an introspection result. Otherwise, it is GraphQL.
    is_introspection: bool,
}

/// Reads schema files matched by given globs.
/// Schemas defined in JavaScript are executed here, and their type extensions are loaded into plugins.
/// Files that fail to load are reported in place of their sources.
pub(crate) async fn read_schema_files(
    root_dir: &Path,
    schema_globs: &[String],
    plugins: &mut [Plugin],
) -> Result<Vec<Result<SchemaSource, PositionedError>>, CommandError> {
    let schema_files = load_glob_files(root_dir, schema_globs)?;
    let mut sources = vec![];
    for (path, buf) in schema_files {
        let source = match schema_kind_by_path(&path) {
            SchemaFileKind::GraphQL => Ok(SchemaSource {
                path,
                content: buf,
                is_introspection: false,
            }),
            SchemaFileKind::IntrospectionJson => Ok(SchemaSource {
                path,
                content: buf,
                is_introspection: true,
            }),
            SchemaFileKind::SchemaJavaScript => {
                info!("loading schema js {}", path.to_string_lossy());
                load_schema_js(&path)
                    .await
                    .map(
                        |LoadSchemaJsResult {
                             schema,
                             type_extensions,
                         }| {
                            for p in plugins.iter_mut() {
                                p.load_schema_extensions(PluginSchemaExtensions {
                                    type_extensions: &type_extensions,
                                });
                            }
                            SchemaSource {
                                path,
                                content: schema,
                                is_introspection: false,
                            }
                        },
                    )
                    .map_err(PositionedError::from)
            }
        };
        sources.push(source);
    }
    Ok(sources)
}

/// Parses schema files read by `read_schema_files` and adds them to the file store.
/// Returned schema is merged and extended with builtins and plugins.
pub(crate) fn parse_schema_files<'src>(
    sources: Vec<Result<SchemaSource, PositionedError>>,
    file_store: &mut FileStore<'src>,
    plugins: &[Plugin],
) -> Result<LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>, CommandError> {
    let mut schema_docs = vec![];
    let mut schema_errors = vec![];
    for source in sources {
        let res = source.and_then(|source| {
            let file_idx = file_store.add_file(source.path, source.content, FileKind::Schema);
            let (path, buf, _) = file_store.get_file(file_idx).unwrap();
            if source.is_introspection {
                info!("parsing(introspection) {}", path.to_string_lossy());
                let doc = schema_from_introspection_json(buf)?;
                Ok(LoadedSchema::Introspection(doc))
            } else {
                info!("parsing(schema) {} {}", path.to_string_lossy(), file_idx);
                let doc = parse_type_system_document(buf, file_idx)?;
                Ok(LoadedSchema::GraphQL(doc))
            }
        });
        match res {
            Ok(doc) => schema_docs.push(doc),
            Err(err) => schema_errors.push(CommandError::from(err)),
        }
    }
    if !schema_errors.is_empty() {
        return Err(CommandError::merge(schema_errors));
    }

    let mut merged_schema_doc = resolve_loaded_schema(schema_docs)?;
    extend_loaded_schema(&mut merged_schema_doc, file_store, plugins)?;
    Ok(merged_schema_doc)
}

fn resolve_loaded_schema<'src>(
    schema_docs: Vec<LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>>,
) -> Result<LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>, CliError> {
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
    file_store: &mut FileStore<'src>,
    plugins: &[Plugin],
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
#[unsafe(no_mangle)]
pub extern "C" fn alloc_string(len_bytes: usize) -> *mut u8 {
    let str = String::with_capacity(len_bytes);
    let mut str = ManuallyDrop::new(str);
    str.as_mut_ptr()
}

/// Frees a string buffer returned by `alloc_string`.
///
/// # Safety
/// `free_string` should only be called with a pointer returned by `alloc_string` with the same value of `len_bytes` argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_string(ptr: *mut u8, len_bytes: usize) {
    let _ = unsafe { String::from_raw_parts(ptr, 0, len_bytes) };
}
