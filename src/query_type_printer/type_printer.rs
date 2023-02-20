use graphql_parser::query::{
    Definition, Document, FragmentDefinition, Mutation, OperationDefinition, Query, Selection,
    SelectionSet, Subscription, TypeCondition, VariableDefinition,
};

use crate::{
    source_map_writer::{has_pos::HasPos, writer::SourceMapWriter},
    utils::capitalize::capitalize,
};

use super::{
    printer::QueryTypePrinterOptions, ts_types::TSType, ts_types_util::ts_intersection,
    type_to_ts_type::get_ts_type_of_type,
};

pub trait TypePrinter {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter);
}

impl TypePrinter for Document<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        for d in self.definitions.iter() {
            d.print_type(options, writer);
        }
    }
}

impl TypePrinter for Definition<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        match self {
            Definition::Operation(ref op) => match op {
                OperationDefinition::Query(ref query) => query.print_type(options, writer),
                OperationDefinition::Mutation(ref mutation) => mutation.print_type(options, writer),
                OperationDefinition::Subscription(ref subscription) => {
                    subscription.print_type(options, writer)
                }
                OperationDefinition::SelectionSet(ref selection_set) => {
                    let actual_query = Query {
                        position: selection_set.span.0,
                        name: None,
                        variable_definitions: vec![],
                        directives: vec![],
                        selection_set: selection_set.clone(),
                    };
                    actual_query.print_type(options, writer)
                }
            },
            Definition::Fragment(ref fragment) => {
                fragment.print_type(options, writer);
            }
        }
    }
}

impl TypePrinter for Query<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        print_operation_type(
            self,
            self.name.as_ref(),
            &self.selection_set,
            &self.variable_definitions,
            options,
            &options.query_result_suffix,
            &options.query_variable_suffix,
            writer,
        );
    }
}

impl TypePrinter for Subscription<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        print_operation_type(
            self,
            self.name.as_ref(),
            &self.selection_set,
            &self.variable_definitions,
            options,
            &options.subscription_result_suffix,
            &options.subscription_variable_suffix,
            writer,
        );
    }
}

impl TypePrinter for Mutation<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        print_operation_type(
            self,
            self.name.as_ref(),
            &self.selection_set,
            &self.variable_definitions,
            options,
            &options.mutation_result_suffix,
            &options.mutation_variable_suffix,
            writer,
        );
    }
}

fn print_operation_type(
    node: &impl HasPos,
    operation_name: Option<&String>,
    selection_set: &SelectionSet<'_, String>,
    variable_definitions: &[VariableDefinition<'_, String>],
    options: &QueryTypePrinterOptions,
    result_type_suffix: &str,
    variable_suffix: &str,
    writer: &mut impl SourceMapWriter,
) {
    let query_name = operation_name
        .map(|name| capitalize(name))
        .unwrap_or(String::new());
    let query_type_name = format!("{}{}", query_name, result_type_suffix);

    writer.write("type ");
    writer.write_for(&query_type_name, node);
    writer.write_for(" = ", selection_set);
    get_type_for_selection_set(
        &selection_set,
        TSType::TypeVariable(options.schema_root.clone()),
    )
    .print_type(options, writer);
    writer.write(";\n\n");

    let input_variable_type = get_type_for_variable_definitions(&variable_definitions);
    let input_variable_name = format!("{}{}", query_name, options.variable_type_suffix);

    writer.write("type ");
    writer.write_for(&input_variable_name, node);
    writer.write(" = ");
    input_variable_type.print_type(options, writer);
    writer.write(";\n\n");

    let query_var_name = format!("{}{}", query_name, variable_suffix);

    writer.write("export const ");
    writer.write_for(&query_var_name, node);
    writer.write(": TypedDocumentNode<");
    writer.write(&query_type_name);
    writer.write(", ");
    writer.write(&input_variable_name);
    writer.write(">;\n\n");
}

impl TypePrinter for FragmentDefinition<'_, String> {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
        writer.write("type ");
        writer.write_for(&self.name, self);
        writer.write(" = ");

        let TypeCondition::On(ref type_name) = self.type_condition;
        let parent_type = TSType::IndexType(
            Box::new(TSType::TypeVariable(options.schema_root.clone())),
            Box::new(TSType::StringLiteral(type_name.clone())),
        );
        get_type_for_selection_set(&self.selection_set, parent_type).print_type(options, writer);
        writer.write(";\n\n");
    }
}

fn get_type_for_selection_set(
    selection_set: &SelectionSet<'_, String>,
    parent_type: TSType,
) -> TSType {
    if selection_set.items.is_empty() {
        // empty selection set means selecting the whole parent
        return parent_type;
    }
    let types_for_each_field = selection_set
        .items
        .iter()
        .map(|sel| match sel {
            Selection::Field(ref field) => {
                let property_name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                let key = TSType::StringLiteral(field.name.clone());
                let field_type = TSType::IndexType(Box::new(parent_type.clone()), Box::new(key));
                let field_sel_type = get_type_for_selection_set(&field.selection_set, field_type);
                TSType::Object(vec![(property_name, field_sel_type)])
            }
            Selection::FragmentSpread(ref fragment) => {
                TSType::TypeVariable(fragment.fragment_name.clone())
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => get_type_for_selection_set(&fragment.selection_set, parent_type.clone()),
                Some(TypeCondition::On(ref cond)) =>
                // TODO: this isn't correct
                {
                    get_type_for_selection_set(
                        &fragment.selection_set,
                        TSType::TypeVariable(cond.clone()),
                    )
                }
            },
        })
        .collect();
    ts_intersection(types_for_each_field)
}

fn get_type_for_variable_definitions(definitions: &[VariableDefinition<'_, String>]) -> TSType {
    let types_for_each_field: Vec<_> = definitions
        .iter()
        .map(|def| {
            let property_name = def.name.clone();
            let field_type = get_ts_type_of_type(&def.var_type);
            TSType::Object(vec![(property_name, field_type)])
        })
        .collect();

    if types_for_each_field.is_empty() {
        TSType::Object(vec![])
    } else {
        ts_intersection(types_for_each_field)
    }
}
