use std::ops::Not;

use super::TSType;

/// Calculates intersection of given list of types.
pub fn ts_intersection(types: Vec<TSType>) -> TSType {
    let mut types = types.into_iter();

    let Some(fst) = types.next() else {
        return TSType::Unknown;
    };
    let Some(snd) = types.next() else {
        return fst;
    };
    let types = vec![fst, snd].into_iter().chain(types).collect::<Vec<_>>();

    let (object_props, others) =
        types
            .into_iter()
            .fold((vec![], vec![]), |(mut props, mut others), ty| {
                if let TSType::Object(mut ps) = ty {
                    props.append(&mut ps);
                    (props, others)
                } else {
                    others.push(ty);
                    (props, others)
                }
            });
    let object_type = object_props
        .is_empty()
        .not()
        .then(|| TSType::Object(object_props));
    if others.is_empty() {
        object_type.unwrap_or(TSType::Unknown)
    } else {
        TSType::Intersection(object_type.into_iter().chain(others).collect())
    }
}
