use std::{collections::HashMap, convert::identity, iter::once};

use crate::{
    ts_types::{ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type, ObjectField},
    utils::interface_implementers,
};
use graphql_type_system::{NamedType, ObjectDefinition, Schema, Text, Type, TypeDefinition};
use itertools::{Either, Itertools};
use nitrogql_ast::{
    base::Pos,
    directive::Directive,
    operation::{FragmentDefinition, OperationDocument},
    selection_set::{Selection, SelectionSet},
    value::Value,
    variable::VariablesDefinition,
};
use nitrogql_semantics::direct_fields_of_output_type;
use sourcemap_writer::SourceMapWriter;

use super::{
    super::ts_types::{ts_types_util::ts_intersection, TSType},
    branching::BranchingCondition,
    selection_set_visitor::visit_fields_in_selection_set,
    visitor::OperationTypePrinterOptions,
};

pub struct QueryTypePrinterContext<'a, 'src, S: Text<'src>> {
    pub options: &'a OperationTypePrinterOptions,
    pub schema: &'a Schema<S, Pos>,
    pub operation: &'a OperationDocument<'src>,
    pub fragment_definitions: &'a HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

pub trait TypePrinter<'src, S: Text<'src>> {
    fn print_type(
        &self,
        options: &QueryTypePrinterContext<'_, 'src, S>,
        writer: &mut impl SourceMapWriter,
    );
}

pub fn get_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    parent_type: &NamedType<S, Pos>,
) -> TSType {
    let branches = generate_branching_conditions(context, selection_set, parent_type);
    ts_union(
        branches
            .into_iter()
            .map(|branch| get_object_type_for_selection_set(context, selection_set, &branch)),
    )
}

/// Generates a set of branching conditions for a given selection set.
fn generate_branching_conditions<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    parent_type: &'a NamedType<S, Pos>,
) -> Vec<BranchingCondition<'a, S>> {
    let parent_type_def = context
        .schema
        .get_type(&parent_type)
        .expect("Type system error");
    let parent_objects = match **parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            panic!("Type system error")
        }
        TypeDefinition::Object(ref obj_def) => {
            vec![obj_def]
        }
        TypeDefinition::Interface(ref interface_def) => {
            let object_defs = interface_implementers(context.schema, &interface_def.name);
            object_defs.collect()
        }
        TypeDefinition::Union(ref union_def) => {
            let object_defs = union_def.possible_types.iter().map(|member| {
                context
                    .schema
                    .get_type(member)
                    .and_then(|def| def.as_object())
                    .expect("Type system error")
            });
            object_defs.collect()
        }
    };
    // multi_cartesian_product cannot handle the case where there are no variables.
    // See: https://github.com/rust-itertools/itertools/issues/337
    let boolean_variables = get_boolean_variables(context, selection_set);
    let boolean_variables = if boolean_variables.is_empty() {
        Either::Left(once(vec![]))
    } else {
        Either::Right(
            boolean_variables
                .into_iter()
                .unique()
                .map(|v| vec![(v, false), (v, true)])
                .multi_cartesian_product(),
        )
    };
    let branches = parent_objects
        .into_iter()
        .cartesian_product(boolean_variables)
        .map(|(obj, vars)| BranchingCondition {
            parent_obj: obj,
            boolean_variables: vars,
        })
        .collect();
    branches
}

/// Get boolean variables involved in a selection set.
fn get_boolean_variables<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
) -> Vec<&'src str> {
    let mut variables = Vec::new();
    visit_fields_in_selection_set(context, selection_set, |selection| {
        let directives = selection.directives();
        for directive in directives {
            if directive.name.name == "skip" || directive.name.name == "include" {
                let variable = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find_map(|(arg, value)| {
                        if arg.name != "if" {
                            return None;
                        }
                        if let Value::Variable(ref v) = value {
                            Some(v.name)
                        } else {
                            None
                        }
                    });
                if let Some(variable) = variable {
                    variables.push(variable);
                }
            }
        }
    });
    variables
}

