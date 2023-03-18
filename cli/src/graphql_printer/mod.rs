use crate::{
    ast::{
        base::Variable,
        directive::Directive,
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, VariableDefinition,
            VariablesDefinition,
        },
        selection_set::{Selection, SelectionSet},
        type_system::{
            ArgumentsDefinition, DirectiveDefinition, EnumValueDefinition, FieldDefinition,
            InputValueDefinition, SchemaDefinition, SchemaExtension, TypeDefinition, TypeExtension,
            TypeSystemDefinition, TypeSystemDefinitionOrExtension,
        },
        value::Arguments,
        OperationDocument, TypeSystemDocument, TypeSystemOrExtensionDocument,
    },
    source_map_writer::writer::SourceMapWriter,
};

mod base;

pub trait GraphQLPrinter {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter);
}

impl GraphQLPrinter for OperationDocument<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        for def in self.definitions.iter() {
            def.print_graphql(writer);
            writer.write("\n");
        }
    }
}

impl GraphQLPrinter for ExecutableDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            ExecutableDefinition::OperationDefinition(op) => {
                op.print_graphql(writer);
            }
            ExecutableDefinition::FragmentDefinition(fragment) => {
                fragment.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for OperationDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write(self.operation_type.as_str());
        if let Some(ref name) = self.name {
            writer.write(" ");
            name.print_graphql(writer);
        }
        if let Some(ref def) = self.variables_definition {
            def.print_graphql(writer);
        }
        for d in self.directives.iter() {
            writer.write(" ");
            d.print_graphql(writer);
        }
        writer.write(" ");
        self.selection_set.print_graphql(writer);
    }
}

impl GraphQLPrinter for VariablesDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("(");
        let len = self.definitions.len();
        if len < 2 {
            for d in self.definitions.iter() {
                d.print_graphql(writer);
            }
        } else {
            writer.write("\n");
            writer.indent();
            for (idx, d) in self.definitions.iter().enumerate() {
                if idx > 0 {
                    writer.write(",\n")
                }
                d.print_graphql(writer);
            }
            writer.dedent();
            writer.write("\n");
        }
        writer.write(")");
    }
}

impl GraphQLPrinter for VariableDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        self.name.print_graphql(writer);
        writer.write(": ");
        self.r#type.print_graphql(writer);
    }
}

impl GraphQLPrinter for Variable<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write_for("$", self);
        writer.write(self.name);
    }
}

impl GraphQLPrinter for Directive<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("@");
        self.name.print_graphql(writer);
        if let Some(ref arguments) = self.arguments {
            arguments.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for Arguments<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("(");
        if self.arguments.len() < 2 {
            for (name, value) in self.arguments.iter() {
                name.print_graphql(writer);
                writer.write(": ");
                value.print_graphql(writer);
            }
        } else {
            writer.write("\n");
            writer.indent();
            for (name, value) in self.arguments.iter() {
                name.print_graphql(writer);
                writer.write(": ");
                value.print_graphql(writer);
                writer.write("\n");
            }
            writer.dedent();
        }
        writer.write(")");
    }
}

impl GraphQLPrinter for SelectionSet<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("{\n");
        writer.indent();
        for selection in self.selections.iter() {
            selection.print_graphql(writer);
            writer.write("\n");
        }
        writer.dedent();
        writer.write("}");
    }
}

impl GraphQLPrinter for Selection<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Selection::Field(field) => {
                if let Some(ref name) = field.alias {
                    name.print_graphql(writer);
                    writer.write(": ");
                }
                field.name.print_graphql(writer);
                if let Some(ref arguments) = field.arguments {
                    arguments.print_graphql(writer);
                }
                for d in field.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if let Some(ref selection_set) = field.selection_set {
                    writer.write(" ");
                    selection_set.print_graphql(writer);
                }
            }
            Selection::FragmentSpread(spread) => {
                writer.write("... ");
                spread.fragment_name.print_graphql(writer);
                for d in spread.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
            }
            Selection::InlineFragment(fragment) => {
                writer.write("... ");
                if let Some(ref type_condition) = fragment.type_condition {
                    writer.write("on ");
                    type_condition.print_graphql(writer);
                    writer.write(" ");
                }
                for d in fragment.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                fragment.selection_set.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for FragmentDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("fragment ");
        self.name.print_graphql(writer);
        writer.write(" on ");
        self.type_condition.print_graphql(writer);
        writer.write(" ");
        todo!()
    }
}

impl GraphQLPrinter for TypeSystemOrExtensionDocument<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        for def in self.definitions.iter() {
            def.print_graphql(writer);
            writer.write("\n");
        }
    }
}

impl GraphQLPrinter for TypeSystemDocument<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        for def in self.definitions.iter() {
            def.print_graphql(writer);
            writer.write("\n");
        }
    }
}

