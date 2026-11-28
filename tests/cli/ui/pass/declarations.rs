//! Compile-pass fixture: the accepted command declarations keep deriving.
//!
//! Each struct mirrors the declaration in `src/cli/args/`: the same derives,
//! command name, localization base, and field attributes. The fixture performs
//! no runtime work; it exists so a derive or attribute change that stops
//! compiling the accepted shapes fails the `cli` target with the fixture's
//! diagnostic.


use camino::Utf8PathBuf;

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
struct MapArgs {
    /// Base ref override.
    #[arg(long, value_name = "REF")]
    base: Option<String>,
    /// Manifest directory override.
    #[arg(long, value_name = "DIR")]
    manifest_dir: Option<Utf8PathBuf>,
}

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
struct ValidateArgs {
    /// Proposal identifier.
    #[arg(value_name = "PROPOSAL")]
    proposal: String,
    /// Manifest directory override.
    #[arg(long, value_name = "DIR")]
    manifest_dir: Option<Utf8PathBuf>,
    /// Base ref override.
    #[arg(long, value_name = "REF")]
    base: Option<String>,
    /// Operator check command.
    #[arg(long, value_name = "COMMAND")]
    check: Option<String>,
}

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
struct ExtractArgs {
    /// Proposal identifier.
    #[arg(value_name = "PROPOSAL")]
    proposal: String,
    /// Manifest directory override.
    #[arg(long, value_name = "DIR")]
    manifest_dir: Option<Utf8PathBuf>,
    /// Base ref override.
    #[arg(long, value_name = "REF")]
    base: Option<String>,
    /// Branch name override.
    #[arg(long, value_name = "NAME")]
    branch: Option<String>,
    /// Publish switch.
    #[arg(
        long,
        value_name = "BOOL",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        hide_possible_values = true
    )]
    publish: Option<bool>,
}

fn main() {}