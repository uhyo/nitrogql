#[cfg(test)]
mod operation {
    use insta::assert_snapshot;

    use crate::parser::parse_operation_document;
    use nitrogql_printer::GraphQLPrinter;
    use sourcemap_writer::JustWriter;

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
    #[test]
    fn comments() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                "# Comment
query #comment
sample#comment
($foo: Int!, #comment
# # # # #
     $bar: Int!) {
    # comment
    foo
    # comment
    bar
} # comment
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
    use crate::parser::parse_type_system_document;
    use insta::assert_snapshot;
    use nitrogql_printer::GraphQLPrinter;
    use sourcemap_writer::JustWriter;

    #[test]
    fn scalar_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                scalar Int
                \"Description\"
                scalar String @string

                extend scalar Int @heyhey(foo: \"bar\")
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

                extend type Foo {
                    foo2: String!
                }
                extend type Bar implements J & K & L
                extend type Baz @wow
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

                extend interface MyI {
                    hey: Hey!
                }
                extend interface _Lion implements MyI2 @abc
                extend interface aiu @heyhey
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

                extend union ABC = D
                extend union XYZ @xyz
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

                extend enum Ehh { EEEEE, EEEE, \"heyhey\" EEE }
                extend enum EEE @eeeee
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

                extend input Foo {
                    foo2: String! @foo(num: 2)
                }
                extend input Bar @barber
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn directive_definition() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "
                directive @foo on QUERY | MUTATION | SUBSCRIPTION
                \"\"\"
                Hey \"\"hey\"\" \\\"\"\"Hey\\\"\"\"
                \"\"\"
                directive @bar repeatable on INPUT_FIELD_DEFINITION
                directive @baz(arg1: Int! @arg, arg2: Int! @arg) on INPUT_OBJECT
                "
            )
            .unwrap()
        ));
    }

    #[test]
    fn comments() {
        assert_snapshot!(print_graphql(
            parse_type_system_document(
                "# Comment
scalar Date # Comment
# Comment
# comment # comment ### comment

type #comment
Foo # comment
# comment
{
    # comment
    foo: String! # comment
    # comment
    bar: String! # comment
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
