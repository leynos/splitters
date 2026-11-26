//! Structural tests for the loader's marker and fence rules.

use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{DocExample, ExampleError, load};

#[test]
fn loads_a_marked_tilde_fence() {
    let document = concat!(
        "<!-- tested-example: tilde -->\n",
        "~~~console\n",
        "$ splitters --help\n",
        "Usage: splitters\n",
        "~~~\n",
    );
    let examples = load(document).expect("the tilde example should load");
    let example = examples
        .first()
        .expect("a tilde fence should load one example");
    assert_eq!(example.id, "tilde", "the marker names the tilde example");
}

#[test]
fn an_unmarked_tilde_fence_is_rejected() {
    let document = "~~~console\n$ splitters --help\n~~~\n";
    assert_eq!(
        load(document),
        Err(ExampleError::UnmarkedFence { line: 0 }),
        "a tilde fence must carry a marker too",
    );
}

#[test]
fn a_shorter_closing_fence_does_not_close_a_longer_fence() {
    let document = concat!(
        "<!-- tested-example: long -->\n",
        "````console\n",
        "```rust\n",
        "```\n",
    );
    assert_eq!(
        load(document),
        Err(ExampleError::UnterminatedFence { line: 1 }),
        "only a fence at least as long as the opener may close it",
    );
}

#[test]
fn a_four_space_indented_delimiter_with_no_closer_is_unterminated() {
    let document = concat!(
        "<!-- tested-example: indented-close -->\n",
        "```console\n",
        "$ splitters --help\n",
        "    ```\n",
    );
    assert_eq!(
        load(document),
        Err(ExampleError::UnterminatedFence { line: 1 }),
        "a four-space-indented delimiter must not close the block",
    );
}

#[test]
fn a_delimiter_followed_by_unicode_whitespace_does_not_close() {
    let document = concat!(
        "<!-- tested-example: unicode-space -->\n",
        "```console\n",
        "$ splitters --help\n",
        "```\u{00A0}\n",
    );
    assert_eq!(
        load(document),
        Err(ExampleError::UnterminatedFence { line: 1 }),
        "only spaces or tabs may follow a closing fence",
    );
}

#[test]
fn a_closer_may_have_three_spaces_and_trailing_blanks() {
    let document = concat!(
        "<!-- tested-example: padded-close -->\n",
        "```console\n",
        "$ splitters --help\n",
        "   ```  \t\n",
    );
    let examples = load(document).expect("the padded closer should close the block");
    let example = examples.first().expect("one example should load");
    assert_eq!(
        example.body, "$ splitters --help\n",
        "up to three leading spaces and trailing blanks are allowed on a closer",
    );
}

#[test]
fn a_backtick_info_string_with_a_backtick_is_content() {
    let document = concat!(
        "<!-- tested-example: info -->\n",
        "```console\n",
        "```rust`not-a-fence\n",
        "```\n",
    );
    let examples = load(document).expect("the backtick block should load");
    let example = examples.first().expect("one backtick example should load");
    assert_eq!(
        example.body, "```rust`not-a-fence\n",
        "a backtick info string may not contain a backtick, so the line is content",
    );
}

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
        "a marked example must load with its identifier and fence body",
    );
}

#[test]
fn a_four_space_indented_fence_is_content_inside_a_block() {
    let document = concat!(
        "<!-- tested-example: indented -->\n",
        "```console\n",
        "$ splitters --help\n",
        "    ```\n",
        "Usage: splitters\n",
        "```\n",
    );
    let examples = load(document).expect("the block should load");
    let example = examples.first().expect("one example should load");
    assert_eq!(
        example.body, "$ splitters --help\n    ```\nUsage: splitters\n",
        "a four-space-indented fence line is content, not a closer",
    );
}

#[test]
fn a_three_space_indented_fence_opens_a_block() {
    let document = concat!(
        "<!-- tested-example: indented-open -->\n",
        "   ```console\n",
        "   $ splitters --help\n",
        "   ```\n",
    );
    let examples = load(document).expect("the indented block should load");
    let example = examples.first().expect("one example should load");
    assert_eq!(
        example.body, "$ splitters --help\n",
        "the opening fence's indentation is removed from content lines",
    );
    assert_eq!(
        example.command_and_output(),
        Ok(("splitters --help", "")),
        "an indented example must still split into its command and output",
    );
}

#[test]
fn a_four_space_indented_fence_does_not_open_a_block() {
    let document = concat!(
        "<!-- tested-example: not-a-fence -->\n",
        "    ```console\n",
        "    $ splitters --help\n",
        "    ```\n",
    );
    let examples = load(document).expect("an indented code block is not a fence");
    assert_eq!(
        examples.len(),
        0,
        "four spaces make an indented code block, which the loader must ignore",
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
