//! Value-preservation tests for the typed command surface.
//!
//! Fixed witnesses containing `/`, `=`, spaces, and non-ASCII text prove that
//! each field stores its input unchanged rather than normalizing it.

use camino::Utf8PathBuf;
use pretty_assertions::assert_eq;
use rstest::rstest;

use super::parse;

/// A witness that every string-carrying field must store verbatim.
const WITNESS: &str = "feature/añadir = x";

#[rstest]
#[case::base("extract", "--base")]
#[case::branch("extract", "--branch")]
#[case::manifest_dir("extract", "--manifest-dir")]
#[case::check("validate", "--check")]
fn option_values_are_stored_verbatim(#[case] command: &str, #[case] option: &str) {
    let invocation = parse(&["splitters", command, "p", option, WITNESS])
        .expect("the witness value should parse");
    let stored = match invocation.cli.command {
        crate::cli::Commands::Extract(args) => match option {
            "--base" => args.base,
            "--branch" => args.branch,
            _ => args.manifest_dir.map(Utf8PathBuf::into_string),
        },
        crate::cli::Commands::Validate(args) => match option {
            "--check" => args.check,
            _ => args.manifest_dir.map(Utf8PathBuf::into_string),
        },
        crate::cli::Commands::Map(_) => panic!("the map command has no witness option"),
    };
    assert_eq!(
        stored.as_deref(),
        Some(WITNESS),
        "{option} must store its value unchanged",
    );
}

#[rstest]
#[case::validate("validate")]
#[case::extract("extract")]
fn the_proposal_operand_is_stored_verbatim(#[case] command: &str) {
    let invocation =
        parse(&["splitters", command, WITNESS]).expect("the witness operand should parse");
    let stored = match invocation.cli.command {
        crate::cli::Commands::Validate(args) => args.proposal,
        crate::cli::Commands::Extract(args) => args.proposal,
        crate::cli::Commands::Map(_) => panic!("the map command has no proposal operand"),
    };
    assert_eq!(
        stored, WITNESS,
        "the proposal operand must be stored unchanged",
    );
}

#[test]
fn operand_may_start_with_a_dash_after_the_separator() {
    let invocation = parse(&["splitters", "validate", "--", "-weird"])
        .expect("the separator should allow a dashed operand");
    let crate::cli::Commands::Validate(actual) = invocation.cli.command else {
        panic!("expected the validate command");
    };
    assert_eq!(
        actual.proposal, "-weird",
        "an operand after `--` must be stored verbatim",
    );
}
