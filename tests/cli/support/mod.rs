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
    baseline: Vec<Entry>,
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

    /// Asserts the child left the sandbox exactly as the fixture created it.
    ///
    /// The comparison with the baseline happens *before* the scan probe writes
    /// anything, so a child-created entry can never be masked by the probe.
    /// The probe runs afterwards on a scratch file that is removed again,
    /// which proves the scan itself can see files.
    ///
    /// # Errors
    ///
    /// Returns the first filesystem error from scanning, probing, or removing
    /// the probe file.
    pub fn assert_untouched(&self, context: &str) -> io::Result<()> {
        let dir = open(self.root.path())?;
        let after_child = snapshot(&dir)?;
        if after_child != self.baseline {
            return Err(io::Error::other(format!(
                "the binary must not touch its sandbox ({context}); found {after_child:?}, \
                 expected {:?}",
                self.baseline,
            )));
        }
        assert_scan_sees_files(&dir)
    }
}

/// Writes and removes a probe file to prove a sandbox scan is not vacuous.
///
/// # Errors
///
/// Returns the first filesystem error from writing, scanning, or removing the
/// probe file.
fn assert_scan_sees_files(dir: &Dir) -> io::Result<()> {
    dir.write("scan-probe", b"probe")?;
    let saw_probe = snapshot(dir)?
        .iter()
        .any(|entry| entry.path == Path::new("scan-probe"));
    dir.remove_file("scan-probe")?;
    if saw_probe {
        return Ok(());
    }
    Err(io::Error::other(
        "the sandbox scan missed its probe file; the emptiness check is vacuous",
    ))
}

/// Opens the sandbox root as a capability directory.
fn open(path: &Path) -> io::Result<Dir> { Dir::open_ambient_dir(path, ambient_authority()) }

/// One sandbox entry's path and kind, so a type change is visible.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    /// The entry's path, relative to the sandbox root.
    path: PathBuf,
    /// Whether the entry is a directory, file, or something else.
    kind: EntryKind,
}

/// The filesystem kind recorded for a sandbox entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EntryKind {
    /// A directory.
    Directory,
    /// A regular file.
    File,
    /// A symbolic link.
    Symlink,
    /// Any other entry kind, such as a socket or device.
    Other,
}

impl EntryKind {
    /// Classifies one directory entry.
    fn of(entry: &cap_std::fs::DirEntry) -> io::Result<Self> {
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            return Ok(Self::Directory);
        }
        if file_type.is_file() {
            return Ok(Self::File);
        }
        if file_type.is_symlink() {
            return Ok(Self::Symlink);
        }
        Ok(Self::Other)
    }
}

/// Returns every entry under `dir`, relative to it, in sorted order.
fn snapshot(dir: &Dir) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    collect(dir, Path::new(""), &mut entries)?;
    entries.sort();
    Ok(entries)
}

/// Appends `dir`'s entries, recursing into directories, to `entries`.
fn collect(dir: &Dir, prefix: &Path, entries: &mut Vec<Entry>) -> io::Result<()> {
    for entry in dir.entries()? {
        let item = entry?;
        let name = item.file_name();
        let path = prefix.join(&name);
        let kind = EntryKind::of(&item)?;
        entries.push(Entry {
            path: path.clone(),
            kind,
        });
        if kind == EntryKind::Directory {
            collect(&dir.open_dir(&name)?, &path, entries)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Regression tests for the sandbox comparison.

    use super::Sandbox;

    #[test]
    fn a_kind_change_is_detected() {
        let sandbox = Sandbox::new().expect("the sandbox should be created");
        let dir = super::open(sandbox.root()).expect("the sandbox root should open");
        dir.remove_dir("home")
            .expect("the empty home directory should be removable");
        dir.write("home", b"now a file")
            .expect("a same-named file should be creatable");
        let result = sandbox.assert_untouched("kind change");
        assert!(
            result.is_err(),
            "replacing an empty directory with a file must fail the comparison",
        );
    }

    #[test]
    fn a_child_created_probe_is_detected_and_preserved() {
        let sandbox = Sandbox::new().expect("the sandbox should be created");
        let dir = super::open(sandbox.root()).expect("the sandbox root should open");
        dir.write("scan-probe", b"child contents")
            .expect("the collision file should be creatable");
        let result = sandbox.assert_untouched("probe collision");
        assert!(
            result.is_err(),
            "a child-created scan-probe must fail the comparison",
        );
        let contents = dir
            .read("scan-probe")
            .expect("the collision file should still exist");
        assert_eq!(
            contents, b"child contents",
            "the failed comparison must not overwrite the child's file",
        );
    }

    #[test]
    fn an_unchanged_sandbox_passes() {
        let sandbox = Sandbox::new().expect("the sandbox should be created");
        sandbox
            .assert_untouched("unchanged")
            .expect("an untouched sandbox must pass");
    }
}
