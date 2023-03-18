use crate::{
    ts_types::{ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type},
    utils::interface_implementers,
};
use nitrogql_ast::{
    operations::{
        ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
        VariablesDefinition,
    },
    r#type::{NamedType, Type},
    selection_set::{Selection, SelectionSet},
    type_system::{ObjectTypeDefinition, TypeDefinition},
    value::StringValue,
    OperationDocument,
};
use nitrogql_checker::operation_checker::direct_fields_of_output_type;
use nitrogql_json_printer::print_to_json_string;
use nitrogql_utils::capitalize;
use sourcemap_writer::SourceMapWriter;

use super::{
    super::ts_types::{ts_types_util::ts_intersection, TSType},
    printer::QueryTypePrinterContext,
};

pub trait TypePrinter {
    fn print_type(&self, options: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter);
}

impl TypePrinter for OperationDocument<'_> {
    fn print_type(&self, options: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter) {
        for d in self.definitions.iter() {
            d.print_type(options, writer);
        }
    }
}

impl TypePrinter for ExecutableDefinition<'_> {
    fn print_type(&self, options: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter) {
        match self {
            ExecutableDefinition::OperationDefinition(ref op) => op.print_type(options, writer),
            ExecutableDefinition::FragmentDefinition(ref fragment) => {
                fragment.print_type(options, writer);
            }
        }
    }
}

/// Calculates a variable name for given operation.
pub fn operation_variable_name(
    context: &QueryTypePrinterContext,
    operation: &OperationDefinition,
) -> String {
    let capitalized_name = operation
        .name
        .map(|name| capitalize(&name.name))
        .unwrap_or(String::new());
    format!(
        "{}{}",
        capitalized_name,
        match operation.operation_type {
            OperationType::Query => &context.options.query_variable_suffix,
            OperationType::Mutation => &context.options.mutation_variable_suffix,
            OperationType::Subscription => &context.options.subscription_variable_suffix,
        }
    )
}

impl TypePrinter for OperationDefinition<'_> {
    fn print_type(&self, context: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter) {
        let operation = self
            .name
            .map(|name| capitalize(&name.name))
            .unwrap_or(String::new());
        let operation_type_name = format!(
            "{}{}",
            operation,
            match self.operation_type {
                OperationType::Query => &context.options.query_result_suffix,
                OperationType::Mutation => &context.options.mutation_result_suffix,
                OperationType::Subscription => &context.options.subscription_result_suffix,
            }
        );

        writer.write("type ");
        writer.write_for(&operation_type_name, &self.name_pos());
        writer.write_for(" = ", &self.selection_set);

        let parent_type = context
            .schema_definitions
            .root_type(self.operation_type)
            .expect("Type system error");
        let parent_type = NamedType {
            name: parent_type.name().clone(),
        };
        get_type_for_selection_set(context, &self.selection_set, &parent_type).print_type(writer);
        writer.write(";\n\n");

        let input_variable_type = self
            .variables_definition
            .as_ref()
            .map_or(TSType::empty_object(), get_type_for_variable_definitions);
        let input_variable_name = format!("{}{}", operation, context.options.variable_type_suffix);

        writer.write("type ");
        writer.write_for(&input_variable_name, &self.name_pos());
        writer.write(" = ");
        input_variable_type.print_type(writer);
        writer.write(";\n\n");

        let var_name = operation_variable_name(context, self);

        writer.write("export const ");
        writer.write_for(&var_name, &self.name_pos());
        writer.write_for(": ", &self.selection_set);
        writer.write("TypedDocumentNode<");
        writer.write(&operation_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        if !context.options.print_values {
            writer.write(">;\n\n");
            return;
        }
        writer.write("> = ");
        // To follow the community conventions, generated JSON has only one operation in it
        let this_document = context
            .operation
            .definitions
            .iter()
            .filter(|def| match def {
                ExecutableDefinition::FragmentDefinition(_) => true,
                ExecutableDefinition::OperationDefinition(op) => {
                    op.name.map(|ident| ident.name) == self.name.map(|ident| ident.name)
                }
            })
            .collect::<Vec<_>>();
        writer.write(&print_to_json_string(&this_document[..]));
        writer.write(";\n\n");
    }
}

