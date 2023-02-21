use crate::{
    graphql_parser::ast::{
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariablesDefinition,
        },
        selection_set::{Selection, SelectionSet},
        OperationDocument,
    },
    source_map_writer::writer::SourceMapWriter,
    utils::capitalize::capitalize,
};

use super::{
    printer::QueryTypePrinterOptions, ts_types::TSType, ts_types_util::ts_intersection,
    type_to_ts_type::get_ts_type_of_type,
};

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
        writer.write_for(&query_type_name, self);
        writer.write_for(" = ", &self.selection_set);
        get_type_for_selection_set(
            &self.selection_set,
            TSType::TypeVariable(options.schema_root.clone()),
        )
        .print_type(options, writer);
        writer.write(";\n\n");

        let input_variable_type = self
            .variables_definition
            .as_ref()
            .map_or(TSType::Object(vec![]), get_type_for_variable_definitions);
        let input_variable_name = format!("{}{}", query_name, options.variable_type_suffix);

        writer.write("type ");
        writer.write_for(&input_variable_name, self);
        writer.write(" = ");
        input_variable_type.print_type(options, writer);
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
        writer.write_for(&query_var_name, self);
        writer.write(": TypedDocumentNode<");
        writer.write(&query_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
    }
}

impl TypePrinter for FragmentDefinition<'_> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        writer.write("type ");
        writer.write_for(&self.name.name, self);
        writer.write(" = ");

        let parent_type = TSType::IndexType(
            Box::new(TSType::TypeVariable(options.schema_root.clone())),
            Box::new(TSType::StringLiteral(self.type_condition.name.to_owned())),
        );
        get_type_for_selection_set(&self.selection_set, parent_type).print_type(options, writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(selection_set: &SelectionSet, parent_type: TSType) -> TSType {
    let types_for_each_field = selection_set
        .selections
        .iter()
        .map(|sel| match sel {
            Selection::Field(ref field) => {
                let property_name = field
                    .alias
                    .unwrap_or_else(|| field.name.clone())
                    .name
                    .to_owned();
                let key = TSType::StringLiteral(field.name.name.to_owned());
                let field_type = TSType::IndexType(Box::new(parent_type.clone()), Box::new(key));
                let field_sel_type = match field.selection_set {
                    None => field_type,
                    Some(ref set) => get_type_for_selection_set(set, field_type),
                };
                TSType::Object(vec![(property_name, field_sel_type)])
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
    ts_intersection(types_for_each_field)
}

fn get_type_for_variable_definitions(definitions: &VariablesDefinition) -> TSType {
    let types_for_each_field: Vec<_> = definitions
        .definitions
        .iter()
        .map(|def| {
            let property_name = def.name.name.to_owned();
            let field_type = get_ts_type_of_type(&def.r#type);
            TSType::Object(vec![(property_name, field_type)])
        })
        .collect();

    if types_for_each_field.is_empty() {
        TSType::Object(vec![])
    } else {
        ts_intersection(types_for_each_field)
    }
}
