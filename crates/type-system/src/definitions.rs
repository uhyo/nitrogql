use crate::{
    cloning_utils::{map_option_node, map_vec_node},
    node::{Node, OriginalNodeRef},
    text::Text,
    r#type::Type,
};

/// Definition of a schema type.
#[derive(Debug, Clone)]
pub enum TypeDefinition<Str, OriginalNode> {
    Scalar(ScalarDefinition<Str, OriginalNode>),
    Object(ObjectDefinition<Str, OriginalNode>),
    Interface(InterfaceDefinition<Str, OriginalNode>),
    Union(UnionDefinition<Str, OriginalNode>),
    Enum(EnumDefinition<Str, OriginalNode>),
    InputObject(InputObjectDefinition<Str, OriginalNode>),
}

impl<'a, Str: Text<'a>, OriginalNode> TypeDefinition<Str, OriginalNode> {
    /// Get name of this type.
    pub fn name(&self) -> &Str {
        match self {
            TypeDefinition::Scalar(def) => &def.name,
            TypeDefinition::Object(def) => &def.name,
            TypeDefinition::Interface(def) => &def.name,
            TypeDefinition::Union(def) => &def.name,
            TypeDefinition::Enum(def) => &def.name,
            TypeDefinition::InputObject(def) => &def.name,
        }
    }
    /// Get description of this type.
    pub fn description(&self) -> Option<&str> {
        match self {
            TypeDefinition::Scalar(def) => def.description.as_ref().map(|opt| opt.borrow()),
            TypeDefinition::Object(def) => def.description.as_ref().map(|opt| opt.borrow()),
            TypeDefinition::Interface(def) => def.description.as_ref().map(|opt| opt.borrow()),
            TypeDefinition::Union(def) => def.description.as_ref().map(|opt| opt.borrow()),
            TypeDefinition::Enum(def) => def.description.as_ref().map(|opt| opt.borrow()),
            TypeDefinition::InputObject(def) => def.description.as_ref().map(|opt| opt.borrow()),
        }
    }

    /// Returns Some if self is an output object type.
    pub fn as_object(&self) -> Option<&ObjectDefinition<Str, OriginalNode>> {
        match self {
            TypeDefinition::Object(def) => Some(def),
            _ => None,
        }
    }

    /// Returns Some if self is an interface type.
    pub fn as_interface(&self) -> Option<&InterfaceDefinition<Str, OriginalNode>> {
        match self {
            TypeDefinition::Interface(def) => Some(def),
            _ => None,
        }
    }

    /// Returns Some if self is a union type.
    pub fn as_union(&self) -> Option<&UnionDefinition<Str, OriginalNode>> {
        match self {
            TypeDefinition::Union(def) => Some(def),
            _ => None,
        }
    }

    /// Returns Some if self is an input object type.
    pub fn as_input_object(&self) -> Option<&InputObjectDefinition<Str, OriginalNode>> {
        match self {
            TypeDefinition::InputObject(def) => Some(def),
            _ => None,
        }
    }
}

