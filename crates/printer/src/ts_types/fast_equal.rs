use super::TSType;

/// Returns true if two types can be determined to be equal in a fast way.
pub fn fast_equal(left: &TSType, right: &TSType) -> bool {
    match (left, right) {
        (TSType::TypeVariable(l), TSType::TypeVariable(r)) => l.name == r.name,
        (TSType::TypeFunc(l, l_args), TSType::TypeFunc(r, r_args)) => {
            fast_equal(l, r)
                && l_args.len() == r_args.len()
                && l_args.iter().zip(r_args).all(|(l, r)| fast_equal(l, r))
        }
        (TSType::StringLiteral(l), TSType::StringLiteral(r)) => l == r,
        (TSType::NamespaceMember(l1, l2), TSType::NamespaceMember(r1, r2)) => l1 == r1 && l2 == r2,
        (TSType::NamespaceMember3(l1, l2, l3), TSType::NamespaceMember3(r1, r2, r3)) => {
            l1 == r1 && l2 == r2 && l3 == r3
        }
        (TSType::Object(l), TSType::Object(r)) => l.is_empty() && r.is_empty(),
        (TSType::Array(l), TSType::Array(r)) => fast_equal(l, r),
        (TSType::ReadonlyArray(l), TSType::ReadonlyArray(r)) => fast_equal(l, r),
        (TSType::Union(_), TSType::Union(_)) => false,
        (TSType::Intersection(_), TSType::Intersection(_)) => false,
        (TSType::Raw(l), TSType::Raw(r)) => l == r,
        (TSType::Never, TSType::Never) => true,
        (TSType::Null, TSType::Null) => true,
        (TSType::Undefined, TSType::Undefined) => true,
        _ => false,
    }
}
