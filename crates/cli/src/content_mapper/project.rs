use std::{
    borrow::Cow,
    cell::OnceCell,
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use graphql_type_system::Schema;
use log::{debug, info};
use nitrogql_ast::{TypeSystemDocument, base::Pos};
use nitrogql_config_file::{Config, load_config};
use nitrogql_error::PositionedError;
use nitrogql_plugin::Plugin;
use nitrogql_semantics::ast_to_type_system;
use nitrogql_utils::normalize_path;
use self_cell::self_cell;
use sourcemap_writer::RecordedSpan;
use thiserror::Error;
use typed_arena::Arena;

use crate::{
    CommandError, SchemaSource,
    check::resolve_schema,
    error::CliError,
    file_store::{FileKind, FileStore},
    load_plugins, parse_schema_files, read_schema_files,
    schema_loader::{LoadedSchema, SchemaFileKind, schema_kind_by_path},
};

/// Options given to the content mapper via the `options` field of tsconfig's `contentMappers` entry.
#[derive(Debug, Default)]
pub struct MapperOptions {
    /// Path to nitrogql config file, relative to tsconfig.json.
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("nitrogql content mapper requires a tsconfig.json file")]
    NoTsConfig,
    #[error(
        "nitrogql config file was not found in {0}. Specify the 'configFile' option of the content mapper to use a config file in another location."
    )]
    ConfigNotFound(PathBuf),
}

/// A project opened by TypeScript.
pub struct Project {
    /// tsconfig.json given by TypeScript.
    pub ts_config_file: PathBuf,
    /// Directory that contains the nitrogql config file.
    pub root_dir: PathBuf,
    pub config: Config,
    /// Fingerprint of everything that affects transform results except for the transformed file itself.
    pub config_identity: String,
    /// Files whose changes should cause the project to be reloaded.
    pub watched_files: Vec<PathBuf>,
    /// Absolute paths of loaded schema files, along with their indices in the file store.
    pub schema_files: Vec<(PathBuf, usize)>,
    files: LoadedFiles,
    /// Schema types printed for the schema module, computed on first request.
    schema_module: OnceCell<Result<SchemaModule, PositionedError>>,
}

/// Schema types printed for the file that provides them.
pub struct SchemaModule {
    pub text: String,
    pub spans: Vec<RecordedSpan>,
}

self_cell!(
    /// Sources of loaded files, along with data that borrows from them.
    struct LoadedFiles {
        owner: Arena<String>,
        #[covariant]
        dependent: ProjectFiles,
    }
);

struct ProjectFiles<'src> {
    file_store: FileStore<'src>,
    state: ProjectState<'src>,
}

impl LoadedFiles {
    /// Creates a project that could not be loaded.
    fn from_errors(errors: Vec<PositionedError>) -> Self {
        LoadedFiles::new(Arena::new(), |sources| ProjectFiles {
            file_store: FileStore::new(sources),
            state: ProjectState::Error(errors),
        })
    }
}

enum ProjectState<'src> {
    Loaded(Box<LoadedProjectSchema<'src>>),
    /// Project could not be loaded. Errors are reported on transform.
    Error(Vec<PositionedError>),
}

pub struct LoadedProjectSchema<'src> {
    pub schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
    pub type_system: Schema<Cow<'src, str>, Pos>,
}

