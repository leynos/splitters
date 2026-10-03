//! Contract tests for the build standard's code-generation backend.
//!
//! The standard makes Cranelift the development profile's backend, which needs
//! the unstable Cargo key and the toolchain component. Two builds must not use
//! it: coverage, because Cranelift cannot build `-Cinstrument-coverage`, and
//! Whitaker's Dylint driver, which builds outside the workspace configuration.
//! These tests hold the configuration, the coverage step's overrides and the
//! `make lint` Whitaker boundary, which the flag contract does not reach.
//!
//! File access goes through `cap_std` directory handles: one rooted at the
//! crate manifest directory, and one at a scratch directory under
//! `CARGO_TARGET_TMPDIR` for the fake tools.

use std::{
    error::Error,
    process::{Command, Output},
};

use cap_std::{ambient_authority, fs::Dir};
use rstest::rstest;

/// The result of a reader, which the tests unwrap.
type Read<T> = Result<T, Box<dyn Error>>;

/// The toolchain component Cranelift needs.
const CRANELIFT_COMPONENT: &str = "rustc-codegen-cranelift-preview";

/// The overrides a build outside the workspace configuration needs to take
/// LLVM: the unstable feature, then the profile's backend.
const LLVM_OVERRIDES: [(&str, &str); 2] = [
    ("CARGO_UNSTABLE_CODEGEN_BACKEND", "true"),
    ("CARGO_PROFILE_DEV_CODEGEN_BACKEND", "llvm"),
];

/// Reads a file relative to the crate manifest directory.
fn read(path: &str) -> Read<String> {
    let root = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())?;
    Ok(root.read_to_string(path)?)
}

/// Reads a TOML file relative to the crate manifest directory.
fn read_toml(path: &str) -> Read<toml::Value> { Ok(toml::from_str(&read(path)?)?) }

/// Follows a path of table keys through a TOML value.
fn value_at<'a>(root: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
    path.iter().try_fold(root, |value, key| value.get(key))
}

#[test]
fn cranelift_is_the_development_backend() {
    let config = read_toml(".cargo/config.toml").expect("read the Cargo configuration");
    assert_eq!(
        value_at(&config, &["unstable", "codegen-backend"]).and_then(toml::Value::as_bool),
        Some(true),
        "`[unstable] codegen-backend = true` is what lets Cargo accept the profile key"
    );
    assert_eq!(
        value_at(&config, &["profile", "dev", "codegen-backend"]).and_then(toml::Value::as_str),
        Some("cranelift"),
        "the development profile no longer selects Cranelift"
    );
    let toolchain = read_toml("rust-toolchain.toml").expect("read the toolchain file");
    let has_component = value_at(&toolchain, &["toolchain", "components"])
        .and_then(toml::Value::as_array)
        .is_some_and(|components| {
            components
                .iter()
                .any(|c| c.as_str() == Some(CRANELIFT_COMPONENT))
        });
    assert!(
        has_component,
        "the pinned toolchain lacks `{CRANELIFT_COMPONENT}`"
    );
}

/// Returns the number of leading spaces on a line.
fn indent(line: &str) -> usize { line.len() - line.trim_start().len() }

/// Returns whether a line is blank or a comment, and so evidence of nothing.
fn is_inert(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Returns whether a line begins a YAML sequence item, which is a workflow step.
fn starts_step(line: &str) -> bool { line.trim_start().starts_with("- ") }

/// Returns the lines of the workflow step whose text holds `needle`.
///
/// A step starts at a `- ` line and runs to the next `- ` line at the same or
/// a shallower indent, so nested lists inside the step stay with it.
fn step_containing<'a>(workflow: &'a str, needle: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = workflow.lines().collect();
    let step_starts: Vec<usize> = (0..lines.len())
        .filter(|&i| lines.get(i).is_some_and(|line| starts_step(line)))
        .collect();
    let bounds = lines
        .iter()
        .position(|line| !is_inert(line) && line.contains(needle))
        .and_then(|found| {
            let start = step_starts.iter().copied().rfind(|&i| i <= found)?;
            let start_indent = indent(lines.get(start)?);
            let end = step_starts
                .iter()
                .copied()
                .find(|&i| {
                    i > found
                        && lines
                            .get(i)
                            .is_some_and(|line| indent(line) <= start_indent)
                })
                .unwrap_or(lines.len());
            Some(start..end)
        });
    bounds
        .and_then(|range| lines.get(range))
        .map(<[&str]>::to_vec)
        .unwrap_or_default()
}

/// Returns the `key: value` entries of a step's own `env` mapping.
///
/// Only lines nested under the step's `env:` key count, so the same names under
/// `with:`, in a comment, or in another step are not evidence. Values lose
/// their quotes.
fn env_entries(step: &[&str]) -> Vec<(String, String)> {
    // The step's keys sit two columns in from its `- ` marker.
    let key_indent = step.first().map_or(0, |first| indent(first) + 2);
    let Some(env) = step
        .iter()
        .position(|line| indent(line) == key_indent && line.trim() == "env:")
    else {
        return Vec::new();
    };
    step.iter()
        .skip(env + 1)
        .take_while(|line| is_inert(line) || indent(line) > key_indent)
        .filter(|line| !is_inert(line))
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(key, value)| (key.to_owned(), value.trim().trim_matches('"').to_owned()))
        .collect()
}

