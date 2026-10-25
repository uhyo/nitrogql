use std::{
    fmt::Debug,
    ops::Index,
    path::{Path, PathBuf},
};

use typed_arena::Arena;

/// Struct that holds files loaded by CLI.
///
/// Contents of files are owned by an arena, so that they can be borrowed (`'src`)
/// while more files are added.
pub struct FileStore<'src> {
    sources: &'src Arena<String>,
    schema_files: Vec<(PathBuf, &'src str, FileKind)>,
    operation_files: Vec<(PathBuf, &'src str, FileKind)>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FileKind {
    Schema,
    Operation,
}

impl<'src> FileStore<'src> {
    /// Create a new file store whose contents are stored in given arena.
    pub fn new(sources: &'src Arena<String>) -> Self {
        Self {
            sources,
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
        let content = self.sources.alloc(content).as_str();
        match kind {
            FileKind::Schema => {
                self.schema_files.push((path, content, kind));
                schema_len
            }
            FileKind::Operation => {
                self.operation_files.push((path, content, kind));
                schema_len + self.operation_files.len() - 1
            }
        }
    }

    /// Get a file by index.
    pub fn get_file(&self, index: usize) -> Option<&(PathBuf, &'src str, FileKind)> {
        if index < self.schema_files.len() {
            self.schema_files.get(index)
        } else {
            self.operation_files.get(index - self.schema_files.len())
        }
    }

    /// Iterate over all files.
    pub fn iter(&self) -> impl Iterator<Item = (usize, (&Path, &'src str, FileKind))> {
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

impl Debug for FileStore<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStore")
            .field("schema_files", &self.schema_files)
            .field("operation_files", &self.operation_files)
            .finish_non_exhaustive()
    }
}

impl<'src> Index<usize> for FileStore<'src> {
    type Output = (PathBuf, &'src str, FileKind);

    fn index(&self, index: usize) -> &Self::Output {
        self.get_file(index).expect("File index out of range")
    }
}
