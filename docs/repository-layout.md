# Repository layout

This guide describes the current tree and the responsibilities of its
long-lived paths. Cargo writes generated build output to `target/`, which Git
ignores.

## Top-level paths

- `.cargo/` contains repository-wide Cargo defaults for development builds.
- `.github/` contains CI workflows, dependency automation, and workflow
  contract configuration.
- `docs/` contains contributor guidance, design and planning documents, and
  testing references. Store RFCs in `docs/rfcs/` and ADRs in `docs/adrs/`,
  following the naming rules in the
  [documentation style guide](documentation-style-guide.md). When a task needs
  an execution plan, store it in `docs/execplans/`.
- `src/` contains the Rust library and binary code.
- `tests/` contains integration tests, build-standard tests, and workflow
  contract checks. Build-standard test support is grouped under
  `tests/build_standard/`; the workflow contract check is under
  `tests/workflow_contracts/`.

## Root files

- `AGENTS.md` records contributor instructions for this repository.
- `.gitignore` excludes `target/`, `**/*.rs.bk`, and `*:Zone.Identifier`
  files.
- `Cargo.lock` is tracked and records resolved dependency versions.
- `Cargo.toml` defines the package, dependencies, and lint configuration.
- `Makefile` provides the development, validation, and release targets.
- `rust-toolchain.toml` selects the Rust toolchain used by the repository.
- `README.md` introduces the project and links to the documentation contents.