#[test]
fn coverage_builds_on_llvm() {
    let workflow = read(".github/workflows/ci.yml").expect("read ci.yml");
    let step = step_containing(&workflow, "generate-coverage@");
    assert!(!step.is_empty(), "ci.yml has no `generate-coverage` step");
    let entries = env_entries(&step);
    for (name, wanted) in LLVM_OVERRIDES {
        assert!(
            entries
                .iter()
                .any(|(key, value)| key == name && value == wanted),
            "the coverage step's env does not set {name}={wanted}"
        );
    }
}

/// Returns the lines of the job that contains the first line holding `needle`.
///
/// A job is a two-space-indented key under `jobs:`, running to the next one.
fn job_containing<'a>(workflow: &'a str, needle: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = workflow.lines().collect();
    let starts_job = |line: &str| indent(line) == 2 && line.trim_end().ends_with(':');
    let bounds = lines
        .iter()
        .position(|line| !is_inert(line) && line.contains(needle))
        .and_then(|found| {
            let start = (0..=found).rfind(|&i| lines.get(i).is_some_and(|l| starts_job(l)))?;
            let end = (found + 1..lines.len())
                .find(|&i| lines.get(i).is_some_and(|l| starts_job(l)))
                .unwrap_or(lines.len());
            Some(start..end)
        });
    bounds
        .and_then(|range| lines.get(range))
        .map(<[&str]>::to_vec)
        .unwrap_or_default()
}

/// Returns the offset of the first line that installs mold: an `apt` install
/// naming it, or setup-rust's `install-mold` input set to true.
fn mold_install_offset(job: &[&str]) -> Option<usize> {
    job.iter().position(|line| {
        let words = line.trim();
        !is_inert(line)
            && ((words.contains("apt") && words.contains("install") && words.contains("mold"))
                || (words.starts_with("install-mold:") && words.contains("true")))
    })
}

/// Returns the offset of the first non-comment line holding `needle`.
fn offset_of(job: &[&str], needle: &str) -> Option<usize> {
    job.iter()
        .position(|line| !is_inert(line) && line.contains(needle))
}

#[test]
fn ci_installs_mold_before_lint_and_coverage() {
    let workflow = read(".github/workflows/ci.yml").expect("read ci.yml");
    let job = job_containing(&workflow, "generate-coverage@");
    let installed = mold_install_offset(&job).expect("the coverage job never installs mold");
    for needle in ["make lint", "generate-coverage@"] {
        let at = offset_of(&job, needle).expect("the job lacks a gate step");
        assert!(installed < at, "mold is installed after `{needle}`");
    }
}

/// A workflow whose coverage step carries its overrides in `env`, as required.
#[cfg(test)]
const GOOD_WORKFLOW: &str = concat!(
    "jobs:\n",
    "  build-test:\n",
    "    steps:\n",
    "      - name: Install mold linker\n",
    "        run: sudo apt-get install --yes mold\n",
    "      - run: make lint\n",
    "      - name: Coverage\n",
    "        uses: org/actions/generate-coverage@abc\n",
    "        env:\n",
    "          CARGO_UNSTABLE_CODEGEN_BACKEND: \"true\"\n",
    "          CARGO_PROFILE_DEV_CODEGEN_BACKEND: llvm\n",
    "        with:\n",
    "          format: lcov\n",
);

#[test]
fn the_env_reader_accepts_overrides_under_env() {
    let step = step_containing(GOOD_WORKFLOW, "generate-coverage@");
    let entries = env_entries(&step);
    assert!(entries.contains(&("CARGO_PROFILE_DEV_CODEGEN_BACKEND".into(), "llvm".into())));
}

#[test]
fn the_env_reader_rejects_overrides_moved_under_with() {
    let moved = GOOD_WORKFLOW.replace("        env:\n", "        with:\n");
    let step = step_containing(&moved, "generate-coverage@");
    assert!(env_entries(&step).is_empty(), "with: entries read as env");
}

#[test]
fn the_env_reader_rejects_commented_and_foreign_overrides() {
    let commented = GOOD_WORKFLOW.replace(
        "          CARGO_PROFILE_DEV_CODEGEN_BACKEND: llvm\n",
        "          # CARGO_PROFILE_DEV_CODEGEN_BACKEND: llvm\n",
    );
    let step = step_containing(&commented, "generate-coverage@");
    assert!(
        !env_entries(&step)
            .iter()
            .any(|(key, _)| key.starts_with("CARGO_PROFILE"))
    );
}

