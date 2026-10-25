use pest::iterators::Pair;

use super::{super::Rule, build_description};
use crate::{
    parser::builder::{
        directives::build_directives, r#type::build_type, utils::PairExt, value::build_value,
    },
    parts,
};
use nitrogql_ast::{
    base::Ident,
    type_system::{
        ArgumentsDefinition, EnumTypeDefinition, EnumValueDefinition, FieldDefinition,
        InputObjectTypeDefinition, InputValueDefinition, InterfaceTypeDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition, UnionTypeDefinition,
    },
};

pub fn build_type_definition(pair: Pair<Rule>, file: usize) -> TypeDefinition {
    let pair = pair.only_child();
    match pair.as_rule() {
        Rule::ScalarTypeDefinition => {
            TypeDefinition::Scalar(build_scalar_type_definition(pair, file))
        }
        Rule::ObjectTypeDefinition => {
            TypeDefinition::Object(build_object_type_definition(pair, file))
        }
        Rule::InterfaceTypeDefinition => {
            TypeDefinition::Interface(build_interface_type_definition(pair, file))
        }
        Rule::UnionTypeDefinition => TypeDefinition::Union(build_union_type_definition(pair, file)),
        Rule::EnumTypeDefinition => TypeDefinition::Enum(build_enum_type_definition(pair, file)),
        Rule::InputObjectTypeDefinition => {
            TypeDefinition::InputObject(build_input_object_type_definition(pair, file))
        }
        rule => panic!("Unexpected child of TypeDefinition: {:?}", rule),
    }
}

fn build_scalar_type_definition(pair: Pair<Rule>, file: usize) -> ScalarTypeDefinition {
    let (description, keyword, name, directives) = parts!(
        pair,
        Description opt,
        KEYWORD_scalar,
        Name,
        Directives opt
    );
    ScalarTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        scalar_keyword: keyword.to_keyword(file),
    }
}

fn build_object_type_definition(pair: Pair<Rule>, file: usize) -> ObjectTypeDefinition {
    let (description, keyword, name, implements, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_type,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );

    ObjectTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        implements: implements.map_or(vec![], |pair| build_implements_interfaces(pair, file)),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        fields: fields.map_or(vec![], |pair| build_fields_definition(pair, file)),
        type_keyword: keyword.to_keyword(file),
    }
}

fn build_interface_type_definition(pair: Pair<Rule>, file: usize) -> InterfaceTypeDefinition {
    let (description, keyword, name, implements, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_interface,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );
    InterfaceTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        implements: implements.map_or(vec![], |pair| build_implements_interfaces(pair, file)),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        fields: fields.map_or(vec![], |pair| build_fields_definition(pair, file)),
        interface_keyword: keyword.to_keyword(file),
    }
}

fn build_union_type_definition(pair: Pair<Rule>, file: usize) -> UnionTypeDefinition {
    let (description, keyword, name, directives, members) = parts!(
        pair,
        Description opt,
        KEYWORD_union,
        Name,
        Directives opt,
        UnionMemberTypes opt
    );
    UnionTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        members: members.map_or(vec![], |members| {
            let pairs = members.all_children(Rule::NamedType);
            pairs.into_iter().map(|pair| pair.to_ident(file)).collect()
        }),
        union_keyword: keyword.to_keyword(file),
    }
}

fn build_enum_type_definition(pair: Pair<Rule>, file: usize) -> EnumTypeDefinition {
    let (description, keyword, name, directives, values) = parts!(
        pair,
        Description opt,
        KEYWORD_enum,
        Name,
        Directives opt,
        EnumValuesDefinition opt
    );
    EnumTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        values: values.map_or(vec![], |pair| {
            let pairs = pair.all_children(Rule::EnumValueDefinition);
            pairs
                .into_iter()
                .map(|pair| build_enum_value_definition(pair, file))
                .collect()
        }),
        enum_keyword: keyword.to_keyword(file),
    }
}

fn build_input_object_type_definition(pair: Pair<Rule>, file: usize) -> InputObjectTypeDefinition {
    let (description, keyword, name, directives, fields) = parts!(
        pair,
        Description opt,
        KEYWORD_input,
        Name,
        Directives opt,
        InputFieldsDefinition opt
    );
    InputObjectTypeDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        fields: fields.map_or(vec![], |pair| build_input_fields_definition(pair, file)),
        input_keyword: keyword.to_keyword(file),
    }
}

pub fn build_implements_interfaces(pair: Pair<Rule>, file: usize) -> Vec<Ident> {
    let mut pairs = pair.into_inner();
    let Some(first_pair) = pairs.next() else {
        panic!("No child of ImplementsInterfaces, expected KEYWORD_implements");
    };
    if first_pair.as_rule() != Rule::KEYWORD_implements {
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
            pair.to_ident(file)
        })
        .collect()
}

pub fn build_fields_definition(pair: Pair<Rule>, file: usize) -> Vec<FieldDefinition> {
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
                description: description.map(|pair| build_description(pair, file)),
                name: name.to_ident(file),
                arguments: arguments.map(|pair| build_arguments_definition(pair, file)),
                r#type: build_type(ty, file),
                directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
            }
        })
        .collect()
}

pub fn build_arguments_definition(pair: Pair<Rule>, file: usize) -> ArgumentsDefinition {
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
                description: description.map(|pair| build_description(pair, file)),
                position: name.to_pos(file),
                name: name.to_ident(file),
                r#type: build_type(ty, file),
                default_value: default_value.map(|pair| {
                    let child = pair.only_child();
                    build_value(child, file)
                }),
                directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
            }
        })
        .collect();
    ArgumentsDefinition { input_values }
}

pub fn build_enum_value_definition(pair: Pair<Rule>, file: usize) -> EnumValueDefinition {
    let (description, value, directives) = parts!(
        pair,
        Description opt,
        EnumValue,
        Directives opt
    );
    EnumValueDefinition {
        description: description.map(|pair| build_description(pair, file)),
        name: value.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
    }
}

pub fn build_input_fields_definition(pair: Pair<Rule>, file: usize) -> Vec<InputValueDefinition> {
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
                description: description.map(|pair| build_description(pair, file)),
                position: name.to_pos(file),
                name: name.to_ident(file),
                r#type: build_type(ty, file),
                default_value: default_value.map(|pair| build_value(pair.only_child(), file)),
                directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
            }
        })
        .collect()
}
