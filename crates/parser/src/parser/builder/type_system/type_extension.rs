use pest::iterators::Pair;

use super::{
    super::Rule,
    type_definition::{
        build_enum_value_definition, build_fields_definition, build_implements_interfaces,
        build_input_fields_definition,
    },
};
use crate::{
    parser::builder::{directives::build_directives, utils::PairExt},
    parts,
};
use nitrogql_ast::type_system::{
    EnumTypeExtension, InputObjectTypeExtension, InterfaceTypeExtension, ObjectTypeExtension,
    ScalarTypeExtension, TypeExtension, UnionTypeExtension,
};

pub fn build_type_extension(pair: Pair<Rule>) -> TypeExtension {
    let pair = pair.only_child();
    match pair.as_rule() {
        Rule::ScalarTypeExtension => TypeExtension::Scalar(build_scalar_type_extension(pair)),
        Rule::ObjectTypeExtension => TypeExtension::Object(build_object_type_extension(pair)),
        Rule::InterfaceTypeExtension => {
            TypeExtension::Interface(build_interface_type_extension(pair))
        }
        Rule::UnionTypeExtension => TypeExtension::Union(build_union_type_extension(pair)),
        Rule::EnumTypeExtension => TypeExtension::Enum(build_enum_type_extension(pair)),
        Rule::InputObjectTypeExtension => {
            TypeExtension::InputObject(build_input_object_type_extension(pair))
        }
        rule => panic!("Unexpected child of TypeExtension: {:?}", rule),
    }
}

fn build_scalar_type_extension(pair: Pair<Rule>) -> ScalarTypeExtension {
    let (keyword, _, name, directives) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_scalar,
        Name,
        Directives opt
    );
    ScalarTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        directives: directives.map_or(vec![], build_directives),
    }
}

fn build_object_type_extension(pair: Pair<Rule>) -> ObjectTypeExtension {
    let (keyword, _, name, implements, directives, fields) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_type,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );

    ObjectTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        implements: implements.map_or(vec![], build_implements_interfaces),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_fields_definition),
    }
}

fn build_interface_type_extension(pair: Pair<Rule>) -> InterfaceTypeExtension {
    let (keyword, _, name, implements, directives, fields) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_interface,
        Name,
        ImplementsInterfaces opt,
        Directives opt,
        FieldsDefinition opt
    );
    InterfaceTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        implements: implements.map_or(vec![], build_implements_interfaces),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_fields_definition),
    }
}

fn build_union_type_extension(pair: Pair<Rule>) -> UnionTypeExtension {
    let (keyword, _, name, directives, members) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_union,
        Name,
        Directives opt,
        UnionMemberTypes opt
    );
    UnionTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        directives: directives.map_or(vec![], build_directives),
        members: members.map_or(vec![], |members| {
            let pairs = members.all_children(Rule::NamedType);
            pairs.into_iter().map(|pair| pair.to_ident()).collect()
        }),
    }
}

fn build_enum_type_extension(pair: Pair<Rule>) -> EnumTypeExtension {
    let (keyword, _, name, directives, values) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_enum,
        Name,
        Directives opt,
        EnumValuesDefinition opt
    );
    EnumTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        directives: directives.map_or(vec![], build_directives),
        values: values.map_or(vec![], |pair| {
            let pairs = pair.all_children(Rule::EnumValueDefinition);
            pairs.into_iter().map(build_enum_value_definition).collect()
        }),
    }
}

fn build_input_object_type_extension(pair: Pair<Rule>) -> InputObjectTypeExtension {
    let (keyword, _, name, directives, fields) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_input,
        Name,
        Directives opt,
        InputFieldsDefinition opt
    );
    InputObjectTypeExtension {
        position: keyword.to_pos(),
        name: name.to_ident(),
        directives: directives.map_or(vec![], build_directives),
        fields: fields.map_or(vec![], build_input_fields_definition),
    }
}
