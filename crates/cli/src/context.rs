use std::{path::PathBuf, str::FromStr};

use nitrogql_ast::{
    operation::OperationDocument,
    type_system::{TypeSystemDocument, TypeSystemOrExtensionDocument},
    OperationDocumentExt,
};
use nitrogql_config_file::Config;
use nitrogql_plugin::Plugin;
use nitrogql_semantics::OperationExtension;
use thiserror::Error;

use crate::{file_store::FileStore, output::CliOutput, schema_loader::LoadedSchema};

pub enum CliContext<'src> {
    SchemaUnresolved {
        config: CliConfig<'src>,
        schema: LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
        operations: Vec<(PathBuf, OperationDocumentExt<'src>, usize)>,
        file_store: &'src mut FileStore,
        output: &'src mut CliOutput,
    },
    SchemaResolved {
        config: CliConfig<'src>,
        schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
        operations: Vec<(
            PathBuf,
            OperationDocument<'src>,
            OperationExtension<'src>,
            usize,
        )>,
        file_store: &'src FileStore,
        output: &'src mut CliOutput,
    },
}

#[derive(Debug)]
pub struct CliConfig<'file_store> {
    /// Root directory for other paths.
    pub root_dir: PathBuf,
    pub config: Config,
    /// Loaded plugins.
    pub plugins: Vec<Plugin<'file_store>>,
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
