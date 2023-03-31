use std::{collections::HashMap, convert::identity};

use crate::{
    ts_types::{ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type},
    utils::interface_implementers,
};
use graphql_type_system::{NamedType, ObjectDefinition, Schema, Text, Type, TypeDefinition};
use itertools::{Either, Itertools};
use nitrogql_ast::{
    base::Pos,
    operation::{FragmentDefinition, OperationDocument},
    selection_set::{Selection, SelectionSet},
    value::StringValue,
    variable::VariablesDefinition,
};
use nitrogql_semantics::direct_fields_of_output_type;
use sourcemap_writer::SourceMapWriter;

use super::{
    super::ts_types::{ts_types_util::ts_intersection, TSType},
    branching::BranchingCondition,
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
    selection_set: &SelectionSet,
    parent_type: &NamedType<S, Pos>,
) -> TSType {
    let parent_type_def = context
        .schema
        .get_type(&parent_type)
        .expect("Type system error");
    let branches = match **parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            panic!("Type system error")
        }
        TypeDefinition::Object(ref obj_def) => {
            let branch = BranchingCondition {
                parent_obj: obj_def,
            };
            Either::Left(std::iter::once(branch))
        }
        TypeDefinition::Interface(ref interface_def) => {
            let object_defs = interface_implementers(context.schema, &interface_def.name);
            let branches = object_defs.map(|obj_def| BranchingCondition {
                parent_obj: obj_def,
            });
            Either::Right(Either::Left(branches))
        }
        TypeDefinition::Union(ref union_def) => {
            let object_defs = union_def.possible_types.iter().map(|member| {
                context
                    .schema
                    .get_type(member)
                    .and_then(|def| def.as_object())
                    .expect("Type system error")
            });
            let branches = object_defs.map(|obj_def| BranchingCondition {
                parent_obj: obj_def,
            });
            Either::Right(Either::Right(branches))
        }
    };
    ts_union(
        branches.map(|branch| get_object_type_for_selection_set(context, selection_set, &branch)),
    )
}

fn get_object_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet,
    branch: &BranchingCondition<S>,
) -> TSType {
    let (unaliased, aliased): (Vec<_>, Vec<_>) =
        get_fields_for_selection_set(context, selection_set, branch.parent_obj)
            .into_iter()
            .partition_map(identity);
    let unaliased = TSType::object(unaliased);
    let aliased = TSType::object(aliased);
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
/// First element of returned tuple is type definitions for non-aliased fields.
/// Second element is for aliased fields.
fn get_fields_for_selection_set<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'a>,
    parent_type: &'a ObjectDefinition<S, Pos>,
) -> Vec<Either<(&'a str, TSType, Option<StringValue>), (&'a str, TSType, Option<StringValue>)>> {
    let parent_type_def = context
        .schema
        .get_type(&parent_type.name)
        .expect("Type system error");

    let parent_fields = direct_fields_of_output_type(parent_type_def).expect("Type system error");

    let types_for_simple_fields =
        selection_set
            .selections
            .iter()
            .filter_map(move |sel| match sel {
                Selection::Field(ref field) => {
                    let field_name = field.name.name;
                    let field_type = if field_name == "__typename" {
                        // Special handling of reflection
                        TSType::StringLiteral(parent_type.name.to_string())
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
                        None => Some(Either::Left((field_name, field_type, None))),
                        Some(aliased) => Some(Either::Right((aliased.name, field_type, None))),
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
                if check_fragment_condition(context, parent_type, fragment_def.type_condition.name)
                {
                    get_fields_for_selection_set(context, &fragment_def.selection_set, &parent_type)
                } else {
                    vec![]
                }
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_fields_for_selection_set(context, &fragment.selection_set, parent_type),
                Some(ref cond) => {
                    if check_fragment_condition(context, parent_type, cond.name) {
                        get_fields_for_selection_set(context, &fragment.selection_set, &parent_type)
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
