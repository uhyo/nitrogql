#[cfg(test)]
mod tests {
    use crate::{
        extension_resolver::resolve_extensions, graphql_parser::parser::parse_type_system_document,
        graphql_printer::GraphQLPrinter, source_map_writer::just_writer::JustWriter,
    };
    use anyhow::Result;
    use insta::assert_snapshot;

    #[test]
    fn resolve() {
        let doc = parse_type_system_document(
            "
            schema { query: Query }

            extend schema { mutation: Mutation }

            type Query {
                foo: Int!
                bar(arg: String): Bar!
            }

            interface I {
                foo: Int!
            }

            extend type Query {
                baz: Baz!
            }

            extend interface I @heyhey

            union XYZ = | X | Y 
            enum ABC {A B}

            extend union XYZ = Z
            extend enum ABC @wow { C }

            extend input Input1 {
                p: Boolean!
            }

            input Input1 {
                i: Boolean!
                n: Boolean!
            }

            extend input Input1 {
                u: Boolean!
                t: Boolean!
            }

            ",
        )
        .unwrap();
        let resolved = resolve_extensions(doc).unwrap();

        assert_snapshot!(print_graphql(resolved));
    }

    fn print_graphql<T: GraphQLPrinter>(value: T) -> String {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        value.print_graphql(&mut writer);
        result
    }
}
