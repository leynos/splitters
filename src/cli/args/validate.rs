//! Arguments for the `validate` command.

use camino::Utf8PathBuf;

/// `validate` proves that one proposal can be extracted cleanly.
///
/// The proposal is a required operand because it names the object being
/// validated; an invocation without one is a usage error.
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
#[command(name = "validate")]
#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.validate")]
pub struct ValidateArgs {
    /// Identifier of the proposal to validate.
    #[arg(value_name = "PROPOSAL")]
    pub proposal: String,

    /// Manifest directory override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "DIR")]
    pub manifest_dir: Option<Utf8PathBuf>,

    /// Base ref override; `None` defers to lower configuration layers.
    #[arg(long, value_name = "REF")]
    pub base: Option<String>,

    /// Operator check command; `None` skips the check in both proof worktrees.
    #[arg(long, value_name = "COMMAND")]
    pub check: Option<String>,
}
