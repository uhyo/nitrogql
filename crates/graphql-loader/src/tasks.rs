use std::{collections::HashMap, path::PathBuf};

use nitrogql_ast::OperationDocument;
use nitrogql_semantics::OperationExtension;

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
    pub loaded_files: HashMap<PathBuf, (OperationDocument<'static>, OperationExtension<'static>)>,
}

impl Task {
    pub fn get_root_document(&self) -> (&OperationDocument<'static>, &OperationExtension<'static>) {
        let (doc, extension) = self
            .loaded_files
            .get(&self.root_file_name)
            .expect("Root file should be present");
        (doc, extension)
    }
}
