//! Runs the users' guide's tested examples against the binary.
//!
//! The guide owns the only exact copy of rendered help. Each marked example is
//! executed and compared here, and an identifier the test does not know about
//! fails the suite so a new example cannot be added without being exercised.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::support::{
    Splitters,
    doc_examples::{self, DocExample},
    splitters_cmd,
};

/// The guide under test.
const GUIDE: &str = include_str!("../../docs/users-guide.md");

/// The identifiers this test knows how to run.
const HANDLED: &[&str] = &[
    "splitters-help",
    "splitters-map-help",
    "splitters-validate-help",
    "splitters-extract-help",
];

#[test]
fn the_guide_declares_exactly_the_handled_examples() {
    let examples = doc_examples::load(GUIDE).expect("the users' guide must load");
    let declared: BTreeSet<String> = examples.into_iter().map(|example| example.id).collect();
    let handled: BTreeSet<String> = HANDLED.iter().map(ToString::to_string).collect();
    assert_eq!(
        declared, handled,
        "every marked example must have a known identifier",
    );
}

#[rstest]
#[case::help("splitters-help")]
#[case::map_help("splitters-map-help")]
#[case::validate_help("splitters-validate-help")]
#[case::extract_help("splitters-extract-help")]
fn the_documented_output_matches_the_binary(splitters_cmd: Splitters, #[case] identifier: &str) {
    let examples = doc_examples::load(GUIDE).expect("the users' guide must load");
    let example = examples
        .iter()
        .find(|example| example.id == identifier)
        .expect("the guide must declare the requested example");
    let actual =
        run_documented(&splitters_cmd, example).expect("the documented example must run cleanly");
    let (_, documented) = example
        .command_and_output()
        .expect("the documented example must start with a command");
    assert_eq!(
        actual, documented,
        "the {identifier} example must match the binary",
    );
}

/// Runs one documented example and returns its captured standard output.
fn run_documented(splitters_cmd: &Splitters, example: &DocExample) -> Result<String, String> {
    let (command, _) = example
        .command_and_output()
        .map_err(|error| error.to_string())?;
    let argv = shlex::split(command).ok_or_else(|| format!("cannot parse {command:?}"))?;
    let program = argv
        .first()
        .ok_or_else(|| format!("the example {command:?} names no program"))?;
    if program != "splitters" {
        return Err(format!("the example {command:?} must invoke splitters"));
    }
    let run = splitters_cmd.run(argv.iter().skip(1));
    if run.status != Some(0) {
        return Err(format!(
            "the example {command:?} exited {:?} with stderr {:?}",
            run.status, run.stderr,
        ));
    }
    Ok(run.stdout)
}
