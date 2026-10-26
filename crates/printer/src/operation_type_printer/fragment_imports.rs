//! Fragments imported from other modules.
//!
//! Spreads of imported fragments are not expanded. Each module exports selection trees of its
//! fragments, and importing modules merge them at the type level with utility types.
//! This way, printed types do not depend on contents of other modules.

use nitrogql_ast::base::{HasPos, Pos};

/// Name of the type exported from each module, which maps names of fragments to their selection trees.
pub const FRAGMENTS_EXPORT_NAME: &str = "__NitrogqlFragments";
/// Namespace of utility types for merging imported fragments.
pub const UTILITIES_NAMESPACE: &str = "__Nitrogql";

/// Fragments imported from another module.
#[derive(Clone, Debug)]
pub struct FragmentImport {
    /// Module specifier to import from.
    pub module: String,
    /// Names of imported fragments, or `None` if all fragments of the module are imported.
    pub fragments: Option<Vec<String>>,
    /// The import path as written in the source and its position,
    /// to which the module specifier is mapped.
    pub source: Option<(Pos, String)>,
}

/// Name of the namespace that given import is imported as.
pub fn import_namespace(index: usize) -> String {
    format!("__NitrogqlImport{index}")
}

/// Returns whether a fragment of given name is imported.
/// Fragments of any names may be imported by wildcard imports.
pub fn is_imported_fragment(imports: &[FragmentImport], name: &str) -> bool {
    imports.iter().any(|import| match import.fragments {
        None => true,
        Some(ref fragments) => fragments.iter().any(|fragment| fragment == name),
    })
}

/// Returns a TypeScript type that refers to the selection tree of given imported fragment.
pub fn imported_fragment_tree(imports: &[FragmentImport], name: &str) -> String {
    let fragments = |index: usize| format!("{}.{FRAGMENTS_EXPORT_NAME}", import_namespace(index));
    // A specific import is preferred over wildcard imports.
    let specific = imports.iter().position(|import| {
        import
            .fragments
            .as_ref()
            .is_some_and(|fragments| fragments.iter().any(|fragment| fragment == name))
    });
    let maps = match specific {
        Some(index) => fragments(index),
        None => imports
            .iter()
            .enumerate()
            .filter(|(_, import)| import.fragments.is_none())
            .map(|(index, _)| fragments(index))
            .collect::<Vec<_>>()
            .join(" & "),
    };
    format!("{UTILITIES_NAMESPACE}.Fragment<{maps}, \"{name}\">")
}

/// Node that is printed for a node in the source.
pub(super) struct SourceNode<'a> {
    pub position: &'a Pos,
    /// Text of the node in the source.
    pub text: &'a str,
}

impl HasPos for SourceNode<'_> {
    fn position(&self) -> &Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.text)
    }
}
