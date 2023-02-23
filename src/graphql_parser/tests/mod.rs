#[cfg(test)]
mod operation {
    use insta::assert_snapshot;

    use crate::{
        graphql_parser::parser::parse_operation_document, graphql_printer::GraphQLPrinter,
        source_map_writer::just_writer::JustWriter,
    };

    #[test]
    fn simple_query() {
        assert_snapshot!(print_graphql(
            parse_operation_document("query { foo bar }").unwrap()
        ))
    }
    #[test]
    fn query_with_name() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "
                query sample {
                    foo
                    bar {
                        nested1
                        nested2
                    }
                 }
"
            )
            .unwrap()
        ))
    }
    #[test]
    fn query_with_variables() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "
                query sample($foo: Int!, $bar: Int!) {
                    foo @dedede(foo: $foo)
                 }
"
            )
            .unwrap()
        ))
    }

    #[test]
    fn query_with_directives() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "
                query sample($foo: [Int!]!) @a @b(c: D) {
                    foo
                    bar
                 }
"
            )
            .unwrap()
        ))
    }

    #[test]
    fn mutation_with_fragment() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "
                mutation($foo: Boolean) {
                    foo
                    ...Fragment
                }
                "
            )
            .unwrap()
        ))
    }
    #[test]
    fn subscription_with_inline_fragment() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "
                subscription a {
                    foo
                    ... on F {
                        bar
                        baz
                    }
                }
                "
            )
            .unwrap()
        ));
    }

    fn print_graphql<T: GraphQLPrinter>(value: T) -> String {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        value.print_graphql(&mut writer);
        result
    }
}

#[cfg(test)]
mod definition {
    use crate::{
        graphql_parser::parser::parse_type_system_document, graphql_printer::GraphQLPrinter,
        source_map_writer::just_writer::JustWriter,
    };
    use insta::assert_snapshot;

    #[test]
    fn scalar_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                scalar Int
                \"Description\"
                scalar String @string
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn object_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                type Foo {
                    foo: String!
                    bar: String!
                }
                type Bar implements I @wow {
                    \"this is foo\" foo: String @wow 
                }
                \"\"\"
                Description of type
                \"\"\"
                type Baz implements I & J {
                    func(arg1: Int): Int
                }
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn interface_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                interface MyI {
                    foo: String!
                    bar: [Int!]!
                }
                \"Gaooo 🦁\" interface _Lion implements MyI {
                    foobar: [[[Int]]]
                }
                interface aiu implements MyI & _Lion @wow
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn union_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                union ABC = A | B | C,
                \"XYZ Dragon Cannon\"union XYZ @x @y @z = | X | Y
                | Z
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn enum_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                enum Ehh { E H h }
                \"This\\nis\\nenum\" enum EEE {
                    E @desc(message: \"Hello\")
                    E2 @desc(message: null)
                    E3 @desc(message: false)
                }
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn input_object_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                input Foo {
                    foo: String!
                    bar: String!
                }
                input Bar @wow {
                    \"this is foo\" foo: String = \"aaa\"  @wow
                    bar: Bar
                }
                \"\"\"
                Description of type
                \"\"\"
                input Baz {
                    baz: Int! = 3
                }
                "
            )
            .unwrap()
        ));
    }

    fn print_graphql<T: GraphQLPrinter>(value: T) -> String {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        value.print_graphql(&mut writer);
        result
    }
}
