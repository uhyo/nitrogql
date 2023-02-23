use pest::iterators::Pair;

use super::{super::Rule, build_description};
use crate::{
    graphql_parser::{
        ast::{
            base::Ident,
            type_system::{
                ArgumentsDefinition, EnumTypeDefinition, EnumValueDefinition, FieldDefinition,
                InputObjectTypeDefinition, InputValueDefinition, InterfaceTypeDefinition,
                ObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition, UnionTypeDefinition,
            },
        },
        parser::builder::{
            directives::build_directives, r#type::build_type, utils::PairExt, value::build_value,
        },
    },
    parts,
};

pub fn build_type_definition(pair: Pair<Rule>) -> TypeDefinition {
    let pair = pair.only_child();
    match pair.as_rule() {
        Rule::ScalarTypeDefinition => TypeDefinition::Scalar(build_scalar_type_definition(pair)),
        Rule::ObjectTypeDefinition => TypeDefinition::Object(build_object_type_definition(pair)),
        Rule::InterfaceTypeDefinition => {
            TypeDefinition::Interface(build_interface_type_definition(pair))
        }
        Rule::UnionTypeDefinition => TypeDefinition::Union(build_union_type_definition(pair)),
        Rule::EnumTypeDefinition => TypeDefinition::Enum(build_enum_type_definition(pair)),
        Rule::InputObjectTypeDefinition => {
            TypeDefinition::InputObject(build_input_object_type_definition(pair))
        }
        rule => panic!("Unexpected child of TypeDefinition: {:?}", rule),
    }
}

fn build_scalar_type_definition(pair: Pair<Rule>) -> ScalarTypeDefinition {
    let (description, _, name, directives) = parts!(
        pair,
        Description opt,
        KEYWORD_scalar,
        Name,
        Directives opt
    );
    ScalarTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        directives: directives.map_or(vec![], build_directives),
    }
}

fn build_object_type_definition(pair: Pair<Rule>) -> ObjectTypeDefinition {
    let (description, _, name, implements, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_type,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );

    ObjectTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        implements: implements.map_or(vec![], build_implements_interfaces),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_fields_definition),
    }
}

fn build_interface_type_definition(pair: Pair<Rule>) -> InterfaceTypeDefinition {
    let (description, _, name, implements, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_interface,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );
    InterfaceTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        implements: implements.map_or(vec![], build_implements_interfaces),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_fields_definition),
    }
}

fn build_union_type_definition(pair: Pair<Rule>) -> UnionTypeDefinition {
    let (description, _, name, directives, members) = parts!(
        pair,
        Description opt,
        KEYWORD_union,
        Name,
        Directives opt,
        UnionMemberTypes opt
    );
    UnionTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        directives: directives.map_or(vec![], build_directives),
        members: members.map_or(vec![], |members| {
            let pairs = members.all_children(Rule::NamedType);
            pairs.into_iter().map(|pair| pair.into()).collect()
        }),
    }
}

fn build_enum_type_definition(pair: Pair<Rule>) -> EnumTypeDefinition {
    let (description, _, name, directives, values) = parts!(
        pair,
        Description opt,
        KEYWORD_enum,
        Name,
        Directives opt,
        EnumValuesDefinition opt
    );
    EnumTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        directives: directives.map_or(vec![], build_directives),
        values: values.map_or(vec![], |pair| {
            let pairs = pair.all_children(Rule::EnumValueDefinition);
            pairs.into_iter().map(build_enum_value_definition).collect()
        }),
    }
}

fn build_input_object_type_definition(pair: Pair<Rule>) -> InputObjectTypeDefinition {
    let (description, _, name, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_input,
        Name,
        Directives opt,
        InputFieldsDefinition opt
    );
    InputObjectTypeDefinition {
        description: description.map(build_description),
        name: name.into(),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_input_fields_definition),
    }
}

pub fn build_implements_interfaces(pair: Pair<Rule>) -> Vec<Ident> {
    let mut pairs = pair.into_inner();
    let Some(first_pair) = pairs.next() else {
        panic!("No child of ImplementsInterfaces, expected KEYWORD_implements");
    };
    if first_pair.as_rule() != Rule::KEYWORD_extend {
        panic!(
            "Unexpected child {:?} of ImplementsInterfaces, expected KEYWORD_implements",
            first_pair.as_rule()
        );
    }
    pairs
        .map(|pair| {
            if pair.as_rule() != Rule::NamedType {
                panic!(
                    "Unexpected child {:?} of ImplementsInterfaces, expected NamedType",
                    first_pair.as_rule()
                );
            }
            pair.into()
        })
        .collect()
}

pub fn build_fields_definition(pair: Pair<Rule>) -> Vec<FieldDefinition> {
    let pairs = pair.all_children(Rule::FieldDefinition);
    pairs
        .into_iter()
        .map(|pair| {
            let (description, name, arguments, ty, directives) = parts!(
                pair,
                Description opt,
                Name,
                ArgumentsDefinition opt,
                Type,
                Directives opt
            );
            FieldDefinition {
                description: description.map(build_description),
                name: name.into(),
                arguments: arguments.map(build_arguments_definition),
                r#type: build_type(ty),
                directives: directives.map_or(vec![], build_directives),
            }
        })
        .collect()
}

pub fn build_arguments_definition(pair: Pair<Rule>) -> ArgumentsDefinition {
    let pairs = pair.all_children(Rule::InputValueDefinition);
    let input_values = pairs
        .into_iter()
        .map(|pair| {
            let (description, name, ty, default_value, directives) = parts!(
                pair,
                Description opt,
                Name,
                Type,
                DefaultValue opt,
                Directives opt
            );
            InputValueDefinition {
                description: description.map(build_description),
                name: name.into(),
                r#type: build_type(ty),
                default_value: default_value.map(|pair| {
                    let child = pair.only_child();
                    build_value(child)
                }),
                directives: directives.map_or(vec![], build_directives),
            }
        })
        .collect();
    ArgumentsDefinition { input_values }
}

pub fn build_enum_value_definition(pair: Pair<Rule>) -> EnumValueDefinition {
    let (description, value, directives) = parts!(
        pair,
        Description opt,
        EnumValue,
        Directives opt
    );
    EnumValueDefinition {
        description: description.map(build_description),
        name: value.into(),
        directives: directives.map_or(vec![], build_directives),
    }
}

pub fn build_input_fields_definition(pair: Pair<Rule>) -> Vec<InputValueDefinition> {
    let pairs = pair.all_children(Rule::InputValueDefinition);
    pairs
        .into_iter()
        .map(|pair| {
            let (description, name, ty, default_value, directives) = parts!(
                pair,
                Description opt,
                Name,
                Type,
                DefaultValue opt,
                Directives opt
            );
            InputValueDefinition {
                description: description.map(build_description),
                name: name.into(),
                r#type: build_type(ty),
                default_value: default_value.map(build_value),
                directives: directives.map_or(vec![], build_directives),
            }
        })
        .collect()
}
