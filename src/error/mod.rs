use colored::Colorize;
use std::cmp::min;
use std::fmt::{Display, Write};
use std::path::PathBuf;
use std::result::Result as StdResult;

use crate::graphql_parser::ast::base::Pos;

/// Error that may be positioned.
pub struct PositionedError {
    inner: anyhow::Error,
    position: Option<Pos>,
    additional_info: Vec<(Pos, String)>,
}

pub type Result<T> = StdResult<T, PositionedError>;

impl PositionedError {
    pub fn new(
        inner: anyhow::Error,
        position: Option<Pos>,
        additional_info: Vec<(Pos, String)>,
    ) -> Self {
        PositionedError {
            inner,
            position,
            additional_info,
        }
    }
}

impl<E> From<E> for PositionedError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        PositionedError {
            inner: value.into(),
            position: None,
            additional_info: vec![],
        }
    }
}

static INDENT: &str = "    ";

pub fn print_positioned_error(error: &PositionedError, files: &[(PathBuf, &str)]) -> String {
    let inner = &error.inner;
    let Some(position) = error.position else {
        return format!("{inner}");
    };

    let source = files.get(position.file);
    let Some((file_path, source)) = source else {
        return format!("{inner}");
    };

    let mut message = message_for_line(file_path, source, position, inner, false);

    for (pos, mes) in error.additional_info.iter() {
        let source = files.get(pos.file);
        let Some((file_path, source)) = source else {
            continue;
        };

        write!(
            message,
            "\n\n{}",
            message_for_line(file_path, source, *pos, mes, true)
        )
        .unwrap();
    }

    message
}

fn message_for_line(
    file_path: &PathBuf,
    source: &str,
    pos: Pos,
    error: &impl Display,
    is_additional: bool,
) -> String {
    // print -2 ~ +2 lines
    let relevant_lines = source
        .lines()
        .enumerate()
        .skip(pos.line.saturating_sub(2))
        .take(5)
        .collect::<Vec<_>>();
    if relevant_lines
        .iter()
        .all(|(line_no, _)| *line_no != pos.line)
    {
        // No targeted line (?)
        return format!("{error}");
    }

    let minimum_indent = relevant_lines
        .iter()
        .filter_map(|(_, line)| first_non_space_byte_index(line).map(|(char_idx, _)| char_idx))
        .min();
    let Some(minimum_indent) = minimum_indent else {
        return format!("{error}");
    };

    // VSCode uses 1-based indices, so we follow here
    let src_string = format!(
        "{}:{}:{}\n",
        file_path.display(),
        pos.line + 1,
        pos.column + 1
    );

    let trimmed_column = pos.column.saturating_sub(minimum_indent);

    let mut result = src_string;

    for (line_no, source_line) in relevant_lines {
        let trimmed_line = skip_chars(source_line, minimum_indent);
        let spaces = " ".repeat(trimmed_column);
        if line_no != pos.line {
            let printed_line = trimmed_line.bright_black();
            if is_additional {
                result.push_str(&format!("{INDENT}{printed_line}\n"));
            } else {
                result.push_str(&format!("{printed_line}\n"));
            }
        } else {
            if is_additional {
                let error_str = format!("{error}").bright_green().underline();
                result.push_str(&format!(
                    "{INDENT}{trimmed_line}\n{INDENT}{spaces}^\n{INDENT}{spaces}{error_str}\n"
                ));
            } else {
                let error_str = format!("{error}").bright_yellow().underline();
                result.push_str(&format!("{trimmed_line}\n{spaces}^\n{spaces}{error_str}\n"));
            }
        }
    }
    result
}

/// Returns the (char_index, byte_index) of first non-space char.
fn first_non_space_byte_index(line: &str) -> Option<(usize, usize)> {
    line.char_indices()
        .enumerate()
        .find(|(_, (_, ch))| !ch.is_whitespace())
        .map(|(char_idx, (byte_idx, _))| (char_idx, byte_idx))
}

fn skip_chars(line: &str, chars: usize) -> &str {
    let bytes = line.chars().take(chars).map(|c| c.len_utf8()).sum();
    let (_, rest) = line.split_at(bytes);
    rest
}
