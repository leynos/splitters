//! `Splitters` application entry point.

// TODO: Remove this stub and implement actual application functionality.
use splitters::greeting;

/// Application entry point.
#[expect(clippy::print_stdout, reason = "CLI output is the intended behaviour")]
fn main() {
    println!("{}", greeting());
}
