//! Parsing behaviour for the typed command surface.
//!
//! The tables pin accepted spellings, rejected spellings, and the matches
//! clap records; `values` pins the values that reach each argument struct.

use camino::Utf8PathBuf;
use clap::error::ErrorKind;
use pretty_assertions::assert_eq;
use rstest::rstest;

use super::parse;
use crate::cli::args::{ExtractArgs, MapArgs, ValidateArgs};

/// Returns the error kind produced by a rejected command line.
fn error_kind(args: &[&str]) -> ErrorKind {
    match parse(args) {
        Ok(_) => panic!("expected {args:?} to be rejected"),
        Err(error) => error.kind(),
    }
}

#[rstest]
#[case::map_accepts_both_options(
    &["splitters", "map", "--base", "main", "--manifest-dir", "splitters"],
    MapArgs {
        base: Some("main".to_owned()),
        manifest_dir: Some(Utf8PathBuf::from("splitters")),
    },
)]
#[case::map_defers_both_options(&["splitters", "map"], MapArgs::default())]
fn map_parses_typed_fields(#[case] args: &[&str], #[case] expected: MapArgs) {
    let invocation = parse(args).expect("the command line should parse");
    let crate::cli::Commands::Map(actual) = invocation.cli.command else {
        panic!("expected the map command");
    };
    assert_eq!(actual, expected);
}

#[rstest]
#[case::all_fields(
    &[
        "splitters", "validate", "add-parser", "--manifest-dir", "splitters",
        "--base", "main", "--check", "make test",
    ],
    ValidateArgs {
        proposal: "add-parser".to_owned(),
        manifest_dir: Some(Utf8PathBuf::from("splitters")),
        base: Some("main".to_owned()),
        check: Some("make test".to_owned()),
    },
)]
#[case::only_proposal(
    &["splitters", "validate", "add-parser"],
    ValidateArgs {
        proposal: "add-parser".to_owned(),
        ..ValidateArgs::default()
    },
)]
fn validate_parses_typed_fields(#[case] args: &[&str], #[case] expected: ValidateArgs) {
    let invocation = parse(args).expect("the command line should parse");
    let crate::cli::Commands::Validate(actual) = invocation.cli.command else {
        panic!("expected the validate command");
    };
    assert_eq!(actual, expected);
}

#[rstest]
#[case::all_fields(
    &[
        "splitters", "extract", "add-parser", "--manifest-dir", "splitters",
        "--base", "main", "--branch", "feature/add-parser", "--publish=false",
    ],
    ExtractArgs {
        proposal: "add-parser".to_owned(),
        manifest_dir: Some(Utf8PathBuf::from("splitters")),
        base: Some("main".to_owned()),
        branch: Some("feature/add-parser".to_owned()),
        publish: Some(false),
    },
)]
#[case::only_proposal(
    &["splitters", "extract", "add-parser"],
    ExtractArgs {
        proposal: "add-parser".to_owned(),
        ..ExtractArgs::default()
    },
)]
fn extract_parses_typed_fields(#[case] args: &[&str], #[case] expected: ExtractArgs) {
    let invocation = parse(args).expect("the command line should parse");
    let crate::cli::Commands::Extract(actual) = invocation.cli.command else {
        panic!("expected the extract command");
    };
    assert_eq!(actual, expected);
}

#[rstest]
#[case::bare_flag(&["splitters", "extract", "p", "--publish"], Some(true))]
#[case::explicit_true(&["splitters", "extract", "p", "--publish=true"], Some(true))]
#[case::explicit_false(&["splitters", "extract", "p", "--publish=false"], Some(false))]
#[case::omitted(&["splitters", "extract", "p"], None)]
fn publish_uses_the_documented_spellings(#[case] args: &[&str], #[case] expected: Option<bool>) {
    let invocation = parse(args).expect("the command line should parse");
    let crate::cli::Commands::Extract(actual) = invocation.cli.command else {
        panic!("expected the extract command");
    };
    assert_eq!(actual.publish, expected);
}

#[rstest]
#[case::detached_value(&["splitters", "extract", "p", "--publish", "false"], ErrorKind::UnknownArgument)]
#[case::non_boolean(&["splitters", "extract", "p", "--publish=maybe"], ErrorKind::InvalidValue)]
#[case::missing_proposal(&["splitters", "validate"], ErrorKind::MissingRequiredArgument)]
#[case::unknown_option(&["splitters", "map", "--colour"], ErrorKind::UnknownArgument)]
fn malformed_command_lines_are_rejected(#[case] args: &[&str], #[case] expected: ErrorKind) {
    assert_eq!(
        error_kind(args),
        expected,
        "unexpected error kind for {args:?}"
    );
}

#[test]
fn matches_record_the_supplied_arguments() {
    let invocation = parse(&["splitters", "extract", "p", "--branch", "feature/x"])
        .expect("the command line should parse");
    let subcommand = invocation
        .matches
        .subcommand_matches("extract")
        .expect("the extract subcommand should be recorded");
    assert_eq!(
        subcommand.get_one::<String>("branch"),
        Some(&"feature/x".to_owned()),
        "the branch value must be preserved in clap's matches",
    );
    assert!(
        subcommand.get_one::<String>("base").is_none(),
        "an omitted argument must stay absent in clap's matches",
    );
    assert!(
        invocation.matches.subcommand_matches("map").is_none(),
        "only the chosen subcommand may be recorded",
    );
}
