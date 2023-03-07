use std::{
    io,
    path::{Component, Path, PathBuf},
};

/// Calculates relative path between two.
pub fn relative_path(from: &Path, to: &Path) -> io::Result<PathBuf> {
    let mut from = from.canonicalize()?;
    let to = to.canonicalize()?;
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
    Ok(result)
}
