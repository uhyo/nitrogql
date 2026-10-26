use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use log::debug;
use nitrogql_ast::{OperationDocument, base::Pos, operation::ExecutableDefinition};
use nitrogql_error::{ErrorCode, PositionedError};
use nitrogql_utils::{normalize_path, resolve_relative_path};
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
    // The document itself may be imported back through a cycle of imports.
    let document_path = normalize_path(document.0);
    let mut appended: HashSet<_> = document
        .1
        .definitions
        .iter()
        .filter_map(|def| match def {
            ExecutableDefinition::FragmentDefinition(def) => {
                Some((document_path.clone(), def.name.name))
            }
            ExecutableDefinition::OperationDefinition(_) => None,
        })
        .collect();
    let mut visited = HashSet::from([document_path]);
    resolve_operation_imports_rec(
        document,
        operation_resolver,
        &mut visited,
        &mut appended,
        &mut definitions,
    )?;
    let result = OperationDocument {
        definitions,
        position: document.1.position,
    };
    Ok(result)
}

/// `visited` is the set of files whose imports have been resolved.
/// `appended` is the set of fragments (file and name) that have been appended to `definitions`.
/// A file may be imported multiple times with different targets,
/// so targets of every import are appended even if the file has been visited.
fn resolve_operation_imports_rec<'src>(
    document: (&Path, &OperationDocument<'src>, &OperationExtension<'src>),
    operation_resolver: &impl OperationResolver<'src>,
    visited: &mut HashSet<PathBuf>,
    appended: &mut HashSet<(PathBuf, &'src str)>,
    definitions: &mut Vec<ExecutableDefinition<'src>>,
) -> Result<(), ExtensionError> {
    let (document_path, _, extensions) = document;
    for import in extensions.imports.iter() {
        let imported_path = resolve_relative_path(document_path, Path::new(&import.path.value));
        let Some(imported_op) = operation_resolver.resolve(&imported_path) else {
            return Err(ExtensionError {
                message: ExtensionErrorMessage::FileNotFound {
                    file: import.path.value.clone(),
                    position: import.path.position,
                },
            });
        };
        if visited.insert(imported_path.clone()) {
            resolve_operation_imports_rec(
                (&imported_path, imported_op.0, imported_op.1),
                operation_resolver,
                visited,
                appended,
                definitions,
            )?;
        }
        let imported_fragments = imported_op
            .0
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::FragmentDefinition(def) => Some(def),
                ExecutableDefinition::OperationDefinition(_) => None,
            });
        let target_fragments: Vec<_> = match import.targets {
            ImportTargets::Wildcard => imported_fragments.collect(),
            ImportTargets::Specific(ref targets) => {
                // check that all targets exist (report the first missing one)
                let missing_target = targets.iter().find(|target| {
                    !imported_op.0.definitions.iter().any(|def| match def {
                        ExecutableDefinition::FragmentDefinition(def) => {
                            target.name == def.name.name
                        }
                        _ => false,
                    })
                });
                if let Some(missing_target) = missing_target {
                    return Err(ExtensionError {
                        message: ExtensionErrorMessage::FragmentNotFound {
                            name: missing_target.name.to_string(),
                            file: import.path.value.clone(),
                            position: missing_target.position,
                        },
                    });
                }
                imported_fragments
                    .filter(|def| {
                        debug!("targets: {:?} def.name.name: {}", targets, def.name.name);
                        targets.iter().any(|target| target.name == def.name.name)
                    })
                    .collect()
            }
        };
        for target in target_fragments {
            // the same fragment may be imported through multiple paths
            if appended.insert((imported_path.clone(), target.name.name)) {
                definitions.push(ExecutableDefinition::FragmentDefinition(target.clone()));
            }
        }
    }
    Ok(())
}

/// Returns names of fragments that given document imports by itself:
/// ones named in specific imports, and ones defined in files imported with wildcard imports.
/// Fragments that imported files import in turn are not included.
/// Imports that cannot be resolved are ignored; they are reported by `resolve_operation_imports`.
pub fn directly_imported_fragments<'src>(
    document: (&Path, &OperationExtension<'src>),
    operation_resolver: &impl OperationResolver<'src>,
) -> HashSet<&'src str> {
    let (document_path, extensions) = document;
    let mut result = HashSet::new();
    for import in extensions.imports.iter() {
        match import.targets {
            ImportTargets::Wildcard => {
                let imported_path =
                    resolve_relative_path(document_path, Path::new(&import.path.value));
                let Some((imported_doc, _)) = operation_resolver.resolve(&imported_path) else {
                    continue;
                };
                result.extend(imported_doc.definitions.iter().filter_map(|def| match def {
                    ExecutableDefinition::FragmentDefinition(def) => Some(def.name.name),
                    ExecutableDefinition::OperationDefinition(_) => None,
                }));
            }
            ImportTargets::Specific(ref targets) => {
                result.extend(targets.iter().map(|target| target.name));
            }
        }
    }
    result
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

        let code = match &value.message {
            ExtensionErrorMessage::FileNotFound { .. } => ErrorCode::ImportFileNotFound,
            ExtensionErrorMessage::FragmentNotFound { .. } => ErrorCode::ImportedFragmentNotFound,
        };

        PositionedError::new(value.message.into(), Some(position), additional_info).with_code(code)
    }
}
