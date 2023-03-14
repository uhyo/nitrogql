use std::path::{Component, Path, PathBuf};

/// Calculates relative path between two.
pub fn relative_path(from: &Path, to: &Path) -> PathBuf {
    eprintln!("relative_path {} {}", from.display(), to.display());
    let mut from = normalize(from);
    eprintln!("relative_path {} {}", from.display(), to.display());
    let to = normalize(to);
    eprintln!("relative_path {} {}", from.display(), to.display());
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
    for component in result_components {
        result.push(component.as_os_str());
    }
    result
}

fn normalize(path: &Path) -> PathBuf {
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