fn get_object_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    branch: &BranchingCondition<S>,
) -> TSType {
    let (unaliased, aliased): (Vec<_>, Vec<_>) =
        get_fields_for_selection_set(context, selection_set, branch)
            .into_iter()
            .partition_map(identity);
    let unaliased = TSType::Object(unaliased);
    let aliased = TSType::Object(aliased);
    let schema_type = TSType::NamespaceMember(
        context.options.schema_root_namespace.clone(),
        branch.parent_obj.name.to_string(),
    );
    TSType::TypeFunc(
        Box::new(TSType::NamespaceMember(
            context.options.schema_root_namespace.clone(),
            "__SelectionSet".into(),
        )),
        vec![schema_type, unaliased, aliased],
    )
}

/// Returns an iterator of object fields.
/// Left is for non-aliased fields.
/// Right is for aliased fields.
fn get_fields_for_selection_set<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    branch: &'a BranchingCondition<'a, S>,
) -> Vec<Either<ObjectField, ObjectField>> {
    let parent_type_def = context
        .schema
        .get_type(&branch.parent_obj.name)
        .expect("Type system error");

    let parent_fields = direct_fields_of_output_type(parent_type_def).expect("Type system error");

    let types_for_simple_fields =
        selection_set
            .selections
            .iter()
            .filter_map(move |sel| match sel {
                Selection::Field(ref field) => {
                    let field_name = field.name.name;
                    let field_type = if check_skip_directive(branch, &field.directives) {
                        TSType::Never
                    } else if field_name == "__typename" {
                        // Special handling of reflection
                        TSType::StringLiteral(branch.parent_obj.name.to_string())
                    } else {
                        let field_def = parent_fields
                            .iter()
                            .find(|parent_field| {
                                parent_field.name.inner_ref().borrow() == field_name
                            })
                            .expect("Type system error");

                        map_to_tstype(&field_def.r#type, |ty| match field.selection_set {
                            None => TSType::NamespaceMember(
                                context.options.schema_root_namespace.clone(),
                                ty.to_string(),
                            ),
                            Some(ref set) => get_type_for_selection_set(context, set, ty),
                        })
                    };

                    match field.alias {
                        None => Some(Either::Left(ObjectField {
                            key: field_name.into(),
                            optional: field_type.is_never(),
                            r#type: field_type,
                            readonly: false,
                            description: None,
                        })),
                        Some(aliased) => Some(Either::Right(ObjectField {
                            key: aliased.name.into(),
                            optional: field_type.is_never(),
                            r#type: field_type,
                            readonly: false,
                            description: None,
                        })),
                    }
                }
                _ => None,
            });

    let types_for_fragments = selection_set
        .selections
        .iter()
        .flat_map(move |sel| match sel {
            Selection::Field(_) => vec![],
            Selection::FragmentSpread(ref fragment) => {
                let fragment_def = context
                    .fragment_definitions
                    .get(fragment.fragment_name.name)
                    .expect("Type system error");
                if check_fragment_condition(
                    context,
                    branch.parent_obj,
                    fragment_def.type_condition.name,
                ) {
                    let fields =
                        get_fields_for_selection_set(context, &fragment_def.selection_set, &branch);
                    if check_skip_directive(branch, &fragment.directives) {
                        fields
                            .into_iter()
                            .map(|field| {
                                field.map(|field| ObjectField {
                                    optional: true,
                                    r#type: TSType::Never,
                                    ..field
                                })
                            })
                            .collect()
                    } else {
                        fields
                    }
                } else {
                    vec![]
                }
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => {
                    let fields =
                        get_fields_for_selection_set(context, &fragment.selection_set, branch);
                    if check_skip_directive(branch, &fragment.directives) {
                        fields
                            .into_iter()
                            .map(|field| {
                                field.map(|field| ObjectField {
                                    optional: true,
                                    r#type: TSType::Never,
                                    ..field
                                })
                            })
                            .collect()
                    } else {
                        fields
                    }
                }
                Some(ref cond) => {
                    if check_fragment_condition(context, branch.parent_obj, cond.name) {
                        let fields =
                            get_fields_for_selection_set(context, &fragment.selection_set, &branch);
                        if check_skip_directive(branch, &fragment.directives) {
                            fields
                                .into_iter()
                                .map(|field| {
                                    field.map(|field| ObjectField {
                                        optional: true,
                                        r#type: TSType::Never,
                                        ..field
                                    })
                                })
                                .collect()
                        } else {
                            fields
                        }
                    } else {
                        vec![]
                    }
                }
            },
        });
    let res = types_for_simple_fields
        .chain(types_for_fragments)
        .collect::<Vec<_>>();
    res
}

