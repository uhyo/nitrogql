#[cfg(test)]
mod operation {
    use insta::assert_snapshot;

    use crate::parser::parse_operation_document;
    use nitrogql_printer::GraphQLPrinter;
    use sourcemap_writer::JustWriter;

    #[test]
    fn simple_query() {
        assert_snapshot!(print_graphql(
            parse_operation_document("query { foo bar }", 0).unwrap()
        ))
    }
    #[test]
    fn shorthand_query() {
        assert_snapshot!(print_graphql(
            parse_operation_document("{ foo bar }", 0).unwrap()
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
",
                0
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
",
                0
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
",
                0
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
                ",
                0
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
                ",
                0
            )
            .unwrap()
        ));
    }
    #[test]
    fn descriptions() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                "This is a query."
                query sample(
                    "This is a variable."
                    $foo: Int!
                    """
                    This is another variable.
                    """
                    $bar: String = "bar" @deprecated
                ) {
                    foo
                    ...F
                }
                """
                This is a fragment.
                """
                fragment F on Foo {
                    bar
                }
                "#,
                0
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
",
                0
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
mod import_syntax {
    use insta::assert_snapshot;

    use crate::parser::parse_operation_document;
    use nitrogql_printer::GraphQLPrinter;
    use sourcemap_writer::JustWriter;

    #[test]
    fn specific_import() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #import Frag1 from "./frag1.graphql"
                query Foo {
                    foo {
                        ...Frag1
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }

    #[test]
    fn wildcard_import() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #import * from "./frag1.graphql"
                query Foo {
                    foo {
                        ...Frag1
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }

    #[test]
    fn multiple_imports() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #import Frag1, Frag2 from "./frag1.graphql"
                query Foo {
                    foo {
                        ...Frag1
                        ...Frag2
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }

    #[test]
    fn multiple_imports_with_wildcard() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #import Frag1, * from "./frag1.graphql"
                query Foo {
                    foo {
                        ...Frag1
                        ...Frag2
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }
    #[test]
    fn multiple_import_statements() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #import Frag1 from "./frag1.graphql"
                #import Frag2 from "./frag2.graphql"
                query Foo {
                    foo {
                        ...Frag1
                        ...Frag2
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }

    #[test]
    fn space_before_import() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                # import Frag1 from "./frag1.graphql"
                query Foo {
                    foo {
                        ...Frag1
                    }
                }
                "#,
                0
            )
            .unwrap()
        ));
    }

    #[test]
    fn resembling_comments() {
        assert_snapshot!(print_graphql(
            parse_operation_document(
                r#"
                #IMPORT * from "./frag1.graphql"
                #import Frag2 from './frag2.graphql'
                # import something!
                #import * from
                query Foo {
                    foo {
                        id
                    }
                }
                "#,
                0
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
                ",
                0
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
                ",
                0
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
                ",
                0
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
                ",
                0
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
                ",
                0
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
                ",
                0
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
                ",
                0
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
",
                0
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
mod file_index {
    use crate::parser::{parse_operation_document, parse_type_system_document};
    use nitrogql_ast::{
        operation_ext::ExecutableDefinitionExt,
        selection_set::Selection,
        type_system::{TypeDefinition, TypeSystemDefinitionOrExtension},
    };
    use nitrogql_error::PositionedError;

    #[test]
    fn operation_positions_have_given_file() {
        let doc = parse_operation_document("query Q { foo }", 3).unwrap();
        assert_eq!(doc.position.file, 3);
        let ExecutableDefinitionExt::OperationDefinition(op) = &doc.definitions[0] else {
            panic!("expected an operation");
        };
        assert_eq!(op.position.file, 3);
        assert_eq!(op.name.unwrap().position.file, 3);
        let Selection::Field(field) = &op.selection_set.selections[0] else {
            panic!("expected a field");
        };
        assert_eq!(field.name.position.file, 3);
        assert!(!field.name.position.builtin);
    }

    #[test]
    fn type_system_positions_have_given_file() {
        let doc = parse_type_system_document("type Query { foo: String }", 5).unwrap();
        let TypeSystemDefinitionOrExtension::TypeDefinition(TypeDefinition::Object(def)) =
            &doc.definitions[0]
        else {
            panic!("expected an object type");
        };
        assert_eq!(def.position.file, 5);
        assert_eq!(def.name.position.file, 5);
        assert_eq!(def.fields[0].name.position.file, 5);
    }

    #[test]
    fn parse_error_has_given_file() {
        let err: PositionedError = parse_operation_document("query {", 7).unwrap_err().into();
        assert_eq!(err.position().unwrap().file, 7);
        let err: PositionedError = parse_type_system_document("type {", 2).unwrap_err().into();
        assert_eq!(err.position().unwrap().file, 2);
    }

    #[test]
    fn parses_are_independent() {
        // The file index of one parse does not leak into the next.
        let first = parse_operation_document("{ foo }", 1).unwrap();
        let second = parse_operation_document("{ foo }", 2).unwrap();
        assert_eq!(first.position.file, 1);
        assert_eq!(second.position.file, 2);
    }
}