impl<Str, OriginalNode> TypeDefinition<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> TypeDefinition<U, OriginalNode> {
        match self {
            TypeDefinition::Scalar(def) => TypeDefinition::Scalar(ScalarDefinition {
                name: def.name.as_ref().map(&f),
                description: map_option_node(&def.description, &f),
            }),
            TypeDefinition::Object(def) => TypeDefinition::Object(ObjectDefinition {
                name: def.name.as_ref().map(&f),
                description: map_option_node(&def.description, &f),
                fields: def.fields.iter().map(|x| x.map_str(&f)).collect(),
                interfaces: map_vec_node(&def.interfaces, &f),
            }),
            TypeDefinition::Interface(def) => TypeDefinition::Interface(InterfaceDefinition {
                name: def.name.as_ref().map(&f),
                description: map_option_node(&def.description, &f),
                fields: def.fields.iter().map(|x| x.map_str(&f)).collect(),
                interfaces: map_vec_node(&def.interfaces, &f),
            }),
            TypeDefinition::Union(def) => TypeDefinition::Union(UnionDefinition {
                name: def.name.as_ref().map(&f),
                description: map_option_node(&def.description, &f),
                possible_types: map_vec_node(&def.possible_types, &f),
            }),
            TypeDefinition::Enum(def) => TypeDefinition::Enum(EnumDefinition {
                name: def.name.as_ref().map(&f),
                description: map_option_node(&def.description, &f),
                members: def.members.iter().map(|x| x.map_str(&f)).collect(),
            }),
            TypeDefinition::InputObject(def) => {
                TypeDefinition::InputObject(InputObjectDefinition {
                    name: def.name.as_ref().map(&f),
                    description: map_option_node(&def.description, &f),
                    fields: def.fields.iter().map(|x| x.map_str(&f)).collect(),
                })
            }
        }
    }
}

impl<Str, OriginalNode> OriginalNodeRef<OriginalNode> for TypeDefinition<Str, OriginalNode> {
    fn original_node_ref(&self) -> &OriginalNode {
        match self {
            TypeDefinition::Scalar(def) => def.name.original_node_ref(),
            TypeDefinition::Object(def) => def.name.original_node_ref(),
            TypeDefinition::Interface(def) => def.name.original_node_ref(),
            TypeDefinition::Union(def) => def.name.original_node_ref(),
            TypeDefinition::Enum(def) => def.name.original_node_ref(),
            TypeDefinition::InputObject(def) => def.name.original_node_ref(),
        }
    }
}

/// Definition of a scalar type.
#[derive(Debug, Clone)]
pub struct ScalarDefinition<Str, OriginalNode> {
    /// Name of scalar.
    pub name: Node<Str, OriginalNode>,
    /// Description of scalar.
    pub description: Option<Node<Str, OriginalNode>>,
}

/// Definition of an (output) object type.
#[derive(Debug, Clone)]
pub struct ObjectDefinition<Str, OriginalNode> {
    /// Name of object.
    pub name: Node<Str, OriginalNode>,
    /// Description of object.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Field definitions.
    pub fields: Vec<Field<Str, OriginalNode>>,
    /// List of interfaces implemented by this object.
    pub interfaces: Vec<Node<Str, OriginalNode>>,
}

/// Definition of an interface type.
#[derive(Debug, Clone)]
pub struct InterfaceDefinition<Str, OriginalNode> {
    /// Name of interface.
    pub name: Node<Str, OriginalNode>,
    /// Description of interface.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Field definitions.
    pub fields: Vec<Field<Str, OriginalNode>>,
    /// List of interfaces implemented by this interface.
    pub interfaces: Vec<Node<Str, OriginalNode>>,
}

/// Definition of a union type.
#[derive(Debug, Clone)]
pub struct UnionDefinition<Str, OriginalNode> {
    /// Name of union.
    pub name: Node<Str, OriginalNode>,
    /// Description of union.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Possible object types.
    pub possible_types: Vec<Node<Str, OriginalNode>>,
}

/// Definition of a union type.
#[derive(Debug, Clone)]
pub struct EnumDefinition<Str, OriginalNode> {
    /// Name of enum.
    pub name: Node<Str, OriginalNode>,
    /// Description of enum.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Enum members.
    pub members: Vec<EnumMember<Str, OriginalNode>>,
}

/// Definition of an input object type.
#[derive(Debug, Clone)]
pub struct InputObjectDefinition<Str, OriginalNode> {
    /// Name of object.
    pub name: Node<Str, OriginalNode>,
    /// Description of object.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Field definitions.
    pub fields: Vec<InputValue<Str, OriginalNode>>,
}

