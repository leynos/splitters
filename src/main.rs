//! `splitters` application entry point.
//!
//! The binary reads `std::env::args_os()` once and parses it through
//! [`splitters::cli::try_parse_from`]. Display requests and usage errors keep
//! clap's own behaviour, because they are rendered and exited by
//! `clap::Error::exit`. A successful parse is passed to
//! [`splitters::cli::dispatch::dispatch`], and the resulting outcome is written
//! by [`splitters::cli::dispatch::report`]. Interim outcomes and a failure to
//! load the en-GB catalogue both map to exit status `2`, the design's
//! operational-failure class; roadmap item 1.1.2 owns the final mapping.

use std::{
    io::{self, Write},
    process::ExitCode,
};

use splitters::cli::{
    self,
    dispatch::{self, CommandOutcome},
    localization,
};

/// Exit status for the design's operational-failure class.
const OPERATIONAL_FAILURE: u8 = 2;

/// Application entry point.
fn main() -> ExitCode {
    let localizer = match localization::help_localizer() {
        Ok(localizer) => localizer,
        Err(error) => {
            return fail(&format!(
                "splitters: cannot load the en-GB help catalogue: {error}"
            ));
        }
    };
    let invocation = match cli::try_parse_from(std::env::args_os(), &localizer) {
        Ok(invocation) => invocation,
        Err(error) => error.exit(),
    };
    report_and_fail(dispatch::dispatch(&invocation.cli))
}

/// Writes `outcome` to standard error and returns the operational-failure code.
fn report_and_fail(outcome: CommandOutcome) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if dispatch::report(outcome, &mut stderr).is_err() {
        // Standard error is unwritable, so the status is the only signal left.
        return ExitCode::from(OPERATIONAL_FAILURE);
    }
    ExitCode::from(OPERATIONAL_FAILURE)
}

/// Writes `message` to standard error and returns the operational-failure code.
fn fail(message: &str) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if writeln!(stderr, "{message}").is_err() {
        // Standard error is unwritable, so the status is the only signal left.
        return ExitCode::from(OPERATIONAL_FAILURE);
    }
    ExitCode::from(OPERATIONAL_FAILURE)
}
