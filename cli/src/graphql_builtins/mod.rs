use crate::ast::{
    base::{Ident, Keyword, Pos},
    r#type::{NamedType, NonNullType, Type},
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, InputValueDefinition, ScalarTypeDefinition,
        TypeDefinition, TypeSystemDefinitionOrExtension,
    },
    value::{StringValue, Value},
};

/// Generate built-in definitions.
/// TODO: make this configurable
pub fn generate_builtins() -> Vec<TypeSystemDefinitionOrExtension<'static>> {
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

    type_definitions
        .into_iter()
        .map(|def| TypeSystemDefinitionOrExtension::TypeDefinition(def))
        .chain(
            directive_definitions
                .into_iter()
                .map(|def| TypeSystemDefinitionOrExtension::DirectiveDefinition(def)),
        )
        .collect()
}

fn scalar(name: &str) -> TypeDefinition {
    TypeDefinition::Scalar(ScalarTypeDefinition {
        description: None,
        position: Pos::builtin(),
        name: ident(name),
        directives: vec![],
        scalar_keyword: keyword("scalar"),
    })
}

fn directive<'a>(
    name: &'a str,
    arguments: Vec<(&'a str, Type<'a>, Option<Value<'a>>)>,
    locations: Vec<&'a str>,
) -> DirectiveDefinition<'a> {
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
        directive_keyword: keyword("directive"),
    }
}

fn ident(name: &str) -> Ident {
    Ident {
        name,
        position: Pos::builtin(),
    }
}

fn keyword(name: &str) -> Keyword {
    Keyword {
        name,
        position: Pos::builtin(),
    }
}
