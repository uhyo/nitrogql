/// If `value` is Some, clone contained value into given `target`.
pub fn clone_into<T: Clone>(value: &Option<T>, target: &mut T) {
    if let Some(value) = value {
        *target = value.clone();
    }
}
