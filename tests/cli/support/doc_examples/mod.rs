//! Loader for marked, executable examples in user-facing Markdown documents.
//!
//! A fenced block is executable when the line immediately before its opening
//! fence is `<!-- tested-example: <identifier> -->`. The loader extracts and
//! validates those fences; comparing their output with the document stays with
//! the calling test. Any fence without a marker, any unterminated fence, a
//! marker without an identifier, and a repeated identifier are structural
//! defects, so a document cannot silently carry unchecked examples.

use std::{collections::BTreeSet, fmt};

/// The marker prefix that makes the next fence executable.
const MARKER_PREFIX: &str = "<!-- tested-example:";
/// The marker suffix that closes an executable marker.
const MARKER_SUFFIX: &str = "-->";

/// One executable example extracted from a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocExample {
    /// The stable identifier from the marker.
    pub id: String,
    /// The fence body, including its command line.
    pub body: String,
}

impl DocExample {
    /// Splits the fence body into the documented command and expected output.
    ///
    /// # Errors
    ///
    /// Returns [`ExampleError::MissingCommand`] when the first line does not
    /// start with the documented shell prompt.
    pub fn command_and_output(&self) -> Result<(&str, &str), ExampleError> {
        let (first, rest) = self.body.split_once('\n').unwrap_or((&self.body, ""));
        let command = first
            .strip_prefix("$ ")
            .ok_or_else(|| ExampleError::MissingCommand {
                identifier: self.id.clone(),
            })?;
        Ok((command, rest))
    }
}

/// A structural defect in a tested-example document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExampleError {
    /// A fenced block has no tested-example marker before it.
    UnmarkedFence {
        /// Zero-based line number of the opening fence.
        line: usize,
    },
    /// A fence opener has no matching closer.
    UnterminatedFence {
        /// Zero-based line number of the opening fence.
        line: usize,
    },
    /// A marker carries no identifier.
    MissingIdentifier {
        /// Zero-based line number of the marker.
        line: usize,
    },
    /// Two markers use the same identifier.
    DuplicateIdentifier {
        /// The repeated identifier.
        identifier: String,
        /// Zero-based line number of the second marker.
        line: usize,
    },
    /// An example's first line is not a documented command.
    MissingCommand {
        /// The example's identifier.
        identifier: String,
    },
}

impl fmt::Display for ExampleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmarkedFence { line } => {
                write!(
                    formatter,
                    "fence at line {line} has no tested-example marker"
                )
            }
            Self::UnterminatedFence { line } => {
                write!(formatter, "fence opened at line {line} is never closed")
            }
            Self::MissingIdentifier { line } => {
                write!(
                    formatter,
                    "tested-example marker at line {line} has no identifier"
                )
            }
            Self::DuplicateIdentifier { identifier, line } => write!(
                formatter,
                "tested-example identifier {identifier:?} is repeated at line {line}",
            ),
            Self::MissingCommand { identifier } => write!(
                formatter,
                "tested example {identifier:?} must start with a `$ ` command line",
            ),
        }
    }
}

impl std::error::Error for ExampleError {}

/// Loads every executable example from `document`, in document order.
///
/// # Errors
///
/// Returns the first structural defect in [`ExampleError`].
pub fn load(document: &str) -> Result<Vec<DocExample>, ExampleError> {
    let lines = document.lines().collect::<Vec<_>>();
    let mut examples = Vec::new();
    let mut seen = BTreeSet::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines.get(index).copied().unwrap_or_default();
        let Some(opening) = opening_fence(line) else {
            index += 1;
            continue;
        };
        let marker = index.checked_sub(1).and_then(|before| lines.get(before));
        let marker_identifier = marker
            .and_then(|previous| parse_marker(previous, index - 1))
            .transpose()?;
        let Some(identifier) = marker_identifier else {
            return Err(ExampleError::UnmarkedFence { line: index });
        };
        if !seen.insert(identifier.clone()) {
            return Err(ExampleError::DuplicateIdentifier {
                identifier,
                line: index - 1,
            });
        }
        let (body, next) = fence_body(&lines, index, opening)?;
        examples.push(DocExample {
            id: identifier,
            body,
        });
        index = next;
    }
    Ok(examples)
}

/// One fenced block's delimiter, as CommonMark defines it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fence {
    /// The delimiter character: a backtick or a tilde.
    marker: char,
    /// How many delimiter characters the opening line had.
    length: usize,
    /// How many spaces the opening delimiter was indented by.
    indent: usize,
}

/// Returns the fence opened by `line`, or `None` when it opens no block.
///
/// A fence is three or more backticks or tildes. A backtick fence's info
/// string may not contain a backtick, so a line such as ```` ```foo`bar ````
/// is content rather than a delimiter.
fn opening_fence(line: &str) -> Option<Fence> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let trimmed = line.get(indent..)?;
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let length = trimmed.chars().take_while(|ch| *ch == marker).count();
    if length < 3 {
        return None;
    }
    let info = trimmed.get(length..).unwrap_or_default();
    if marker == '`' && info.contains('`') {
        return None;
    }
    Some(Fence {
        marker,
        length,
        indent,
    })
}

/// Removes up to `max` leading spaces from a content line.
///
/// CommonMark removes the opening fence's indentation from each content line
/// and keeps any indentation beyond it.
fn strip_indent(line: &str, max: usize) -> &str {
    let spaces = line.chars().take_while(|ch| *ch == ' ').count();
    line.get(spaces.min(max)..).unwrap_or(line)
}

/// Returns whether `line` closes the block opened by `opening`.
///
/// A closing fence uses the same character, is at least as long as the
/// opening fence, and has only spaces or tabs after its delimiter; other
/// Unicode whitespace does not close a block.
fn closing_fence(line: &str, opening: Fence) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }
    let trimmed = line.get(indent..).unwrap_or_default();
    let marker_count = trimmed
        .chars()
        .take_while(|ch| *ch == opening.marker)
        .count();
    if marker_count < opening.length {
        return false;
    }
    let rest = trimmed.get(marker_count..).unwrap_or_default();
    rest.chars().all(|ch| ch == ' ' || ch == '\t')
}

/// Parses one marker line.
///
/// Returns `Ok(None)` when the line is not a marker at all, and
/// [`ExampleError::MissingIdentifier`] when it is a marker with an empty body.
fn parse_marker(line: &str, at: usize) -> Option<Result<String, ExampleError>> {
    let without_prefix = line.trim().strip_prefix(MARKER_PREFIX)?;
    let inner = without_prefix.strip_suffix(MARKER_SUFFIX)?.trim();
    if inner.is_empty() {
        return Some(Err(ExampleError::MissingIdentifier { line: at }));
    }
    Some(Ok(inner.to_owned()))
}

/// Extracts the body of the fence opened at `open`, returning it and the index
/// of the first line after the closing fence.
fn fence_body(
    lines: &[&str],
    open: usize,
    opening: Fence,
) -> Result<(String, usize), ExampleError> {
    let mut body = Vec::new();
    let mut index = open + 1;
    while let Some(line) = lines.get(index).copied() {
        if closing_fence(line, opening) {
            let mut text = body.join("\n");
            text.push('\n');
            return Ok((text, index + 1));
        }
        body.push(strip_indent(line, opening.indent));
        index += 1;
    }
    Err(ExampleError::UnterminatedFence { line: open })
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
