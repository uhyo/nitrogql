use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use nitrogql_ast::OperationDocument;
use nitrogql_error::Result;
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::{OperationExtension, resolve_operation_extensions};

/// Set of tasks.
#[derive(Debug)]
pub struct Tasks {
    next_task_id: usize,
    tasks: HashMap<usize, Task>,
}

impl Tasks {
    /// Creates a new set of tasks.
    pub fn new() -> Self {
        Self {
            next_task_id: 1,
            tasks: HashMap::new(),
        }
    }

    /// Add a new task.
    pub fn add_task(&mut self, task: Task) -> usize {
        let task_id = self.next_task_id;
        self.next_task_id += 1;
        self.tasks.insert(task_id, task);
        task_id
    }

    /// Get a task.
    pub fn get_task(&self, task_id: usize) -> Option<&Task> {
        self.tasks.get(&task_id)
    }

    /// Get a mutable task.
    pub fn get_task_mut(&mut self, task_id: usize) -> Option<&mut Task> {
        self.tasks.get_mut(&task_id)
    }

    /// Remove a task.
    pub fn remove_task(&mut self, task_id: usize) -> Option<Task> {
        self.tasks.remove(&task_id)
    }
}

/// One task of printing.
#[derive(Debug)]
pub struct Task {
    /// Name of root file.
    pub root_file_name: PathBuf,
    /// Set of loaded operation files.
    /// The root file should be present when initiating the task.
    loaded_files: HashMap<PathBuf, (OperationDocument<'static>, OperationExtension<'static>)>,
    source_drop_list: Vec<(*mut u8, usize, usize)>,
}

impl Task {
    /// Creates a new task.
    pub fn new(root_file_name: PathBuf) -> Self {
        Self {
            root_file_name,
            loaded_files: HashMap::new(),
            source_drop_list: Vec::new(),
        }
    }
    /// Gets the root document.
    /// Panics if the root file is not registered yet.
    pub fn get_root_document(&self) -> (&OperationDocument, &OperationExtension) {
        let (doc, extension) = self
            .loaded_files
            .get(&self.root_file_name)
            .expect("Root file should be present");
        (doc, extension)
    }
    /// Checks if the given file is loaded.
    pub fn contains_file(&self, file_name: &Path) -> bool {
        self.loaded_files.contains_key(file_name)
    }
    /// Gets the document and extension for the given file.
    pub fn get_file(&self, file_name: &Path) -> Option<&(OperationDocument, OperationExtension)> {
        self.loaded_files.get(file_name)
    }
    /// Returns an iterator over loaded files.
    pub fn iter_loaded_files(
        &self,
    ) -> impl Iterator<
        Item = (
            &PathBuf,
            &(OperationDocument<'static>, OperationExtension<'static>),
        ),
    > {
        self.loaded_files.iter()
    }

    /// Registers a source.
    /// Source will be dropped when the task is dropped.
    pub fn register_file(&mut self, file_name: PathBuf, mut source: String) -> Result<()> {
        let raw_parts = (source.as_mut_ptr(), source.len(), source.capacity());
        let source = Box::leak(source.into_boxed_str());
        self.source_drop_list.push(raw_parts);
        let document = parse_operation_document(source)?;
        let (document, extensions) = resolve_operation_extensions(document)?;
        self.loaded_files.insert(file_name, (document, extensions));
        Ok(())
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        // need to drop loaded_files first
        self.loaded_files.clear();
        // then drop sources
        for (ptr, len, capacity) in self.source_drop_list.drain(..) {
            let _ = unsafe { String::from_raw_parts(ptr, len, capacity) };
        }
    }
}
