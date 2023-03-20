/// Returns the (char_index, byte_index) of first non-space char.
pub fn first_non_space_byte_index(line: &str) -> Option<(usize, usize)> {
    line.char_indices()
        .enumerate()
        .find(|(_, (_, ch))| !ch.is_whitespace())
        .map(|(char_idx, (byte_idx, _))| (char_idx, byte_idx))
}

/// Skip `chars` characters and returns sub slice.
pub fn skip_chars(line: &str, chars: usize) -> &str {
    let bytes = line.chars().take(chars).map(|c| c.len_utf8()).sum();
    let (_, rest) = line.split_at(bytes);
    rest
}
