use std::{
    error::Error,
    fmt::{Display, Write},
};

use crate::graphql_parser::ast::base::Pos;

/// Error that have position information,
pub trait PositionedError {
    fn position(&self) -> Pos;
    fn additional_info(&self) -> Vec<(Pos, String)>;
}

static INDENT: &str = "    ";

pub fn print_positioned_error<Err>(error: &Err, files: &[&str]) -> String
where
    Err: PositionedError + Error,
{
    let position = error.position();
    let source = files.get(position.file);
    let Some(source) = source else {
        return format!("{error}");
    };

    let Some(source_line) = source.lines().nth(position.line) else {
        return format!("{error}");
    };

    let mut message = message_for_line(source_line, position, error, false);

    for (pos, mes) in error.additional_info() {
        let source = files.get(position.file);
        let Some(source) = source else {
            continue;
        };
        let Some(source_line) = source.lines().nth(position.line) else {
            continue;
        };

        write!(
            message,
            "\n\n{}",
            message_for_line(source_line, pos, &mes, true)
        )
        .unwrap();
    }

    message
}

fn message_for_line(
    source_line: &str,
    pos: Pos,
    error: &impl Display,
    is_additional: bool,
) -> String {
    let Some((char_idx, byte_idx)) = first_non_space_byte_index(source_line) else {
        return format!("{error}");
    };

    let (_, trimmed_line) = source_line.split_at(byte_idx);
    let trimmed_column = pos.column.saturating_sub(char_idx);

    let spaces = " ".repeat(trimmed_column);
    if is_additional {
        format!("{INDENT}{trimmed_line}\n{INDENT}{spaces}^\n{INDENT}{spaces}{error}")
    } else {
        format!("{trimmed_line}\n{spaces}^\n{spaces}{error}")
    }
}

/// Returns the (char_index, byte_index) of first non-space char.
fn first_non_space_byte_index(line: &str) -> Option<(usize, usize)> {
    line.char_indices()
        .enumerate()
        .find(|(_, (_, ch))| !ch.is_whitespace())
        .map(|(char_idx, (byte_idx, _))| (char_idx, byte_idx))
}
