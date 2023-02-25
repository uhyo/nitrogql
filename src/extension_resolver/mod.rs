use crate::graphql_parser::ast::{
    type_system::{
        EnumTypeDefinition, EnumTypeExtension, InputObjectTypeDefinition, InputObjectTypeExtension,
        InterfaceTypeDefinition, InterfaceTypeExtension, ObjectTypeDefinition, ObjectTypeExtension,
        ScalarTypeDefinition, ScalarTypeExtension, SchemaDefinition, SchemaExtension,
        TypeDefinition, TypeExtension, TypeSystemDefinition, TypeSystemDefinitionOrExtension,
        UnionTypeDefinition, UnionTypeExtension,
    },
    TypeSystemDocument, TypeSystemOrExtensionDocument,
};

use self::extension_list::{ExtensionError, ExtensionList};

mod extension_list;
mod tests;

pub fn resolve_extensions(
    document: TypeSystemOrExtensionDocument,
) -> Result<TypeSystemDocument, ExtensionError> {
    let mut schema_list = ExtensionList::new("schema");
    let mut scalar_list = ExtensionList::new("scalar");
    let mut object_list = ExtensionList::new("type");
    let mut interface_list = ExtensionList::new("interface");
    let mut union_list = ExtensionList::new("union");
    let mut enum_list = ExtensionList::new("enum");
    let mut input_object_list = ExtensionList::new("input object");
    let mut directive_list = Vec::new();
    for def in document.definitions {
        match def {
            TypeSystemDefinitionOrExtension::SchemaDefinition(schema) => {
                schema_list.set_original(schema)?;
            }
            TypeSystemDefinitionOrExtension::TypeDefinition(def) => match def {
                TypeDefinition::Scalar(def) => {
                    scalar_list.set_original(def)?;
                }
                TypeDefinition::Object(def) => {
                    object_list.set_original(def)?;
                }
                TypeDefinition::Interface(def) => {
                    interface_list.set_original(def)?;
                }
                TypeDefinition::Union(def) => {
                    union_list.set_original(def)?;
                }
                TypeDefinition::Enum(def) => {
                    enum_list.set_original(def)?;
                }
                TypeDefinition::InputObject(def) => {
                    input_object_list.set_original(def)?;
                }
            },
            TypeSystemDefinitionOrExtension::DirectiveDefinition(def) => {
                directive_list.push(def);
            }
            TypeSystemDefinitionOrExtension::SchemaExtension(schema) => {
                schema_list.add_extension(schema);
            }
            TypeSystemDefinitionOrExtension::TypeExtension(def) => match def {
                TypeExtension::Scalar(def) => {
                    scalar_list.add_extension(def);
                }
                TypeExtension::Object(def) => {
                    object_list.add_extension(def);
                }
                TypeExtension::Interface(def) => {
                    interface_list.add_extension(def);
                }
                TypeExtension::Union(def) => {
                    union_list.add_extension(def);
                }
                TypeExtension::Enum(def) => {
                    enum_list.add_extension(def);
                }
                TypeExtension::InputObject(def) => {
                    input_object_list.add_extension(def);
                }
            },
        }
    }

    let schema_definitions = schema_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_schema_definition)
        .map(|def| TypeSystemDefinition::SchemaDefinition(def));
    let scalar_definitions = scalar_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_scalar_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::Scalar(def)));
    let object_definitions = object_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_object_type_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(def)));
    let interface_definitions = interface_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_interface_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::Interface(def)));
    let union_definitions = union_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_union_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::Union(def)));
    let enum_definitions = enum_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_enum_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::Enum(def)));
    let input_object_definitions = input_object_list
        .into_original_and_extensions()?
        .into_iter()
        .map(merge_input_object_definition)
        .map(|def| TypeSystemDefinition::TypeDefinition(TypeDefinition::InputObject(def)));

    let new_definitions = directive_list
        .into_iter()
        .map(TypeSystemDefinition::DirectiveDefinition)
        .chain(schema_definitions)
        .chain(scalar_definitions)
        .chain(object_definitions)
        .chain(interface_definitions)
        .chain(union_definitions)
        .chain(enum_definitions)
        .chain(input_object_definitions)
        .into_iter()
        .collect();
    Ok(TypeSystemDocument {
        definitions: new_definitions,
    })
}

fn merge_schema_definition<'a>(
    input: (SchemaDefinition<'a>, Vec<SchemaExtension<'a>>),
) -> SchemaDefinition<'a> {
    let (
        SchemaDefinition {
            description,
            position,
            directives,
            definitions,
        },
        extensions,
    ) = input;
    let (ext_directives, ext_definitions) =
        unzip2(extensions, |ext| (ext.directives, ext.definitions));
    SchemaDefinition {
        description,
        position,
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
        definitions: definitions
            .into_iter()
            .chain(ext_definitions.into_iter().flatten())
            .collect(),
    }
}

fn merge_scalar_definition<'a>(
    input: (ScalarTypeDefinition<'a>, Vec<ScalarTypeExtension<'a>>),
) -> ScalarTypeDefinition<'a> {
    let (
        ScalarTypeDefinition {
            description,
            position,
            name,
            directives,
        },
        extensions,
    ) = input;

    ScalarTypeDefinition {
        description,
        position,
        name,
        directives: directives
            .into_iter()
            .chain(extensions.into_iter().flat_map(|ext| ext.directives))
            .collect(),
    }
}

