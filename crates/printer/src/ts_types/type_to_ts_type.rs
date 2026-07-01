use nitrogql_ast::r#type::{NamedType, Type};

use super::super::ts_types::TSType;

pub fn get_ts_type_of_type(ty: &Type, map_name: impl FnOnce(&NamedType) -> TSType) -> TSType {
    let (ty, nullable) = get_ts_type_of_type_impl(ty, map_name);
    if nullable {
        TSType::Union(vec![ty, TSType::Null])
    } else {
        ty
    }
}

/// Variant of `get_ts_type_of_type` that does not add `| null`
/// even if given type is nullable.
pub fn get_ts_type_of_type_non_null(
    ty: &Type,
    map_name: impl FnOnce(&NamedType) -> TSType,
) -> TSType {
    let (ty, _) = get_ts_type_of_type_impl(ty, map_name);
    ty
}

/// With nullability flag
fn get_ts_type_of_type_impl(
    ty: &Type,
    map_name: impl FnOnce(&NamedType) -> TSType,
) -> (TSType, bool) {
    match ty {
        Type::Named(name) => (map_name(name), true),
        Type::List(ty) => (
            TSType::Array(Box::new(get_ts_type_of_type(&ty.r#type, map_name))),
            true,
        ),
        Type::NonNull(ty) => {
            let (tsty, _) = get_ts_type_of_type_impl(&ty.r#type, map_name);
            (tsty, false)
        }
    }
}
