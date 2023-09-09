use std::{borrow::Cow, path::PathBuf};

use log::{debug, info};

use graphql_builtins::generate_builtins;
use nitrogql_ast::{OperationDocument, TypeSystemDocument, TypeSystemOrExtensionDocument};
use nitrogql_checker::{
    check_operation_document, check_type_system_document, CheckError, CheckErrorMessage,
};
use nitrogql_error::Result;
use nitrogql_plugin::Plugin;
use nitrogql_semantics::{ast_to_type_system, resolve_extensions};

use crate::{
    context::LoadedSchema, file_store::FileStore, output::InputFileKind, plugin_host::PluginHost,
};

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
                operations: &operations,
                plugins: &config.plugins,
                file_store,
            })?;
            match result {
                CheckImplOutput::Ok { schema, file_store } => {
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
    pub operations: &'a [(PathBuf, OperationDocument<'src>, usize)],
    pub plugins: &'a [Plugin<'src>],
    pub file_store: &'src mut FileStore,
}

enum CheckImplOutput<'src> {
    Ok {
        schema: LoadedSchema<'src, TypeSystemDocument<'src>>,
        file_store: &'src mut FileStore,
    },
    Err {
        errors: Vec<(InputFileKind, CheckError)>,
    },
}

fn check_impl<'src>(input: CheckImplInput<'src, '_>) -> Result<CheckImplOutput<'src>> {
    let CheckImplInput {
        schema,
        operations,
        plugins,
        file_store,
    } = input;

    let mut plugin_host = PluginHost::new(file_store);

    let loaded_schema =
        {
            match schema {
                LoadedSchema::GraphQL(mut document) => {
                    document.extend(generate_builtins());
                    // extend schema with plugins
                    for plugin in plugins {
                        if let Some(addition) = plugin.schema_addition(&mut plugin_host)? {
                            document.extend(addition.definitions);
                        }
                    }
                    let resolved = resolve_extensions(document)?;
                    let mut errors = check_type_system_document(&resolved);
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

                    if !errors.is_empty() {
                        return Ok(CheckImplOutput::Err {
                            errors: errors
                                .into_iter()
                                .map(|err| (InputFileKind::Schema, err))
                                .collect(),
                        });
                    }
                    LoadedSchema::GraphQL(resolved)
                }
                LoadedSchema::Introspection(schema) => LoadedSchema::Introspection(schema),
            }
        };
    let schema = loaded_schema.map_into(|doc| Cow::Owned(ast_to_type_system(doc)), Cow::Borrowed);
    let errors = operations
        .iter()
        .flat_map(|(_, doc, file_by_index)| {
            check_operation_document(&schema, doc)
                .into_iter()
                .map(move |err| (err, file_by_index))
        })
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        Ok(CheckImplOutput::Err {
            errors: errors
                .into_iter()
                .map(|(err, _)| (InputFileKind::Operation, err))
                .collect(),
        })
    } else {
        Ok(CheckImplOutput::Ok {
            schema: loaded_schema,
            file_store: plugin_host.file_store,
        })
    }
}
