use graphql_parser::schema::Type;

use super::ts_types::TSType;

pub fn get_ts_type_of_type(ty: &Type<'_, String>) -> TSType {
    let (ty, nullable) = get_ts_type_of_type_impl(ty);
    if nullable {
        TSType::Union(vec![ty, TSType::Null])
    } else {
        ty
    }
}

/// With nullability flag
fn get_ts_type_of_type_impl(ty: &Type<'_, String>) -> (TSType, bool) {
    match ty {
        Type::NamedType(name) => (TSType::TypeVariable(name.clone()), true),
        Type::ListType(ty) => (TSType::Array(Box::new(get_ts_type_of_type(ty))), true),
        Type::NonNullType(ty) => {
            let (tsty, _) = get_ts_type_of_type_impl(ty);
            (tsty, false)
        }
    }
}
