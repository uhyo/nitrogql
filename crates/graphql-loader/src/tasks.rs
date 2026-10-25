use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use nitrogql_error::Result;
use nitrogql_operation_set::OperationSet;
use self_cell::self_cell;
use typed_arena::Arena;

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
    files: TaskFiles,
    /// Index of the next registered file.
    next_file_index: usize,
}

self_cell!(
    /// Sources of loaded files, along with operations parsed from them.
    struct TaskFiles {
        owner: Arena<String>,
        #[covariant]
        dependent: OperationSet,
    }
);

impl Debug for TaskFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow_dependent().fmt(f)
    }
}

impl Task {
    /// Creates a new task.
    pub fn new(root_file_name: PathBuf) -> Self {
        Self {
            root_file_name,
            files: TaskFiles::new(Arena::new(), |_| OperationSet::new()),
            next_file_index: 0,
        }
    }
    /// Gets the set of loaded operation files.
    pub fn operations(&self) -> &OperationSet<'_> {
        self.files.borrow_dependent()
    }

    /// Registers a source.
    /// Source will be dropped when the task is dropped.
    pub fn register_file(&mut self, file_name: PathBuf, source: String) -> Result<()> {
        // Each file gets its own index, so that only definitions in the root file
        // are exported, as in `nitrogql generate`.
        let file_index = self.next_file_index;
        self.next_file_index += 1;
        self.files.with_dependent_mut(|sources, operations| {
            let source = sources.alloc(source);
            operations.add_file(&file_name, source, file_index);
            let syntax_errors = operations.take_syntax_errors();
            match syntax_errors
                .into_iter()
                .chain(operations.take_errors())
                .next()
            {
                Some(err) => Err(err),
                None => Ok(()),
            }
        })
    }
}
