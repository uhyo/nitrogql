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
        get_type_for_selection_set(
            &self.selection_set,
            TSType::NamespaceMember(
                options.schema_root_namespace.clone(),
                match self.operation_type {
                    OperationType::Query => options.schema_root_types.query.clone(),
                    OperationType::Mutation => options.schema_root_types.mutation.clone(),
                    OperationType::Subscription => options.schema_root_types.subscription.clone(),
                },
            ),
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

        let parent_type = TSType::IndexType(
            Box::new(TSType::TypeVariable(options.schema_root_namespace.clone())),
            Box::new(TSType::StringLiteral(self.type_condition.name.to_owned())),
        );
        get_type_for_selection_set(&self.selection_set, parent_type).print_type(writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(selection_set: &SelectionSet, parent_type: TSType) -> TSType {
    let types_for_each_field = selection_set
        .selections
        .iter()
        .map(|sel| match sel {
            Selection::Field(ref field) => {
                let property_name = field.alias.unwrap_or_else(|| field.name.clone()).name;
                let key = TSType::StringLiteral(field.name.name.to_owned());
                let field_type = TSType::IndexType(Box::new(parent_type.clone()), Box::new(key));
                let field_sel_type = match field.selection_set {
                    None => field_type,
                    Some(ref set) => get_type_for_selection_set(set, field_type),
                };
                TSType::object(vec![(property_name, field_sel_type, None)])
            }
            Selection::FragmentSpread(ref fragment) => {
                TSType::TypeVariable(fragment.fragment_name.name.to_owned())
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_type_for_selection_set(&fragment.selection_set, parent_type.clone()),
                Some(ref cond) =>
                // TODO: this isn't correct
                {
                    get_type_for_selection_set(
                        &fragment.selection_set,
                        TSType::TypeVariable(cond.name.to_owned()),
                    )
                }
            },
        })
        .collect();
    wrap_with_keepdoc(parent_type, ts_intersection(types_for_each_field))
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

/// Wraps object type with the KeepDoc utility.
fn wrap_with_keepdoc(original_type: TSType, wrapped_type: TSType) -> TSType {
    TSType::TypeFunc(
        Box::new(TSType::TypeVariable("KeepDoc".into())),
        vec![original_type, wrapped_type],
    )
}
