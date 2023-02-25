use super::{
    base::{HasPos, Ident, Pos},
    directive::Directive,
    operations::OperationType,
    r#type::Type,
    value::{StringValue, Value},
};

#[derive(Clone, Debug)]
pub enum TypeSystemDefinition<'a> {
    SchemaDefinition(SchemaDefinition<'a>),
    TypeDefinition(TypeDefinition<'a>),
    DirectiveDefinition(DirectiveDefinition<'a>),
}

#[derive(Clone, Debug)]
pub enum TypeSystemDefinitionOrExtension<'a> {
    SchemaDefinition(SchemaDefinition<'a>),
    TypeDefinition(TypeDefinition<'a>),
    DirectiveDefinition(DirectiveDefinition<'a>),
    SchemaExtension(SchemaExtension<'a>),
    TypeExtension(TypeExtension<'a>),
}

#[derive(Clone, Debug)]
pub struct SchemaDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub directives: Vec<Directive<'a>>,
    pub definitions: Vec<(OperationType, Ident<'a>)>,
}

impl HasPos for SchemaDefinition<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum TypeDefinition<'a> {
    Scalar(ScalarTypeDefinition<'a>),
    Object(ObjectTypeDefinition<'a>),
    Interface(InterfaceTypeDefinition<'a>),
    Union(UnionTypeDefinition<'a>),
    Enum(EnumTypeDefinition<'a>),
    InputObject(InputObjectTypeDefinition<'a>),
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
}

impl HasPos for ScalarTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub implements: Vec<Ident<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<FieldDefinition<'a>>,
}

impl HasPos for ObjectTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct FieldDefinition<'a> {
    pub description: Option<StringValue>,
    pub name: Ident<'a>,
    pub arguments: Option<ArgumentsDefinition<'a>>,
    pub r#type: Type<'a>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub implements: Vec<Ident<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<FieldDefinition<'a>>,
}

impl HasPos for InterfaceTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub members: Vec<Ident<'a>>,
}

impl HasPos for UnionTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct DirectiveDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub arguments: Option<ArgumentsDefinition<'a>>,
    pub repeatable: Option<Ident<'a>>,
    pub locations: Vec<Ident<'a>>,
}

impl HasPos for DirectiveDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct ArgumentsDefinition<'a> {
    pub input_values: Vec<InputValueDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct InputValueDefinition<'a> {
    pub description: Option<StringValue>,
    pub name: Ident<'a>,
    pub r#type: Type<'a>,
    pub default_value: Option<Value<'a>>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub values: Vec<EnumValueDefinition<'a>>,
}

impl HasPos for EnumTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct EnumValueDefinition<'a> {
    pub description: Option<StringValue>,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<InputValueDefinition<'a>>,
}

impl HasPos for InputObjectTypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct SchemaExtension<'a> {
    pub position: Pos,
    pub directives: Vec<Directive<'a>>,
    pub definitions: Vec<(OperationType, Ident<'a>)>,
}

impl HasPos for SchemaExtension<'_> {
    fn name(&self) -> Option<&str> {
        None
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub enum TypeExtension<'a> {
    Scalar(ScalarTypeExtension<'a>),
    Object(ObjectTypeExtension<'a>),
    Interface(InterfaceTypeExtension<'a>),
    Union(UnionTypeExtension<'a>),
    Enum(EnumTypeExtension<'a>),
    InputObject(InputObjectTypeExtension<'a>),
}

#[derive(Clone, Debug)]
pub struct ScalarTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
}

impl HasPos for ScalarTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct ObjectTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub implements: Vec<Ident<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<FieldDefinition<'a>>,
}

impl HasPos for ObjectTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub implements: Vec<Ident<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<FieldDefinition<'a>>,
}

impl HasPos for InterfaceTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct UnionTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub members: Vec<Ident<'a>>,
}

impl HasPos for UnionTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct EnumTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub values: Vec<EnumValueDefinition<'a>>,
}

impl HasPos for EnumTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeExtension<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub fields: Vec<InputValueDefinition<'a>>,
}

impl HasPos for InputObjectTypeExtension<'_> {
    fn name(&self) -> Option<&str> {
        Some(&self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}
