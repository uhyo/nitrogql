use nitrogql_ast::{
    OperationDocumentExt,
    operation_ext::{ExecutableDefinitionExt, ImportDefinition, ImportTarget},
};
use sourcemap_writer::SourceMapWriter;

use crate::GraphQLPrinter;

impl GraphQLPrinter for OperationDocumentExt<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        for def in self.definitions.iter() {
            def.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for ExecutableDefinitionExt<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            ExecutableDefinitionExt::OperationDefinition(op) => {
                op.print_graphql(writer);
            }
            ExecutableDefinitionExt::FragmentDefinition(fragment) => {
                fragment.print_graphql(writer);
            }
            ExecutableDefinitionExt::Import(import) => {
                import.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for ImportDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("#import ");
        for (idx, target) in self.targets.iter().enumerate() {
            if idx > 0 {
                writer.write(", ");
            }
            match target {
                ImportTarget::Wildcard => {
                    writer.write("*");
                }
                ImportTarget::Name(name) => {
                    writer.write(name.name);
                }
            }
        }
        writer.write(" from ");
        self.path.print_graphql(writer);
        writer.write("\n");
    }
}
