use std::path::PathBuf;

use crate::ast::{OperationDocument, TypeSystemDocument, TypeSystemOrExtensionDocument};

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

pub struct CliConfig {
    /// Root directory for other paths.
    pub root_dir: PathBuf,
    pub schema_output: Option<PathBuf>,
}
