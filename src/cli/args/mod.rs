//! Typed argument structs for each subcommand.
//!
//! Each struct is the single declaration of a command's command-line flags,
//! configuration keys, environment variables, and catalogue identifiers.

mod extract;
mod map;
mod validate;

pub use extract::ExtractArgs;
pub use map::MapArgs;
pub use validate::ValidateArgs;
