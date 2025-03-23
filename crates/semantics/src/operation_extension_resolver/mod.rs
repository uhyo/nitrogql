use nitrogql_ast::{
    OperationDocument, OperationDocumentExt,
    base::Pos,
    operation::ExecutableDefinition,
    operation_ext::{ExecutableDefinitionExt, ImportTarget},
};
use nitrogql_error::PositionedError;
use thiserror::Error;

use self::operation_extension::{Import, ImportTargets, OperationExtension};

pub mod operation_extension;

pub fn resolve_operation_extensions(
    document: OperationDocumentExt,
) -> Result<(OperationDocument, OperationExtension), ExtensionError> {
    let mut definitions = vec![];
    let mut imports = vec![];
    for def in document.definitions {
        match def {
            ExecutableDefinitionExt::OperationDefinition(def) => {
                definitions.push(ExecutableDefinition::OperationDefinition(def));
            }
            ExecutableDefinitionExt::FragmentDefinition(def) => {
                definitions.push(ExecutableDefinition::FragmentDefinition(def));
            }
            ExecutableDefinitionExt::Import(import) => {
                let existing = imports
                    .iter()
                    .position(|i: &Import| i.path.value == import.path.value);
                let initial = if let Some(existing) = existing {
                    imports.remove(existing).targets
                } else {
                    ImportTargets::Specific(vec![])
                };

                let targets = import.targets.into_iter().try_fold(
                    initial,
                    |acc, target| {
                        match (acc, target) {
                            (ImportTargets::Wildcard, ImportTarget::Wildcard) => {
                                Err(ExtensionError {
                                    message: ExtensionErrorMessage::WildcardOnlyOnce {
                                        pos: import.position,
                                    },
                                })
                            }
                            (ImportTargets::Wildcard, ImportTarget::Name(_)) => {
                                Err(ExtensionError {
                                    message: ExtensionErrorMessage::WildcardCannotBeCombinedWithSpecific {
                                        pos: import.position,
                                    },
                                })
                            }
                            (ImportTargets::Specific(idents), ImportTarget::Wildcard) => {
                                if idents.is_empty() {
                                    Ok(ImportTargets::Wildcard)
                                } else {
                                    Err(ExtensionError {
                                        message: ExtensionErrorMessage::WildcardCannotBeCombinedWithSpecific {
                                            pos: import.position,
                                        },
                                    })
                                }
                            }
                            (ImportTargets::Specific(mut idents), ImportTarget::Name(ident)) => {
                                idents.push(ident);
                                Ok(ImportTargets::Specific(idents))
                            }
                        }
                    },
                );
                imports.push(Import {
                    path: import.path,
                    targets: targets?,
                });
            }
        }
    }
    Ok((
        OperationDocument {
            definitions,
            position: document.position,
        },
        OperationExtension { imports },
    ))
}

#[derive(Error, Debug)]
pub enum ExtensionErrorMessage {
    #[error("Wildcard import should be specified only once")]
    WildcardOnlyOnce { pos: Pos },
    #[error("Wildcard import cannot be combined with specific import")]
    WildcardCannotBeCombinedWithSpecific { pos: Pos },
}

#[derive(Debug)]
pub struct ExtensionError {
    pub message: ExtensionErrorMessage,
}

impl From<ExtensionError> for PositionedError {
    fn from(value: ExtensionError) -> Self {
        let position = match &value.message {
            ExtensionErrorMessage::WildcardOnlyOnce { pos } => *pos,
            ExtensionErrorMessage::WildcardCannotBeCombinedWithSpecific { pos } => *pos,
        };
        let additional_info = match &value.message {
            ExtensionErrorMessage::WildcardOnlyOnce { .. } => vec![],
            ExtensionErrorMessage::WildcardCannotBeCombinedWithSpecific { .. } => vec![],
        };

        PositionedError::new(value.message.into(), Some(position), additional_info)
    }
}
