//! Unit tests for the typed command-line surface.
//!
//! The tests are split by obligation: `declaration` pins the declared command
//! tree, `parsing` pins the accepted and rejected spellings, and `values`
//! pins the values that reach the argument structs.

mod declaration;
mod parsing;
mod values;

use ortho_config::NoOpLocalizer;

use crate::cli::Invocation;

/// Parses `args` (including the program name) with the no-op localizer.
fn parse(args: &[&str]) -> Result<Invocation, clap::Error> {
    crate::cli::try_parse_from(args.iter().copied(), &NoOpLocalizer::new())
}
