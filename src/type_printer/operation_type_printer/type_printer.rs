use crate::{
    ast::{
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariablesDefinition,
        },
        selection_set::{Selection, SelectionSet},
        OperationDocument,
    },
    source_map_writer::writer::SourceMapWriter,
    type_printer::ts_types::type_to_ts_type::get_ts_type_of_type,
    utils::capitalize::capitalize,
};

use super::super::ts_types::{ts_types_util::ts_intersection, TSType};
use super::printer::QueryTypePrinterOptions;

pub trait TypePrinter {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter);
}

impl TypePrinter for OperationDocument<'_> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        for d in self.definitions.iter() {
            d.print_type(options, writer);
        }
    }
}

impl TypePrinter for ExecutableDefinition<'_> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        match self {
            ExecutableDefinition::OperationDefinition(ref op) => op.print_type(options, writer),
            ExecutableDefinition::FragmentDefinition(ref fragment) => {
                fragment.print_type(options, writer);
            }
        }
    }
}

impl TypePrinter for OperationDefinition<'_> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        let query_name = self
            .name
            .map(|name| capitalize(&name.name))
            .unwrap_or(String::new());
        let query_type_name = format!(
            "{}{}",
            query_name,
            match self.operation_type {
                OperationType::Query => &options.query_result_suffix,
                OperationType::Mutation => &options.mutation_result_suffix,
                OperationType::Subscription => &options.subscription_result_suffix,
            }
        );

        writer.write("type ");
        writer.write_for(&query_type_name, &self.name_pos());
        writer.write_for(" = ", &self.selection_set);
        let parent_type = TSType::NamespaceMember(
            options.schema_root_namespace.clone(),
            match self.operation_type {
                OperationType::Query => options.schema_root_types.query.clone(),
                OperationType::Mutation => options.schema_root_types.mutation.clone(),
                OperationType::Subscription => options.schema_root_types.subscription.clone(),
            },
        );
        wrap_with_selection_set_helper(
            options,
            parent_type.clone(),
            get_type_for_selection_set(options, &self.selection_set, parent_type),
        )
        .print_type(writer);
        writer.write(";\n\n");

        let input_variable_type = self
            .variables_definition
            .as_ref()
            .map_or(TSType::empty_object(), get_type_for_variable_definitions);
        let input_variable_name = format!("{}{}", query_name, options.variable_type_suffix);

        writer.write("type ");
        writer.write_for(&input_variable_name, &self.name_pos());
        writer.write(" = ");
        input_variable_type.print_type(writer);
        writer.write(";\n\n");

        let query_var_name = format!(
            "{}{}",
            query_name,
            match self.operation_type {
                OperationType::Query => &options.query_variable_suffix,
                OperationType::Mutation => &options.mutation_variable_suffix,
                OperationType::Subscription => &options.subscription_variable_suffix,
            }
        );

        writer.write("export const ");
        writer.write_for(&query_var_name, &self.name_pos());
        writer.write_for(": ", &self.selection_set);
        writer.write("TypedDocumentNode<");
        writer.write(&query_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
    }
}

impl TypePrinter for FragmentDefinition<'_> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        writer.write("export type ");
        writer.write_for(&self.name.name, self);
        writer.write(" = ");

        let parent_type = TSType::NamespaceMember(
            options.schema_root_namespace.clone(),
            self.type_condition.name.to_owned(),
        );
        let fragment_type = wrap_with_selection_set_helper(
            options,
            parent_type.clone(),
            get_type_for_selection_set(options, &self.selection_set, parent_type),
        );
        fragment_type.print_type(writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(
    options: &QueryTypePrinterOptions,
    selection_set: &SelectionSet,
    parent_type: TSType,
) -> TSType {
    let types_for_each_field = selection_set
        .selections
        .iter()
        .map(|sel| match sel {
            Selection::Field(ref field) => {
                let property_name = field.alias.unwrap_or_else(|| field.name.clone()).name;
                let field_type =
                    wrap_with_selection_field_helper(options, parent_type.clone(), field.name.name);
                let field_type_var = TSType::TypeVariable("__1".into());
                let field_sel_type = match field.selection_set {
                    None => field_type,
                    Some(ref set) => TSType::Let {
                        var: "__1".to_owned(),
                        r#type: Box::new(field_type),
                        r#in: Box::new(wrap_with_selection_set_helper(
                            options,
                            field_type_var.clone(),
                            get_type_for_selection_set(options, set, field_type_var),
                        )),
                    },
                };
                TSType::object(vec![(property_name, field_sel_type, None)])
            }
            Selection::FragmentSpread(ref fragment) => {
                TSType::TypeVariable((&fragment.fragment_name).into())
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_type_for_selection_set(
                    options,
                    &fragment.selection_set,
                    parent_type.clone(),
                ),
                Some(ref cond) =>
                // TODO: this isn't correct
                {
                    get_type_for_selection_set(
                        options,
                        &fragment.selection_set,
                        TSType::TypeVariable(cond.into()),
                    )
                }
            },
        })
        .collect();
    ts_intersection(types_for_each_field)
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

/// Wraps object type with the __SelectionSet utility.
fn wrap_with_selection_set_helper(
    options: &QueryTypePrinterOptions,
    original_type: TSType,
    wrapped_type: TSType,
) -> TSType {
    TSType::TypeFunc(
        Box::new(TSType::NamespaceMember(
            options.schema_root_namespace.clone(),
            "__SelectionSet".into(),
        )),
        vec![original_type, wrapped_type],
    )
}

/// Wraps type with the __SelectionField utility.
fn wrap_with_selection_field_helper(
    options: &QueryTypePrinterOptions,
    parent: TSType,
    key: &str,
) -> TSType {
    TSType::TypeFunc(
        Box::new(TSType::NamespaceMember(
            options.schema_root_namespace.clone(),
            "__SelectionField".into(),
        )),
        vec![parent, TSType::StringLiteral(key.into())],
    )
}
