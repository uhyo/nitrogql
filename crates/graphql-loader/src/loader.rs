use std::path::{Path, PathBuf};

use nitrogql_ast::OperationDocument;
use nitrogql_config_file::Config;
use nitrogql_error::{PositionedError, Result};
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::{
    resolve_operation_extensions, resolve_operation_imports, OperationExtension, OperationResolver,
};
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
pub fn initiate_task(
    tasks: &mut Tasks,
    file_name: PathBuf,
    input_source: &'static str,
) -> Result<usize> {
    let document = parse_operation_document(input_source)?;
    let (document, extensions) = resolve_operation_extensions(document)?;
    let task = Task {
        root_file_name: file_name.clone(),
        loaded_files: vec![(file_name, (document, extensions))]
            .into_iter()
            .collect(),
    };
    let task_id = tasks.add_task(task);
    Ok(task_id)
}

/// Get the list of additionally required files for the given task.
pub fn get_required_files(tasks: &mut Tasks, task_id: usize) -> Result<Vec<PathBuf>> {
    let task = tasks
        .get_task_mut(task_id)
        .ok_or_else(|| PositionedError::new(LoaderError::TaskNotFound.into(), None, vec![]))?;

    let mut required_files = vec![];
    for (from_file, (_, extensions)) in task.loaded_files.iter() {
        for import in extensions.imports.iter() {
            let path = Path::new(import.path.value.as_str());
            let path = resolve_relative_path(from_file, path);
            if task.loaded_files.contains_key(&path) || required_files.contains(&path) {
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
    input_source: &'static str,
) -> Result<()> {
    let task = tasks
        .get_task_mut(task_id)
        .ok_or_else(|| PositionedError::new(LoaderError::TaskNotFound.into(), None, vec![]))?;
    let document = parse_operation_document(input_source)?;
    let (document, extensions) = resolve_operation_extensions(document)?;
    task.loaded_files.insert(file_name, (document, extensions));
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

impl<'a> OperationResolver<'static> for TaskOperationResolver<'a> {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(&OperationDocument<'static>, &OperationExtension<'static>)> {
        let task = self.0;
        let path = resolve_relative_path(&task.root_file_name, path);
        let (document, extension) = task.loaded_files.get(&path)?;
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
            "#,
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
            "#,
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
            "#,
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
            "#,
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
            "#,
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
            "#,
        )
        .unwrap();
        let required_files = get_required_files(&mut tasks, task_id).unwrap();
        assert_eq!(required_files.len(), 0);
        let js = emit_js(&tasks, task_id, &Default::default()).unwrap();
        assert_snapshot!(js);
    }
}