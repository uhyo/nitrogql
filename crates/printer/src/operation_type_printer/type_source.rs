use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    convert::Infallible,
};

use graphql_type_system::{NamedType, Node, NonNullType, RootTypes, Schema, Type};
use nitrogql_ast::{
    OperationDocument,
    base::Pos,
    operation::{ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType},
};

use crate::ts_types::TSType;

use super::{
    error::OperationTypePrinterError,
    fragment_imports::exported_fragment_tree,
    selection_tree::{
        GenerateSelectionTreeTypeContext, SelectionTree, generate_selection_tree_type,
        generate_unrendered_branches_type, render_type,
    },
    type_printer::{
        QueryTypePrinterContext, get_type_for_selection_set, get_type_for_variable_definitions,
    },
    visitor::OperationTypePrinterOptions,
};

/// Provides TypeScript types of operations and fragments to the operation type printer.
///
/// The printer itself decides which declarations are emitted and how they are named;
/// a type source only decides what their types are.
pub trait OperationTypeSource {
    type Error;
    /// Whether printed types refer to the schema types namespace,
    /// which then needs to be imported.
    fn uses_schema_types(&self) -> bool;
    /// Type of the result of given operation.
    fn operation_result_type(&self, operation: &OperationDefinition)
    -> Result<TSType, Self::Error>;
    /// Type of the variables of given operation.
    fn operation_variables_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error>;
    /// Types of given fragment.
    /// With `export_tree`, its selection tree is computed too,
    /// and the type of its data may refer to the tree exported from the module.
    fn fragment_types(
        &self,
        fragment: &FragmentDefinition,
        export_tree: bool,
    ) -> Result<FragmentTypes, Self::Error>;
}

/// Types of a fragment.
pub struct FragmentTypes {
    /// Type of the data of the fragment.
    pub data: TSType,
    /// Selection tree of the fragment, which other modules merge into their selections.
    /// `None` if the tree is not exported.
    pub tree: Option<TSType>,
}

impl FragmentTypes {
    /// Types of a fragment in which all types are `any`.
    fn any(export_tree: bool) -> Self {
        Self {
            data: TSType::Any,
            tree: export_tree.then_some(TSType::Any),
        }
    }
}

/// Type source that computes types from the schema.
/// The document must be valid against the schema; otherwise an error is returned.
pub struct SchemaTypeSource<'a, 'src> {
    options: &'a OperationTypePrinterOptions,
    schema: &'a Schema<Cow<'src, str>, Pos>,
    fragment_definitions: HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

impl<'a, 'src> SchemaTypeSource<'a, 'src> {
    pub fn new(
        options: &'a OperationTypePrinterOptions,
        schema: &'a Schema<Cow<'src, str>, Pos>,
        operation: &'a OperationDocument<'src>,
    ) -> Self {
        let fragment_definitions = operation
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::OperationDefinition(_) => None,
                ExecutableDefinition::FragmentDefinition(fragment_def) => {
                    Some((fragment_def.name.name, fragment_def))
                }
            })
            .collect();
        Self {
            options,
            schema,
            fragment_definitions,
        }
    }

    fn type_printer_context(&self) -> QueryTypePrinterContext<'_, 'src, Cow<'src, str>> {
        QueryTypePrinterContext {
            options: self.options,
            schema: self.schema,
            fragment_definitions: &self.fragment_definitions,
        }
    }

    fn selection_tree_type_context(&self) -> GenerateSelectionTreeTypeContext<'_, Cow<'src, str>> {
        GenerateSelectionTreeTypeContext {
            schema: self.schema,
            schema_root_namespace: &self.options.schema_root_namespace,
            fragment_imports: &self.options.fragment_imports,
        }
    }
}

impl OperationTypeSource for SchemaTypeSource<'_, '_> {
    type Error = OperationTypePrinterError;

    fn uses_schema_types(&self) -> bool {
        true
    }

    fn operation_result_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        let root_types = self.schema.root_types().unwrap_or_default();
        let parent_type = select_root_type(&root_types, operation.operation_type);
        let parent_type = Type::NonNull(Box::new(NonNullType::from(Type::Named(NamedType::from(
            parent_type.clone(),
        )))));
        let operation_type = get_type_for_selection_set(
            &self.type_printer_context(),
            &operation.selection_set,
            &parent_type,
        )?;
        Ok(generate_selection_tree_type(
            &self.selection_tree_type_context(),
            &operation_type,
        ))
    }

    fn operation_variables_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(operation
            .variables_definition
            .as_ref()
            .map_or(TSType::empty_object(), |v| {
                get_type_for_variable_definitions(&self.type_printer_context(), v)
            }))
    }

    fn fragment_types(
        &self,
        fragment: &FragmentDefinition,
        export_tree: bool,
    ) -> Result<FragmentTypes, Self::Error> {
        let parent_type = Type::Named(NamedType::from(Node::from(
            fragment.type_condition.name,
            fragment.type_condition.position,
        )));
        let SelectionTree::Object(branches) = get_type_for_selection_set(
            &self.type_printer_context(),
            &fragment.selection_set,
            &parent_type,
        )?
        else {
            unreachable!("selection tree of a named type is an object");
        };
        let context = self.selection_tree_type_context();
        let tree = export_tree.then(|| generate_unrendered_branches_type(&context, &branches));
        // Data of a fragment is never null.
        let selection = SelectionTree::NonNull(Box::new(SelectionTree::Object(branches)));
        let data = match tree {
            // The data is the rendered tree, which would print the exported tree again.
            Some(_) if selection.has_fragment_refs() => {
                render_type(TSType::Raw(exported_fragment_tree(fragment.name.name)))
            }
            _ => generate_selection_tree_type(&context, &selection),
        };
        Ok(FragmentTypes { data, tree })
    }
}

