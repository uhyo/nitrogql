use sourcemap_writer::SourceMapWriter;

/// Print string in the GraphQL string literal format.
pub fn print_string(s: &str, writer: &mut impl SourceMapWriter) {
    let mut result = String::with_capacity(s.len());
    let is_multiline = s.find('\n').is_some();
    if is_multiline {
        // print as multiline string
        result.push_str("\"\"\"");
        let mut dq_count: usize = 0;
        for c in s.chars() {
            if c != '"' {
                if dq_count > 0 {
                    result.push_str(&"\"".repeat(dq_count));
                    dq_count = 0;
                }
                result.push(c);
                continue;
            }
            dq_count += 1;
            if dq_count == 3 {
                // """ in string
                result.push_str("\\\"\"\"");
                dq_count = 0;
            }
        }
        if dq_count > 0 {
            result.push_str(&"\"".repeat(dq_count));
        }
        result.push_str("\"\"\"");
        writer.write(&result);
    } else {
        // single line string
        result.push('"');
        for c in s.chars() {
            match c {
                '\r' => result.push_str("\\r"),
                '\n' => result.push_str("\\n"),
                c if c.is_control() => {
                    result.push_str(&format!("\\u{{{:x}}}", c as u32));
                }
                c => result.push(c),
            }
        }
        result.push('"');
        writer.write(&result);
    }
}
