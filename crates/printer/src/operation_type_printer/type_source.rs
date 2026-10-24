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
    selection_tree::{GenerateSelectionTreeTypeContext, generate_selection_tree_type},
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
    /// Type of the data of given fragment.
    fn fragment_type(&self, fragment: &FragmentDefinition) -> Result<TSType, Self::Error>;
}

/// Type source that computes types from the schema.
/// The document must be valid against the schema; otherwise an error is returned.
pub struct SchemaTypeSource<'a, 'src> {
    options: OperationTypePrinterOptions,
    schema: &'a Schema<Cow<'src, str>, Pos>,
    fragment_definitions: HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

impl<'a, 'src> SchemaTypeSource<'a, 'src> {
    pub fn new(
        options: OperationTypePrinterOptions,
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
            options: &self.options,
            schema: self.schema,
            fragment_definitions: &self.fragment_definitions,
        }
    }

    fn selection_tree_type_context(&self) -> GenerateSelectionTreeTypeContext<'_> {
        GenerateSelectionTreeTypeContext {
            schema_root_namespace: &self.options.schema_root_namespace,
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

    fn fragment_type(&self, fragment: &FragmentDefinition) -> Result<TSType, Self::Error> {
        let parent_type = Type::NonNull(Box::new(NonNullType::from(Type::Named(NamedType::from(
            Node::from(
                fragment.type_condition.name,
                fragment.type_condition.position,
            ),
        )))));
        let fragment_type = get_type_for_selection_set(
            &self.type_printer_context(),
            &fragment.selection_set,
            &parent_type,
        )?;
        Ok(generate_selection_tree_type(
            &self.selection_tree_type_context(),
            &fragment_type,
        ))
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
        options: OperationTypePrinterOptions,
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

    fn type_for(
        &self,
        position: &Pos,
        get_type: impl FnOnce() -> Result<TSType, OperationTypePrinterError>,
    ) -> TSType {
        if (self.use_fallback)(position) || self.failed.borrow().contains(position) {
            return TSType::Any;
        }
        get_type().unwrap_or_else(|err| {
            self.failed.borrow_mut().insert(*position);
            self.errors.borrow_mut().push(err);
            TSType::Any
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
        Ok(self.type_for(&operation.position, || {
            self.schema.operation_result_type(operation)
        }))
    }

    fn operation_variables_type(
        &self,
        operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(self.type_for(&operation.position, || {
            self.schema.operation_variables_type(operation)
        }))
    }

    fn fragment_type(&self, fragment: &FragmentDefinition) -> Result<TSType, Self::Error> {
        Ok(self.type_for(&fragment.position, || self.schema.fragment_type(fragment)))
    }
}

fn select_root_type<T>(root_types: &RootTypes<T>, operation_type: OperationType) -> &T {
    match operation_type {
        OperationType::Query => &root_types.query_type,
        OperationType::Mutation => &root_types.mutation_type,
        OperationType::Subscription => &root_types.subscription_type,
    }
}