impl Project {
    /// Opens a project for given tsconfig.json.
    ///
    /// `previous` is a closed project for the same tsconfig.json.
    /// Its schema is reused if the config and schema files are the same.
    pub async fn open(
        ts_config_file: &Path,
        options: &MapperOptions,
        previous: Option<Project>,
    ) -> Project {
        let ts_config_dir = ts_config_file
            .parent()
            .map(normalize_path)
            .unwrap_or_default();
        // The project is rooted at the directory of the config file if it is found.
        let (config_path, root_dir, config, sources) =
            match find_config(ts_config_file, &ts_config_dir, options).await {
                Ok((config_path, config)) => {
                    let root_dir = normalize_path(config_path.parent().unwrap_or(Path::new("")));
                    let sources = load_sources(&root_dir, &config).await;
                    (Some(config_path), root_dir, config, sources)
                }
                Err(errors) => (None, ts_config_dir, Config::default(), Err(errors)),
            };

        let (files, schema_module) = match sources {
            Ok((plugins, schema_sources)) => {
                match previous.filter(|previous| previous.has_same_schema(&config, &schema_sources))
                {
                    Some(previous) => {
                        debug!("Reusing the schema of the previous project");
                        (previous.files, previous.schema_module)
                    }
                    None => (
                        LoadedFiles::new(Arena::new(), |sources| {
                            let mut file_store = FileStore::new(sources);
                            let state =
                                match parse_schema(schema_sources, &mut file_store, &plugins) {
                                    Ok(schema) => ProjectState::Loaded(Box::new(schema)),
                                    Err(errors) => ProjectState::Error(errors),
                                };
                            ProjectFiles { file_store, state }
                        }),
                        OnceCell::new(),
                    ),
                }
            }
            Err(errors) => (LoadedFiles::from_errors(errors), OnceCell::new()),
        };

        let file_store = &files.borrow_dependent().file_store;
        let schema_files = file_store
            .iter_kind(FileKind::Schema)
            .map(|(idx, (path, _, _))| (normalize_path(path), idx))
            .collect();
        let (watched_files, config_identity) =
            watched_files_and_identity(&config, config_path.as_deref(), file_store);

        Project {
            ts_config_file: ts_config_file.to_owned(),
            root_dir,
            config,
            config_identity,
            watched_files,
            schema_files,
            files,
            schema_module,
        }
    }

    /// Returns whether this project loaded the same schema as given config and sources would load.
    fn has_same_schema(
        &self,
        config: &Config,
        schema_sources: &[Result<SchemaSource, PositionedError>],
    ) -> bool {
        // Options are compared by their debug output, as in the project identity.
        if format!("{:?}", self.config) != format!("{config:?}") {
            return false;
        }
        let mut sources = vec![];
        for source in schema_sources {
            let Ok(source) = source else {
                return false;
            };
            // JavaScript schemas also provide type extensions to plugins, which are not compared.
            if matches!(
                schema_kind_by_path(&source.path),
                SchemaFileKind::SchemaJavaScript
            ) {
                return false;
            }
            sources.push((source.path.as_path(), source.content.as_str()));
        }
        let loaded = self
            .file_store()
            .iter_kind(FileKind::Schema)
            .map(|(_, (path, content, _))| (path, content))
            .collect::<Vec<_>>();
        // A project that failed to load has no files, and is never reused.
        !loaded.is_empty() && loaded == sources
    }

    /// Returns the store of loaded schema files.
    pub fn file_store(&self) -> &FileStore<'_> {
        &self.files.borrow_dependent().file_store
    }

    /// Returns the printed schema module, which is computed on first use.
    pub fn schema_module(&self) -> &OnceCell<Result<SchemaModule, PositionedError>> {
        &self.schema_module
    }

    /// Returns the loaded schema, or errors if the project could not be loaded.
    pub fn schema(&self) -> Result<&LoadedProjectSchema<'_>, &[PositionedError]> {
        match &self.files.borrow_dependent().state {
            ProjectState::Loaded(schema) => Ok(schema),
            ProjectState::Error(errors) => Err(errors),
        }
    }

    /// Looks up the given file among loaded schema files.
    /// Returns the index of the file in the file store.
    pub fn schema_file_index(&self, path: &Path) -> Option<usize> {
        self.schema_files
            .iter()
            .find(|(schema_path, _)| schema_path == path)
            .map(|(_, idx)| *idx)
    }
}

