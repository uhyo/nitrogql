#[cfg(test)]
mod test_impl {
    use insta::assert_snapshot;
    use json_writer::JSONObjectWriter;

    use super::super::to_json::JsonPrintable;

    use crate::graphql_parser::ast::{
        base::{Ident, Pos},
        value::{
            BooleanValue, EnumValue, FloatValue, IntValue, ListValue, NullValue, ObjectValue,
            StringValue, Value,
        },
    };

    static POS: Pos = Pos { line: 0, column: 0 };
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
            value: ""
        })));
        assert_snapshot!(print_json_to_string(Value::StringValue(StringValue {
            position: POS,
            value: "foobar"
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