impl TypePrinter for FragmentDefinition<'_> {
    fn print_type(&self, context: &QueryTypePrinterContext, writer: &mut impl SourceMapWriter) {
        writer.write("export type ");
        writer.write_for(&self.name.name, self);
        writer.write(" = ");

        let parent_type = NamedType {
            name: self.type_condition.clone(),
        };
        let fragment_type = get_type_for_selection_set(context, &self.selection_set, &parent_type);
        fragment_type.print_type(writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(
    context: &QueryTypePrinterContext,
    selection_set: &SelectionSet,
    parent_type: &NamedType,
) -> TSType {
    let parent_type_def = context
        .schema_definitions
        .types
        .get(parent_type.name.name)
        .expect("Type system error");
    match parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            panic!("Type system error")
        }
        TypeDefinition::Object(obj_def) => {
            get_object_type_for_selection_set(context, selection_set, obj_def)
        }
        TypeDefinition::Interface(interface_def) => {
            let object_defs = interface_implementers(context.schema, interface_def.name.name);
            ts_union(
                object_defs.map(|obj_def| {
                    get_object_type_for_selection_set(context, selection_set, obj_def)
                }),
            )
        }
        TypeDefinition::Union(union_def) => {
            let object_defs = union_def.members.iter().map(|member| {
                match context.schema_definitions.types.get(member.name) {
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
    parent_type: &ObjectTypeDefinition,
) -> TSType {
    let actual = TSType::object(get_fields_for_selection_set(
        context,
        selection_set,
        parent_type,
    ));
    let schema_type = TSType::NamespaceMember(
        context.options.schema_root_namespace.clone(),
        parent_type.name.name.to_owned(),
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
    parent_type: &'a ObjectTypeDefinition,
) -> impl Iterator<Item = (&'a str, TSType, Option<StringValue>)> + 'a {
    let parent_type_def = context
        .schema_definitions
        .types
        .get(parent_type.name.name)
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
                            TSType::StringLiteral(parent_type.name.name.to_owned()),
                            None,
                        ));
                    }

                    let field_def = parent_fields
                        .iter()
                        .find(|parent_field| parent_field.name.name == field.name.name)
                        .expect("Type system error");

                    let field_sel_type =
                        map_to_tstype(&field_def.r#type, |ty| match field.selection_set {
                            None => TSType::NamespaceMember(
                                context.options.schema_root_namespace.clone(),
                                ty.name.name.to_owned(),
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
    object_def: &ObjectTypeDefinition,
    cond: &str,
) -> bool {
    let cond_type = context
        .schema_definitions
        .types
        .get(cond)
        .expect("Type system error");
    match cond_type {
        TypeDefinition::Object(obj) => object_def.name.name == obj.name.name,
        TypeDefinition::Interface(interface) => object_def
            .implements
            .iter()
            .any(|imp| imp.name == interface.name.name),
        TypeDefinition::Union(union) => union
            .members
            .iter()
            .any(|mem| mem.name == object_def.name.name),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            false
        }
    }
}

fn get_type_for_variable_definitions(definitions: &VariablesDefinition) -> TSType {
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
fn map_to_tstype(ty: &Type, mapper: impl FnOnce(&NamedType) -> TSType) -> TSType {
    let (res, nullable) = map_to_tstype_impl(ty, mapper);
    if nullable {
        ts_union(vec![res, TSType::Null])
    } else {
        res
    }
}

fn map_to_tstype_impl(ty: &Type, mapper: impl FnOnce(&NamedType) -> TSType) -> (TSType, bool) {
    match ty {
        Type::Named(name) => (mapper(name), true),
        Type::List(inner) => (
            TSType::Array(Box::new(map_to_tstype(&inner.r#type, mapper))),
            true,
        ),
        Type::NonNull(inner) => {
            let (inner_ty, _) = map_to_tstype_impl(&inner.r#type, mapper);
            (inner_ty, false)
        }
    }
}
