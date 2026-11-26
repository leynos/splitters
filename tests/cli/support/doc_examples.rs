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
        if !is_fence(line) {
            index += 1;
            continue;
        }
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
        let (body, next) = fence_body(&lines, index)?;
        examples.push(DocExample {
            id: identifier,
            body,
        });
        index = next;
    }
    Ok(examples)
}

/// Returns whether `line` opens or closes a fenced block.
fn is_fence(line: &str) -> bool { line.trim_start().starts_with("```") }

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
fn fence_body(lines: &[&str], open: usize) -> Result<(String, usize), ExampleError> {
    let mut body = Vec::new();
    let mut index = open + 1;
    while let Some(line) = lines.get(index).copied() {
        if is_fence(line) {
            let mut text = body.join("\n");
            text.push('\n');
            return Ok((text, index + 1));
        }
        body.push(line);
        index += 1;
    }
    Err(ExampleError::UnterminatedFence { line: open })
}

#[cfg(test)]
mod tests {
    //! Structural tests for the loader's marker and fence rules.

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{DocExample, ExampleError, load};

    #[test]
    fn loads_one_marked_example() {
        let document = concat!(
            "# Guide\n\n",
            "<!-- tested-example: demo -->\n",
            "```console\n",
            "$ splitters --help\n",
            "Usage: splitters\n",
            "```\n",
        );
        let examples = load(document).expect("the document should load");
        assert_eq!(
            examples,
            vec![DocExample {
                id: "demo".to_owned(),
                body: "$ splitters --help\nUsage: splitters\n".to_owned(),
            }],
        );
    }

    #[test]
    fn an_unmarked_fence_is_rejected() {
        let document = "```console\n$ splitters --help\n```\n";
        assert_eq!(
            load(document),
            Err(ExampleError::UnmarkedFence { line: 0 }),
            "every fence must carry a marker",
        );
    }

    #[test]
    fn an_unterminated_fence_is_rejected() {
        let document = "<!-- tested-example: demo -->\n```console\n$ splitters --help\n";
        assert_eq!(
            load(document),
            Err(ExampleError::UnterminatedFence { line: 1 }),
            "a fence without a closer must fail",
        );
    }

    #[test]
    fn a_marker_without_an_identifier_is_rejected() {
        let document = "<!-- tested-example:  -->\n```console\n$ splitters --help\n```\n";
        assert_eq!(
            load(document),
            Err(ExampleError::MissingIdentifier { line: 0 }),
            "a marker must name its example",
        );
    }

    #[test]
    fn a_duplicate_identifier_is_rejected() {
        let document = concat!(
            "<!-- tested-example: demo -->\n```console\n$ splitters --help\n```\n",
            "<!-- tested-example: demo -->\n```console\n$ splitters --version\n```\n",
        );
        assert_eq!(
            load(document),
            Err(ExampleError::DuplicateIdentifier {
                identifier: "demo".to_owned(),
                line: 4,
            }),
            "identifiers must be unique within a document",
        );
    }

    #[rstest]
    #[case::command("$ splitters --help\nUsage: splitters\n", Ok(("splitters --help", "Usage: splitters\n")))]
    #[case::missing("Usage: splitters\n", Err(ExampleError::MissingCommand { identifier: "demo".to_owned() }))]
    fn splits_the_command_from_the_expected_output(
        #[case] body: &str,
        #[case] expected: Result<(&str, &str), ExampleError>,
    ) {
        let example = DocExample {
            id: "demo".to_owned(),
            body: body.to_owned(),
        };
        assert_eq!(
            example.command_and_output(),
            expected,
            "the command line and expected output must split at the first newline",
        );
    }
}
