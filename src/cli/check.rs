use log::{debug, info};

use crate::{error::Result, extension_resolver::resolve_extensions};

use super::{error::CliError, CliContext};

pub fn run_check(context: CliContext) -> Result<CliContext> {
    debug!("Checking");
    match context {
        CliContext::SchemaUnresolved {
            schema,
            file_by_index,
        } => {
            let resolved = resolve_extensions(schema)?;
            info!("Check succeeded");
            Ok(CliContext::SchemaResolved {
                schema: resolved,
                file_by_index,
            })
        }
        _ => Err(CliError::InvalidCommand(
            "'check' command cannot be called after another command".into(),
        )
        .into()),
    }
}
