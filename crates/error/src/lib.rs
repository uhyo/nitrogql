use colored::Colorize;
use std::fmt::{Display, Write};
use std::ops::Index;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use nitrogql_ast::base::Pos;
use nitrogql_utils::{first_non_space_byte_index, skip_chars};

mod code;

pub use code::ErrorCode;

/// Error that may be positioned.
#[derive(Debug)]
pub struct PositionedError {
    inner: anyhow::Error,
    position: Option<Pos>,
    additional_info: Vec<(Pos, String)>,
    code: Option<ErrorCode>,
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
            code: None,
        }
    }

    /// Sets the code of the error.
    pub fn with_code(mut self, code: impl Into<Option<ErrorCode>>) -> Self {
        self.code = code.into();
        self
    }

    pub fn code(&self) -> Option<ErrorCode> {
        self.code
    }

    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    pub fn position(&self) -> Option<Pos> {
        self.position
    }

    pub fn into_inner(self) -> anyhow::Error {
        self.inner
    }

    pub fn inner(&self) -> &anyhow::Error {
        &self.inner
    }

    pub fn additional_info(&self) -> &[(Pos, String)] {
        &self.additional_info
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
            code: None,
        }
    }
}

static INDENT: &str = "    ";

pub fn print_positioned_error<S: AsRef<str>, T>(
    error: &PositionedError,
    files: &impl Index<usize, Output = (PathBuf, S, T)>,
) -> String {
    let inner = match error.code {
        Some(code) => format!("{code}: {}", error.inner),
        None => error.inner.to_string(),
    };
    let Some(position) = error.position else {
        return inner;
    };
    if position.builtin {
        return inner;
    }

    let (ref file_path, ref source, _) = files[position.file];

    let mut message = message_for_line(file_path, source.as_ref(), position, &inner, false);

    for (pos, mes) in error.additional_info.iter() {
        if pos.builtin {
            continue;
        }
        let (ref file_path, ref source, _) = files[pos.file];

        write!(
            message,
            "\n\n{}",
            message_for_line(file_path, source.as_ref(), *pos, mes, true)
        )
        .unwrap();
    }

    message
}

fn message_for_line(
    file_path: &Path,
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
    )
    .bold();

    let trimmed_column = pos.column.saturating_sub(minimum_indent);

    let mut result = if is_additional {
        format!("{INDENT}{src_string}")
    } else {
        src_string.to_string()
    };

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
        } else if is_additional {
            let error_str = format!("{error}").bright_green().underline();
            result.push_str(&format!(
                "{INDENT}{trimmed_line}\n{INDENT}{spaces}^\n{INDENT}{spaces}{error_str}\n"
            ));
        } else {
            let error_str = format!("{error}").bright_yellow().underline();
            result.push_str(&format!("{trimmed_line}\n{spaces}^\n{spaces}{error_str}\n"));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nitrogql_ast::base::Pos;

    use super::{ErrorCode, PositionedError, print_positioned_error};

    #[test]
    fn print_code() {
        colored::control::set_override(false);
        let files = vec![(PathBuf::from("a.graphql"), "query { foo }", ())];
        let error = PositionedError::new(
            anyhow::anyhow!("Field 'foo' is not found on type 'Query'"),
            Some(Pos::new(0, 8, 8..11, 0)),
            vec![],
        )
        .with_code(ErrorCode::FieldNotFound);
        assert_eq!(
            print_positioned_error(&error, &files),
            "a.graphql:1:9\nquery { foo }\n        ^\n        nitrogql4007: Field 'foo' is not found on type 'Query'\n"
        );

        let error = PositionedError::from(anyhow::anyhow!("Config file not found"))
            .with_code(ErrorCode::ProjectError);
        assert_eq!(
            print_positioned_error(&error, &files),
            "nitrogql9002: Config file not found"
        );

        let error = PositionedError::from(anyhow::anyhow!("Config file not found"));
        assert_eq!(
            print_positioned_error(&error, &files),
            "Config file not found"
        );
    }
}
