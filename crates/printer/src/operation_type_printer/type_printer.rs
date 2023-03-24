use std::collections::HashMap;

use crate::{
    ts_types::{ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type},
    utils::interface_implementers,
};
use graphql_type_system::{NamedType, ObjectDefinition, Schema, Type, TypeDefinition};
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
    visitor::OperationTypePrinterOptions,
};

pub struct QueryTypePrinterContext<'a, 'src> {
    pub options: &'a OperationTypePrinterOptions,
    pub schema: &'a Schema<&'src str, Pos>,
    pub operation: &'a OperationDocument<'src>,
    pub fragment_definitions: &'a HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

pub trait TypePrinter {
    fn print_type(&self, options: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter);
}

pub fn get_type_for_selection_set(
    context: &QueryTypePrinterContext,
    selection_set: &SelectionSet,
    parent_type: &NamedType<&str, Pos>,
) -> TSType {
    let parent_type_def = context
        .schema
        .get_type(&parent_type)
        .expect("Type system error");
    match parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            panic!("Type system error")
        }
        TypeDefinition::Object(obj_def) => {
            get_object_type_for_selection_set(context, selection_set, obj_def)
        }
        TypeDefinition::Interface(interface_def) => {
            let object_defs = interface_implementers(context.schema, &interface_def.name);
            ts_union(
                object_defs.map(|obj_def| {
                    get_object_type_for_selection_set(context, selection_set, obj_def)
                }),
            )
        }
        TypeDefinition::Union(union_def) => {
            let object_defs = union_def.possible_types.iter().map(|member| {
                match context.schema.get_type(member) {
                    Some(TypeDefinition::Object(obj_def)) => obj_def,
                    _ => panic!("Type system error"),
                }
            });
            ts_union(
                object_defs.map(|obj_def| {
                    get_object_type_for_selection_set(context, selection_set, obj_def)
                }),
            )
        }
    }
}

fn get_object_type_for_selection_set(
    context: &QueryTypePrinterContext,
    selection_set: &SelectionSet,
    parent_type: &ObjectDefinition<&str, Pos>,
) -> TSType {
    let actual = TSType::object(get_fields_for_selection_set(
        context,
        selection_set,
        parent_type,
    ));
    let schema_type = TSType::NamespaceMember(
        context.options.schema_root_namespace.clone(),
        parent_type.name.to_string(),
    );
    TSType::TypeFunc(
        Box::new(TSType::NamespaceMember(
            context.options.schema_root_namespace.clone(),
            "__SelectionSet".into(),
        )),
        vec![schema_type, actual],
    )
}

/// Returns an iterator of object fields.
fn get_fields_for_selection_set<'a>(
    context: &'a QueryTypePrinterContext,
    selection_set: &'a SelectionSet,
    parent_type: &'a ObjectDefinition<&str, Pos>,
) -> impl Iterator<Item = (&'a str, TSType, Option<StringValue>)> + 'a {
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
                    let property_name = field.alias.unwrap_or_else(|| field.name.clone()).name;
                    if property_name == "__typename" {
                        // Special handling of reflection
                        return Some((
                            property_name,
                            TSType::StringLiteral(parent_type.name.to_string()),
                            None,
                        ));
                    }

                    let field_def = parent_fields
                        .iter()
                        .find(|parent_field| parent_field.name == field.name.name)
                        .expect("Type system error");

                    let field_sel_type =
                        map_to_tstype(&field_def.r#type, |ty| match field.selection_set {
                            None => TSType::NamespaceMember(
                                context.options.schema_root_namespace.clone(),
                                ty.to_string(),
                            ),
                            Some(ref set) => get_type_for_selection_set(context, set, ty),
                        });
                    Some((property_name, field_sel_type, None))
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
                        .collect()
                } else {
                    vec![]
                }
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_fields_for_selection_set(context, &fragment.selection_set, parent_type)
                    .collect(),
                Some(ref cond) => {
                    if check_fragment_condition(context, parent_type, cond.name) {
                        get_fields_for_selection_set(context, &fragment.selection_set, &parent_type)
                            .collect()
                    } else {
                        vec![]
                    }
                }
            },
        });
    types_for_simple_fields.chain(types_for_fragments)
}

/// Returns whether given object type implements given condition.
fn check_fragment_condition(
    context: &QueryTypePrinterContext,
    object_def: &ObjectDefinition<&str, Pos>,
    cond: &str,
) -> bool {
    let cond_type = context.schema.get_type(cond).expect("Type system error");
    match cond_type {
        TypeDefinition::Object(obj) => object_def.name == obj.name,
        TypeDefinition::Interface(interface) => object_def
            .interfaces
            .iter()
            .any(|imp| *imp == interface.name),
        TypeDefinition::Union(union) => union
            .possible_types
            .iter()
            .any(|mem| *mem == object_def.name),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            false
        }
    }
}

pub fn get_type_for_variable_definitions(definitions: &VariablesDefinition) -> TSType {
    let types_for_each_field: Vec<_> = definitions
        .definitions
        .iter()
        .map(|def| {
            let property_name = def.name.name;
            let field_type = get_ts_type_of_type(&def.r#type);
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
