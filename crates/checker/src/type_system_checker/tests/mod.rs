use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_schema_extensions;

mod directives {
    use graphql_builtins::generate_builtins;
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };
    use nitrogql_parser::parse_type_system_document;
    use nitrogql_semantics::resolve_schema_extensions;

    // https://spec.graphql.org/draft/#sec-Type-System.Directives.Validation
    #[test]
    fn direct_recursion() {
        // A directive definition must not contain the use of a directive which references itself directly.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: Int! @heyhey(arg1: 1)) on ARGUMENT_DEFINITION | SCHEMA
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 8,
                    start: 9,
                    end: 18,
                    file: 0,
                    builtin: false,
                },
                message: RecursingDirective {
                    name: "heyhey",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn indirect_recursion() {
        // A directive definition must not contain the use of a directive which references itself indirectly by referencing a Type or Directive which transitively includes a reference to this directive.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: MyType!) on INPUT_OBJECT
        input MyType @heyhey(arg1: { foo: 3 }) {
            foo: Int!
        }

        directive @wow(arg1: MyType2!) on INPUT_FIELD_DEFINITION
        input MyType2 {
            foo: Int! @wow(arg1: { foo: 3 })
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 8,
                    start: 9,
                    end: 18,
                    file: 0,
                    builtin: false,
                },
                message: RecursingDirective {
                    name: "heyhey",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 6,
                    column: 8,
                    start: 148,
                    end: 157,
                    file: 0,
                    builtin: false,
                },
                message: RecursingDirective {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn reserved_name() {
        // The directive must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @__heyhey(arg1: MyType!) on OBJECT

        input MyType {
            foo: Int!
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 19,
                    start: 20,
                    end: 28,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn argument_reserved_name() {
        // The argument must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(__arg1: MyType!) on OBJECT

        input MyType {
            foo: Int!
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 26,
                    start: 27,
                    end: 33,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn argument_duplicated_name() {
        // The argument must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: MyType!, arg1: Int!) on OBJECT

        input MyType {
            foo: Int!
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 41,
                    start: 42,
                    end: 46,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "arg1",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn argument_input_type() {
        // The argument must accept a type where IsInputType(argumentType) returns true.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(
            arg1: MyType!
            arg2: MyInterface!
            arg3: MyUnion!
        ) on OBJECT
        type MyType {
            foo: String
        }
        interface MyInterface {
            foo: String
        }
        union MyUnion = MyType
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 18,
                    start: 46,
                    end: 52,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyType",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 3,
                    column: 18,
                    start: 72,
                    end: 83,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyInterface",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 4,
                    column: 18,
                    start: 103,
                    end: 110,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyUnion",
                },
                additional_info: [],
            },
        ]
        "###);

        let mut doc = parse_type_system_document(
            "
        directive @heyhey(arg1: MyScalar!, arg2: [InputType], arg3: MyEnum) on OBJECT
        scalar MyScalar
        input InputType { foo: Int! }
        enum MyEnum { A,B,C }
        ",
            0,
        )
        .unwrap();
        doc.extend(generate_builtins());
        let doc = resolve_schema_extensions(doc).unwrap();
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"[]");
    }
}

#[cfg(test)]
mod schemas {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn repeated_directives() {
        let doc = parse_to_type_system_document(
            "
        directive @wow on SCHEMA
        schema @wow @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 20,
                    start: 54,
                    end: 58,
                    file: 0,
                    builtin: false,
                },
                message: RepeatedDirective {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);

        let doc = parse_to_type_system_document(
            "
        directive @wow repeatable on SCHEMA
        schema @wow @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        []
        "###);
    }
    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
        directive @wow repeatable on INPUT_OBJECT
        schema @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 15,
                    start: 66,
                    end: 70,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod scalars {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            scalar __Int
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 19,
                    start: 20,
                    end: 25,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }
    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
        directive @wow repeatable on INPUT_OBJECT
        scalar Wow @wow
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 19,
                    start: 70,
                    end: 74,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod objects {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    // https://spec.graphql.org/draft/#sec-Objects.Type-Validation

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            type __MyType {
                foo: String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 17,
                    start: 18,
                    end: 26,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn duplicated_field() {
        // The field must have a unique name within that Object type; no two fields may share the same name.
        let doc = parse_to_type_system_document(
            "
            type MyType {
                foo: String!
                foo: Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 3,
                    column: 16,
                    start: 72,
                    end: 75,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "foo",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    /// The field must not have a name which begins with the characters "__" (two underscores).
    #[test]
    fn reserved_field_name() {
        let doc = parse_to_type_system_document(
            "
            type MyType {
                __foo: String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 16,
                    start: 43,
                    end: 48,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    /// The field must return a type where IsOutputType(fieldType) returns true.
    #[test]
    fn field_output_type() {
        let doc = parse_to_type_system_document(
            "
            type MyType {
                unknown_field: AAAAA
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 31,
                    start: 58,
                    end: 63,
                    file: 0,
                    builtin: false,
                },
                message: UnknownType {
                    name: "AAAAA",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn unknown_field_type() {}

    #[test]
    fn argument_check() {
        let doc = parse_to_type_system_document(
            "
            type MyType {
                field1(__arg: Int!): MyType!
                field1: MyType!
                field2(arg: Int!, arg: Int!): MyType!
                field3(arg: MyType!): Int!
                field4(
                    arg: Int!
                    @deprecated
                ): String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 23,
                    start: 50,
                    end: 55,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 3,
                    column: 16,
                    start: 88,
                    end: 94,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "field1",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 4,
                    column: 34,
                    start: 138,
                    end: 141,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "arg",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 5,
                    column: 28,
                    start: 186,
                    end: 192,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyType",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn argument_undefined_type() {
        let doc = parse_to_type_system_document(
            "
            type MyType {
                field1(arg: Input!): Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 28,
                    start: 55,
                    end: 60,
                    file: 0,
                    builtin: false,
                },
                message: UnknownType {
                    name: "Input",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    // An object type must be a super-set of all interfaces it implements
    #[test]
    fn implements_interfaces() {
        let doc = parse_to_type_system_document(
            "
            interface IFoo {
                foo: String
                foo2: Int!
            }
            interface IBar {
                bar(arg: Boolean!): Int!
            }
            type MyType implements IFoo & IBar {
                foo: String!
                foo2: Int
            }
            type MyType2 implements IBar {
                bar(differentArg: Boolean!): Int!
            }
            type MyType3 implements IBar {
                bar(arg: Boolean): Int!
            }
            type MyType4 implements IBar {
                bar(arg: Boolean!, arg2: Int!): Int
            }
            type MyType5 implements IBar {
                bar(arg: Boolean!, arg2: Int): Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 10,
                    column: 16,
                    start: 277,
                    end: 281,
                    file: 0,
                    builtin: false,
                },
                message: FieldTypeMisMatchWithInterface {
                    interface_name: "IFoo",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 8,
                    column: 17,
                    start: 200,
                    end: 206,
                    file: 0,
                    builtin: false,
                },
                message: InterfaceFieldNotImplemented {
                    field_name: "bar",
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 13,
                    column: 16,
                    start: 360,
                    end: 363,
                    file: 0,
                    builtin: false,
                },
                message: InterfaceArgumentNotImplemented {
                    argument_name: "arg",
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 13,
                    column: 20,
                    start: 364,
                    end: 376,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeNonNullAgainstInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 16,
                    column: 20,
                    start: 471,
                    end: 474,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeMisMatchWithInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 19,
                    column: 35,
                    start: 583,
                    end: 587,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeNonNullAgainstInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 19,
                    column: 16,
                    start: 564,
                    end: 567,
                    file: 0,
                    builtin: false,
                },
                message: FieldTypeMisMatchWithInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn undefined_directive_on_object() {
        let doc = parse_to_type_system_document(
            "
            type MyType @wow {
                foo: String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r#"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 25,
                    start: 26,
                    end: 29,
                    file: 0,
                    builtin: false,
                },
                message: UnknownDirective {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "#);
    }

    #[test]
    fn undefined_directive_on_field() {
        let doc = parse_to_type_system_document(
            "
            type MyType {
                foo: String! @wow
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 30,
                    start: 57,
                    end: 60,
                    file: 0,
                    builtin: false,
                },
                message: UnknownDirective {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod interfaces {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            interface __MyInterface {
                foo: String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 22,
                    start: 23,
                    end: 36,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn duplicated_field() {
        let doc = parse_to_type_system_document(
            "
            interface MyType {
                foo: String!
                foo: Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 3,
                    column: 16,
                    start: 77,
                    end: 80,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "foo",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    /// The field must not have a name which begins with the characters "__" (two underscores).
    #[test]
    fn reserved_field_name() {
        let doc = parse_to_type_system_document(
            "
            interface MyInterface {
                __foo: String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 16,
                    start: 53,
                    end: 58,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn field_output_type() {
        let doc = parse_to_type_system_document(
            "
            interface MyType {
                scalar_field: String!
                object_field: [MyObj]!
                interface_field: I
                union_field: XYZ
                enum_field: ABC!
                input_object_field: InputObj
            }
            type MyObj { foo: Int }
            interface I { foo: Int }
            union XYZ = X | Y | Z
            enum ABC { A B C }
            input InputObj {
                field: Int!
                field2: Boolean!
            }
            type X { x: Int! } type Y { y: Int! } type Z { z: Int! }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 7,
                    column: 36,
                    start: 246,
                    end: 254,
                    file: 0,
                    builtin: false,
                },
                message: NoInputType {
                    name: "InputObj",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn argument_check() {
        let doc = parse_to_type_system_document(
            "
            interface MyType {
                field1(__arg: Int!): MyType!
                field1: MyType!
                field2(arg: Int!, arg: Int!): MyType!
                field3(arg: MyType!): Int!
                field4(
                    arg: Int!
                    @deprecated
                ): String!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 23,
                    start: 55,
                    end: 60,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 3,
                    column: 16,
                    start: 93,
                    end: 99,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "field1",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 4,
                    column: 34,
                    start: 143,
                    end: 146,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "arg",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 5,
                    column: 28,
                    start: 191,
                    end: 197,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyType",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    // An object type must be a super-set of all interfaces it implements
    #[test]
    fn implements_interfaces() {
        let doc = parse_to_type_system_document(
            "
            interface IFoo {
                foo: String
                foo2: Int!
            }
            interface IBar {
                bar(arg: Boolean!): Int!
            }
            interface MyType implements IFoo & IBar {
                foo: String!
                foo2: Int
            }
            interface MyType2 implements IBar {
                bar(differentArg: Boolean!): Int!
            }
            interface MyType3 implements IBar {
                bar(arg: Boolean): Int!
            }
            interface MyType4 implements IBar {
                bar(arg: Boolean!, arg2: Int!): Int
            }
            interface MyType5 implements IBar {
                bar(arg: Boolean!, arg2: Int): Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 10,
                    column: 16,
                    start: 282,
                    end: 286,
                    file: 0,
                    builtin: false,
                },
                message: FieldTypeMisMatchWithInterface {
                    interface_name: "IFoo",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 8,
                    column: 22,
                    start: 205,
                    end: 211,
                    file: 0,
                    builtin: false,
                },
                message: InterfaceFieldNotImplemented {
                    field_name: "bar",
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 13,
                    column: 16,
                    start: 370,
                    end: 373,
                    file: 0,
                    builtin: false,
                },
                message: InterfaceArgumentNotImplemented {
                    argument_name: "arg",
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 13,
                    column: 20,
                    start: 374,
                    end: 386,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeNonNullAgainstInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 16,
                    column: 20,
                    start: 486,
                    end: 489,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeMisMatchWithInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 19,
                    column: 35,
                    start: 603,
                    end: 607,
                    file: 0,
                    builtin: false,
                },
                message: ArgumentTypeNonNullAgainstInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 19,
                    column: 16,
                    start: 584,
                    end: 587,
                    file: 0,
                    builtin: false,
                },
                message: FieldTypeMisMatchWithInterface {
                    interface_name: "IBar",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn no_implement_self() {
        let doc = parse_to_type_system_document(
            "
            interface I implements I {
                field: [Int!]!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 35,
                    start: 36,
                    end: 37,
                    file: 0,
                    builtin: false,
                },
                message: NoImplementSelf,
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod unions {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            union __U = A | B | C
            type A { a: Int! }
            type B { a: Int! }
            type C { a: Int! }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 18,
                    start: 19,
                    end: 22,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }
    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
            directive @wow repeatable on INPUT_OBJECT
            union Wow @wow = A | B
            type A { a: Int! }
            type B { a: Int! }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 22,
                    start: 77,
                    end: 81,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "wow",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn unknown_member() {
        let doc = parse_to_type_system_document(
            "
            union ABC = A | B | C
            type A { a: Int! }
            type B { a: Int! }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 32,
                    start: 33,
                    end: 34,
                    file: 0,
                    builtin: false,
                },
                message: UnknownType {
                    name: "C",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn non_object_member() {
        let doc = parse_to_type_system_document(
            "
            union Test = Int | Obj | Union | Enum | Input
            type Obj { a: Int! }
            union Union = Obj
            enum Enum { A B C }
            input Input {
                enum: Enum!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 25,
                    start: 26,
                    end: 29,
                    file: 0,
                    builtin: false,
                },
                message: NonObjectTypeUnionMember {
                    member_name: "Int",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 1,
                    column: 37,
                    start: 38,
                    end: 43,
                    file: 0,
                    builtin: false,
                },
                message: NonObjectTypeUnionMember {
                    member_name: "Union",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 1,
                    column: 45,
                    start: 46,
                    end: 50,
                    file: 0,
                    builtin: false,
                },
                message: NonObjectTypeUnionMember {
                    member_name: "Enum",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 1,
                    column: 52,
                    start: 53,
                    end: 58,
                    file: 0,
                    builtin: false,
                },
                message: NonObjectTypeUnionMember {
                    member_name: "Input",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod enums {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            enum __E { A }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 17,
                    start: 18,
                    end: 21,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }
    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
            directive @x on ENUM_VALUE
            directive @y on ENUM
            directive @z on ENUM | ENUM_VALUE
            enum ABC @x@y@z {
                A @x
                B @y
                C @z
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 4,
                    column: 21,
                    start: 140,
                    end: 142,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "x",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 6,
                    column: 18,
                    start: 188,
                    end: 190,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "y",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod input_objects {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn reserved_name() {
        let doc = parse_to_type_system_document(
            "
            input __Obj { 
                field: Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 18,
                    start: 19,
                    end: 24,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
            directive @x on INPUT_OBJECT
            directive @y on INPUT_FIELD_DEFINITION
            directive @z on INPUT_OBJECT | INPUT_FIELD_DEFINITION
            input Input @x@y@z {
                field: Int! @x
                field2: Int! @y
                field3: Int! @z
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 4,
                    column: 26,
                    start: 185,
                    end: 187,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "y",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 5,
                    column: 28,
                    start: 220,
                    end: 222,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "x",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn field_definition() {
        let doc = parse_to_type_system_document(
            "
            input Input {
                __field: Int!
                field2: Int!
                field2: Int!
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 16,
                    start: 43,
                    end: 50,
                    file: 0,
                    builtin: false,
                },
                message: UnscoUnsco,
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 4,
                    column: 16,
                    start: 102,
                    end: 108,
                    file: 0,
                    builtin: false,
                },
                message: DuplicatedName {
                    name: "field2",
                },
                additional_info: [],
            },
        ]
        "###);
    }

    #[test]
    fn field_output_type() {
        let doc = parse_to_type_system_document(
            "
            input Input {
                scalar: Int!
                object: MyType!
                union: MyUnion!
                enum: MyEnum!
                input: Input
            }
            type MyType {
                field: Int!
            }
            union MyUnion = MyType
            enum MyEnum { A B C }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            CheckError {
                position: Pos {
                    line: 3,
                    column: 24,
                    start: 80,
                    end: 86,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyType",
                },
                additional_info: [],
            },
            CheckError {
                position: Pos {
                    line: 4,
                    column: 23,
                    start: 111,
                    end: 118,
                    file: 0,
                    builtin: false,
                },
                message: NoOutputType {
                    name: "MyUnion",
                },
                additional_info: [],
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod one_of {
    use insta::assert_debug_snapshot;

    use crate::type_system_checker::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn valid_one_of_input_object() {
        let doc = parse_to_type_system_document(
            "
            input UserBy @oneOf {
                id: ID
                email: String
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"[]");
    }

    #[test]
    fn one_of_field_must_be_nullable() {
        let doc = parse_to_type_system_document(
            "
            input UserBy @oneOf {
                id: ID!
                email: String
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 20,
                    start: 55,
                    end: 57,
                    file: 0,
                    builtin: false,
                },
                message: OneOfFieldNotNullable,
                additional_info: [],
            },
        ]
        ");
    }

    #[test]
    fn one_of_field_must_not_have_default_value() {
        let doc = parse_to_type_system_document(
            "
            input UserBy @oneOf {
                id: ID = \"admin\"
                email: String
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"
        [
            CheckError {
                position: Pos {
                    line: 2,
                    column: 25,
                    start: 60,
                    end: 67,
                    file: 0,
                    builtin: false,
                },
                message: OneOfFieldWithDefaultValue,
                additional_info: [],
            },
        ]
        ");
    }

    #[test]
    fn one_of_wrong_location() {
        let doc = parse_to_type_system_document(
            "
            type MyType @oneOf {
                foo: String
            }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r#"
        [
            CheckError {
                position: Pos {
                    line: 1,
                    column: 24,
                    start: 25,
                    end: 31,
                    file: 0,
                    builtin: false,
                },
                message: DirectiveLocationNotAllowed {
                    name: "oneOf",
                },
                additional_info: [],
            },
        ]
        "#);
    }

    #[test]
    fn one_of_applied_by_extension() {
        let doc = parse_to_type_system_document(
            "
            input UserBy {
                id: ID
                email: String!
            }
            extend input UserBy @oneOf
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"
        [
            CheckError {
                position: Pos {
                    line: 3,
                    column: 23,
                    start: 74,
                    end: 80,
                    file: 0,
                    builtin: false,
                },
                message: OneOfFieldNotNullable,
                additional_info: [],
            },
        ]
        ");
    }
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument<'_> {
    use graphql_builtins::generate_builtins;

    let mut doc = parse_type_system_document(source, 0).unwrap();
    doc.extend(generate_builtins());

    resolve_schema_extensions(doc).unwrap()
}