#[test]
fn the_mold_reader_rejects_a_late_or_missing_install() {
    let good = job_containing(GOOD_WORKFLOW, "generate-coverage@");
    let good_install = mold_install_offset(&good).expect("the good workflow installs mold");
    assert!(good_install < offset_of(&good, "make lint").expect("lint"));
    let late_text = GOOD_WORKFLOW
        .replace("run: sudo apt-get install --yes mold", "run: echo skipped")
        .replace(
            "      - run: make lint\n",
            "      - run: make lint\n      - run: sudo apt-get install mold\n",
        );
    let late = job_containing(&late_text, "generate-coverage@");
    let late_install = mold_install_offset(&late).expect("the late workflow installs mold");
    assert!(late_install > offset_of(&late, "make lint").expect("lint"));
    let none_text = GOOD_WORKFLOW.replace("sudo apt-get install --yes mold", "true");
    assert!(mold_install_offset(&job_containing(&none_text, "generate-coverage@")).is_none());
}

/// Runs `make lint` with fake tools first on `PATH`, returning whether it
/// succeeded and the environment the fake Whitaker recorded.
///
/// Cargo is replaced by `true`, so `doc` and `clippy` succeed without
/// building. The fake Whitaker writes its environment to a file and exits with
/// `whitaker_status`. Each test passes its own `scratch` directory name, since
/// the tests run concurrently and a shared directory would be cleared under one
/// of them.
#[cfg(unix)]
fn lint_with_fake_whitaker(scratch: &str, whitaker_status: i32) -> Read<(Output, String)> {
    use cap_std::fs::{OpenOptions, OpenOptionsExt};

    let target_tmp = Dir::open_ambient_dir(env!("CARGO_TARGET_TMPDIR"), ambient_authority())?;
    // A clean directory keeps a record from an earlier run out.
    target_tmp
        .remove_dir_all(scratch)
        .or_else(|error| match error.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(error),
        })?;
    target_tmp.create_dir(scratch)?;
    let dir = target_tmp.open_dir(scratch)?;
    let script = format!("#!/bin/sh\nenv > \"$WHITAKER_RECORD\"\nexit {whitaker_status}\n");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o755);
    std::io::Write::write_all(&mut dir.open_with("whitaker", &options)?, script.as_bytes())?;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(scratch);
    // A fixed PATH keeps the run hermetic: the fake first, then the system
    // directories that hold `sh`, `env` and the `true` standing in for cargo.
    let path = format!("{}:/usr/bin:/bin", root.display());
    let output = Command::new("make")
        .args(["lint", "CARGO=true"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", path)
        .env("WHITAKER_RECORD", root.join("record"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .output()?;
    // A missing record means the fake never ran; surface that rather than
    // reading it as an empty successful one.
    let record = dir.read_to_string("record")?;
    Ok((output, record))
}

/// A Whitaker exit status decides `make lint`, and the fake is always run.
#[cfg(unix)]
#[rstest]
#[case::failing("whitaker-fails", 1)]
#[case::passing("whitaker-passes", 0)]
fn lint_succeeds_exactly_when_whitaker_does(#[case] scratch: &str, #[case] status: i32) {
    let (output, record) = lint_with_fake_whitaker(scratch, status).expect("run `make lint`");
    assert_eq!(
        output.status.success(),
        status == 0,
        "`make lint` ignored Whitaker's exit status {status}: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!record.is_empty(), "the fake Whitaker recorded nothing");
}

#[cfg(unix)]
#[test]
fn whitaker_builds_on_llvm_with_the_composed_flags() {
    let (_, record) = lint_with_fake_whitaker("whitaker-env", 0).expect("run `make lint`");
    let value = |name: &str| {
        record
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{name}=")))
            .map(str::to_owned)
    };
    assert_eq!(
        value("CARGO_UNSTABLE_CODEGEN_BACKEND").as_deref(),
        Some("true")
    );
    assert_eq!(
        value("CARGO_PROFILE_DEV_CODEGEN_BACKEND").as_deref(),
        Some("llvm")
    );
    let flags = value("RUSTFLAGS").expect("Whitaker received no RUSTFLAGS");
    assert!(
        flags.contains("-Zthreads=8"),
        "Whitaker lost the frontend flag: {flags}"
    );
}

/// A missing Whitaker binary skips the check with a message and `make lint`
/// still succeeds, since it is an optional tool; a present one that fails is
/// covered above. `PATH` is narrowed to the system directories, where Whitaker
/// is not installed, and the test refuses to run if it is found there anyway.
#[cfg(unix)]
#[test]
fn a_missing_whitaker_skips_the_check_and_lint_succeeds() {
    let system_path = "/usr/bin:/bin";
    let found = Command::new("sh")
        .args(["-c", "command -v whitaker"])
        .env("PATH", system_path)
        .output()
        .expect("probe for Whitaker");
    assert!(
        !found.status.success(),
        "Whitaker is installed under {system_path}"
    );
    let output = Command::new("make")
        .args(["lint", "CARGO=true"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("PATH", system_path)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .output()
        .expect("run `make lint`");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "`make lint` failed without Whitaker ({}): {stdout}{stderr}",
        output.status
    );
    assert!(
        stdout.contains("skipping whitaker lint"),
        "no skip message in: {stdout}"
    );
}
