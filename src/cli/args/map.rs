//! Arguments for the `map` command.
//!
//! The module declares the `map` subcommand's flags and configuration keys.
//! [`MapArgs`] is the single declaration used by `clap`, `OrthoConfig`, and the
//! en-GB catalogue identifiers, and it is re-exported through `cli::args` for
//! use as `Commands::Map`. Every field is optional: an omitted value defers to
//! the lower configuration layers that roadmap item 1.1.4 adds.

use camino::Utf8PathBuf;

/// `map` decomposes the current branch into fragments and writes a manifest.
///
/// Every field is optional because the lower configuration layers that supply
/// omitted values arrive with roadmap item 1.1.4.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    clap::Parser,
    serde::Serialize,
    serde::Deserialize,
    ortho_config::OrthoConfig,
)]
#[command(name = "map")]
#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.map")]
pub struct MapArgs {
    /// Base ref override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "REF")]
    pub base: Option<String>,

    /// Manifest directory override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<Utf8PathBuf>,
}
