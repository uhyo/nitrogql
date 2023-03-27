use super::{
    base::{HasPos, Ident, Keyword, Pos},
    directive::Directive,
    operation::OperationType,
    r#type::Type,
    value::{StringValue, Value},
};

#[derive(Clone, Debug)]
pub enum TypeSystemDefinition<'a> {
    SchemaDefinition(SchemaDefinition<'a>),
    TypeDefinition(TypeDefinition<'a>),
    DirectiveDefinition(DirectiveDefinition<'a>),
}

impl HasPos for TypeSystemDefinition<'_> {
    fn name(&self) -> Option<&str> {
        match self {
            TypeSystemDefinition::SchemaDefinition(def) => def.name(),
            TypeSystemDefinition::TypeDefinition(def) => HasPos::name(def),
            TypeSystemDefinition::DirectiveDefinition(def) => def.name(),
        }
    }
    fn position(&self) -> &Pos {
        match self {
            TypeSystemDefinition::SchemaDefinition(def) => def.position(),
            TypeSystemDefinition::TypeDefinition(def) => def.position(),
            TypeSystemDefinition::DirectiveDefinition(def) => def.position(),
        }
    }
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

impl TypeDefinition<'_> {
    pub fn name(&self) -> &Ident {
        match self {
            TypeDefinition::Scalar(def) => &def.name,
            TypeDefinition::Object(def) => &def.name,
            TypeDefinition::Interface(def) => &def.name,
            TypeDefinition::Union(def) => &def.name,
            TypeDefinition::Enum(def) => &def.name,
            TypeDefinition::InputObject(def) => &def.name,
        }
    }
}

impl HasPos for TypeDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(self.name().name)
    }
    fn position(&self) -> &Pos {
        match self {
            TypeDefinition::Scalar(def) => def.position(),
            TypeDefinition::Object(def) => def.position(),
            TypeDefinition::Interface(def) => def.position(),
            TypeDefinition::Union(def) => def.position(),
            TypeDefinition::Enum(def) => def.position(),
            TypeDefinition::InputObject(def) => def.position(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    // keywords & punctuations
    pub scalar_keyword: Keyword<'a>,
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
    // keywords & punctuations
    pub type_keyword: Keyword<'a>,
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
    // keywords & punctuations
    pub interface_keyword: Keyword<'a>,
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
    // keywords & punctuations
    pub union_keyword: Keyword<'a>,
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
    // keywords & punctuations
    pub directive_keyword: Keyword<'a>,
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
    pub position: Pos,
    pub name: Ident<'a>,
    pub r#type: Type<'a>,
    pub default_value: Option<Value<'a>>,
    pub directives: Vec<Directive<'a>>,
}

impl HasPos for InputValueDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition<'a> {
    pub description: Option<StringValue>,
    pub position: Pos,
    pub name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub values: Vec<EnumValueDefinition<'a>>,
    // keywords & punctuations
    pub enum_keyword: Keyword<'a>,
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
    // keywords & punctuations
    pub input_keyword: Keyword<'a>,
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

#[derive(Clone, Debug)]
pub struct TypeSystemOrExtensionDocument<'a> {
    pub definitions: Vec<TypeSystemDefinitionOrExtension<'a>>,
}

impl<'a> Extend<TypeSystemDefinitionOrExtension<'a>> for TypeSystemOrExtensionDocument<'a> {
    fn extend<T: IntoIterator<Item = TypeSystemDefinitionOrExtension<'a>>>(&mut self, iter: T) {
        self.definitions.extend(iter)
    }
}

impl TypeSystemOrExtensionDocument<'_> {
    /// Merges multiple documents into one.
    pub fn merge(docs: impl IntoIterator<Item = Self>) -> Self {
        TypeSystemOrExtensionDocument {
            definitions: docs.into_iter().flat_map(|doc| doc.definitions).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeSystemDocument<'a> {
    pub definitions: Vec<TypeSystemDefinition<'a>>,
}

impl TypeSystemDocument<'_> {
    pub fn new() -> Self {
        TypeSystemDocument {
            definitions: Vec::new(),
        }
    }
}
