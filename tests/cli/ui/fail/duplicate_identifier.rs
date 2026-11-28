//! Compile-fail fixture: two fields must not share a normalized argument id.
//!
//! The OrthoConfig derive normalizes each clap argument identifier before it
//! builds the catalogue identifiers, so two fields with the same explicit
//! `#[arg(id = ...)]` would generate one Fluent identifier for two arguments.
//! The expected `.stderr` file pins the diagnostic shape for `ortho_config`
//! 0.11.0 and `trybuild` 1.0.125; refresh it when either version changes. The
//! fixture performs no runtime work and exists only to fail compilation.


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
#[command(name = "duplicate")]
#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.duplicate")]
struct DuplicateArgs {
    /// First field.
    #[arg(id = "shared", long = "first")]
    first: Option<String>,
    /// Second field.
    #[arg(id = "shared", long = "second")]
    second: Option<String>,
}

fn main() {}