impl GraphQLPrinter for TypeSystemDefinitionOrExtension<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TypeSystemDefinitionOrExtension::SchemaDefinition(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinitionOrExtension::TypeDefinition(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinitionOrExtension::DirectiveDefinition(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinitionOrExtension::SchemaExtension(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinitionOrExtension::TypeExtension(def) => {
                def.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for TypeSystemDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TypeSystemDefinition::SchemaDefinition(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                def.print_graphql(writer);
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                def.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for SchemaDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        if let Some(ref description) = self.description {
            description.print_graphql(writer);
            writer.write("\n");
        }
        writer.write("schema ");
        for d in self.directives.iter() {
            d.print_graphql(writer);
        }
        writer.write("{\n");
        writer.indent();
        for (operation_type, name) in self.definitions.iter() {
            writer.write(operation_type.as_str());
            writer.write(": ");
            name.print_graphql(writer);
            writer.write("\n");
        }
        writer.dedent();
        writer.write("}\n");
    }
}

impl GraphQLPrinter for TypeDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TypeDefinition::Scalar(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write("scalar ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                writer.write("\n");
            }
            TypeDefinition::Object(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write_for("type ", &def.type_keyword);
                def.name.print_graphql(writer);
                if !def.implements.is_empty() {
                    writer.write(" implements");
                    for implements in def.implements.iter() {
                        writer.write(" & ");
                        implements.print_graphql(writer);
                    }
                }
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeDefinition::Interface(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write("interface ");
                def.name.print_graphql(writer);
                if !def.implements.is_empty() {
                    writer.write(" implements");
                    for implements in def.implements.iter() {
                        writer.write(" & ");
                        implements.print_graphql(writer);
                    }
                }
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeDefinition::Union(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write("union ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                writer.write(" =");
                for f in def.members.iter() {
                    writer.write(" | ");
                    f.print_graphql(writer);
                }
                writer.write("\n");
            }
            TypeDefinition::Enum(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write("enum ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.values.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for v in def.values.iter() {
                        v.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeDefinition::InputObject(def) => {
                if let Some(ref description) = def.description {
                    description.print_graphql(writer);
                    writer.write("\n");
                }
                writer.write("input ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
        }
    }
}

impl GraphQLPrinter for FieldDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        if let Some(ref description) = self.description {
            description.print_graphql(writer);
            writer.write("\n");
        }
        self.name.print_graphql(writer);
        if let Some(ref arguments) = self.arguments {
            arguments.print_graphql(writer);
        }
        writer.write(": ");
        self.r#type.print_graphql(writer);
        for d in self.directives.iter() {
            writer.write(" ");
            d.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for ArgumentsDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("(");
        for (idx, input) in self.input_values.iter().enumerate() {
            if idx > 0 {
                writer.write(", ");
            }
            input.print_graphql(writer);
        }
        writer.write(")");
    }
}

impl GraphQLPrinter for EnumValueDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        if let Some(ref description) = self.description {
            description.print_graphql(writer);
            writer.write("\n");
        }
        self.name.print_graphql(writer);
        for d in self.directives.iter() {
            writer.write(" ");
            d.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for InputValueDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        if let Some(ref description) = self.description {
            description.print_graphql(writer);
            writer.write("\n");
        }
        self.name.print_graphql(writer);
        writer.write(": ");
        self.r#type.print_graphql(writer);
        if let Some(ref default_value) = self.default_value {
            writer.write(" = ");
            default_value.print_graphql(writer);
        }
        for d in self.directives.iter() {
            writer.write(" ");
            d.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for DirectiveDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        if let Some(ref description) = self.description {
            description.print_graphql(writer);
            writer.write("\n");
        }
        writer.write("directive @");
        self.name.print_graphql(writer);
        if let Some(ref arguments_definition) = self.arguments {
            arguments_definition.print_graphql(writer);
        }
        if let Some(ref token) = self.repeatable {
            writer.write(" ");
            token.print_graphql(writer);
        }
        writer.write(" on");
        for loc in self.locations.iter() {
            writer.write(" | ");
            loc.print_graphql(writer);
        }
        writer.write("\n");
    }
}

impl GraphQLPrinter for SchemaExtension<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("extend schema ");
        for d in self.directives.iter() {
            d.print_graphql(writer);
        }
        writer.write("{\n");
        writer.indent();
        for (operation_type, name) in self.definitions.iter() {
            writer.write(operation_type.as_str());
            writer.write(": ");
            name.print_graphql(writer);
            writer.write("\n");
        }
        writer.dedent();
        writer.write("}\n");
    }
}

impl GraphQLPrinter for TypeExtension<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TypeExtension::Scalar(def) => {
                writer.write("extend scalar ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                writer.write("\n");
            }
            TypeExtension::Object(def) => {
                writer.write("extend type ");
                def.name.print_graphql(writer);
                if !def.implements.is_empty() {
                    writer.write(" implements");
                    for implements in def.implements.iter() {
                        writer.write(" & ");
                        implements.print_graphql(writer);
                    }
                }
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeExtension::Interface(def) => {
                writer.write("extend interface ");
                def.name.print_graphql(writer);
                if !def.implements.is_empty() {
                    writer.write(" implements");
                    for implements in def.implements.iter() {
                        writer.write(" & ");
                        implements.print_graphql(writer);
                    }
                }
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeExtension::Union(def) => {
                writer.write("extend union ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                writer.write(" =");
                for f in def.members.iter() {
                    writer.write(" | ");
                    f.print_graphql(writer);
                }
                writer.write("\n");
            }
            TypeExtension::Enum(def) => {
                writer.write("extend enum ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.values.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for v in def.values.iter() {
                        v.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
            TypeExtension::InputObject(def) => {
                writer.write("extend input ");
                def.name.print_graphql(writer);
                for d in def.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if !def.fields.is_empty() {
                    writer.write(" {\n");
                    writer.indent();
                    for f in def.fields.iter() {
                        f.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                    writer.write("}")
                }
                writer.write("\n");
            }
        }
    }
}
