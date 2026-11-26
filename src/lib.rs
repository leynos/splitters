//! Library entry points for the `splitters` package.
//!
//! Splitters is library-first: the binary parses arguments, reports the
//! selected command's outcome, and maps it to an exit code, while repository
//! behaviour belongs to library modules that tests can drive without spawning
//! the binary. The command-line surface lives in [`cli`].

pub mod cli;
