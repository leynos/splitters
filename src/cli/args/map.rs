//! Arguments for the `map` command.

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
