use std::borrow::Cow;

use log::{debug, info};

use graphql_builtins::generate_builtins;
use nitrogql_checker::{check_operation_document, check_type_system_document};
use nitrogql_error::Result;
use nitrogql_semantics::{ast_to_type_system, resolve_extensions};

use crate::{context::LoadedSchema, output::InputFileKind};

use super::{error::CliError, CliContext};

pub fn run_check(context: CliContext) -> Result<CliContext> {
    debug!("Checking");
    match context {
        CliContext::SchemaUnresolved {
            schema,
            operations,
            file_store,
            config,
            output,
        } => {
            output.command_run("check".to_owned());
            let loaded_schema = {
                match schema {
                    LoadedSchema::GraphQL(mut document) => {
                        document.extend(generate_builtins());
                        let resolved = resolve_extensions(document)?;
                        let errors = check_type_system_document(&resolved);

                        if !errors.is_empty() {
                            output
                                .extend(errors.into_iter().map(|err| (InputFileKind::Schema, err)));
                            return Err(CliError::CommandNotSuccessful("check".into()).into());
                        }
                        LoadedSchema::GraphQL(resolved)
                    }
                    LoadedSchema::Introspection(schema) => LoadedSchema::Introspection(schema),
                }
            };
            let schema =
                loaded_schema.map_into(|doc| Cow::Owned(ast_to_type_system(doc)), Cow::Borrowed);
            let errors = operations
                .iter()
                .flat_map(|(_, doc, file_by_index)| {
                    check_operation_document(&schema, doc)
                        .into_iter()
                        .map(move |err| (err, file_by_index))
                })
                .collect::<Vec<_>>();
            if errors.is_empty() {
                info!("Check succeeded");
                eprintln!("'check' finished");
            } else {
                output.extend(
                    errors
                        .into_iter()
                        .map(|(err, _)| (InputFileKind::Operation, err)),
                );
                return Err(CliError::CommandNotSuccessful("check".into()).into());
            }

            Ok(CliContext::SchemaResolved {
                schema: loaded_schema,
                operations,
                file_store,
                config,
                output,
            })
        }
        _ => Err(CliError::InvalidCommand(
            "'check' command cannot be called after another command".into(),
        )
        .into()),
    }
}
