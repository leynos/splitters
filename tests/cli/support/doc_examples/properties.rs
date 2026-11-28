//! Property tests for the tested-example loader's fenced-block grammar.
//!
//! The example tests pin one point per fence rule. These properties sample the
//! structural axes a wrong comparison would hide: delimiter character, opener
//! and closer lengths, opening and closing indentation, and trailing blanks.
//! Each property names the CommonMark rule it checks, so a shrunken
//! counterexample reads as a defect in that rule rather than in the fixture.

use proptest::prelude::*;

use super::{DocExample, ExampleError, load};

/// Strategy for the two characters CommonMark allows as a fence delimiter.
fn fence_marker() -> impl Strategy<Value = char> { prop_oneof![Just('`'), Just('~')] }

/// Builds one delimiter line with `indent` leading spaces.
fn fence_line(marker: char, length: usize, indent: usize) -> String {
    let mut line = " ".repeat(indent);
    line.extend(std::iter::repeat_n(marker, length));
    line
}

/// Strategy for one generated line that contains no line break.
fn generated_line() -> impl Strategy<Value = String> {
    prop::collection::vec(
        any::<char>().prop_filter("not a line break", |ch| *ch != '\n'),
        0..=12,
    )
    .prop_map(|chars| chars.into_iter().collect::<String>())
}

/// Joins `payloads` as lines, each indented by `indent` spaces.
fn indented_lines(payloads: &[&str], indent: usize) -> String {
    let mut text = String::new();
    let padding = " ".repeat(indent);
    for payload in payloads {
        text.push_str(&padding);
        text.push_str(payload);
        text.push('\n');
    }
    text
}

proptest! {
    /// A closer shorter than its opener is content, never a closing fence.
    #[test]
    fn a_shorter_closer_never_closes_the_block(
        marker in fence_marker(),
        opener_length in 3usize..=8,
        opener_indent in 0usize..=3,
        closer_length in 0usize..=8,
        closer_indent in 0usize..=5,
    ) {
        prop_assume!(closer_length < opener_length);
        let document = format!(
            "<!-- tested-example: demo -->\n{opener}\n$ splitters --help\n{closer}\n",
            opener = fence_line(marker, opener_length, opener_indent),
            closer = fence_line(marker, closer_length, closer_indent),
        );
        prop_assert_eq!(
            load(&document),
            Err(ExampleError::UnterminatedFence { line: 1 }),
            "a closer shorter than the opener must not close the block: {:?}",
            document,
        );
    }

    /// Four or more leading spaces form an indented code block, not a fence.
    #[test]
    fn an_opener_indented_four_or_more_spaces_never_opens_a_block(
        marker in fence_marker(),
        length in 3usize..=8,
        opener_indent in 4usize..=5,
        closer_length in 3usize..=10,
        closer_indent in 4usize..=5,
    ) {
        let document = format!(
            "<!-- tested-example: demo -->\n{fence}\n$ splitters --help\n{closer}\n",
            fence = fence_line(marker, length, opener_indent),
            closer = fence_line(marker, closer_length, closer_indent),
        );
        prop_assert_eq!(
            load(&document),
            Ok(Vec::new()),
            "an indented code block must contribute no example: {:?}",
            document,
        );
    }

    /// A marked block loads its identifier and its indentation-stripped body.
    #[test]
    fn a_marked_block_loads_one_indentation_stripped_example(
        marker in fence_marker(),
        opener_length in 3usize..=8,
        opener_indent in 0usize..=3,
        closer_length in 3usize..=10,
        closer_indent in 0usize..=3,
        trailing in "[ \t]{0,3}",
        extra_indent in 0usize..=2,
        identifier in "[a-z][a-z0-9-]{0,12}",
    ) {
        prop_assume!(closer_length >= opener_length);
        let payloads = ["$ splitters --help", "Usage: splitters"];
        let body = indented_lines(&payloads, opener_indent + extra_indent);
        let expected_body = indented_lines(&payloads, extra_indent);
        let opener = fence_line(marker, opener_length, opener_indent);
        let closer = format!(
            "{}{trailing}",
            fence_line(marker, closer_length, closer_indent),
        );
        let document =
            format!("<!-- tested-example: {identifier} -->\n{opener}\n{body}{closer}\n");
        prop_assert_eq!(
            load(&document),
            Ok(vec![DocExample {
                id: identifier,
                body: expected_body,
            }]),
            "a marked block must load with its identifier and stripped body: {:?}",
            document,
        );
    }

    /// A shorter run of the fence character inside a block is body content.
    #[test]
    fn a_shorter_run_inside_the_body_is_content(
        marker in fence_marker(),
        opener_length in 3usize..=8,
        short_length in 0usize..=7,
        opener_indent in 0usize..=3,
        closer_indent in 0usize..=3,
    ) {
        prop_assume!(short_length < opener_length);
        let content = fence_line(marker, short_length, opener_indent);
        let document = format!(
            "<!-- tested-example: demo -->\n{opener}\n{content}\n{closer}\n",
            opener = fence_line(marker, opener_length, opener_indent),
            closer = fence_line(marker, opener_length, closer_indent),
        );
        let expected_body = format!("{}\n", fence_line(marker, short_length, 0));
        prop_assert_eq!(
            load(&document),
            Ok(vec![DocExample {
                id: "demo".to_owned(),
                body: expected_body,
            }]),
            "a shorter run inside the body must be content: {:?}",
            document,
        );
    }

    /// A fence with no marker before it is rejected at its own line.
    #[test]
    fn an_unmarked_fence_is_rejected(
        marker in fence_marker(),
        opener_length in 3usize..=8,
        opener_indent in 0usize..=3,
        closer_length in 3usize..=10,
        prefix in 0usize..=2,
    ) {
        prop_assume!(closer_length >= opener_length);
        let document = format!(
            "{prefix}{opener}\n$ splitters --help\n{closer}\n",
            prefix = "Intro paragraph\n".repeat(prefix),
            opener = fence_line(marker, opener_length, opener_indent),
            closer = fence_line(marker, closer_length, 0),
        );
        prop_assert_eq!(
            load(&document),
            Err(ExampleError::UnmarkedFence { line: prefix }),
            "an unmarked fence must be rejected at line {}: {:?}",
            prefix,
            document,
        );
    }

    /// Splitting succeeds exactly when the body starts with a `$ ` command.
    #[test]
    fn command_and_output_requires_a_dollar_prompt(
        has_prompt in any::<bool>(),
        content in generated_line(),
        tail in prop::collection::vec(generated_line(), 0..=2),
    ) {
        let first = if has_prompt {
            format!("$ {content}")
        } else {
            prop_assume!(!content.starts_with("$ "));
            content.clone()
        };
        let mut lines = vec![first];
        lines.extend(tail.iter().cloned());
        let expected_output = tail.join("\n");
        let example = DocExample {
            id: "demo".to_owned(),
            body: lines.join("\n"),
        };
        let outcome = example.command_and_output();
        let expected = if has_prompt {
            Ok((content.as_str(), expected_output.as_str()))
        } else {
            Err(ExampleError::MissingCommand {
                identifier: "demo".to_owned(),
            })
        };
        prop_assert_eq!(outcome, expected);
    }
}
