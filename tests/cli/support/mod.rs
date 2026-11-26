//! Shared harness that starts the `splitters` binary in a private sandbox.
//!
//! Every child command is built here so the environment allowlist and the
//! sandbox apply to every test. The sandbox is a temporary directory outside
//! any Git repository, with its own `HOME` and `XDG_CONFIG_HOME`, and
//! `GIT_CEILING_DIRECTORIES` set to its parent so Git discovery cannot escape
//! into the checkout that owns the test.

pub mod doc_examples;

use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use cap_std::{ambient_authority, fs::Dir};
use rstest::fixture;
use tempfile::TempDir;

/// Environment variables the child inherits from the test runner.
const PASS_THROUGH_ENV: &[&str] = &["PATH"];

/// Environment prefixes preserved so coverage instrumentation records the child.
const PASS_THROUGH_PREFIXES: &[&str] = &["LLVM_PROFILE_FILE", "CARGO_LLVM_COV"];

/// The captured result of one child process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// Standard output decoded as UTF-8.
    pub stdout: String,
    /// Standard error decoded as UTF-8.
    pub stderr: String,
    /// Exit status, or `None` when a signal terminated the child.
    pub status: Option<i32>,
}

/// Copies the allowlisted environment into a child command.
fn apply_environment_allowlist(command: &mut Command) {
    for variable in PASS_THROUGH_ENV {
        copy_exact_variable(command, variable);
    }
    for (key, value) in std::env::vars_os() {
        copy_prefixed_variable(command, &key, &value);
    }
}

/// Copies one exact allowlisted variable when the runner defines it.
fn copy_exact_variable(command: &mut Command, variable: &str) {
    if let Some(value) = std::env::var_os(variable) {
        command.env(variable, value);
    }
}

/// Copies one variable whose name matches an allowlisted prefix.
fn copy_prefixed_variable(command: &mut Command, key: &OsStr, value: &OsStr) {
    let name = key.to_string_lossy();
    if name_matches_allowlist(&name) {
        command.env(name.as_ref(), value);
    }
}

/// Returns whether a variable name is preserved for coverage tooling.
fn name_matches_allowlist(name: &str) -> bool {
    PASS_THROUGH_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// The `splitters` binary invoked inside a sandbox.
pub struct Splitters {
    sandbox: Sandbox,
}

impl Splitters {
    /// Builds a child command with a cleared environment and a private home.
    pub fn command(&self) -> Command {
        let Ok(mut command) = Command::cargo_bin("splitters") else {
            panic!("the splitters binary should build");
        };
        command
            .env_clear()
            .current_dir(self.sandbox.root())
            .env("HOME", self.sandbox.home())
            .env("XDG_CONFIG_HOME", self.sandbox.xdg_config_home())
            .env("GIT_CEILING_DIRECTORIES", self.sandbox.ceiling());
        apply_environment_allowlist(&mut command);
        command
    }

    /// Runs the binary with `args` (excluding the program name) and captures it.
    pub fn run<I, S>(&self, args: I) -> Run
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let Ok(output) = self.command().args(args).output() else {
            panic!("the splitters binary should start");
        };
        Run {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            status: output.status.code(),
        }
    }

    /// Returns the sandbox the child may write into.
    pub const fn sandbox(&self) -> &Sandbox { &self.sandbox }
}

/// The fixture that owns one sandboxed `splitters` invocation per test.
#[fixture]
pub fn splitters_cmd() -> Splitters {
    let sandbox = match Sandbox::new() {
        Ok(sandbox) => sandbox,
        Err(error) => panic!("the test sandbox should be creatable: {error}"),
    };
    Splitters { sandbox }
}

/// An empty directory tree that the binary must leave untouched.
pub struct Sandbox {
    root: TempDir,
    baseline: Vec<PathBuf>,
}

impl Sandbox {
    /// Creates the sandbox with its two configuration directories.
    fn new() -> io::Result<Self> {
        let root = tempfile::Builder::new()
            .prefix("splitters-cli-")
            .tempdir()?;
        let dir = open(root.path())?;
        dir.create_dir("home")?;
        dir.create_dir("xdg-config")?;
        let baseline = snapshot(&dir)?;
        Ok(Self { root, baseline })
    }

    /// Returns the sandbox root, which is also the child's working directory.
    pub fn root(&self) -> &Path { self.root.path() }

    /// Returns the child's private `HOME`.
    pub fn home(&self) -> PathBuf { self.root.path().join("home") }

    /// Returns the child's private `XDG_CONFIG_HOME`.
    pub fn xdg_config_home(&self) -> PathBuf { self.root.path().join("xdg-config") }

    /// Returns the directory Git discovery must not walk above.
    pub fn ceiling(&self) -> &Path {
        self.root
            .path()
            .parent()
            .unwrap_or_else(|| self.root.path())
    }

    /// Proves the emptiness check can see files, then asserts no effects.
    ///
    /// # Errors
    ///
    /// Returns the first filesystem error from creating, scanning, or removing
    /// the witness file.
    pub fn assert_untouched(&self, context: &str) -> io::Result<()> {
        let dir = open(self.root.path())?;
        dir.write("witness", b"witness")?;
        let saw_witness = snapshot(&dir)?
            .iter()
            .any(|path| path == Path::new("witness"));
        dir.remove_file("witness")?;
        if !saw_witness {
            return Err(io::Error::other(
                "the sandbox scan missed its witness file; the emptiness check is vacuous",
            ));
        }
        let remaining = snapshot(&dir)?;
        if remaining == self.baseline {
            return Ok(());
        }
        Err(io::Error::other(format!(
            "the binary must not touch its sandbox ({context}); found {remaining:?}, expected {:?}",
            self.baseline,
        )))
    }
}

/// Opens the sandbox root as a capability directory.
fn open(path: &Path) -> io::Result<Dir> { Dir::open_ambient_dir(path, ambient_authority()) }

/// Returns every path under `dir`, relative to it, in sorted order.
fn snapshot(dir: &Dir) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    collect(dir, Path::new(""), &mut entries)?;
    entries.sort();
    Ok(entries)
}

/// Appends `dir`'s entries, recursing into directories, to `entries`.
fn collect(dir: &Dir, prefix: &Path, entries: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in dir.entries()? {
        let item = entry?;
        let name = item.file_name();
        let path = prefix.join(&name);
        entries.push(path.clone());
        if item.file_type()?.is_dir() {
            collect(&dir.open_dir(&name)?, &path, entries)?;
        }
    }
    Ok(())
}