/// Finds and loads the nitrogql config file for given tsconfig.json.
/// Returns the normalized path of the config file and its contents.
async fn find_config(
    ts_config_file: &Path,
    ts_config_dir: &Path,
    options: &MapperOptions,
) -> Result<(PathBuf, Config), Vec<PositionedError>> {
    if ts_config_file.as_os_str().is_empty() {
        return Err(vec![ProjectError::NoTsConfig.into()]);
    }
    let (config_path, config) = load_config(ts_config_dir, options.config_file.as_deref())
        .await
        .map_err(|err| vec![err.into()])?
        .ok_or_else(|| vec![ProjectError::ConfigNotFound(ts_config_dir.to_owned()).into()])?;
    info!("Loaded config file from {}", config_path.display());
    Ok((normalize_path(&config_path), config))
}

/// Loads plugins and reads schema files specified in the config.
async fn load_sources(
    root_dir: &Path,
    config: &Config,
) -> Result<(Vec<Plugin>, Vec<Result<SchemaSource, PositionedError>>), Vec<PositionedError>> {
    if config.schema.is_empty() {
        return Err(vec![CliError::NoSchemaSpecified.into()]);
    }
    let mut plugins = load_plugins(&config.plugins).map_err(|err| vec![err.into()])?;
    let schema_sources = read_schema_files(root_dir, &config.schema, &mut plugins)
        .await
        .map_err(CommandError::into_errors)?;
    Ok((plugins, schema_sources))
}

/// Computes files to watch and identity of a project.
///
/// Watched files are the config file and schema files. Operation files are not watched:
/// the output of an operation file only depends on its own contents and the project,
/// and TypeScript tracks the files it imports fragments from.
///
/// The identity is computed from what affects the output of other files: the config,
/// contents of the config file and schema files, and schema additions of plugins.
/// Additions are included because they can depend on type extensions of JavaScript schemas,
/// which are not part of the printed schema.
fn watched_files_and_identity(
    config: &Config,
    config_path: Option<&Path>,
    files: &FileStore,
) -> (Vec<PathBuf>, String) {
    let mut watched: BTreeMap<PathBuf, Option<Cow<str>>> = BTreeMap::new();
    if let Some(config_path) = config_path {
        let content = fs::read_to_string(config_path).ok().map(Cow::Owned);
        watched.insert(config_path.to_owned(), content);
    }
    for (_, (path, content, _)) in files.iter_kind(FileKind::Schema) {
        watched.insert(normalize_path(path), Some(Cow::Borrowed(content)));
    }

    let mut hasher = Fnv1a::new();
    hasher.write(format!("{config:?}").as_bytes());
    for (path, content) in watched.iter() {
        hasher.write(path.to_string_lossy().as_bytes());
        hasher.write(&[0]);
        if let Some(content) = content {
            hasher.write(content.as_bytes());
        }
        hasher.write(&[0]);
    }
    for (_, (_, content, _)) in files.iter_kind(FileKind::Plugin) {
        hasher.write(content.as_bytes());
        hasher.write(&[0]);
    }
    (
        watched.into_keys().collect(),
        format!("{:016x}", hasher.finish()),
    )
}

/// Parses schema files and resolves the schema.
fn parse_schema<'src>(
    schema_sources: Vec<Result<SchemaSource, PositionedError>>,
    file_store: &mut FileStore<'src>,
    plugins: &[Plugin],
) -> Result<LoadedProjectSchema<'src>, Vec<PositionedError>> {
    let schema = parse_schema_files(schema_sources, file_store, plugins)
        .map_err(CommandError::into_errors)?;
    let schema = resolve_schema(schema, plugins)?;
    let type_system = match schema {
        LoadedSchema::GraphQL(ref doc) => ast_to_type_system(doc),
        LoadedSchema::Introspection(ref schema) => schema.clone(),
    };
    Ok(LoadedProjectSchema {
        schema,
        type_system,
    })
}

/// FNV-1a hash, used for computing project identity.
/// Unlike `DefaultHasher`, results are stable across builds.
struct Fnv1a(u64);

impl Fnv1a {
    fn new() -> Self {
        Fnv1a(0xcbf29ce484222325)
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}
