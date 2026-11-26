//! Typed command-line surface for the `splitters` binary.
//!
//! The root parser is a plain `clap` type that selects a subcommand. Each
//! subcommand is declared once as a `clap` and `OrthoConfig`-derived argument
//! struct, so the command line, configuration keys, environment variables, and
//! catalogue identifiers all come from the same declaration. Configuration
//! discovery and merging are deliberately not wired here; roadmap item 1.1.4
//! owns them.

pub mod args;
pub mod dispatch;
pub mod localization;

#[cfg(test)]
mod tests;

use std::ffi::OsString;

use args::{ExtractArgs, MapArgs, ValidateArgs};
use clap::{Command, CommandFactory};
use ortho_config::{LocalizeCmd, Localizer, parse_localized_command};

/// Catalogue base shared by the root command and every subcommand.
///
/// A subcommand's generated base is this value plus its command name, so the
/// identifier the localizer looks up and the identifier `OrthoConfig`
/// generates agree by construction.
pub(crate) const ROOT_LOCALIZATION_BASE: &str = "splitters.cli";

/// The root command line: one subcommand plus clap's own display flags.
#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Cli {
    /// The subcommand selected on the command line.
    #[command(subcommand)]
    pub command: Commands,
}

/// The three commands declared by design Tables 3 to 5.
#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Carries the `map` arguments into the library.
    #[command(name = "map")]
    Map(MapArgs),
    /// Carries the `validate` arguments into the library.
    #[command(name = "validate")]
    Validate(ValidateArgs),
    /// Carries the `extract` arguments into the library.
    #[command(name = "extract")]
    Extract(ExtractArgs),
}

/// A parsed command line together with clap's record of supplied arguments.
///
/// Roadmap item 1.1.4 needs the matches to tell a parser default from an
/// explicit command-line value, so parsing preserves them.
#[derive(Debug)]
pub struct Invocation {
    /// The typed command selected on the command line.
    pub cli: Cli,
    /// Which arguments the user actually supplied.
    pub matches: clap::ArgMatches,
}

/// Builds the localized command tree at a fixed help width of 100 columns.
#[must_use]
pub fn localized_command(localizer: &dyn Localizer) -> Command {
    Cli::command()
        .term_width(100)
        .with_base(ROOT_LOCALIZATION_BASE)
        .localize(localizer)
}

/// Parses an argument vector that includes the program name.
///
/// # Errors
///
/// Returns the `clap` error when the arguments are malformed, including the
/// display requests `--help` and `--version`.
///
/// # Examples
///
/// ```
/// use ortho_config::NoOpLocalizer;
/// use splitters::cli::{Commands, try_parse_from};
///
/// let invocation = try_parse_from(
///     ["splitters", "map", "--base", "main"],
///     &NoOpLocalizer::new(),
/// )?;
/// assert!(
///     matches!(invocation.cli.command, Commands::Map(_)),
///     "the map subcommand should be selected",
/// );
/// # Ok::<(), clap::Error>(())
/// ```
pub fn try_parse_from<I, T>(args: I, localizer: &dyn Localizer) -> Result<Invocation, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let (cli, matches) =
        parse_localized_command::<Cli, _, _>(localized_command(localizer), args, localizer)?;
    Ok(Invocation { cli, matches })
}