/// Type source that chooses between the schema and `any` per definition.
///
/// Definitions for which `use_fallback` returns true (given the position of the definition)
/// get `any` types. So do definitions whose types cannot be computed from the schema;
/// errors for them are collected instead of aborting the printing.
pub struct PerDefinitionTypeSource<'a, 'src, F> {
    schema: SchemaTypeSource<'a, 'src>,
    use_fallback: F,
    uses_schema_types: bool,
    /// Definitions that failed to print with the schema.
    failed: RefCell<HashSet<Pos>>,
    errors: RefCell<Vec<OperationTypePrinterError>>,
}

impl<'a, 'src, F: Fn(&Pos) -> bool> PerDefinitionTypeSource<'a, 'src, F> {
    pub fn new(
        options: &'a OperationTypePrinterOptions,
        schema: &'a Schema<Cow<'src, str>, Pos>,
        operation: &'a OperationDocument<'src>,
        use_fallback: F,
    ) -> Self {
        let uses_schema_types = operation.definitions.iter().any(|def| {
            !use_fallback(match def {
                ExecutableDefinition::OperationDefinition(def) => &def.position,
                ExecutableDefinition::FragmentDefinition(def) => &def.position,
            })
        });
        Self {
            schema: SchemaTypeSource::new(options, schema, operation),
            use_fallback,
            uses_schema_types,
            failed: RefCell::new(HashSet::new()),
            errors: RefCell::new(vec![]),
        }
    }

    /// Returns errors from definitions that fell back to `any` because they failed to print.
    pub fn into_errors(self) -> Vec<OperationTypePrinterError> {
        self.errors.into_inner()
    }

    /// Returns `get_type()` for the definition at given position, or `fallback` if the
    /// definition falls back to `any`.
    fn type_for<T>(
        &self,
        position: &Pos,
        fallback: T,
        get_type: impl FnOnce() -> Result<T, OperationTypePrinterError>,
    ) -> T {
        if (self.use_fallback)(position) || self.failed.borrow().contains(position) {
            return fallback;
        }
        get_type().unwrap_or_else(|err| {
            self.failed.borrow_mut().insert(*position);
            self.errors.borrow_mut().push(err);
            fallback
        })
    }
}

impl<F: Fn(&Pos) -> bool> OperationTypeSource for PerDefinitionTypeSource<'_, '_, F> {
    type Error = Infallible;

    fn uses_schema_types(&self) -> bool {
        self.uses_schema_types
    }

    fn operation_result_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(self.type_for(&operation.position, TSType::Any, || {
            self.schema.operation_result_type(operation)
        }))
    }

    fn operation_variables_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(self.type_for(&operation.position, TSType::Any, || {
            self.schema.operation_variables_type(operation)
        }))
    }

    fn fragment_types(
        &self,
        fragment: &FragmentDefinition,
        export_tree: bool,
    ) -> Result<FragmentTypes, Self::Error> {
        Ok(
            self.type_for(&fragment.position, FragmentTypes::any(export_tree), || {
                self.schema.fragment_types(fragment, export_tree)
            }),
        )
    }
}

/// Type source in which all types are `any`.
///
/// Unlike `SchemaTypeSource`, this does not require the document to be valid
/// against the schema. This is useful for providing the same set of exports as usual
/// for documents that have errors.
pub struct AnyTypeSource;

impl OperationTypeSource for AnyTypeSource {
    type Error = Infallible;

    fn uses_schema_types(&self) -> bool {
        false
    }

    fn operation_result_type(
        &self,
        _operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(TSType::Any)
    }

    fn operation_variables_type(
        &self,
        _operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(TSType::Any)
    }

    fn fragment_types(
        &self,
        _fragment: &FragmentDefinition,
        export_tree: bool,
    ) -> Result<FragmentTypes, Self::Error> {
        Ok(FragmentTypes::any(export_tree))
    }
}

fn select_root_type<T>(root_types: &RootTypes<T>, operation_type: OperationType) -> &T {
    match operation_type {
        OperationType::Query => &root_types.query_type,
        OperationType::Mutation => &root_types.mutation_type,
        OperationType::Subscription => &root_types.subscription_type,
    }
}
