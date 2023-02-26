use crate::graphql_parser::ast::{type_system::{InterfaceTypeDefinition, ObjectTypeDefinition}, r#type::{Type, NamedType}, base::Ident};

use super::{CheckTypeSystemError, types::is_subtype, definition_map::DefinitionMap};

/// Checks if given object validly implements given interface.
/// https://spec.graphql.org/draft/#IsValidImplementation()
pub fn check_valid_implementation(
    definitions: &DefinitionMap,
    object: &ObjectTypeDefinition,
    interface: &InterfaceTypeDefinition,
    result: &mut Vec<CheckTypeSystemError>,
) {
    // If implementedType declares it implements any interfaces, type must also declare it implements those interfaces.
    for imp in interface.implements.iter() {
        if !object.implements.iter().any(|ident| ident.name == imp.name) {
            result.push(CheckTypeSystemError::InterfaceNotImplemented {
                position: object.name.position,
                name: imp.name.to_owned(),
            });
        }

    }
    // type must include a field of the same name for every field defined in implementedType.
    for imp_field in interface.fields.iter() {
        let Some(field) = object.fields.iter().find(|field| imp_field.name.name == field.name.name) else {
            result.push(CheckTypeSystemError::InterfaceFieldNotImplemented {
                position: object.name.position,
                field_name: imp_field.name.name.to_owned(),
                interface_name: 
                interface.name.name.to_owned()
             });
             continue;
        };

        // field must include an argument of the same name for every argument defined in implementedField.
        for imp_arg in imp_field.arguments.iter().flat_map(|args| args.input_values.iter()) {
            let Some(field_arg) = field.arguments.iter().flat_map(|args| args.input_values.iter())
            .find(|arg| arg.name.name == imp_arg.name.name)
            
             else {
                result.push(CheckTypeSystemError::InterfaceArgumentNotImplemented { position: field.name.position,
                    argument_name: imp_arg.name.name.to_owned(), interface_name: interface.name.name.to_owned() });
                continue;
            };
            // That named argument on field must accept the same type (invariant) as that named argument on implementedField.
            if !field_arg.r#type.is_same(&imp_arg.r#type) {
                result.push(CheckTypeSystemError::ArgumentTypeMisMatchWithInterface { position: field_arg.name.position, interface_name: interface.name.name.to_owned() });
            }
        }
        // field may include additional arguments not defined in implementedField, but any additional argument must not be required, e.g. must not be of a non-nullable type.
        if let Some(ref arguments) = field.arguments {
            for field_arg in arguments.input_values.iter().filter(|arg| {
                imp_field.arguments.iter().flat_map(|imp_args| imp_args.input_values.iter()).find(|imp_arg| imp_arg.name.name == arg.name.name)
                .is_none()
            }) {
                if field_arg.r#type.is_nonnull() {
                    result.push(CheckTypeSystemError::ArgumentTypeNonNullAgainstInterface { position: field_arg.name.position, interface_name: interface.name.name.to_owned() })
                }
            }
        }
        // field must return a type which is equal to or a sub-type of (covariant) the return type of implementedField field’s return type:
        if is_subtype(definitions, &field.r#type, &imp_field.r#type) == Some(false) {
            result.push(CheckTypeSystemError::FieldTypeMisMatchWithInterface {
                position: field.name.position,
                interface_name: interface.name.name.to_owned(),
            });
        }
    }
}
