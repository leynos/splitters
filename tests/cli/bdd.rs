//! Behaviour-driven scenarios for the declared command surface.
//!
//! The scenarios link `REQ-help`, `REQ-parse-coverage`, and `REQ-streams` to
//! the executable.

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, scenario, then, when};

use crate::support::{Run, Splitters, splitters_cmd};

/// The streams and status of the most recent command in this scenario.
#[derive(Default, ScenarioState)]
struct SurfaceState {
    /// Captured output of the last command.
    run: Slot<Run>,
}

#[fixture]
fn surface_state() -> SurfaceState {
    let run = Slot::new();
    SurfaceState { run }
}

#[when("the operator runs {cmd:string}")]
fn the_operator_runs(surface_state: &SurfaceState, splitters_cmd: &Splitters, cmd: String) {
    let Some(argv) = shlex::split(&cmd) else {
        panic!("the scenario command must be shell syntax: {cmd}");
    };
    let Some(program) = argv.first() else {
        panic!("the scenario command must name a program: {cmd}");
    };
    assert_eq!(
        program, "splitters",
        "scenario commands must invoke the splitters binary",
    );
    surface_state
        .run
        .set(splitters_cmd.run(argv.iter().skip(1)));
}

#[then("the exit status is {status:i32}")]
fn the_exit_status_is(surface_state: &SurfaceState, status: i32) {
    let run = last_run(surface_state);
    assert_eq!(run.status, Some(status), "unexpected exit status");
}

#[then("standard output is empty")]
fn standard_output_is_empty(surface_state: &SurfaceState) {
    let run = last_run(surface_state);
    assert_eq!(run.stdout, "", "standard output must be empty");
}

#[then("standard error is empty")]
fn standard_error_is_empty(surface_state: &SurfaceState) {
    let run = last_run(surface_state);
    assert_eq!(run.stderr, "", "standard error must be empty");
}

#[then("standard error says {message:string}")]
fn standard_error_says(surface_state: &SurfaceState, message: String) {
    let run = last_run(surface_state);
    assert!(
        run.stderr.contains(&message),
        "standard error must contain {message:?}; it was {:?}",
        run.stderr,
    );
}

#[then("standard error names {operand:string}")]
fn standard_error_names(surface_state: &SurfaceState, operand: String) {
    let run = last_run(surface_state);
    // Fluent isolates interpolated values with U+2068 and U+2069.
    let flattened = run.stderr.replace(['\u{2068}', '\u{2069}'], "");
    assert!(
        flattened.contains(&operand),
        "standard error must name {operand}; it was {:?}",
        run.stderr,
    );
}

#[then("the commands section lists {first:string}, {second:string}, and {third:string}")]
fn the_commands_section_lists(
    surface_state: &SurfaceState,
    first: String,
    second: String,
    third: String,
) {
    let run = last_run(surface_state);
    for name in [first, second, third] {
        assert!(
            section_contains_row(&run.stdout, "Commands:", &name),
            "the commands section must list {name}; it was {:?}",
            run.stdout,
        );
    }
}

#[then("the options section lists {option:string}")]
fn the_options_section_lists(surface_state: &SurfaceState, option: String) {
    let run = last_run(surface_state);
    // clap renders positionals under `Arguments:` and options under `Options:`.
    let heading = if option.starts_with('<') {
        "Arguments:"
    } else {
        "Options:"
    };
    assert!(
        section_contains_row(&run.stdout, heading, &option),
        "the help must list {option} in the {heading} section; it was {:?}",
        run.stdout,
    );
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The top-level help lists every command"
)]
fn top_level_help(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The map command documents --base"
)]
fn map_documents_base(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The map command documents --manifest-dir"
)]
fn map_documents_manifest_dir(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The validate command documents the proposal operand"
)]
fn validate_documents_proposal(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The validate command documents --check"
)]
fn validate_documents_check(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The extract command documents --branch"
)]
fn extract_documents_branch(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "The extract command documents --publish"
)]
fn extract_documents_publish(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "A well-formed command reports that it is not implemented yet"
)]
fn well_formed_command_reports(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "A missing proposal is a usage error"
)]
fn missing_proposal_is_a_usage_error(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

#[scenario(
    path = "tests/features/command_surface.feature",
    name = "A boolean flag rejects a detached value"
)]
fn boolean_flag_rejects_a_detached_value(surface_state: SurfaceState, splitters_cmd: Splitters) {
    let _ = (surface_state, splitters_cmd);
}

/// Returns the last captured run, failing the step when none is recorded.
fn last_run(surface_state: &SurfaceState) -> Run {
    let Some(run) = surface_state.run.get() else {
        panic!("no command has been run in this scenario");
    };
    run
}

/// Returns whether `heading`'s section has a row that begins with `item`.
///
/// Rows are matched from the start of the trimmed line, so `map` cannot match
/// an `extract` row and `--base` cannot match `--base-ref`. A bracketed value
/// suffix such as `--publish[=<BOOL>]` still matches its option name.
fn section_contains_row(output: &str, heading: &str, item: &str) -> bool {
    let rows = output
        .lines()
        .skip_while(|line| line.trim_end() != heading)
        .skip(1)
        .take_while(|line| !line.trim().is_empty());
    rows.filter_map(|line| line.trim_start().strip_prefix(item))
        .any(|tail| {
            tail.is_empty() || tail.starts_with(char::is_whitespace) || tail.starts_with('[')
        })
}