/// Represents one field in an object type.
#[derive(Debug, Clone)]
pub struct Field<Str, OriginalNode> {
    /// Name of field.
    pub name: Node<Str, OriginalNode>,
    /// Description of field.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Type of field.
    pub r#type: Type<Str, OriginalNode>,
    /// Arguments of this field. Empty list means no args.
    pub arguments: Vec<InputValue<Str, OriginalNode>>,
    /// If deprecated, contains the reason.
    pub deprecation: Option<Str>,
}

impl<Str, OriginalNode> Field<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> Field<U, OriginalNode> {
        Field {
            name: self.name.as_ref().map(&f),
            description: map_option_node(&self.description, &f),
            r#type: self.r#type.map_str(&f),
            arguments: self.arguments.iter().map(|x| x.map_str(&f)).collect(),
            deprecation: self.deprecation.as_ref().map(&f),
        }
    }
}

impl<Str, OriginalNode> OriginalNodeRef<OriginalNode> for Field<Str, OriginalNode> {
    fn original_node_ref(&self) -> &OriginalNode {
        self.name.original_node_ref()
    }
}

/// Represents an argument to a field.
#[derive(Debug, Clone)]
pub struct InputValue<Str, OriginalNode> {
    /// Name of input value.
    pub name: Node<Str, OriginalNode>,
    /// Description of input value.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Type of input value.
    pub r#type: Type<Str, OriginalNode>,
    /// Default value of input value.
    pub default_value: Option<Node<Str, OriginalNode>>,
    /// If deprecated, contains the reason.
    pub deprecation: Option<Str>,
}

impl<Str, OriginalNode> InputValue<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> InputValue<U, OriginalNode> {
        InputValue {
            name: self.name.as_ref().map(&f),
            description: map_option_node(&self.description, &f),
            r#type: self.r#type.map_str(&f),
            default_value: map_option_node(&self.default_value, &f),
            deprecation: self.deprecation.as_ref().map(&f),
        }
    }
}

impl<Str, OriginalNode> OriginalNodeRef<OriginalNode> for InputValue<Str, OriginalNode> {
    fn original_node_ref(&self) -> &OriginalNode {
        self.name.original_node_ref()
    }
}

/// Represents an enum member.
#[derive(Debug, Clone)]
pub struct EnumMember<Str, OriginalNode> {
    /// Name of enum member.
    pub name: Node<Str, OriginalNode>,
    /// Description of enum member.
    pub description: Option<Node<Str, OriginalNode>>,
    /// If deprecated, contains the reason.
    pub deprecation: Option<Str>,
}

impl<Str, OriginalNode> EnumMember<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> EnumMember<U, OriginalNode> {
        EnumMember {
            name: self.name.as_ref().map(&f),
            description: map_option_node(&self.description, &f),
            deprecation: self.deprecation.as_ref().map(&f),
        }
    }
}

/// Represents a directive definition.
#[derive(Debug, Clone)]
pub struct DirectiveDefinition<Str, OriginalNode> {
    /// Name of directive (does not include `@`)
    pub name: Node<Str, OriginalNode>,
    /// Description of directive.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Locations where this directive can be used.
    pub locations: Vec<Node<Str, OriginalNode>>,
    /// Arguments of this directive. Empty list means no args.
    pub arguments: Vec<InputValue<Str, OriginalNode>>,
    /// Whether this is repeatable. Some means this is repeatable.
    pub repeatable: Option<Node<(), OriginalNode>>,
}

impl<Str, OriginalNode> DirectiveDefinition<Str, OriginalNode> {
    /// Get name of this directive.
    pub fn name(&self) -> &Str {
        &self.name
    }
}

impl<Str, OriginalNode> DirectiveDefinition<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> DirectiveDefinition<U, OriginalNode> {
        DirectiveDefinition {
            name: self.name.as_ref().map(&f),
            description: map_option_node(&self.description, &f),
            locations: map_vec_node(&self.locations, &f),
            arguments: self.arguments.iter().map(|x| x.map_str(&f)).collect(),
            repeatable: self.repeatable.clone(),
        }
    }
}
