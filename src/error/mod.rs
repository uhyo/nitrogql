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

    let Some(source_line) = source.lines().nth(position.line) else {
        return format!("{inner}");
    };

    let mut message = message_for_line(file_path, source_line, position, inner, false);

    for (pos, mes) in error.additional_info.iter() {
        let source = files.get(position.file);
        let Some((file_path, source)) = source else {
            continue;
        };
        let Some(source_line) = source.lines().nth(position.line) else {
            continue;
        };

        write!(
            message,
            "\n\n{}",
            message_for_line(file_path, source_line, *pos, mes, true)
        )
        .unwrap();
    }

    message
}

fn message_for_line(
    file_path: &PathBuf,
    source_line: &str,
    pos: Pos,
    error: &impl Display,
    is_additional: bool,
) -> String {
    let Some((char_idx, byte_idx)) = first_non_space_byte_index(source_line) else {
        return format!("{error}");
    };

    // VSCode uses 1-based indices, so we follow here
    let src_string = format!(
        "{}:{}:{}",
        file_path.display(),
        pos.line + 1,
        pos.column + 1
    );

    let (_, trimmed_line) = source_line.split_at(byte_idx);
    let trimmed_column = pos.column.saturating_sub(char_idx);

    let spaces = " ".repeat(trimmed_column);
    if is_additional {
        format!("{INDENT}{src_string}\n{INDENT}{trimmed_line}\n{INDENT}{spaces}^\n{INDENT}{spaces}{error}")
    } else {
        format!("{src_string}\n{trimmed_line}\n{spaces}^\n{spaces}{error}")
    }
}

/// Returns the (char_index, byte_index) of first non-space char.
fn first_non_space_byte_index(line: &str) -> Option<(usize, usize)> {
    line.char_indices()
        .enumerate()
        .find(|(_, (_, ch))| !ch.is_whitespace())
        .map(|(char_idx, (byte_idx, _))| (char_idx, byte_idx))
}
