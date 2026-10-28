use std::{borrow::Cow, collections::HashSet, path::PathBuf};

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

    // Files whose imports cannot be resolved are not validated, but other files still are.
    let (operations, visible_fragments, resolve_errors) = resolve_operations(operations);

    // Fragments are checked in every document that imports them, so errors may be duplicated.
    // Import errors may be duplicated too, when multiple files depend on the same broken import.
    let mut seen_errors = HashSet::new();
    let check_errors = operations
        .iter()
        .zip(visible_fragments.iter())
        .flat_map(|((_, doc, _, _), visible_fragments)| {
            // Each file can only spread fragments it imports by itself,
            // even though transitively imported fragments are appended to the document.
            let is_visible = |name: &str| visible_fragments.contains(name);
            let context = OperationCheckContext::new(&schema).with_visible_fragments(&is_visible);
            check_operation_document(doc, &context)
        })
        .map(PositionedError::from);
    let errors = resolve_errors
        .into_iter()
        .chain(check_errors)
        .filter(|err| seen_errors.insert((err.position(), err.inner().to_string())))
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        CheckImplOutput::Err {
            errors: errors
                .into_iter()
                .map(|err| (InputFileKind::Operation, err))
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

/// Resolved operations, along with names of fragments directly imported by each of them,
/// and errors found while loading or resolving them.
/// Files whose imports cannot be resolved are left out.
type ResolveOperationsResult<'src> = (
    Vec<(
        PathBuf,
        OperationDocument<'src>,
        OperationExtension<'src>,
        usize,
    )>,
    Vec<HashSet<&'src str>>,
    Vec<PositionedError>,
);

fn resolve_operations(mut operations: OperationSet<'_>) -> ResolveOperationsResult<'_> {
    let mut errors = operations.take_errors();
    let mut resolved = vec![];
    let mut visible_fragments = vec![];
    for file in operations.files() {
        let doc = match operations
            .resolve_imports(file.path)
            .expect("file should be loaded")
        {
            Ok(doc) => doc,
            Err(err) => {
                // A file that failed to load has its own error already,
                // so reporting it as not found would only be confusing.
                if !operations.imports_unloaded_file(file.path) {
                    errors.push(err);
                }
                continue;
            }
        };
        resolved.push((
            file.path.to_owned(),
            doc,
            file.extension.clone(),
            file.file_index,
        ));
        visible_fragments.push(
            operations
                .directly_imported_fragments(file.path)
                .expect("file should be loaded"),
        );
    }
    (resolved, visible_fragments, errors)
}
