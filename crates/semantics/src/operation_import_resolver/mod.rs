use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use log::debug;
use nitrogql_ast::{base::Pos, operation::ExecutableDefinition, OperationDocument};
use nitrogql_error::PositionedError;
use nitrogql_utils::resolve_relative_path;
use thiserror::Error;

use crate::{ImportTargets, OperationExtension};

pub use self::path_resolver::OperationResolver;

mod path_resolver;
#[cfg(test)]
mod tests;

/// Resolves operation imports in the given document.
/// Currently, only Fragments can be imported and
/// imported fragments are appended to the document.
pub fn resolve_operation_imports<'src>(
    document: (&Path, &OperationDocument<'src>, &OperationExtension<'src>),
    operation_resolver: &impl OperationResolver<'src>,
) -> Result<OperationDocument<'src>, ExtensionError> {
    let mut definitions: Vec<_> = document.1.definitions.to_vec();
    let mut visited = HashSet::new();
    resolve_operation_imports_rec(document, operation_resolver, &mut visited, &mut definitions)?;
    let result = OperationDocument {
        definitions,
        position: document.1.position,
    };
    Ok(result)
}

fn resolve_operation_imports_rec<'src>(
    document: (&Path, &OperationDocument<'src>, &OperationExtension<'src>),
    operation_resolver: &impl OperationResolver<'src>,
    visited: &mut HashSet<PathBuf>,
    definitions: &mut Vec<ExecutableDefinition<'src>>,
) -> Result<(), ExtensionError> {
    let (document_path, _, extensions) = document;
    for import in extensions.imports.iter() {
        let imported_path = resolve_relative_path(document_path, Path::new(&import.path.value));
        if visited.contains(&imported_path) {
            continue;
        }
        let Some(imported_op) = operation_resolver.resolve(&imported_path) else {
            return Err(ExtensionError {
                message: ExtensionErrorMessage::FileNotFound {
                    file: import.path.value.clone(),
                    position: import.path.position,
                },
            });
        };
        visited.insert(imported_path.clone());
        resolve_operation_imports_rec(
            (&imported_path, imported_op.0, imported_op.1),
            operation_resolver,
            visited,
            definitions,
        )?;
        match import.targets {
            ImportTargets::Wildcard => {
                definitions.extend(
                    imported_op
                        .0
                        .definitions
                        .iter()
                        .filter(|def| matches!(def, ExecutableDefinition::FragmentDefinition(_)))
                        .cloned(),
                );
            }
            ImportTargets::Specific(ref targets) => {
                let mut count = 0;
                let target_fragments = imported_op
                    .0
                    .definitions
                    .iter()
                    .filter(|def| match def {
                        ExecutableDefinition::FragmentDefinition(def) => {
                            debug!("targets: {:?} def.name.name: {}", targets, def.name.name);
                            debug!("def: {:?}", def);
                            targets.iter().any(|target| target.name == def.name.name)
                        }
                        _ => false,
                    })
                    .cloned();
                for target in target_fragments {
                    definitions.push(target);
                    count += 1;
                }
                if count < targets.len() {
                    // figure out which targets are missing (first one)
                    let missing_target = targets
                        .iter()
                        .find(|target| {
                            !imported_op.0.definitions.iter().any(|def| match def {
                                ExecutableDefinition::FragmentDefinition(def) => {
                                    target.name == def.name.name
                                }
                                _ => false,
                            })
                        })
                        .expect("missing target not found");
                    return Err(ExtensionError {
                        message: ExtensionErrorMessage::FragmentNotFound {
                            name: missing_target.name.to_string(),
                            file: import.path.value.clone(),
                            position: missing_target.position,
                        },
                    });
                }
            }
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ExtensionErrorMessage {
    #[error("File '{file}' not found.")]
    FileNotFound { file: String, position: Pos },
    #[error("'{name}' is not found in the imported file '{file}'.")]
    FragmentNotFound {
        name: String,
        file: String,
        position: Pos,
    },
}

#[derive(Debug)]
pub struct ExtensionError {
    pub message: ExtensionErrorMessage,
}

impl From<ExtensionError> for PositionedError {
    fn from(value: ExtensionError) -> Self {
        let position = match &value.message {
            ExtensionErrorMessage::FileNotFound { position, .. } => *position,
            ExtensionErrorMessage::FragmentNotFound { position, .. } => *position,
        };
        let additional_info = match &value.message {
            ExtensionErrorMessage::FileNotFound { .. } => vec![
                (
                    Pos::builtin(),
                    "Hint: imported file must be included in the 'documents' option of the config file.".into(),
                )
            ],
            _ => vec![],
        };

        PositionedError::new(value.message.into(), Some(position), additional_info)
    }
}
