use std::path::{Component, Path, PathBuf};

/// Calculates relative path between two.
pub fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let mut from = normalize_path(from);
    let to = normalize_path(to);
    // from should be file (not directory), so move to parent dir
    from.pop();

    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common_prefix_num = from
        .iter()
        .zip(to.iter())
        .take_while(|(f, t)| f == t)
        .count();

    let result_components = from
        .into_iter()
        .skip(common_prefix_num)
        .flat_map(|component| match component {
            Component::CurDir => Some(Component::CurDir),
            Component::Normal(_) => Some(Component::ParentDir),
            Component::ParentDir => panic!("Cannot calc reverse of ParentDir"),
            Component::Prefix(_) => None,
            Component::RootDir => None,
        })
        .chain(to.into_iter().skip(common_prefix_num));

    let mut result = PathBuf::new();
    for (idx, component) in result_components.enumerate() {
        if idx == 0 && !matches!(component, Component::CurDir | Component::ParentDir) {
            // To align with the custom that relative paths start with `./` or `../`
            result.push(Component::CurDir);
        }
        result.push(component.as_os_str());
    }
    result
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut stack = vec![];
    for component in path.components() {
        match component {
            Component::CurDir => {}
            c @ Component::Normal(_) => {
                stack.push(c);
            }
            Component::ParentDir => {
                stack.pop();
            }
            c @ Component::Prefix(_) | c @ Component::RootDir => {
                stack.clear();
                stack.push(c);
            }
        }
    }

    let mut result = PathBuf::new();
    for component in stack {
        result.push(component.as_os_str());
    }
    result
}

/// Calculates file-to-file relative path.
pub fn resolve_relative_path(from_file: &Path, relative: &Path) -> PathBuf {
    let mut result = from_file.to_path_buf();
    result.pop();
    result.push(relative);
    normalize_path(&result)
}

#[test]
fn test_relative_path() {
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/sub/frag1.graphql")
        ),
        Path::new("./sub/frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/sub/../frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/sub/../../frag1.graphql")
        ),
        Path::new("../frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
    assert_eq!(
        relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("./frag1.graphql")
    );
}

#[test]
fn test_resolve_relative_path() {
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("./frag1.graphql")
        ),
        Path::new("/path/to/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("./sub/frag1.graphql")
        ),
        Path::new("/path/to/sub/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("./sub/../frag1.graphql")
        ),
        Path::new("/path/to/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("./sub/../../frag1.graphql")
        ),
        Path::new("/path/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("../frag1.graphql")
        ),
        Path::new("/path/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("../../frag1.graphql")
        ),
        Path::new("/frag1.graphql")
    );
    assert_eq!(
        resolve_relative_path(
            Path::new("/path/to/main.graphql"),
            Path::new("/path/to/frag1.graphql")
        ),
        Path::new("/path/to/frag1.graphql")
    );
}
