use std::path::{Path, PathBuf};

use nitrogql_ast::OperationDocument;
use nitrogql_config_file::Config;
use nitrogql_error::{PositionedError, Result};
use nitrogql_semantics::{OperationExtension, OperationResolver, resolve_operation_imports};
use nitrogql_utils::resolve_relative_path;
use thiserror::Error;

use crate::{
    js_printer::print_js,
    tasks::{Task, Tasks},
};

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Task not found")]
    TaskNotFound,
}

/// Initiates a task.
/// Returns the task id.
pub fn initiate_task(tasks: &mut Tasks, file_name: PathBuf, input_source: String) -> Result<usize> {
    let mut task = Task::new(file_name.clone());
    task.register_file(file_name, input_source)?;
    let task_id = tasks.add_task(task);
    Ok(task_id)
}

/// Get the list of additionally required files for the given task.
pub fn get_required_files(tasks: &mut Tasks, task_id: usize) -> Result<Vec<PathBuf>> {
    let task = tasks
        .get_task_mut(task_id)
        .ok_or_else(|| PositionedError::new(LoaderError::TaskNotFound.into(), None, vec![]))?;

    let mut required_files = vec![];
    for (from_file, (_, extensions)) in task.iter_loaded_files() {
        for import in extensions.imports.iter() {
            let path = Path::new(import.path.value.as_str());
            let path = resolve_relative_path(from_file, path);
            if task.contains_file(&path) || required_files.contains(&path) {
                continue;
            }
            required_files.push(path);
        }
    }
    Ok(required_files)
}

/// Load an additional file.
pub fn load_file(
    tasks: &mut Tasks,
    task_id: usize,
    file_name: PathBuf,
    input_source: String,
) -> Result<()> {
    let task = tasks
        .get_task_mut(task_id)
        .ok_or_else(|| PositionedError::new(LoaderError::TaskNotFound.into(), None, vec![]))?;
    task.register_file(file_name, input_source)?;
    Ok(())
}

/// Emit JavaScript for the given task.
pub fn emit_js(tasks: &Tasks, task_id: usize, config: &Config) -> Result<String> {
    let task = tasks
        .get_task(task_id)
        .ok_or_else(|| PositionedError::new(LoaderError::TaskNotFound.into(), None, vec![]))?;
    let (document, extensions) = task.get_root_document();
    let document = resolve_operation_imports(
        (&task.root_file_name, document, extensions),
        &TaskOperationResolver(task),
    )?;
    let js = print_js(&document, config);
    Ok(js)
}

struct TaskOperationResolver<'a>(&'a Task);

impl<'a> OperationResolver<'a> for TaskOperationResolver<'a> {
    fn resolve(&self, path: &Path) -> Option<(&OperationDocument<'a>, &OperationExtension<'a>)> {
        let task = self.0;
        let (document, extension) = task.get_file(path)?;
        Some((document, extension))
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_no_import() {
        let mut tasks = Tasks::new();
        let task_id = initiate_task(
            &mut tasks,
            PathBuf::from("/path/to/op.graphql"),
            r#"
            query Test {
                test
            }
            "#
            .to_string(),
        )
        .unwrap();
        // no imports
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(required_files.len(), 0);
        let js = emit_js(&tasks, task_id, &Default::default()).unwrap();
        assert_snapshot!(js);
    }

    #[test]
    fn test_import() {
        let mut tasks = Tasks::new();
        let task_id = initiate_task(
            &mut tasks,
            PathBuf::from("/path/to/op.graphql"),
            r#"
            #import Frag1 from "./frag1.graphql"
            query Test {
                test
                ...Frag1
            }
            "#
            .to_string(),
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(
            required_files,
            vec![PathBuf::from("/path/to/frag1.graphql"),]
        );
        load_file(
            &mut tasks,
            task_id,
            PathBuf::from("/path/to/frag1.graphql"),
            r#"
            fragment Frag1 on Query {
                test2
            }
            "#
            .to_string(),
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(required_files.len(), 0);
        let js = emit_js(&tasks, task_id, &Default::default()).unwrap();
        assert_snapshot!(js);
    }

    #[test]
    fn transitive_import() {
        let mut tasks = Tasks::new();
        let task_id = initiate_task(
            &mut tasks,
            PathBuf::from("/path/to/op.graphql"),
            r#"
            #import Frag1 from "./frag1.graphql"
            query Test {
                test
                ...Frag1
            }
            "#
            .to_string(),
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(
            required_files,
            vec![PathBuf::from("/path/to/frag1.graphql"),]
        );
        load_file(
            &mut tasks,
            task_id,
            PathBuf::from("/path/to/frag1.graphql"),
            r#"
            #import Frag2 from "./frag2.graphql"
            fragment Frag1 on Query {
                test2
                ...Frag2
            }
            "#
            .to_string(),
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(
            required_files,
            vec![PathBuf::from("/path/to/frag2.graphql"),]
        );
        load_file(
            &mut tasks,
            task_id,
            PathBuf::from("/path/to/frag2.graphql"),
            r#"
            fragment Frag2 on Query {
                test3
            }
            "#
            .to_string(),
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(required_files.len(), 0);
        let js = emit_js(&tasks, task_id, &Default::default()).unwrap();
        assert_snapshot!(js);
    }
}
