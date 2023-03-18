#[cfg(test)]
mod tests {
    use crate::resolve_extensions;
    use insta::assert_snapshot;
    use nitrogql_parser::parser::parse_type_system_document;
    use nitrogql_printer::GraphQLPrinter;
    use sourcemap_writer::JustWriter;

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

    #[test]
    fn error_handling() {
        let doc = parse_type_system_document(
            "
            extend schema { mutation: Mutation }
            type A { foo: Int! }
            ",
        )
        .unwrap();

        let resolved = resolve_extensions(doc).unwrap_err();
        assert_snapshot!(resolved.message.to_string());
    }

    #[test]
    fn error_handling2() {
        let doc = parse_type_system_document(
            "
            type A { foo: Int! }
            type A { bar: Int! }
            ",
        )
        .unwrap();

        let resolved = resolve_extensions(doc).unwrap_err();
        assert_snapshot!(resolved.message.to_string());
    }

    fn print_graphql<T: GraphQLPrinter>(value: T) -> String {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        value.print_graphql(&mut writer);
        result
    }
}
