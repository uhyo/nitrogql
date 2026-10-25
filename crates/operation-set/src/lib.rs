//! Loading operation files and resolving `#import`s between them.
//!
//! This is shared by the CLI, the content mapper and the webpack loader,
//! which differ only in how files are found and read.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use nitrogql_ast::{OperationDocument, base::Pos};
use nitrogql_error::PositionedError;
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::{
    OperationExtension, OperationResolver, resolve_operation_extensions, resolve_operation_imports,
};
use nitrogql_utils::{normalize_path, resolve_relative_path};

#[cfg(test)]
mod tests;

/// A set of operation files that can import each other.
///
/// Each file is parsed and its extensions are resolved once, when it is added.
/// Errors found while loading files are collected in the set.
/// Paths are compared after normalization.
#[derive(Debug, Default)]
pub struct OperationSet<'src> {
    /// Added files, in order of addition.
    files: Vec<Entry<'src>>,
    /// Index of each file in `files`, by normalized path.
    by_path: HashMap<PathBuf, usize>,
    syntax_errors: Vec<PositionedError>,
    errors: Vec<PositionedError>,
}

#[derive(Debug)]
struct Entry<'src> {
    path: PathBuf,
    file_index: usize,
    /// None if the file could not be loaded.
    content: Option<(OperationDocument<'src>, OperationExtension<'src>)>,
}

/// A successfully loaded file in an operation set.
#[derive(Debug, Clone, Copy)]
pub struct OperationFile<'a, 'src> {
    /// Path of the file as it was added.
    pub path: &'a Path,
    /// Index of the file, used in positions.
    pub file_index: usize,
    pub document: &'a OperationDocument<'src>,
    pub extension: &'a OperationExtension<'src>,
}

impl<'a> OperationFile<'a, '_> {
    /// Returns files imported by this file (normalized paths), along with positions of the imports.
    pub fn imports(self) -> impl Iterator<Item = (PathBuf, Pos)> + 'a {
        self.extension.imports.iter().map(|import| {
            (
                resolve_relative_path(self.path, Path::new(&import.path.value)),
                import.path.position,
            )
        })
    }
}

impl<'src> OperationSet<'src> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file to the set.
    /// `file_index` is recorded in positions of the parsed document.
    /// If the file cannot be parsed or has invalid extensions, the error is collected
    /// and the file is not available for resolution.
    /// Adding a file that is already in the set has no effect.
    pub fn add_file(&mut self, path: &Path, source: &'src str, file_index: usize) {
        let normalized = normalize_path(path);
        if self.by_path.contains_key(&normalized) {
            return;
        }
        let content = match parse_operation_document(source, file_index) {
            Err(err) => {
                self.syntax_errors.push(err.into());
                None
            }
            Ok(document) => match resolve_operation_extensions(document) {
                Err(err) => {
                    self.errors.push(err.into());
                    None
                }
                Ok(content) => Some(content),
            },
        };
        self.push_entry(normalized, path.to_owned(), file_index, content);
    }

    fn push_entry(
        &mut self,
        normalized: PathBuf,
        path: PathBuf,
        file_index: usize,
        content: Option<(OperationDocument<'src>, OperationExtension<'src>)>,
    ) {
        self.by_path.insert(normalized, self.files.len());
        self.files.push(Entry {
            path,
            file_index,
            content,
        });
    }

    /// Returns imported files that are not in the set, along with the position of the first import of each.
    pub fn missing_imports(&self) -> Vec<(PathBuf, Pos)> {
        let mut result: Vec<(PathBuf, Pos)> = vec![];
        for file in self.files() {
            for (path, position) in file.imports() {
                if !self.by_path.contains_key(&path)
                    && !result.iter().any(|(missing, _)| *missing == path)
                {
                    result.push((path, position));
                }
            }
        }
        result
    }

    /// Adds files imported from files in the set, transitively.
    /// `read` returns the source and the file index of given (normalized) path.
    /// If it fails, the error is collected, positioned at the import.
    pub fn load_imports<E: Into<anyhow::Error>>(
        &mut self,
        mut read: impl FnMut(&Path) -> Result<(&'src str, usize), E>,
    ) {
        loop {
            let missing = self.missing_imports();
            if missing.is_empty() {
                return;
            }
            for (path, position) in missing {
                match read(&path) {
                    Ok((source, file_index)) => self.add_file(&path, source, file_index),
                    Err(err) => {
                        self.errors
                            .push(PositionedError::new(err.into(), Some(position), vec![]));
                        // remember the file so that it is not read again
                        self.push_entry(path.clone(), path, usize::MAX, None);
                    }
                }
            }
        }
    }

    /// Returns whether given file has been added to the set, whether it was loaded successfully or not.
    pub fn contains(&self, path: &Path) -> bool {
        self.by_path.contains_key(&normalize_path(path))
    }

    /// Gets a successfully loaded file.
    pub fn get(&self, path: &Path) -> Option<OperationFile<'_, 'src>> {
        let idx = *self.by_path.get(&normalize_path(path))?;
        to_file(&self.files[idx])
    }

    /// Iterates over successfully loaded files, in order of addition.
    pub fn files(&self) -> impl Iterator<Item = OperationFile<'_, 'src>> {
        self.files.iter().filter_map(to_file)
    }

    /// Returns the path of the file with given file index, whether it was loaded successfully or not.
    pub fn file_path(&self, file_index: usize) -> Option<&Path> {
        self.files
            .iter()
            .find(|entry| entry.file_index == file_index)
            .map(|entry| entry.path.as_path())
    }

    /// Returns given file with imported fragments appended.
    /// Returns None if the file is not loaded successfully.
    pub fn resolve_imports(
        &self,
        path: &Path,
    ) -> Option<Result<OperationDocument<'src>, PositionedError>> {
        let file = self.get(path)?;
        Some(
            resolve_operation_imports((file.path, file.document, file.extension), self)
                .map_err(PositionedError::from),
        )
    }

    /// Syntax errors in added files.
    pub fn syntax_errors(&self) -> &[PositionedError] {
        &self.syntax_errors
    }

    /// Other errors found while loading files: invalid extensions, and imported files that could not be read.
    pub fn errors(&self) -> &[PositionedError] {
        &self.errors
    }

    /// Takes collected syntax errors out of the set.
    pub fn take_syntax_errors(&mut self) -> Vec<PositionedError> {
        std::mem::take(&mut self.syntax_errors)
    }

    /// Takes collected errors other than syntax errors out of the set.
    pub fn take_errors(&mut self) -> Vec<PositionedError> {
        std::mem::take(&mut self.errors)
    }
}

fn to_file<'a, 'src>(entry: &'a Entry<'src>) -> Option<OperationFile<'a, 'src>> {
    let (document, extension) = entry.content.as_ref()?;
    Some(OperationFile {
        path: &entry.path,
        file_index: entry.file_index,
        document,
        extension,
    })
}

impl<'src> OperationResolver<'src> for OperationSet<'src> {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(&OperationDocument<'src>, &OperationExtension<'src>)> {
        let file = self.get(path)?;
        Some((file.document, file.extension))
    }
}
