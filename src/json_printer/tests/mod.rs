#[cfg(test)]
mod tests {
    use json_writer::JSONObjectWriter;

    use crate::graphql_parser::ast::base::Pos;

    use super::super::to_json::JsonPrintable;

    static POS: Pos = Pos { line: 0, column: 0 };

    mod value {
        use super::{print_json_to_string, POS};
        use crate::graphql_parser::ast::{
            base::Ident,
            value::{
                BooleanValue, EnumValue, FloatValue, IntValue, ListValue, NullValue, ObjectValue,
                StringValue, Value,
            },
        };
        use insta::assert_snapshot;

        #[test]
        fn null_value() {
            assert_snapshot!(print_json_to_string(Value::NullValue(NullValue {
                position: POS,
                keyword: "null"
            })))
        }
        #[test]
        fn boolean_value() {
            assert_snapshot!(print_json_to_string(Value::BooleanValue(BooleanValue {
                position: POS,
                keyword: "false",
                value: false,
            })));
            assert_snapshot!(print_json_to_string(Value::BooleanValue(BooleanValue {
                position: POS,
                keyword: "true",
                value: true,
            })));
        }
        #[test]
        fn int_value() {
            assert_snapshot!(print_json_to_string(Value::IntValue(IntValue {
                position: POS,
                value: "123"
            })));
            assert_snapshot!(print_json_to_string(Value::IntValue(IntValue {
                position: POS,
                value: "-5"
            })));
        }
        #[test]
        fn float_value() {
            assert_snapshot!(print_json_to_string(Value::FloatValue(FloatValue {
                position: POS,
                value: "123.4"
            })));
            assert_snapshot!(print_json_to_string(Value::FloatValue(FloatValue {
                position: POS,
                value: "-0.05"
            })));
        }
        #[test]
        fn string_value() {
            assert_snapshot!(print_json_to_string(Value::StringValue(StringValue {
                position: POS,
                value: String::from("")
            })));
            assert_snapshot!(print_json_to_string(Value::StringValue(StringValue {
                position: POS,
                value: String::from("foobar")
            })));
        }
        #[test]
        fn enum_value() {
            assert_snapshot!(print_json_to_string(Value::EnumValue(EnumValue {
                position: POS,
                value: "A"
            })))
        }
        #[test]
        fn list_value() {
            assert_snapshot!(print_json_to_string(Value::ListValue(ListValue {
                position: POS,
                values: vec![
                    Value::IntValue(IntValue {
                        position: POS,
                        value: "1"
                    }),
                    Value::IntValue(IntValue {
                        position: POS,
                        value: "2"
                    }),
                    Value::IntValue(IntValue {
                        position: POS,
                        value: "3"
                    }),
                ]
            })))
        }
        #[test]
        fn object_value() {
            assert_snapshot!(print_json_to_string(Value::ObjectValue(ObjectValue {
                position: POS,
                fields: vec![(
                    Ident {
                        position: POS,
                        name: "foo"
                    },
                    Value::NullValue(NullValue {
                        position: POS,
                        keyword: "null"
                    })
                )]
            })));
        }
    }

    mod r#type {
        use insta::assert_snapshot;

        use crate::{
            graphql_parser::ast::{
                base::Ident,
                r#type::{ListType, NamedType, NonNullType, Type},
            },
            json_printer::tests::tests::{print_json_to_string, POS},
        };

        #[test]
        fn named_type() {
            assert_snapshot!(print_json_to_string(Type::Named(NamedType {
                name: Ident {
                    position: POS,
                    name: "tyty"
                }
            })));
        }

