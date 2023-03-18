use nitrogql_ast::{type_system::{InterfaceTypeDefinition, FieldDefinition}, base::Ident};
use nitrogql_semantics::DefinitionMap;
use crate::{error::CheckError, types::is_subtype};

use super::CheckErrorMessage;

/// Checks if given object or interface validly implements given interface.
/// https://spec.graphql.org/draft/#IsValidImplementation()
pub fn check_valid_implementation(
    definitions: &DefinitionMap,
    object_name: &Ident,
    fields: &[FieldDefinition],
    implements: &[Ident],
    interface: &InterfaceTypeDefinition,
    result: &mut Vec<CheckError>,
) {
    // If implementedType declares it implements any interfaces, type must also declare it implements those interfaces.
    for imp in interface.implements.iter() {
        if !implements.iter().any(|ident| ident.name == imp.name) {
            result.push(CheckErrorMessage::InterfaceNotImplemented {
                name: imp.name.to_owned(),
            }.with_pos(object_name.position));
        }

    }
    // type must include a field of the same name for every field defined in implementedType.
    for imp_field in interface.fields.iter() {
        let Some(field) = fields.iter().find(|field| imp_field.name.name == field.name.name) else {
            result.push(CheckErrorMessage::InterfaceFieldNotImplemented {
                field_name: imp_field.name.name.to_owned(),
                interface_name: 
                interface.name.name.to_owned()
             }.with_pos(object_name.position));
             continue;
        };

        // field must include an argument of the same name for every argument defined in implementedField.
        for imp_arg in imp_field.arguments.iter().flat_map(|args| args.input_values.iter()) {
            let Some(field_arg) =
                field.arguments
                .iter()
                .flat_map(|args| args.input_values.iter())
                .find(|arg| arg.name.name == imp_arg.name.name)
            else {
                result.push(CheckErrorMessage::InterfaceArgumentNotImplemented {
                    argument_name: imp_arg.name.name.to_owned(),
                    interface_name: interface.name.name.to_owned(),
                }.with_pos(field.name.position));
                continue;
            };
            // That named argument on field must accept the same type (invariant) as that named argument on implementedField.
            if !field_arg.r#type.is_same(&imp_arg.r#type) {
                result.push(CheckErrorMessage::ArgumentTypeMisMatchWithInterface { interface_name: interface.name.name.to_owned() }.with_pos(field_arg.name.position));
            }
        }
        // field may include additional arguments not defined in implementedField, but any additional argument must not be required, e.g. must not be of a non-nullable type.
        if let Some(ref arguments) = field.arguments {
            for field_arg in
                arguments.input_values
                .iter()
                .filter(|arg| {
                    imp_field.arguments
                        .iter()
                        .flat_map(|imp_args| imp_args.input_values.iter())
                        .find(|imp_arg| imp_arg.name.name == arg.name.name)
                        .is_none()
                }) {
                if field_arg.r#type.is_nonnull() {
                    result.push(CheckErrorMessage::ArgumentTypeNonNullAgainstInterface { interface_name: interface.name.name.to_owned() }.with_pos(field_arg.name.position));
                }
            }
        }
        // field must return a type which is equal to or a sub-type of (covariant) the return type of implementedField field’s return type:
        if is_subtype(definitions, &field.r#type, &imp_field.r#type) == Some(false) {
            result.push(CheckErrorMessage::FieldTypeMisMatchWithInterface {
                interface_name: interface.name.name.to_owned(),
            }.with_pos(field.name.position));
        }
    }
}
