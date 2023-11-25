use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

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
    ast_to_type_system, resolve_operation_extensions, resolve_operation_imports,
    resolve_schema_extensions, OperationExtension, OperationResolver,
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
            });
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
            schema: loaded_schema,
            operations,
        }
    }
}

fn resolve_schema<'src>(
    schema: LoadedSchema<'src, TypeSystemOrExtensionDocument<'src>>,
    plugins: &[Plugin<'src>],
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

fn resolve_operations(
    operations: Vec<(PathBuf, OperationDocumentExt, usize)>,
) -> ResolveOperationsResult {
    let (operations, resolve_errors): (Vec<_>, Vec<_>) = operations
        .into_iter()
        .map(|(path, doc, file_by_index)| -> std::result::Result<_, _> {
            let (doc, ext) = resolve_operation_extensions(doc)?;
            // resolve_operation_imports((&path, &doc, &ext), operation_resolver);
            Ok((path, doc, ext, file_by_index))
        })
        .partition_result();
    if !resolve_errors.is_empty() {
        return Err(resolve_errors);
    }

    let operation_resolver = Operations::new(&operations);
    let (operations, resolve_errors): (Vec<_>, Vec<_>) = operations
        .iter()
        .map(
            |(path, doc, ext, file_by_index)| -> std::result::Result<_, _> {
                let doc = resolve_operation_imports((&path, &doc, &ext), &operation_resolver)?;
                Ok((path.clone(), doc, ext.clone(), *file_by_index))
            },
        )
        .partition_result();
    if !resolve_errors.is_empty() {
        return Err(resolve_errors);
    }
    Ok(operations)
}

struct Operations<'a, 'src> {
    file_by_path: HashMap<&'a Path, (&'a OperationDocument<'src>, &'a OperationExtension<'src>)>,
}

impl<'a, 'src> Operations<'a, 'src> {
    pub fn new(
        operations: &'a [(
            PathBuf,
            OperationDocument<'src>,
            OperationExtension<'src>,
            usize,
        )],
    ) -> Self {
        let file_by_path = operations
            .iter()
            .map(|(path, doc, ext, _)| (path.as_path(), (doc, ext)))
            .collect();
        Self { file_by_path }
    }
}

impl<'a, 'src> OperationResolver<'src> for Operations<'a, 'src> {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(&OperationDocument<'src>, &OperationExtension<'src>)> {
        self.file_by_path.get(path).copied()
    }
}
