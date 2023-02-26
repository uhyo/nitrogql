use crate::{
    extension_resolver::resolve_extensions,
    graphql_parser::{ast::TypeSystemDocument, parser::parse_type_system_document},
};

#[cfg(test)]
mod directives {
    use insta::assert_debug_snapshot;

    use crate::{
        extension_resolver::resolve_extensions,
        graphql_parser::parser::parse_type_system_document,
        type_system_validator::{check_type_system_document, tests::parse_to_type_system_document},
    };

    // https://spec.graphql.org/draft/#sec-Type-System.Directives.Validation
    #[test]
    fn direct_recursion() {
        // A directive definition must not contain the use of a directive which references itself directly.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: Int! @heyhey) on ARGUMENT_DEFINITION | SCHEMA
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            RecursingDirective {
                position: Pos {
                    line: 1,
                    column: 8,
                    builtin: false,
                },
                name: "heyhey",
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
        input MyType @heyhey {
            foo: Int!
        }

        directive @wow(arg1: MyType2!) on INPUT_FIELD_DEFINITION
        input MyType2 {
            foo: Int! @wow
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            RecursingDirective {
                position: Pos {
                    line: 1,
                    column: 8,
                    builtin: false,
                },
                name: "heyhey",
            },
            RecursingDirective {
                position: Pos {
                    line: 6,
                    column: 8,
                    builtin: false,
                },
                name: "wow",
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
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 19,
                    builtin: false,
                },
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
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 26,
                    builtin: false,
                },
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
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            DuplicatedName {
                position: Pos {
                    line: 1,
                    column: 41,
                    builtin: false,
                },
                name: "arg1",
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
            NoOutputType {
                position: Pos {
                    line: 2,
                    column: 18,
                    builtin: false,
                },
                name: "MyType",
            },
            NoOutputType {
                position: Pos {
                    line: 3,
                    column: 18,
                    builtin: false,
                },
                name: "MyInterface",
            },
            NoOutputType {
                position: Pos {
                    line: 4,
                    column: 18,
                    builtin: false,
                },
                name: "MyUnion",
            },
        ]
        "###);

        let doc = parse_type_system_document(
            "
        directive @heyhey(arg1: MyScalar!, arg2: [InputType], arg3: MyEnum) on OBJECT
        scalar MyScalar
        input InputType { foo: Int! }
        enum MyEnum { A,B,C }
        ",
        )
        .unwrap();
        let doc = resolve_extensions(doc).unwrap();
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"[]");
    }
}

