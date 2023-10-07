use nitrogql_ast::{
    base::{Ident, Keyword, Pos},
    r#type::{NamedType, NonNullType, Type},
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, InputValueDefinition, ScalarTypeDefinition,
        TypeDefinition, TypeSystemDefinition, TypeSystemDefinitionOrExtension,
    },
    TypeSystemDocument,
};

/// Build nitrogql-specific built-in definitions.
pub fn nitrogql_builtins() -> Vec<TypeSystemDefinitionOrExtension<'static>> {
    vec![TypeSystemDefinitionOrExtension::DirectiveDefinition(
        DirectiveDefinition {
            directive_keyword: keyword("directive"),
            position: Pos::builtin(),
            name: ident("nitrogql_ts_type"),
            description: None,
            arguments: Some(ArgumentsDefinition {
                input_values: vec![InputValueDefinition {
                    description: None,
                    position: Pos::builtin(),
                    name: ident("type"),
                    r#type: Type::NonNull(Box::new(NonNullType {
                        r#type: Type::Named(NamedType {
                            name: ident("String"),
                        }),
                    })),
                    default_value: None,
                    directives: vec![],
                }],
            }),
            repeatable: None,
            locations: vec![ident("SCALAR")],
        },
    )]
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

/// Removes built-in definitions from the schema.
pub fn remove_builtins<'src>(schema: &TypeSystemDocument<'src>) -> TypeSystemDocument<'src> {
    let definitions = schema
        .definitions
        .iter()
        .cloned()
        .filter_map(|d| match d {
            TypeSystemDefinition::DirectiveDefinition(def) => (def.name.name != "nitrogql_ts_type")
                .then_some(TypeSystemDefinition::DirectiveDefinition(def)),
            TypeSystemDefinition::SchemaDefinition(_) => Some(d),
            TypeSystemDefinition::TypeDefinition(def) => {
                if let TypeDefinition::Scalar(def) = def {
                    return Some(TypeSystemDefinition::TypeDefinition(
                        TypeDefinition::Scalar(ScalarTypeDefinition {
                            directives: def
                                .directives
                                .into_iter()
                                .filter(|d| d.name.name != "nitrogql_ts_type")
                                .collect(),
                            ..def
                        }),
                    ));
                }
                Some(TypeSystemDefinition::TypeDefinition(def))
            }
        })
        .collect();
    TypeSystemDocument { definitions }
}
