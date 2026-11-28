//! Catalogue tests for localized help text.
//!
//! These tests discharge `INV-catalogue-ids-agree`, `INV-catalogue-complete`,
//! and `INV-help-localized`: the declared identifiers, the catalogue, and the
//! rendered help must agree, and the catalogue may contain nothing stale.

use std::collections::BTreeSet;

use googletest::prelude::*;
use ortho_config::{ArgLocalizationIds, Localizer, OrthoConfigLocalization, message_id_for};
use pretty_assertions::assert_eq;
use rstest::rstest;

use crate::cli::{
    ROOT_LOCALIZATION_BASE,
    args::{ExtractArgs, MapArgs, ValidateArgs},
    localization::{CATALOGUE, help_localizer},
    localized_command,
};

/// One command's generated localization surface.
struct Localization {
    /// The catalogue base declared for the argument struct.
    base: &'static str,
    /// The generated `about` identifier.
    about: &'static str,
    /// The generated identifiers for the command's own arguments.
    args: &'static [ArgLocalizationIds],
}

/// Returns the generated localization surface for one command.
fn localization(name: &str) -> Localization {
    match name {
        "map" => Localization {
            base: MapArgs::LOCALIZATION_BASE,
            about: MapArgs::ABOUT_ID,
            args: MapArgs::ARG_IDS,
        },
        "validate" => Localization {
            base: ValidateArgs::LOCALIZATION_BASE,
            about: ValidateArgs::ABOUT_ID,
            args: ValidateArgs::ARG_IDS,
        },
        "extract" => Localization {
            base: ExtractArgs::LOCALIZATION_BASE,
            about: ExtractArgs::ABOUT_ID,
            args: ExtractArgs::ARG_IDS,
        },
        other => panic!("unknown command {other}"),
    }
}

#[rstest]
#[case::map("map")]
#[case::validate("validate")]
#[case::extract("extract")]
fn localization_base_agrees_with_the_command_name(#[case] name: &str) {
    let localization = localization(name);
    let expected_base = format!("{ROOT_LOCALIZATION_BASE}.{name}");
    assert_eq!(
        localization.base, expected_base,
        "a localization base must be the root base plus the command name",
    );
    assert_eq!(
        localization.about,
        message_id_for(&["splitters", "cli", name], "about"),
        "the generated about identifier must match the localized command path",
    );
}

#[test]
fn catalogue_declares_exactly_the_generated_identifiers() {
    let required = required_identifiers();
    let declared = declared_identifiers();
    assert!(
        !required.is_empty(),
        "the derive must generate identifiers to check",
    );
    assert_that!(&declared, container_eq(required));
}

#[test]
fn the_embedded_catalogue_builds_and_resolves_the_root_about() {
    let localizer = help_localizer().expect("the en-GB catalogue must build");
    let about = message_id_for(&["splitters", "cli"], "about");
    assert!(
        localizer.lookup(&about, None).is_some(),
        "the root about must resolve",
    );
}

#[rstest]
#[case::map("map")]
#[case::validate("validate")]
#[case::extract("extract")]
fn rendered_help_uses_catalogue_text_only(#[case] name: &str) {
    let localizer = help_localizer().expect("the en-GB catalogue must build");
    let localization = localization(name);
    let mut tree = localized_command(&localizer);
    tree.build();
    let Some(subcommand) = tree.find_subcommand(name) else {
        panic!("missing subcommand {name}");
    };
    let mut command = subcommand.clone();
    let help = command.render_help().to_string();
    let about = localizer
        .lookup(localization.about, None)
        .expect("the command about must resolve");
    assert!(
        help.contains(&about),
        "help for {name} must render its catalogue about",
    );
    for arg in localization.args {
        let arg_help = localizer
            .lookup(arg.help_id, None)
            .expect("every argument help must resolve");
        assert!(
            help.contains(&arg_help),
            "help for {name} must render the catalogue text for {}",
            arg.name,
        );
    }
    assert!(
        !help.contains("defers to lower configuration layers"),
        "documentation comments must not leak into rendered help",
    );
}

#[test]
fn rendered_root_help_uses_catalogue_text_only() {
    let localizer = help_localizer().expect("the en-GB catalogue must build");
    let mut tree = localized_command(&localizer);
    let help = tree.render_help().to_string();
    let about = localizer
        .lookup(&message_id_for(&["splitters", "cli"], "about"), None)
        .expect("the root about must resolve");
    let long_about = localizer
        .lookup(&message_id_for(&["splitters", "cli"], "long_about"), None)
        .expect("the root long about must resolve");
    assert_eq!(
        about, long_about,
        "the root short and long about must state the same scope",
    );
    assert!(
        help.contains(&about),
        "the root help must render the catalogue about",
    );
    assert!(
        !help.contains("The root command line"),
        "the root documentation comment must not leak into rendered help",
    );
}

/// Returns the identifiers the derive says the catalogue must declare.
fn required_identifiers() -> BTreeSet<String> {
    let mut identifiers = BTreeSet::from([
        message_id_for(&["splitters", "cli"], "about"),
        message_id_for(&["splitters", "cli"], "long_about"),
    ]);
    for name in ["map", "validate", "extract"] {
        let localization = localization(name);
        identifiers.insert(localization.about.to_owned());
        identifiers.extend(localization.args.iter().map(|arg| arg.help_id.to_owned()));
    }
    identifiers
}

/// Returns every identifier the embedded catalogue declares.
fn declared_identifiers() -> BTreeSet<String> {
    CATALOGUE.lines().filter_map(declared_identifier).collect()
}

/// Extracts one normalized identifier from a catalogue line.
fn declared_identifier(line: &str) -> Option<String> {
    let (left, _right) = line.split_once('=')?;
    let identifier = left.trim();
    let is_valid = identifier
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && identifier
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'));
    is_valid.then(|| identifier.replace('.', "-"))
}
