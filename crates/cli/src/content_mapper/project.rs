use std::{
    borrow::Cow,
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use graphql_type_system::Schema;
use log::{debug, info};
use nitrogql_ast::{TypeSystemDocument, base::Pos};
use nitrogql_config_file::{Config, load_config};
use nitrogql_error::PositionedError;
use nitrogql_operation_set::OperationSet;
use nitrogql_plugin::Plugin;
use nitrogql_semantics::ast_to_type_system;
use nitrogql_utils::normalize_path;
use self_cell::self_cell;
use sourcemap_writer::RecordedSpan;
use thiserror::Error;
use typed_arena::Arena;

use crate::{
    SchemaSource,
    check::resolve_schema,
    error::CliError,
    file_store::FileStore,
    load_glob_files, load_plugins, parse_schema_files,
    plugin_host::PLUGIN_FILE_NAME,
    read_schema_files,
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
    /// Imports of operation files matched by the config and of files imported from them,
    /// by normalized path.
    operation_imports: HashMap<PathBuf, OperationImports>,
}

/// Schema types printed for the file that provides them.
pub struct SchemaModule {
    pub text: String,
    pub spans: Vec<RecordedSpan>,
}

/// Files imported from an operation file, along with the content they were computed from.
struct OperationImports {
    content: String,
    imports: Vec<PathBuf>,
}

impl OperationImports {
    /// Computes imports of given operation file, reusing `previous` if it has the same content.
    fn new(path: &Path, content: String, previous: Option<OperationImports>) -> Self {
        match previous {
            Some(previous) if previous.content == content => previous,
            _ => OperationImports {
                imports: imported_files(path, &content),
                content,
            },
        }
    }
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
    /// Its work is reused where inputs are unchanged: the schema is not parsed again
    /// if the config and schema files are the same, and operation files are not parsed again
    /// if their contents are the same.
    pub async fn open(
        ts_config_file: &Path,
        options: &MapperOptions,
        mut previous: Option<Project>,
    ) -> Project {
        let mut project = Project {
            ts_config_file: ts_config_file.to_owned(),
            root_dir: ts_config_file
                .parent()
                .map(normalize_path)
                .unwrap_or_default(),
            config: Config::default(),
            config_identity: String::new(),
            watched_files: vec![],
            schema_files: vec![],
            files: LoadedFiles::from_errors(vec![]),
            schema_module: OnceCell::new(),
            operation_imports: HashMap::new(),
        };
        let previous_imports = previous
            .as_mut()
            .map(|previous| std::mem::take(&mut previous.operation_imports))
            .unwrap_or_default();
        project.files = match project.load(ts_config_file, options).await {
            Ok((plugins, schema_sources)) => {
                match previous
                    .filter(|previous| previous.has_same_schema(&project.config, &schema_sources))
                {
                    Some(previous) => {
                        debug!("Reusing the schema of the previous project");
                        project.schema_module = previous.schema_module;
                        previous.files
                    }
                    None => LoadedFiles::new(Arena::new(), |sources| {
                        let mut file_store = FileStore::new(sources);
                        let state = match parse_schema(schema_sources, &mut file_store, &plugins) {
                            Ok(schema) => ProjectState::Loaded(Box::new(schema)),
                            Err(errors) => ProjectState::Error(errors),
                        };
                        ProjectFiles { file_store, state }
                    }),
                }
            }
            Err(errors) => LoadedFiles::from_errors(errors),
        };
        project.schema_files = project
            .file_store()
            .iter()
            // exclude virtual files added by plugins
            .filter(|(_, (path, _, _))| *path != Path::new(PLUGIN_FILE_NAME))
            .map(|(idx, (path, _, _))| (normalize_path(path), idx))
            .collect();
        project.compute_watched_files(previous_imports);
        project
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
            .iter()
            .filter(|(_, (path, _, _))| *path != Path::new(PLUGIN_FILE_NAME))
            .map(|(_, (path, content, _))| (path, content))
            .collect::<Vec<_>>();
        // A project that failed to load has no files, and is never reused.
        !loaded.is_empty() && loaded == sources
    }

    /// Loads the config and reads schema files.
    async fn load(
        &mut self,
        ts_config_file: &Path,
        options: &MapperOptions,
    ) -> Result<(Vec<Plugin>, Vec<Result<SchemaSource, PositionedError>>), Vec<PositionedError>>
    {
        if ts_config_file.as_os_str().is_empty() {
            return Err(vec![ProjectError::NoTsConfig.into()]);
        }
        let ts_config_dir = self.root_dir.clone();
        let (config_path, config) = load_config(&ts_config_dir, options.config_file.as_deref())
            .await
            .map_err(|err| vec![err.into()])?
            .ok_or_else(|| vec![ProjectError::ConfigNotFound(ts_config_dir).into()])?;
        info!("Loaded config file from {}", config_path.display());
        self.root_dir = normalize_path(config_path.parent().unwrap_or(Path::new("")));
        self.watched_files.push(normalize_path(&config_path));
        self.config = config;

        if self.config.schema.is_empty() {
            return Err(vec![CliError::NoSchemaSpecified.into()]);
        }
        let mut plugins = load_plugins(&self.config.plugins).map_err(|err| vec![err.into()])?;
        let schema_sources = read_schema_files(&self.root_dir, &self.config.schema, &mut plugins)
            .await
            .map_err(|err| err.inner)?;
        Ok((plugins, schema_sources))
    }

    /// Computes files to watch and identity of this project.
    ///
    /// Watched files are the config file, schema files, operation files matched by the config,
    /// and operation files imported from them.
    /// Imported operation files affect the output of importing files. Other operation files are
    /// watched too, as adding an `#import` to one of them adds such a dependency.
    ///
    /// The identity is computed from what affects the output of other files: the config,
    /// contents of the config file, schema files and imported operation files,
    /// and imports of each operation file. Other changes to operation files keep the identity,
    /// so that TypeScript does not transform every file again.
    ///
    /// Imports of operation files are taken from `previous_imports` if their contents are unchanged.
    fn compute_watched_files(&mut self, mut previous_imports: HashMap<PathBuf, OperationImports>) {
        // Files whose contents affect the identity.
        let mut watched: BTreeMap<PathBuf, Option<String>> = BTreeMap::new();
        for path in self.watched_files.drain(..) {
            let content = fs::read_to_string(&path).ok();
            watched.insert(path, content);
        }
        for (path, idx) in self.schema_files.iter() {
            let (_, content, _) = &self.file_store()[*idx];
            watched.insert(path.clone(), Some(content.to_string()));
        }
        if !self.config.operations.is_empty() {
            match load_glob_files(&self.root_dir, &self.config.operations) {
                Ok(operation_files) => {
                    for (path, content) in operation_files {
                        let path = normalize_path(&path);
                        let file =
                            OperationImports::new(&path, content, previous_imports.remove(&path));
                        self.operation_imports.insert(path, file);
                    }
                    // Imported files may not be matched by the config, and may import other files.
                    let mut queue = self
                        .operation_imports
                        .values()
                        .flat_map(|file| file.imports.iter().cloned())
                        .collect::<Vec<_>>();
                    while let Some(path) = queue.pop() {
                        if watched.contains_key(&path) {
                            continue;
                        }
                        let content = match self.operation_imports.get(&path) {
                            Some(file) => Some(file.content.clone()),
                            None => fs::read_to_string(&path).ok().map(|content| {
                                let file = OperationImports::new(
                                    &path,
                                    content,
                                    previous_imports.remove(&path),
                                );
                                queue.extend(file.imports.iter().cloned());
                                let content = file.content.clone();
                                self.operation_imports.insert(path.clone(), file);
                                content
                            }),
                        };
                        watched.insert(path, content);
                    }
                }
                Err(err) => {
                    debug!("Failed to load operation files: {err}");
                }
            }
        }

        let mut hasher = Fnv1a::new();
        hasher.write(format!("{:?}", self.config).as_bytes());
        for (path, content) in watched.iter() {
            hasher.write(path.to_string_lossy().as_bytes());
            hasher.write(&[0]);
            if let Some(content) = content {
                hasher.write(content.as_bytes());
            }
            hasher.write(&[0]);
        }
        hasher.write(&[1]);
        // Files imported from each operation file, in a stable order.
        let imports = self
            .operation_imports
            .iter()
            .map(|(path, file)| (path, &file.imports))
            .collect::<BTreeMap<_, _>>();
        for (path, imported) in imports {
            hasher.write(path.to_string_lossy().as_bytes());
            for imported in imported {
                hasher.write(&[0]);
                hasher.write(imported.to_string_lossy().as_bytes());
            }
            hasher.write(&[1]);
        }
        self.config_identity = format!("{:016x}", hasher.finish());
        let mut watched_files = watched.into_keys().collect::<BTreeSet<_>>();
        watched_files.extend(self.operation_imports.keys().cloned());
        self.watched_files = watched_files.into_iter().collect();
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

/// Parses schema files and resolves the schema.
fn parse_schema<'src>(
    schema_sources: Vec<Result<SchemaSource, PositionedError>>,
    file_store: &mut FileStore<'src>,
    plugins: &[Plugin],
) -> Result<LoadedProjectSchema<'src>, Vec<PositionedError>> {
    let schema =
        parse_schema_files(schema_sources, file_store, plugins).map_err(|err| err.inner)?;
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

/// Returns files imported from given operation file.
/// Files that cannot be parsed are treated as importing nothing.
fn imported_files(path: &Path, content: &str) -> Vec<PathBuf> {
    let mut operations = OperationSet::new();
    // file index is not used, as positions are not reported from here.
    operations.add_file(path, content, usize::MAX);
    operations
        .files()
        .flat_map(|file| file.imports().map(|(path, _)| path))
        .collect()
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
