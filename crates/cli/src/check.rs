use std::{borrow::Cow, path::PathBuf};

use itertools::Itertools;
use log::{debug, info};

use nitrogql_ast::{OperationDocument, TypeSystemDocument, TypeSystemOrExtensionDocument};
use nitrogql_checker::{
    CheckError, CheckErrorMessage, OperationCheckContext, check_operation_document,
    check_type_system_document,
};
use nitrogql_error::{PositionedError, Result};
use nitrogql_operation_set::OperationSet;
use nitrogql_plugin::Plugin;
use nitrogql_semantics::{OperationExtension, ast_to_type_system, resolve_schema_extensions};

use crate::{output::InputFileKind, schema_loader::LoadedSchema};

use super::{CliContext, error::CliError};

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
            let result = check_impl(CheckImplInput {
                schema,
                operations,
                plugins: &config.plugins,
            });
            match result {
                CheckImplOutput::Ok { schema, operations } => {
                    info!("Check succeeded");
                    eprintln!("'check' finished");
                    Ok(CliContext::SchemaResolved {
                        schema: *schema,
                        operations,
                        file_store,
                        config,
                        output,
                    })
                }
                CheckImplOutput::Err { errors } => {
                    output.extend(errors);
                    Err(CliError::CommandNotSuccessful("check".into()).into())
                }
            }
        }
        _ => Err(CliError::InvalidCommand(
            "'check' command cannot be called after another command".into(),
        )
        .into()),
    }
}

struct CheckImplInput<'src, 'a> {
    pub schema: LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
    pub operations: OperationSet<'src>,
    pub plugins: &'a [Plugin],
}

enum CheckImplOutput<'src> {
    Ok {
        schema: Box<LoadedSchema<'src, TypeSystemDocument<'src>>>,
        operations: Vec<(
            PathBuf,
            OperationDocument<'src>,
            OperationExtension<'src>,
            usize,
        )>,
    },
    Err {
        errors: Vec<(InputFileKind, PositionedError)>,
    },
}

fn check_impl<'src>(input: CheckImplInput<'src, '_>) -> CheckImplOutput<'src> {
    let CheckImplInput {
        schema,
        operations,
        plugins,
    } = input;

    let loaded_schema = match resolve_schema(schema, plugins) {
        Ok(schema) => schema,
        Err(errors) => {
            return CheckImplOutput::Err {
                errors: errors
                    .into_iter()
                    .map(|err| (InputFileKind::Schema, err))
                    .collect(),
            };
        }
    };
    let schema = loaded_schema.map_into(|doc| Cow::Owned(ast_to_type_system(doc)), Cow::Borrowed);

    let operations = match resolve_operations(operations) {
        Ok(operations) => operations,
        Err(errors) => {
            return CheckImplOutput::Err {
                errors: errors
                    .into_iter()
                    .map(|err| (InputFileKind::Operation, err))
                    .collect(),
            };
        }
    };

    let context = OperationCheckContext::new(&schema);
    let errors = operations
        .iter()
        .flat_map(|(_, doc, _, file_by_index)| {
            check_operation_document(doc, &context)
                .into_iter()
                .map(move |err| (err, file_by_index))
        })
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        CheckImplOutput::Err {
            errors: errors
                .into_iter()
                .map(|(err, _)| (InputFileKind::Operation, err.into()))
                .collect(),
        }
    } else {
        CheckImplOutput::Ok {
            schema: Box::new(loaded_schema),
            operations,
        }
    }
}

pub(crate) fn resolve_schema<'src>(
    schema: LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
    plugins: &[Plugin],
) -> std::result::Result<LoadedSchema<'src, TypeSystemDocument<'src>>, Vec<PositionedError>> {
    match schema {
        LoadedSchema::GraphQL(document) => {
            let resolved = resolve_schema_extensions(document).map_err(|err| vec![err.into()])?;
            let mut errors = check_type_system_document(&resolved);
            // If basic schema check fails, we don't need to check with plugins.
            if errors.is_empty() {
                // check schema with plugins
                for plugin in plugins {
                    errors.extend(
                        plugin
                            .check_schema(&resolved)
                            .errors
                            .into_iter()
                            .map(|error| CheckError {
                                position: error.position,
                                message: CheckErrorMessage::Plugin {
                                    message: error.message,
                                },
                                additional_info: error
                                    .additional_info
                                    .into_iter()
                                    .map(|(pos, message)| {
                                        (pos, CheckErrorMessage::Plugin { message })
                                    })
                                    .collect(),
                            }),
                    );
                }
            }

            if !errors.is_empty() {
                return Err(errors.into_iter().map(|err| err.into()).collect());
            }
            Ok(LoadedSchema::GraphQL(resolved))
        }
        LoadedSchema::Introspection(schema) => Ok(LoadedSchema::Introspection(schema)),
    }
}

type ResolveOperationsResult<'src> = std::result::Result<
    Vec<(
        PathBuf,
        OperationDocument<'src>,
        OperationExtension<'src>,
        usize,
    )>,
    Vec<PositionedError>,
>;

fn resolve_operations(mut operations: OperationSet<'_>) -> ResolveOperationsResult<'_> {
    let resolve_errors = operations.take_errors();
    if !resolve_errors.is_empty() {
        return Err(resolve_errors);
    }

    let (operations, resolve_errors): (Vec<_>, Vec<_>) = operations
        .files()
        .map(|file| -> std::result::Result<_, _> {
            let doc = operations
                .resolve_imports(file.path)
                .expect("file should be loaded")?;
            Ok((
                file.path.to_owned(),
                doc,
                file.extension.clone(),
                file.file_index,
            ))
        })
        .partition_result();
    if !resolve_errors.is_empty() {
        return Err(resolve_errors);
    }
    Ok(operations)
}
