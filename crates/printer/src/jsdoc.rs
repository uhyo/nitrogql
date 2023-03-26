use nitrogql_utils::{first_non_space_byte_index, skip_chars};
use sourcemap_writer::SourceMapWriter;

pub fn print_description(description: &str, writer: &mut impl SourceMapWriter) {
    let desc = dedent(&description);
    writer.write("/**\n");
    for line in desc.lines() {
        writer.write(" * ");
        writer.write(line);
        writer.write("\n");
    }
    writer.write(" */\n");
}

/// Removes extra indentation and surrounding empty lines from given string.
/// Note: returned string includes final '\n'.
fn dedent(value: &str) -> String {
    let lines = value.lines().skip_while(|s| s.is_empty());
    let lines = skip_last_if(lines, |s| s.is_empty());
    let lines = lines.collect::<Vec<_>>();

    let minimum_indent = lines
        .iter()
        .filter_map(|line| first_non_space_byte_index(line).map(|(char_idx, _)| char_idx))
        .min()
        .unwrap_or(0);

    let mut result = String::new();
    for line in lines {
        let line = skip_chars(line, minimum_indent);
        if line.is_empty() {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

fn skip_last_if<T>(
    iter: impl Iterator<Item = T>,
    pred: impl Fn(&T) -> bool,
) -> impl Iterator<Item = T> {
    SkipLastIf {
        buffer: vec![],
        iter,
        pred,
        maybe_end: true,
    }
}

struct SkipLastIf<T, I, F>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> bool,
{
    buffer: Vec<T>,
    iter: I,
    pred: F,
    maybe_end: bool,
}

impl<T, I, F> Iterator for SkipLastIf<T, I, F>
where
    I: Iterator<Item = T>,
    F: Fn(&T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            if self.maybe_end {
                match self.iter.next() {
                    Some(elem) if (self.pred)(&elem) => {
                        self.buffer.push(elem);
                        continue;
                    }
                    Some(elem) => {
                        self.buffer.push(elem);
                        self.maybe_end = false;
                        self.buffer.reverse();
                    }
                    None => return None,
                }
            } else {
                match self.buffer.pop() {
                    Some(elem) => return Some(elem),
                    None => {
                        self.maybe_end = true;
                    }
                };
            };
        }
    }
}
