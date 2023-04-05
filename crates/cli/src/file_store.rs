use std::{
    ops::Index,
    path::{Path, PathBuf},
};

/// Struct that holds files loaded by CLI.
#[derive(Debug)]
pub struct FileStore {
    schema_files: Vec<(PathBuf, &'static str, FileKind)>,
    operation_files: Vec<(PathBuf, &'static str, FileKind)>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FileKind {
    Schema,
    Operation,
}

impl FileStore {
    /// Create a new file store.
    pub fn new() -> Self {
        Self {
            schema_files: vec![],
            operation_files: vec![],
        }
    }

    /// Add a file to the store.
    /// Returns the index of the file in the store.
    /// Once the index is issued, it will not change.
    /// After an operation is added, schema cannot be added.
    pub fn add_file(&mut self, path: PathBuf, content: String, kind: FileKind) -> usize {
        if !self.operation_files.is_empty() && kind == FileKind::Schema {
            panic!("Cannot add schema file after operation file is added");
        }
        let schema_len = self.schema_files.len();
        match kind {
            FileKind::Schema => {
                self.schema_files
                    .push((path, Box::leak(content.into_boxed_str()), kind));
                schema_len
            }
            FileKind::Operation => {
                self.operation_files
                    .push((path, Box::leak(content.into_boxed_str()), kind));
                schema_len + self.operation_files.len() - 1
            }
        }
    }

    /// Get a file by index.
    pub fn get_file(&self, index: usize) -> Option<&(PathBuf, &'static str, FileKind)> {
        if index < self.schema_files.len() {
            self.schema_files.get(index)
        } else {
            self.operation_files.get(index - self.schema_files.len())
        }
    }

    /// Iterate over all files.
    pub fn iter(&self) -> impl Iterator<Item = (usize, (&Path, &'static str, FileKind))> {
        self.schema_files
            .iter()
            .chain(self.operation_files.iter())
            .map(|(path, content, kind)| {
                let path = path.as_path();
                (path, *content, *kind)
            })
            .enumerate()
    }

    /// Returns the number of schema files in the store.
    pub fn schema_len(&self) -> usize {
        self.schema_files.len()
    }
}

impl Index<usize> for FileStore {
    type Output = (PathBuf, &'static str, FileKind);

    fn index(&self, index: usize) -> &Self::Output {
        self.get_file(index).expect("File index out of range")
    }
}
