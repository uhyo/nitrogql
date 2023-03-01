use std::path::PathBuf;

use crate::graphql_parser::ast::{TypeSystemDocument, TypeSystemOrExtensionDocument};

pub enum CliContext<'src> {
    SchemaUnresolved {
        schema: TypeSystemOrExtensionDocument<'src>,
        file_by_index: Vec<(PathBuf, &'src str)>,
    },
    SchemaResolved {
        schema: TypeSystemDocument<'src>,
        file_by_index: Vec<(PathBuf, &'src str)>,
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
