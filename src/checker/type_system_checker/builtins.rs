use crate::graphql_parser::ast::{
    base::{Ident, Pos},
    r#type::{NamedType, NonNullType, Type},
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, InputValueDefinition, ScalarTypeDefinition,
        TypeDefinition,
    },
    value::{StringValue, Value},
};

/// Generate built-in definitions.
/// TODO: make this configurable
pub fn generate_builtins() -> (
    Vec<(&'static str, TypeDefinition<'static>)>,
    Vec<(&'static str, DirectiveDefinition<'static>)>,
) {
    let type_definitions = vec![
        scalar("Int"),
        scalar("Float"),
        scalar("String"),
        scalar("Boolean"),
        scalar("ID"),
    ];

    let directive_definitions = vec![
        directive(
            "skip",
            vec![(
                "if",
                Type::NonNull(Box::new(NonNullType {
                    r#type: Type::Named(NamedType {
                        name: ident("Boolean"),
                    }),
                })),
                None,
            )],
            vec!["FIELD", "FRAGMENT_SPREAD", "INLINE_FRAGMENT"],
        ),
        directive(
            "include",
            vec![(
                "if",
                Type::NonNull(Box::new(NonNullType {
                    r#type: Type::Named(NamedType {
                        name: ident("Boolean"),
                    }),
                })),
                None,
            )],
            vec!["FIELD", "FRAGMENT_SPREAD", "INLINE_FRAGMENT"],
        ),
        directive(
            "deprecated",
            vec![(
                "reason",
                Type::Named(NamedType {
                    name: ident("String"),
                }),
                Some(Value::StringValue(StringValue {
                    position: Pos::builtin(),
                    value: String::from("No longer supported"),
                })),
            )],
            vec![
                "FIELD_DEFINITION",
                "ARGUMENT_DEFINITION",
                "INPUT_FIELD_DEFINITION",
                "ENUM_VALUE",
            ],
        ),
        directive(
            "specifiedBy",
            vec![(
                "url",
                Type::NonNull(Box::new(NonNullType {
                    r#type: Type::Named(NamedType {
                        name: ident("String"),
                    }),
                })),
                None,
            )],
            vec!["SCALAR"],
        ),
    ];

    (type_definitions, directive_definitions)
}

fn scalar(name: &str) -> (&str, TypeDefinition) {
    (
        name,
        TypeDefinition::Scalar(ScalarTypeDefinition {
            description: None,
            position: Pos::builtin(),
            name: ident(name),
            directives: vec![],
        }),
    )
}

fn directive<'a>(
    name: &'a str,
    arguments: Vec<(&'a str, Type<'a>, Option<Value<'a>>)>,
    locations: Vec<&'a str>,
) -> (&'a str, DirectiveDefinition<'a>) {
    (
        name,
        DirectiveDefinition {
            description: None,
            position: Pos::builtin(),
            name: ident(name),
            arguments: if arguments.is_empty() {
                None
            } else {
                Some(ArgumentsDefinition {
                    input_values: arguments
                        .into_iter()
                        .map(|(name, ty, default_value)| InputValueDefinition {
                            description: None,
                            position: Pos::builtin(),
                            name: ident(name),
                            r#type: ty,
                            default_value,
                            directives: vec![],
                        })
                        .collect(),
                })
            },
            repeatable: None,
            locations: locations.into_iter().map(ident).collect(),
        },
    )
}

fn ident(name: &str) -> Ident {
    Ident {
        name,
        position: Pos::builtin(),
    }
}
