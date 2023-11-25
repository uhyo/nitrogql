use std::{borrow::Cow, path::PathBuf};

use itertools::Itertools;
use log::{debug, info};

use nitrogql_ast::{
    OperationDocument, OperationDocumentExt, TypeSystemDocument, TypeSystemOrExtensionDocument,
};
use nitrogql_checker::{
    check_operation_document, check_type_system_document, CheckError, CheckErrorMessage,
    OperationCheckContext,
};
use nitrogql_error::{PositionedError, Result};
use nitrogql_plugin::Plugin;
use nitrogql_semantics::{
    ast_to_type_system, resolve_operation_extensions, resolve_schema_extensions, OperationExtension,
};

use crate::{output::InputFileKind, schema_loader::LoadedSchema};

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
            let result = check_impl(CheckImplInput {
                schema,
                operations,
                plugins: &config.plugins,
            })?;
            match result {
                CheckImplOutput::Ok { schema, operations } => {
                    info!("Check succeeded");
                    eprintln!("'check' finished");
                    Ok(CliContext::SchemaResolved {
                        schema,
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
    pub operations: Vec<(PathBuf, OperationDocumentExt<'src>, usize)>,
    pub plugins: &'a [Plugin<'src>],
}

enum CheckImplOutput<'src> {
    Ok {
        schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
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

fn check_impl<'src>(input: CheckImplInput<'src, '_>) -> Result<CheckImplOutput<'src>> {
    let CheckImplInput {
        schema,
        operations,
        plugins,
    } = input;

    let loaded_schema = {
        match schema {
            LoadedSchema::GraphQL(document) => {
                let resolved = resolve_schema_extensions(document)?;
                let mut errors = check_type_system_document(&resolved);
                // If basic schema check fails, we don't need to check with plugins.
                if errors.is_empty() {
                    // check schema with plugins
                    for plugin in plugins {
                        errors.extend(plugin.check_schema(&resolved).errors.into_iter().map(
                            |error| {
                                CheckError {
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
                                }
                            },
                        ));
                    }
                }

                if !errors.is_empty() {
                    return Ok(CheckImplOutput::Err {
                        errors: errors
                            .into_iter()
                            .map(|err| (InputFileKind::Schema, err.into()))
                            .collect(),
                    });
                }
                LoadedSchema::GraphQL(resolved)
            }
            LoadedSchema::Introspection(schema) => LoadedSchema::Introspection(schema),
        }
    };
    let schema = loaded_schema.map_into(|doc| Cow::Owned(ast_to_type_system(doc)), Cow::Borrowed);

    let (operations, resolve_errors): (Vec<_>, Vec<_>) = operations
        .into_iter()
        .map(|(path, doc, file_by_index)| -> std::result::Result<_, _> {
            let (doc, ext) = resolve_operation_extensions(doc)?;
            Ok((path, doc, ext, file_by_index))
        })
        .partition_result();

    if !resolve_errors.is_empty() {
        return Ok(CheckImplOutput::Err {
            errors: resolve_errors
                .into_iter()
                .map(|err| (InputFileKind::Operation, err))
                .collect(),
        });
    }

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
        Ok(CheckImplOutput::Err {
            errors: errors
                .into_iter()
                .map(|(err, _)| (InputFileKind::Operation, err.into()))
                .collect(),
        })
    } else {
        Ok(CheckImplOutput::Ok {
            schema: loaded_schema,
            operations,
        })
    }
}
