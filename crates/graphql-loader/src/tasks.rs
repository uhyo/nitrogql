use std::{collections::HashMap, path::PathBuf};

use nitrogql_error::Result;
use nitrogql_operation_set::OperationSet;

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
    operations: OperationSet<'static>,
    source_drop_list: Vec<(*mut u8, usize, usize)>,
}

impl Task {
    /// Creates a new task.
    pub fn new(root_file_name: PathBuf) -> Self {
        Self {
            root_file_name,
            operations: OperationSet::new(),
            source_drop_list: Vec::new(),
        }
    }
    /// Gets the set of loaded operation files.
    pub fn operations(&self) -> &OperationSet<'_> {
        &self.operations
    }

    /// Registers a source.
    /// Source will be dropped when the task is dropped.
    pub fn register_file(&mut self, file_name: PathBuf, mut source: String) -> Result<()> {
        let raw_parts = (source.as_mut_ptr(), source.len(), source.capacity());
        let source = Box::leak(source.into_boxed_str());
        self.source_drop_list.push(raw_parts);
        // All files of a task share file index 0. The printer exports definitions
        // from the same file as the root document, so imported fragments are exported
        // from the emitted module too (unlike in `nitrogql generate`).
        self.operations.add_file(&file_name, source, 0);
        let syntax_errors = self.operations.take_syntax_errors();
        match syntax_errors
            .into_iter()
            .chain(self.operations.take_errors())
            .next()
        {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        // need to drop loaded files first
        self.operations = OperationSet::new();
        // then drop sources
        for (ptr, len, capacity) in self.source_drop_list.drain(..) {
            let _ = unsafe { String::from_raw_parts(ptr, len, capacity) };
        }
    }
}
