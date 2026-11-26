//! `Fluent` localization for help text and parse diagnostics.
//!
//! The catalogue is the only owner of user-facing help text. The declarations
//! in [`crate::cli::args`] describe Rust semantics, and the prose a reader sees
//! lives in `locales/en-GB/messages.ftl`. Locale detection is deliberately
//! absent: it needs an injected environment, which roadmap item 1.1.4 owns.

use ortho_config::{FluentLocalizer, FluentLocalizerError, langid};

/// The en-GB catalogue shipped with this release.
pub(crate) const CATALOGUE: &str = include_str!("../../locales/en-GB/messages.ftl");

/// Builds the en-GB help localizer from the embedded catalogue.
///
/// # Errors
///
/// Returns a [`FluentLocalizerError`] when `OrthoConfig`'s embedded catalogue
/// or this crate's catalogue fails to parse or register. The catalogue is fixed
/// at compile time and covered by tests, so a failure here is a code defect.
///
/// # Examples
///
/// ```
/// use ortho_config::Localizer;
/// use splitters::cli::localization::help_localizer;
///
/// let localizer = help_localizer()?;
/// assert!(
///     localizer.lookup("splitters-cli-about", None).is_some(),
///     "the root about text must resolve",
/// );
/// # Ok::<(), ortho_config::FluentLocalizerError>(())
/// ```
pub fn help_localizer() -> Result<FluentLocalizer, FluentLocalizerError> {
    FluentLocalizer::builder(langid!("en-GB"))
        .with_consumer_resources([CATALOGUE])
        .try_build()
}