/// Examine directives and returns whether field should be skipped.
fn check_skip_directive<'src, S: Text<'src>>(
    branch: &BranchingCondition<S>,
    directives: &[Directive<'src>],
) -> bool {
    for directive in directives {
        match directive.name.name {
            "skip" => {
                let (_, skip) = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find(|(arg, _)| arg.name == "if")
                    .expect("Type system error");
                match skip {
                    Value::Variable(var) => {
                        let (_, var_value) = branch
                            .boolean_variables
                            .iter()
                            .find(|(name, _)| *name == var.name)
                            .expect("Type system error");
                        if *var_value {
                            return true;
                        }
                    }
                    Value::BooleanValue(b) => {
                        if b.value {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            "include" => {
                let (_, include) = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find(|(arg, _)| arg.name == "if")
                    .expect("Type system error");
                match include {
                    Value::Variable(var) => {
                        let (_, var_value) = branch
                            .boolean_variables
                            .iter()
                            .find(|(name, _)| *name == var.name)
                            .expect("Type system error");
                        if !var_value {
                            return true;
                        }
                    }
                    Value::BooleanValue(b) => {
                        if !b.value {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    false
}

/// Returns whether given object type implements given condition.
fn check_fragment_condition<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    object_def: &ObjectDefinition<S, Pos>,
    cond: &str,
) -> bool {
    let cond_type = context.schema.get_type(cond).expect("Type system error");
    match **cond_type {
        TypeDefinition::Object(ref obj) => object_def.name.inner_ref() == &*obj.name,
        TypeDefinition::Interface(ref interface) => object_def
            .interfaces
            .iter()
            .any(|imp| imp.inner_ref() == &*interface.name),
        TypeDefinition::Union(ref union) => union
            .possible_types
            .iter()
            .any(|mem| mem.inner_ref() == &*object_def.name),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            false
        }
    }
}

pub fn get_type_for_variable_definitions<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    definitions: &VariablesDefinition,
) -> TSType {
    let types_for_each_field: Vec<_> = definitions
        .definitions
        .iter()
        .map(|def| {
            let property_name = def.name.name;
            let field_type = get_ts_type_of_type(&def.r#type, |name| {
                TSType::NamespaceMember(
                    context.options.schema_root_namespace.clone(),
                    name.name.to_string(),
                )
            });
            TSType::object(vec![(property_name, field_type, None)])
        })
        .collect();

    if types_for_each_field.is_empty() {
        TSType::empty_object()
    } else {
        ts_intersection(types_for_each_field)
    }
}

/// Map given Type to TSType.
fn map_to_tstype<Str, OriginalNode>(
    ty: &Type<Str, OriginalNode>,
    mapper: impl FnOnce(&NamedType<Str, OriginalNode>) -> TSType,
) -> TSType {
    let (res, nullable) = map_to_tstype_impl(ty, mapper);
    if nullable {
        ts_union(vec![res, TSType::Null])
    } else {
        res
    }
}

fn map_to_tstype_impl<Str, OriginalNode>(
    ty: &Type<Str, OriginalNode>,
    mapper: impl FnOnce(&NamedType<Str, OriginalNode>) -> TSType,
) -> (TSType, bool) {
    match ty {
        Type::Named(name) => (mapper(name), true),
        Type::List(inner) => (TSType::Array(Box::new(map_to_tstype(&inner, mapper))), true),
        Type::NonNull(inner) => {
            let (inner_ty, _) = map_to_tstype_impl(&inner, mapper);
            (inner_ty, false)
        }
    }
}
