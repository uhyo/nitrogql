use log::{debug, info};

use graphql_builtins::generate_builtins;
use nitrogql_checker::{
    operation_checker::check_operation_document, type_system_checker::check_type_system_document,
};
use nitrogql_error::{print_positioned_error, Result};
use nitrogql_extension_resolver::resolve_extensions;

use super::{error::CliError, CliContext};

pub fn run_check(context: CliContext) -> Result<CliContext> {
    debug!("Checking");
    match context {
        CliContext::SchemaUnresolved {
            mut schema,
            operations,
            file_by_index,
            config,
        } => {
            schema.extend(generate_builtins());
            let resolved = resolve_extensions(schema)?;
            let errors = check_type_system_document(&resolved);
            if !errors.is_empty() {
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
            let errors = operations
                .iter()
                .flat_map(|(_, doc, file_by_index)| {
                    check_operation_document(&resolved, doc)
                        .into_iter()
                        .map(move |err| (err, file_by_index))
                })
                .collect::<Vec<_>>();
            if errors.is_empty() {
                info!("Check succeeded");
                eprintln!("'check' finished");
            } else {
                eprintln!(
                    "Found {} error{} in operations:\n",
                    errors.len(),
                    if errors.len() > 1 { "s" } else { "" }
                );
                for (err, file_by_index) in errors {
                    eprintln!("{}", print_positioned_error(&err.into(), file_by_index));
                }
                eprintln!("");
                return Err(CliError::CommandNotSuccessful("check".into()).into());
            }

            Ok(CliContext::SchemaResolved {
                schema: resolved,
                operations,
                file_by_index,
                config,
            })
        }
        _ => Err(CliError::InvalidCommand(
            "'check' command cannot be called after another command".into(),
        )
        .into()),
    }
}
