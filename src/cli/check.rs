use log::{debug, info};

use crate::{
    error::{print_positioned_error, Result},
    extension_resolver::resolve_extensions,
    type_system_validator::check_type_system_document,
};

use super::{error::CliError, CliContext};

pub fn run_check(context: CliContext) -> Result<CliContext> {
    debug!("Checking");
    match context {
        CliContext::SchemaUnresolved {
            schema,
            file_by_index,
        } => {
            let resolved = resolve_extensions(schema)?;
            let errors = check_type_system_document(&resolved);
            if errors.is_empty() {
                info!("Check succeeded");
                eprintln!("'check' succeeded without errors");
            } else {
                eprintln!(
                    "Found {} error{} in schema:\n",
                    errors.len(),
                    if errors.len() > 1 { "s" } else { "" }
                );
                for err in errors {
                    eprintln!("{}", print_positioned_error(&err.into(), &file_by_index));
                }
                eprintln!("");
                return Err(CliError::CommandNotSuccessful("check".into()).into());
            }
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
