//! Command dispatch and interim outcome reporting.
//!
//! Roadmap item 1.1.1 declares the command surface but implements no command,
//! so every parsed invocation resolves to
//! [`CommandOutcome::NotYetImplemented`]. The error hierarchy that replaces
//! this module arrives with roadmap item 1.1.2.

use std::{
    fmt,
    io::{self, Write},
};

use crate::cli::{Cli, Commands};

/// One command of the declared surface, named exactly as `clap` names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    /// The `map` command.
    Map,
    /// The `validate` command.
    Validate,
    /// The `extract` command.
    Extract,
}

impl CommandName {
    /// Returns the command-line spelling of this command.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Validate => "validate",
            Self::Extract => "extract",
        }
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What the binary should do after a successful parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOutcome {
    /// The selected command is declared but not implemented yet.
    NotYetImplemented(CommandName),
}

/// Resolves a parsed invocation to the outcome the binary reports.
#[must_use]
pub const fn dispatch(cli: &Cli) -> CommandOutcome {
    let name = match cli.command {
        Commands::Map(_) => CommandName::Map,
        Commands::Validate(_) => CommandName::Validate,
        Commands::Extract(_) => CommandName::Extract,
    };
    CommandOutcome::NotYetImplemented(name)
}

/// Writes the interim diagnostic for `outcome` to `err`.
///
/// # Errors
///
/// Returns the write error when `err` rejects the diagnostic, such as a closed
/// standard error.
pub fn report(outcome: CommandOutcome, err: &mut dyn Write) -> io::Result<()> {
    match outcome {
        CommandOutcome::NotYetImplemented(name) => {
            writeln!(err, "splitters {name}: not implemented yet")
        }
    }
}

#[cfg(test)]
mod tests {
    //! Dispatch, naming, and reporting tests.

    use ortho_config::NoOpLocalizer;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{CommandName, CommandOutcome, dispatch, report};
    use crate::cli::{
        Cli,
        Commands,
        args::{ExtractArgs, MapArgs, ValidateArgs},
        localized_command,
    };

    #[rstest]
    #[case::map(Commands::Map(MapArgs::default()), CommandName::Map)]
    #[case::validate(Commands::Validate(ValidateArgs::default()), CommandName::Validate)]
    #[case::extract(Commands::Extract(ExtractArgs::default()), CommandName::Extract)]
    fn dispatch_names_the_selected_command(
        #[case] command: Commands,
        #[case] expected: CommandName,
    ) {
        let cli = Cli { command };
        assert_eq!(
            dispatch(&cli),
            CommandOutcome::NotYetImplemented(expected),
            "dispatch must name the selected command",
        );
    }

    #[test]
    fn display_agrees_with_the_clap_subcommand_name() {
        let mut tree = localized_command(&NoOpLocalizer::new());
        tree.build();
        for name in [
            CommandName::Map,
            CommandName::Validate,
            CommandName::Extract,
        ] {
            assert!(
                tree.find_subcommand(name.as_str()).is_some(),
                "CommandName::{name:?} must name a declared subcommand",
            );
            assert_eq!(
                name.to_string(),
                name.as_str(),
                "Display and the command-line spelling must agree",
            );
        }
    }

    #[test]
    fn report_writes_the_interim_diagnostic() {
        let mut output = Vec::new();
        report(
            CommandOutcome::NotYetImplemented(CommandName::Validate),
            &mut output,
        )
        .expect("writing to a vector cannot fail");
        assert_eq!(
            String::from_utf8(output).expect("the diagnostic is UTF-8"),
            "splitters validate: not implemented yet\n",
        );
    }
}
