use anyhow::Result;

use crate::extension_resolver::resolve_extensions;

use super::{error::CliError, CliContext};

pub fn run_check(context: CliContext) -> Result<CliContext> {
    match context {
        CliContext::SchemaUnresolved { schema } => {
            let resolved = resolve_extensions(schema)?;
            Ok(CliContext::SchemaResolved { schema: resolved })
        }
        _ => Err(CliError::InvalidCommand(
            "'check' command cannot be called after another command".into(),
        )
        .into()),
    }
}
