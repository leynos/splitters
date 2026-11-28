//! Declaration tests for the localized command surface.
//!
//! One `insta` snapshot pins the structured command tree, and `rstest` tables
//! prove that `clap`'s arguments and `OrthoConfig`'s generated identifiers
//! come from the same declarations.

use std::collections::BTreeSet;

use clap::{Command, CommandFactory};
use googletest::prelude::*;
use ortho_config::{ArgLocalizationIds, Localizer, NoOpLocalizer, OrthoConfigLocalization};
use rstest::rstest;

use crate::cli::{
    args::{ExtractArgs, MapArgs, ValidateArgs},
    localized_command,
};

/// One command of the declared surface, used to select its declarations.
#[derive(Debug, Clone, Copy)]
struct CommandUnderTest(&'static str);

impl CommandUnderTest {
    /// Returns the command's clap name.
    #[must_use]
    const fn as_str(self) -> &'static str { self.0 }
}

/// The `map` command under test.
const MAP: CommandUnderTest = CommandUnderTest("map");
/// The `validate` command under test.
const VALIDATE: CommandUnderTest = CommandUnderTest("validate");
/// The `extract` command under test.
const EXTRACT: CommandUnderTest = CommandUnderTest("extract");

/// Renders the localized command tree as deterministic, reviewed plain text.
fn describe_surface(localizer: &dyn Localizer) -> String {
    let mut command = localized_command(localizer);
    command.build();
    let mut lines = Vec::new();
    describe_command(&command, 0, &mut lines);
    lines.join("\n")
}

/// Appends one command, its arguments, and its real subcommands to `lines`.
fn describe_command(command: &Command, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    lines.push(format!("{indent}command {}", command.get_name()));
    lines.push(format!("{indent}  about: {}", text(command.get_about())));
    let mut args = command.get_arguments().collect::<Vec<_>>();
    args.sort_by_key(|arg| arg.get_id().to_string());
    for arg in args {
        describe_arg(arg, depth + 1, lines);
    }
    for child in command.get_subcommands() {
        if child.get_name() != "help" {
            describe_command(child, depth + 1, lines);
        }
    }
}

/// Appends one argument's declared properties to `lines`.
fn describe_arg(arg: &clap::Arg, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let id = arg.get_id();
    let long = arg.get_long().unwrap_or("<none>");
    let value_names = arg
        .get_value_names()
        .map_or_else(Vec::new, |names| {
            names.iter().map(ToString::to_string).collect::<Vec<_>>()
        })
        .join(",");
    let index = arg
        .get_index()
        .map_or_else(|| "<none>".to_owned(), |index| index.to_string());
    let arity = arg.get_num_args().map_or_else(
        || "<none>".to_owned(),
        |range| format!("{}..={}", range.min_values(), range.max_values()),
    );
    lines.push(format!(
        "{indent}arg {id}: long={long} value_names={value_names} index={index} required={} \
         num_args={arity} require_equals={} help={}",
        arg.is_required_set(),
        arg.is_require_equals_set(),
        text(arg.get_help()),
    ));
}

/// Renders `clap`'s styled text without emitting any style information.
fn text(value: Option<&clap::builder::StyledStr>) -> String {
    value.map_or_else(|| "<none>".to_owned(), ToString::to_string)
}

#[test]
fn command_surface_is_pinned() {
    let localizer =
        crate::cli::localization::help_localizer().expect("the en-GB catalogue must build");
    insta::assert_snapshot!("surface", describe_surface(&localizer));
}

#[test]
fn command_tree_declares_the_three_commands_and_help() {
    let mut command = localized_command(&NoOpLocalizer::new());
    command.build();
    let names = command
        .get_subcommands()
        .map(|child| child.get_name().to_owned())
        .collect::<Vec<_>>();
    assert!(!names.is_empty(), "the root must declare subcommands");
    assert_that!(
        names,
        unordered_elements_are![eq("map"), eq("validate"), eq("extract"), eq("help")]
    );
}

#[rstest]
#[case::map(MAP)]
#[case::validate(VALIDATE)]
#[case::extract(EXTRACT)]
fn argument_struct_names_its_command(#[case] command: CommandUnderTest) {
    assert_eq!(
        args_command(command).get_name(),
        command.as_str(),
        "each argument struct must name its own command for OrthoConfig namespacing",
    );
}

#[rstest]
#[case::map(MAP, &["base", "manifest_dir"])]
#[case::validate(VALIDATE, &["proposal", "manifest_dir", "base", "check"])]
#[case::extract(EXTRACT, &["proposal", "manifest_dir", "base", "branch", "publish"])]
fn clap_arguments_match_ortho_config_ids(
    #[case] command: CommandUnderTest,
    #[case] expected: &[&str],
) {
    let name = command.as_str();
    let clap_ids = clap_argument_ids(command);
    let ortho_ids = ortho_argument_ids(command);
    assert!(
        !clap_ids.is_empty(),
        "clap must declare arguments for {name}"
    );
    assert!(
        !ortho_ids.is_empty(),
        "OrthoConfig must generate identifiers for {name}"
    );
    let expected_ids: BTreeSet<String> = expected.iter().map(ToString::to_string).collect();
    assert_that!(&clap_ids, container_eq(expected_ids.clone()));
    assert_that!(&ortho_ids, container_eq(expected_ids));
}

#[test]
fn root_declares_only_help_and_version() {
    let mut command = localized_command(&NoOpLocalizer::new());
    command.build();
    let ids = command
        .get_arguments()
        .map(|arg| arg.get_id().to_string())
        .collect::<Vec<_>>();
    assert!(
        !ids.is_empty(),
        "the root must declare clap's own arguments"
    );
    assert_that!(ids, unordered_elements_are![eq("help"), eq("version")]);
}

/// Returns the argument struct's own command, without the subcommand wrapper.
fn args_command(command: CommandUnderTest) -> Command {
    match command.0 {
        "map" => MapArgs::command(),
        "validate" => ValidateArgs::command(),
        "extract" => ExtractArgs::command(),
        other => panic!("unknown command {other}"),
    }
}

/// Returns the argument identifiers `clap` sees for one subcommand.
fn clap_argument_ids(command: CommandUnderTest) -> BTreeSet<String> {
    let name = command.as_str();
    let mut tree = localized_command(&NoOpLocalizer::new());
    tree.build();
    let Some(subcommand) = tree.find_subcommand(name) else {
        panic!("missing subcommand {name}");
    };
    subcommand
        .get_arguments()
        .map(|arg| arg.get_id().to_string())
        .filter(|id| id != "help")
        .collect()
}

/// Returns the argument identifiers `OrthoConfig` generated for one subcommand.
fn ortho_argument_ids(command: CommandUnderTest) -> BTreeSet<String> {
    let ids = match command.0 {
        "map" => MapArgs::ARG_IDS,
        "validate" => ValidateArgs::ARG_IDS,
        "extract" => ExtractArgs::ARG_IDS,
        other => panic!("unknown command {other}"),
    };
    ids.iter()
        .map(|arg: &ArgLocalizationIds| arg.name.to_owned())
        .collect()
}
