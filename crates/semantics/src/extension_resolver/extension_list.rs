use std::collections::HashMap;

use thiserror::Error;

use nitrogql_ast::base::{HasPos, Pos};
use nitrogql_error::PositionedError;

pub struct ExtensionList<'a, OriginalType: HasPos, ExtensionType: HasPos> {
    name_of_elem: &'a str,
    items: HashMap<Option<String>, ExtensionItem<OriginalType, ExtensionType>>,
}

struct ExtensionItem<OriginalType, ExtensionType> {
    original: Option<OriginalType>,
    extensions: Vec<ExtensionType>,
}

impl<OriginalType, ExtensionType> Default for ExtensionItem<OriginalType, ExtensionType> {
    fn default() -> Self {
        ExtensionItem {
            original: None,
            extensions: vec![],
        }
    }
}

#[derive(Error, Debug)]
pub enum ExtensionErrorMessage {
    #[error("Duplicated declaration of {name_of_elem} '{name}'")]
    DuplicateOriginal {
        name_of_elem: String,
        name: String,
        first: Pos,
        second: Pos,
    },
    #[error("{name_of_elem} is extended, but there is no original declaration of {name_of_elem}")]
    NoOriginal {
        name_of_elem: String,
        first_extension: Pos,
    },
}

#[derive(Debug)]
pub struct ExtensionError {
    pub message: ExtensionErrorMessage,
}

impl From<ExtensionError> for PositionedError {
    fn from(value: ExtensionError) -> Self {
        let position = match &value.message {
            ExtensionErrorMessage::DuplicateOriginal { first, .. } => *first,
            ExtensionErrorMessage::NoOriginal {
                first_extension, ..
            } => *first_extension,
        };
        let additional_info = match &value.message {
            ExtensionErrorMessage::DuplicateOriginal { name, second, .. } => {
                vec![(*second, format!("Another declaration of '{name}'"))]
            }
            ExtensionErrorMessage::NoOriginal { .. } => vec![],
        };

        PositionedError::new(value.message.into(), Some(position), additional_info)
    }
}

impl<OriginalType: HasPos, ExtensionType: HasPos> ExtensionList<'_, OriginalType, ExtensionType> {
    pub fn new<'a>(name_of_elem: &'a str) -> ExtensionList<'a, OriginalType, ExtensionType> {
        ExtensionList {
            name_of_elem,
            items: HashMap::new(),
        }
    }

    pub fn set_original(&mut self, original: OriginalType) -> Result<(), ExtensionError> {
        let name = original.name().map(|str| str.to_owned());
        let mut item = self.items.entry(name.clone()).or_default();
        if let Some(ref first) = item.original {
            return Err(ExtensionError {
                message: ExtensionErrorMessage::DuplicateOriginal {
                    name_of_elem: self.name_of_elem.to_owned(),
                    name: name.unwrap_or_else(|| String::new()),
                    first: *first.position(),
                    second: *original.position(),
                },
            });
        }
        item.original = Some(original);
        Ok(())
    }
    pub fn add_extension(&mut self, extension: ExtensionType) {
        let name = extension.name().map(|str| str.to_owned());
        let item = self.items.entry(name).or_default();
        item.extensions.push(extension);
    }

    pub fn into_original_and_extensions(
        self,
    ) -> Result<Vec<(OriginalType, Vec<ExtensionType>)>, ExtensionError> {
        let result: Result<Vec<_>, _> = self
            .items
            .into_iter()
            .filter_map(|(_, item)| match item.original {
                None => match item.extensions.into_iter().next() {
                    None => None,
                    Some(first) => Some(Err(ExtensionError {
                        message: ExtensionErrorMessage::NoOriginal {
                            name_of_elem: self.name_of_elem.to_owned(),
                            first_extension: *first.position(),
                        },
                    })),
                },
                Some(orig) => Some(Ok((orig, item.extensions))),
            })
            .collect();
        let mut result = result?;
        // Sort by AST position for stable results
        result.sort_by_key(|(orig, _)| *orig.position());
        Ok(result)
    }
}
