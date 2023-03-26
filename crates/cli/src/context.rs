use std::{borrow::Cow, path::PathBuf};

use graphql_type_system::Schema;
use nitrogql_ast::{
    base::Pos,
    operation::OperationDocument,
    type_system::{TypeSystemDocument, TypeSystemOrExtensionDocument},
};
use nitrogql_config_file::Config;

/// List of (path, source)
pub type FileByIndex<'src> = Vec<(PathBuf, &'src str)>;

pub enum LoadedSchema<'src, Gql> {
    GraphQL(Gql),
    Introspection(Schema<Cow<'src, str>, Pos>),
}

impl<'src, Gql> LoadedSchema<'src, Gql> {
    pub fn map_into<'a, F, G, R>(&'a self, graphql: F, introspection: G) -> R
    where
        F: FnOnce(&'a Gql) -> R,
        G: FnOnce(&'a Schema<Cow<'src, str>, Pos>) -> R,
    {
        match self {
            LoadedSchema::GraphQL(gql) => graphql(gql),
            LoadedSchema::Introspection(schema) => introspection(schema),
        }
    }
}

pub enum CliContext<'src> {
    SchemaUnresolved {
        config: CliConfig,
        schema: LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
        operations: Vec<(PathBuf, OperationDocument<'src>, FileByIndex<'src>)>,
        /// List of (path, source)
        file_by_index: Vec<(PathBuf, &'src str)>,
    },
    SchemaResolved {
        config: CliConfig,
        schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
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
    pub config: Config,
}
