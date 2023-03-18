use nitrogql_ast::r#type::Type;

use super::super::ts_types::TSType;

pub fn get_ts_type_of_type(ty: &Type) -> TSType {
    let (ty, nullable) = get_ts_type_of_type_impl(ty);
    if nullable {
        TSType::Union(vec![ty, TSType::Null])
    } else {
        ty
    }
}

/// With nullability flag
fn get_ts_type_of_type_impl(ty: &Type) -> (TSType, bool) {
    match ty {
        Type::Named(name) => (TSType::TypeVariable((&name.name).into()), true),
        Type::List(ty) => (
            TSType::Array(Box::new(get_ts_type_of_type(&ty.r#type))),
            true,
        ),
        Type::NonNull(ty) => {
            let (tsty, _) = get_ts_type_of_type_impl(&ty.r#type);
            (tsty, false)
        }
    }
}