        #[test]
        fn non_null_type() {
            assert_snapshot!(print_json_to_string(Type::NonNull(Box::new(NonNullType {
                r#type: Type::Named(NamedType {
                    name: Ident {
                        position: POS,
                        name: "tyty"
                    }
                })
            }))));
        }

        #[test]
        fn list_type() {
            assert_snapshot!(print_json_to_string(Type::List(Box::new(ListType {
                position: POS,
                r#type: Type::Named(NamedType {
                    name: Ident {
                        position: POS,
                        name: "String"
                    }
                })
            }))));
        }
    }

    mod selection_set {
        use insta::assert_snapshot;

        use crate::graphql_parser::ast::{
            base::Ident,
            directive::Directive,
            selection_set::{Field, FragmentSpread, InlineFragment, Selection, SelectionSet},
            value::{Arguments, IntValue, StringValue, Value},
        };

        use super::{print_json_to_string, POS};

        #[test]
        fn simple_field() {
            assert_snapshot!(print_json_to_string(Field {
                alias: None,
                name: Ident {
                    position: POS,
                    name: "field"
                },
                arguments: None,
                directives: vec![],
                selection_set: None
            }))
        }

        #[test]
        fn aliased_field() {
            assert_snapshot!(print_json_to_string(Field {
                alias: Some(Ident {
                    position: POS,
                    name: "orig"
                }),
                name: Ident {
                    position: POS,
                    name: "field"
                },
                arguments: None,
                directives: vec![],
                selection_set: None
            }))
        }

        #[test]
        fn field_with_arguments() {
            assert_snapshot!(print_json_to_string(Field {
                alias: None,
                name: Ident {
                    position: POS,
                    name: "field"
                },
                arguments: Some(Arguments {
                    position: POS,
                    arguments: vec![(
                        Ident {
                            name: "a1",
                            position: POS
                        },
                        Value::StringValue(StringValue {
                            position: POS,
                            value: String::from("aaa")
                        })
                    )]
                }),
                directives: vec![],
                selection_set: None
            }))
        }

        #[test]
        fn field_with_directives() {
            assert_snapshot!(print_json_to_string(Field {
                alias: None,
                name: Ident {
                    position: POS,
                    name: "field"
                },
                arguments: None,
                directives: vec![
                    Directive {
                        position: POS,
                        name: Ident {
                            name: "dedede",
                            position: POS
                        },
                        arguments: None
                    },
                    Directive {
                        position: POS,
                        name: Ident {
                            name: "dedede2",
                            position: POS
                        },
                        arguments: Some(Arguments {
                            position: POS,
                            arguments: vec![(
                                Ident {
                                    name: "limit",
                                    position: POS
                                },
                                Value::IntValue(IntValue {
                                    position: POS,
                                    value: "10"
                                })
                            )]
                        })
                    }
                ],
                selection_set: None
            }))
        }

        #[test]
        fn fragment_spread() {
            assert_snapshot!(print_json_to_string(FragmentSpread {
                position: POS,
                fragment_name: Ident {
                    position: POS,
                    name: "F"
                },
                directives: vec![]
            }))
        }

        #[test]
        fn fragment_spread_with_directives() {
            assert_snapshot!(print_json_to_string(FragmentSpread {
                position: POS,
                fragment_name: Ident {
                    position: POS,
                    name: "F"
                },
                directives: vec![Directive {
                    position: POS,
                    name: Ident {
                        name: "abc",
                        position: POS
                    },
                    arguments: None
                },]
            }))
        }

        #[test]
        fn simple_selection_set() {
            assert_snapshot!(print_json_to_string(SelectionSet {
                position: POS,
                selections: vec![Selection::Field(Field {
                    alias: None,
                    name: Ident {
                        position: POS,
                        name: "field"
                    },
                    arguments: None,
                    directives: vec![],
                    selection_set: None
                })]
            }))
        }

        #[test]
        fn inline_fragment() {
            assert_snapshot!(print_json_to_string(InlineFragment {
                position: POS,
                type_condition: None,
                directives: vec![],
                selection_set: SelectionSet {
                    position: POS,
                    selections: vec![Selection::Field(Field {
                        alias: None,
                        name: Ident {
                            position: POS,
                            name: "field"
                        },
                        arguments: None,
                        directives: vec![],
                        selection_set: None
                    })]
                }
            }));
        }
    }

    fn print_json_to_string<V>(value: V) -> String
    where
        V: JsonPrintable,
    {
        let mut result = String::new();
        let mut writer = JSONObjectWriter::new(&mut result);
        value.print_json(&mut writer);
        writer.end();
        result
    }
}
