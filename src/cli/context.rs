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
