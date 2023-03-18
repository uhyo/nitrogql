use std::path::PathBuf;

use nitrogql_ast::{
    operation::OperationDocument,
    type_system::{TypeSystemDocument, TypeSystemOrExtensionDocument},
};
use nitrogql_config_file::GenerateConfig;

/// List of (path, source)
pub type FileByIndex<'src> = Vec<(PathBuf, &'src str)>;

pub enum CliContext<'src> {
    SchemaUnresolved {
        config: CliConfig,
        schema: TypeSystemOrExtensionDocument<'src>,
        operations: Vec<(PathBuf, OperationDocument<'src>, FileByIndex<'src>)>,
        /// List of (path, source)
        file_by_index: Vec<(PathBuf, &'src str)>,
    },
    SchemaResolved {
        config: CliConfig,
        schema: TypeSystemDocument<'src>,
        operations: Vec<(PathBuf, OperationDocument<'src>, FileByIndex<'src>)>,
        file_by_index: FileByIndex<'src>,
    },
}

impl<'src> CliContext<'src> {
    pub fn file_by_index(&self) -> Vec<(PathBuf, &'src str)> {
        match self {
            CliContext::SchemaUnresolved { file_by_index, .. }
            | CliContext::SchemaResolved { file_by_index, .. } => file_by_index.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CliConfig {
    /// Root directory for other paths.
    pub root_dir: PathBuf,
    pub generate_config: GenerateConfig,
}
