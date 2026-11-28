//! Process-boundary tests for stream discipline, exit status, and no effects.
//!
//! The obligations are `INV-streams` and `INV-no-effects`: display requests use
//! standard output and exit `0`, every other diagnostic uses standard error and
//! exits `2`, and no invocation writes to the sandbox.

use std::ffi::OsStr;

use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::support::{Splitters, splitters_cmd};

/// The interim diagnostic every well-formed command ends with.
const NOT_IMPLEMENTED: &str = "not implemented yet";

#[rstest]
#[case::root_help(&["--help"])]
#[case::map_help(&["map", "--help"])]
#[case::validate_help(&["validate", "--help"])]
#[case::extract_help(&["extract", "--help"])]
#[case::help_subcommand(&["help", "validate"])]
#[case::version(&["--version"])]
fn display_requests_use_standard_output(splitters_cmd: Splitters, #[case] args: &[&str]) {
    let run = splitters_cmd.run(args);
    assert_eq!(run.status, Some(0), "display request {args:?} must succeed",);
    assert!(
        !run.stdout.is_empty(),
        "display request {args:?} must write to standard output",
    );
    assert_eq!(
        run.stderr, "",
        "display request {args:?} must leave standard error empty",
    );
}

#[rstest]
#[case::missing_validate_proposal(&["validate"])]
#[case::missing_extract_proposal(&["extract"])]
#[case::no_subcommand(&[])]
#[case::unknown_option(&["map", "--colour"])]
#[case::detached_boolean(&["extract", "p", "--publish", "false"])]
fn usage_errors_use_standard_error(splitters_cmd: Splitters, #[case] args: &[&str]) {
    let run = splitters_cmd.run(args);
    assert_eq!(run.status, Some(2), "usage error {args:?} must exit 2");
    assert_eq!(
        run.stdout, "",
        "usage error {args:?} must leave standard output empty",
    );
    assert!(
        !run.stderr.is_empty(),
        "usage error {args:?} must explain itself on standard error",
    );
    assert!(
        !run.stderr.contains(NOT_IMPLEMENTED),
        "usage error {args:?} must be a usage diagnostic, not the interim dispatch report; stderr \
         was {:?}",
        run.stderr,
    );
}

#[rstest]
fn a_detached_boolean_is_rejected_as_a_usage_error(splitters_cmd: Splitters) {
    let run = splitters_cmd.run(["extract", "p", "--publish", "false"]);
    assert_eq!(
        run.status,
        Some(2),
        "a detached boolean value must be a usage error",
    );
    assert_eq!(
        run.stdout, "",
        "a detached boolean value must leave standard output empty",
    );
    assert!(
        run.stderr.contains("Unknown argument"),
        "a detached boolean value must be an unknown-argument error; stderr was {:?}",
        run.stderr,
    );
    assert!(
        !run.stderr.contains(NOT_IMPLEMENTED),
        "a detached boolean value must not reach dispatch; stderr was {:?}",
        run.stderr,
    );
}

#[rstest]
#[case::map(&["map"], "map")]
#[case::validate(&["validate", "add-parser", "--check", "make test"], "validate")]
#[case::extract(&["extract", "add-parser"], "extract")]
#[case::extract_publish_false(&["extract", "add-parser", "--publish=false"], "extract")]
fn well_formed_commands_report_not_yet_implemented(
    splitters_cmd: Splitters,
    #[case] args: &[&str],
    #[case] command: &str,
) {
    let run = splitters_cmd.run(args);
    assert_eq!(
        run.status,
        Some(2),
        "well-formed {command} must report an operational failure",
    );
    assert_eq!(
        run.stdout, "",
        "well-formed {command} must keep standard output empty",
    );
    assert_eq!(
        run.stderr,
        format!("splitters {command}: {NOT_IMPLEMENTED}\n"),
        "well-formed {command} must report the interim outcome exactly",
    );
}

#[rstest]
fn version_reports_the_package_version(splitters_cmd: Splitters) {
    let run = splitters_cmd.run(["--version"]);
    assert_eq!(run.status, Some(0), "`--version` must succeed");
    assert_eq!(
        run.stdout,
        format!("splitters {}\n", env!("CARGO_PKG_VERSION")),
        "`--version` must print the package version",
    );
}

#[rstest]
#[case::root_help(&["--help"])]
#[case::map(&["map"])]
#[case::validate(&["validate", "add-parser"])]
#[case::extract(&["extract", "add-parser", "--publish"])]
#[case::usage_error(&["extract"])]
fn commands_leave_the_sandbox_untouched(splitters_cmd: Splitters, #[case] args: &[&str]) {
    let run = splitters_cmd.run(args);
    assert!(
        run.status.is_some(),
        "the child must exit normally for {args:?}",
    );
    splitters_cmd
        .sandbox()
        .assert_untouched(&format!("{args:?}"))
        .expect("the sandbox scan must complete and report an untouched tree");
}

#[cfg(unix)]
#[rstest]
fn a_non_utf8_operand_is_a_usage_error(splitters_cmd: Splitters) {
    use std::os::unix::ffi::OsStrExt;

    let operand = OsStr::from_bytes(b"\xff");
    let Ok(output) = splitters_cmd
        .command()
        .arg("validate")
        .arg(operand)
        .output()
    else {
        panic!("the splitters binary should start");
    };
    assert_eq!(
        output.status.code(),
        Some(2),
        "a non-UTF-8 operand must be a usage error, not a panic",
    );
    assert!(
        output.stdout.is_empty(),
        "a non-UTF-8 operand must leave standard output empty",
    );
    assert!(
        !output.stderr.is_empty(),
        "a non-UTF-8 operand must be explained on standard error",
    );
}
