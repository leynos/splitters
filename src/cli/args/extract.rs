//! Arguments for the `extract` command.

use camino::Utf8PathBuf;

/// `extract` moves one validated proposal onto a new branch.
///
/// The proposal is a required operand, and `publish` uses the boolean model
/// from the `OrthoConfig` guide so an omitted flag defers to lower
/// configuration layers while `--publish=false` can still switch a configured
/// `true` off.
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
#[command(name = "extract")]
#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.extract")]
pub struct ExtractArgs {
    /// Identifier of the proposal to extract.
    #[arg(value_name = "PROPOSAL")]
    pub proposal: String,

    /// Manifest directory override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<Utf8PathBuf>,

    /// Base ref override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "REF")]
    pub base: Option<String>,

    /// Branch name override; `None` derives the name from the proposal.
    #[arg(long, value_name = "NAME")]
    pub branch: Option<String>,

    /// Whether to publish the extracted branch; `None` defers to lower layers.
    #[arg(
        long,
        value_name = "BOOL",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        hide_possible_values = true
    )]
    pub publish: Option<bool>,
}
