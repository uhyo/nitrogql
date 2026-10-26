use graphql_type_system::Schema;
use nitrogql_ast::base::Pos;

pub struct OperationCheckContext<'schema, 'src, S> {
    pub definitions: &'schema Schema<S, Pos>,
    /// Returns whether a fragment of given name is imported from another document.
    imported_fragments: Option<&'schema dyn Fn(&str) -> bool>,
    /// Returns whether a fragment of given name is imported by the checked file itself.
    visible_fragments: Option<&'schema dyn Fn(&str) -> bool>,
    // For some reasons 'src is considered unused
    phantom: std::marker::PhantomData<&'src ()>,
}

impl<'schema, S> OperationCheckContext<'schema, '_, S> {
    pub fn new(definitions: &'schema Schema<S, Pos>) -> Self {
        Self {
            definitions,
            imported_fragments: None,
            visible_fragments: None,
            phantom: std::marker::PhantomData,
        }
    }

    /// Treats fragments for which given function returns true as imported from other documents.
    /// Spreads of imported fragments are not checked, as their definitions are not available.
    pub fn with_imported_fragments(
        mut self,
        imported_fragments: &'schema dyn Fn(&str) -> bool,
    ) -> Self {
        self.imported_fragments = Some(imported_fragments);
        self
    }

    /// Restricts fragments that the checked file can spread to ones defined in the file
    /// and ones for which given function returns true.
    /// This is for documents with imported fragments appended (including transitively imported ones):
    /// fragments that are appended but not imported by the file itself are reported as unknown.
    pub fn with_visible_fragments(
        mut self,
        visible_fragments: &'schema dyn Fn(&str) -> bool,
    ) -> Self {
        self.visible_fragments = Some(visible_fragments);
        self
    }

    /// Returns whether a fragment of given name, defined in another file, can be spread from the checked file.
    pub fn is_visible_fragment(&self, name: &str) -> bool {
        self.visible_fragments
            .is_none_or(|visible_fragments| visible_fragments(name))
    }

    /// Returns whether a fragment of given name is imported from another document.
    pub fn is_imported_fragment(&self, name: &str) -> bool {
        self.imported_fragments
            .is_some_and(|imported_fragments| imported_fragments(name))
    }
}