fn merge_object_type_definition<'a>(
    input: (ObjectTypeDefinition<'a>, Vec<ObjectTypeExtension<'a>>),
) -> ObjectTypeDefinition<'a> {
    let (
        ObjectTypeDefinition {
            description,
            position,
            name,
            implements,
            fields,
            directives,
        },
        extensions,
    ) = input;
    let (ext_implements, ext_fields, ext_directives) = unzip3(extensions, |ext| {
        (ext.implements, ext.fields, ext.directives)
    });

    ObjectTypeDefinition {
        description,
        position,
        name,
        implements: implements
            .into_iter()
            .chain(ext_implements.into_iter().flatten())
            .collect(),
        fields: fields
            .into_iter()
            .chain(ext_fields.into_iter().flatten())
            .collect(),
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
    }
}

fn merge_interface_definition<'a>(
    input: (InterfaceTypeDefinition<'a>, Vec<InterfaceTypeExtension<'a>>),
) -> InterfaceTypeDefinition<'a> {
    let (
        InterfaceTypeDefinition {
            description,
            position,
            name,
            implements,
            fields,
            directives,
        },
        extensions,
    ) = input;
    let (ext_implements, ext_fields, ext_directives) = unzip3(extensions, |ext| {
        (ext.implements, ext.fields, ext.directives)
    });

    InterfaceTypeDefinition {
        description,
        position,
        name,
        implements: implements
            .into_iter()
            .chain(ext_implements.into_iter().flatten())
            .collect(),
        fields: fields
            .into_iter()
            .chain(ext_fields.into_iter().flatten())
            .collect(),
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
    }
}

fn merge_union_definition<'a>(
    input: (UnionTypeDefinition<'a>, Vec<UnionTypeExtension<'a>>),
) -> UnionTypeDefinition<'a> {
    let (
        UnionTypeDefinition {
            description,
            position,
            name,
            members,
            directives,
        },
        extensions,
    ) = input;
    let (ext_members, ext_directives) = unzip2(extensions, |ext| (ext.members, ext.directives));

    UnionTypeDefinition {
        description,
        position,
        name,
        members: members
            .into_iter()
            .chain(ext_members.into_iter().flatten())
            .collect(),
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
    }
}

fn merge_enum_definition<'a>(
    input: (EnumTypeDefinition<'a>, Vec<EnumTypeExtension<'a>>),
) -> EnumTypeDefinition<'a> {
    let (
        EnumTypeDefinition {
            description,
            position,
            name,
            values,
            directives,
        },
        extensions,
    ) = input;
    let (ext_values, ext_directives) = unzip2(extensions, |ext| (ext.values, ext.directives));

    EnumTypeDefinition {
        description,
        position,
        name,
        values: values
            .into_iter()
            .chain(ext_values.into_iter().flatten())
            .collect(),
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
    }
}

fn merge_input_object_definition<'a>(
    input: (
        InputObjectTypeDefinition<'a>,
        Vec<InputObjectTypeExtension<'a>>,
    ),
) -> InputObjectTypeDefinition<'a> {
    let (
        InputObjectTypeDefinition {
            description,
            position,
            name,
            fields,
            directives,
        },
        extensions,
    ) = input;
    let (ext_fields, ext_directives) = unzip2(extensions, |ext| (ext.fields, ext.directives));

    InputObjectTypeDefinition {
        description,
        position,
        name,
        fields: fields
            .into_iter()
            .chain(ext_fields.into_iter().flatten())
            .collect(),
        directives: directives
            .into_iter()
            .chain(ext_directives.into_iter().flatten())
            .collect(),
    }
}

fn unzip2<T, A, B>(iter: impl IntoIterator<Item = T>, f: impl Fn(T) -> (A, B)) -> (Vec<A>, Vec<B>) {
    let mut result_a = vec![];
    let mut result_b = vec![];
    for t in iter {
        let (a, b) = f(t);
        result_a.push(a);
        result_b.push(b);
    }
    (result_a, result_b)
}

fn unzip3<T, A, B, C>(
    iter: impl IntoIterator<Item = T>,
    f: impl Fn(T) -> (A, B, C),
) -> (Vec<A>, Vec<B>, Vec<C>) {
    let mut result_a = vec![];
    let mut result_b = vec![];
    let mut result_c = vec![];
    for t in iter {
        let (a, b, c) = f(t);
        result_a.push(a);
        result_b.push(b);
        result_c.push(c);
    }
    (result_a, result_b, result_c)
}

fn unzip4<T, A, B, C, D>(
    iter: impl IntoIterator<Item = T>,
    f: impl Fn(T) -> (A, B, C, D),
) -> (Vec<A>, Vec<B>, Vec<C>, Vec<D>) {
    let mut result_a = vec![];
    let mut result_b = vec![];
    let mut result_c = vec![];
    let mut result_d = vec![];
    for t in iter {
        let (a, b, c, d) = f(t);
        result_a.push(a);
        result_b.push(b);
        result_c.push(c);
        result_d.push(d);
    }
    (result_a, result_b, result_c, result_d)
}