#[cfg(test)]
mod schemas {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            RepeatedDirective {
                position: Pos {
                    line: 2,
                    column: 20,
                    builtin: false,
                },
                name: "wow",
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
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 2,
                    column: 15,
                    builtin: false,
                },
                name: "wow",
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod scalars {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 19,
                    builtin: false,
                },
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
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 2,
                    column: 19,
                    builtin: false,
                },
                name: "wow",
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod objects {
    use insta::{assert_debug_snapshot, assert_snapshot};

    use crate::{
        graphql_printer::GraphQLPrinter,
        source_map_writer::just_writer::JustWriter,
        type_system_validator::{check_type_system_document, tests::parse_to_type_system_document},
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 17,
                    builtin: false,
                },
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
            DuplicatedName {
                position: Pos {
                    line: 3,
                    column: 16,
                    builtin: false,
                },
                name: "foo",
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
            UnscoUnsco {
                position: Pos {
                    line: 2,
                    column: 16,
                    builtin: false,
                },
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
            NoInputType {
                position: Pos {
                    line: 7,
                    column: 36,
                    builtin: false,
                },
                name: "InputObj",
            },
        ]
        "###);
    }

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
            UnscoUnsco {
                position: Pos {
                    line: 2,
                    column: 23,
                    builtin: false,
                },
            },
            DuplicatedName {
                position: Pos {
                    line: 3,
                    column: 16,
                    builtin: false,
                },
                name: "field1",
            },
            DuplicatedName {
                position: Pos {
                    line: 4,
                    column: 34,
                    builtin: false,
                },
                name: "arg",
            },
            NoOutputType {
                position: Pos {
                    line: 5,
                    column: 28,
                    builtin: false,
                },
                name: "MyType",
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
            FieldTypeMisMatchWithInterface {
                position: Pos {
                    line: 10,
                    column: 16,
                    builtin: false,
                },
                interface_name: "IFoo",
            },
            InterfaceFieldNotImplemented {
                position: Pos {
                    line: 8,
                    column: 17,
                    builtin: false,
                },
                field_name: "bar",
                interface_name: "IBar",
            },
            InterfaceArgumentNotImplemented {
                position: Pos {
                    line: 13,
                    column: 16,
                    builtin: false,
                },
                argument_name: "arg",
                interface_name: "IBar",
            },
            ArgumentTypeNonNullAgainstInterface {
                position: Pos {
                    line: 13,
                    column: 20,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            ArgumentTypeMisMatchWithInterface {
                position: Pos {
                    line: 16,
                    column: 20,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            ArgumentTypeNonNullAgainstInterface {
                position: Pos {
                    line: 19,
                    column: 35,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            FieldTypeMisMatchWithInterface {
                position: Pos {
                    line: 19,
                    column: 16,
                    builtin: false,
                },
                interface_name: "IBar",
            },
        ]
        "###);
    }

    fn print_graphql<T: GraphQLPrinter>(value: &T) -> String {
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        value.print_graphql(&mut writer);
        result
    }
}

#[cfg(test)]
mod interfaces {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 22,
                    builtin: false,
                },
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
            DuplicatedName {
                position: Pos {
                    line: 3,
                    column: 16,
                    builtin: false,
                },
                name: "foo",
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
            UnscoUnsco {
                position: Pos {
                    line: 2,
                    column: 16,
                    builtin: false,
                },
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
            NoInputType {
                position: Pos {
                    line: 7,
                    column: 36,
                    builtin: false,
                },
                name: "InputObj",
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
            UnscoUnsco {
                position: Pos {
                    line: 2,
                    column: 23,
                    builtin: false,
                },
            },
            DuplicatedName {
                position: Pos {
                    line: 3,
                    column: 16,
                    builtin: false,
                },
                name: "field1",
            },
            DuplicatedName {
                position: Pos {
                    line: 4,
                    column: 34,
                    builtin: false,
                },
                name: "arg",
            },
            NoOutputType {
                position: Pos {
                    line: 5,
                    column: 28,
                    builtin: false,
                },
                name: "MyType",
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
            FieldTypeMisMatchWithInterface {
                position: Pos {
                    line: 10,
                    column: 16,
                    builtin: false,
                },
                interface_name: "IFoo",
            },
            InterfaceFieldNotImplemented {
                position: Pos {
                    line: 8,
                    column: 22,
                    builtin: false,
                },
                field_name: "bar",
                interface_name: "IBar",
            },
            InterfaceArgumentNotImplemented {
                position: Pos {
                    line: 13,
                    column: 16,
                    builtin: false,
                },
                argument_name: "arg",
                interface_name: "IBar",
            },
            ArgumentTypeNonNullAgainstInterface {
                position: Pos {
                    line: 13,
                    column: 20,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            ArgumentTypeMisMatchWithInterface {
                position: Pos {
                    line: 16,
                    column: 20,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            ArgumentTypeNonNullAgainstInterface {
                position: Pos {
                    line: 19,
                    column: 35,
                    builtin: false,
                },
                interface_name: "IBar",
            },
            FieldTypeMisMatchWithInterface {
                position: Pos {
                    line: 19,
                    column: 16,
                    builtin: false,
                },
                interface_name: "IBar",
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
            NoImplementSelf {
                position: Pos {
                    line: 1,
                    column: 35,
                    builtin: false,
                },
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod unions {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 18,
                    builtin: false,
                },
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
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 2,
                    column: 22,
                    builtin: false,
                },
                name: "wow",
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
            UnknownType {
                position: Pos {
                    line: 1,
                    column: 32,
                    builtin: false,
                },
                name: "C",
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
            NonObjectTypeUnionMember {
                position: Pos {
                    line: 1,
                    column: 25,
                    builtin: false,
                },
                member_name: "Int",
            },
            NonObjectTypeUnionMember {
                position: Pos {
                    line: 1,
                    column: 37,
                    builtin: false,
                },
                member_name: "Union",
            },
            NonObjectTypeUnionMember {
                position: Pos {
                    line: 1,
                    column: 45,
                    builtin: false,
                },
                member_name: "Enum",
            },
            NonObjectTypeUnionMember {
                position: Pos {
                    line: 1,
                    column: 52,
                    builtin: false,
                },
                member_name: "Input",
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod enums {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 17,
                    builtin: false,
                },
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
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 4,
                    column: 21,
                    builtin: false,
                },
                name: "x",
            },
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 6,
                    column: 18,
                    builtin: false,
                },
                name: "y",
            },
        ]
        "###);
    }
}

#[cfg(test)]
mod input_objects {
    use insta::assert_debug_snapshot;

    use crate::type_system_validator::{
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
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 18,
                    builtin: false,
                },
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
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 4,
                    column: 26,
                    builtin: false,
                },
                name: "y",
            },
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 5,
                    column: 28,
                    builtin: false,
                },
                name: "x",
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
            UnscoUnsco {
                position: Pos {
                    line: 2,
                    column: 16,
                    builtin: false,
                },
            },
            DuplicatedName {
                position: Pos {
                    line: 4,
                    column: 16,
                    builtin: false,
                },
                name: "field2",
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
            NoOutputType {
                position: Pos {
                    line: 3,
                    column: 24,
                    builtin: false,
                },
                name: "MyType",
            },
            NoOutputType {
                position: Pos {
                    line: 4,
                    column: 23,
                    builtin: false,
                },
                name: "MyUnion",
            },
        ]
        "###);
    }
}

#[cfg(test)]
fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let doc = parse_type_system_document(source).unwrap();
    let doc = resolve_extensions(doc).unwrap();
    doc
}
