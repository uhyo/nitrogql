use crate::graphql_parser::ast::{TypeSystemDocument, TypeSystemOrExtensionDocument};

pub enum CliContext<'src> {
    SchemaUnresolved {
        schema: TypeSystemOrExtensionDocument<'src>,
    },
    SchemaResolved {
        schema: TypeSystemDocument<'src>,
    },
}
