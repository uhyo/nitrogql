/// Capitalizes given string.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => {
            // empty string
            String::new()
        }
        Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
    }
}
