use std::{borrow::Cow, path::PathBuf, str::FromStr};

use graphql_type_system::Schema;
use nitrogql_ast::{
    base::Pos,
    operation::OperationDocument,
    type_system::{TypeSystemDocument, TypeSystemOrExtensionDocument},
};
use nitrogql_config_file::Config;
use thiserror::Error;

use crate::{file_store::FileStore, output::CliOutput};

#[allow(clippy::large_enum_variant)]
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
        operations: Vec<(PathBuf, OperationDocument<'src>, usize)>,
        file_store: &'src FileStore,
        output: &'src mut CliOutput,
    },
    SchemaResolved {
        config: CliConfig,
        schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
        operations: Vec<(PathBuf, OperationDocument<'src>, usize)>,
        file_store: &'src FileStore,
        output: &'src mut CliOutput,
    },
}

#[derive(Debug)]
pub struct CliConfig {
    /// Root directory for other paths.
    pub root_dir: PathBuf,
    pub config: Config,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    Human,
    Json,
    Rdjson,
}

#[derive(Debug, Error)]
#[error("invalid output format {0}")]
pub struct FromStrError(String);

impl FromStr for OutputFormat {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "rdjson" => Ok(OutputFormat::Rdjson),
            s => Err(FromStrError(s.to_owned())),
        }
    }
}
