# 1.1.1 Replace the stub binary with the `map`, `validate`, and `extract` command skeletons

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (awaiting expert review and user approval)

Branch: `1-1-1-replace-the-stub-binary-with-command-skeletons`

Roadmap item: `1.1.1` in `docs/roadmap.md`.

## Purpose / big picture

Splitters is a planned Rust command-line tool that splits one Git branch into
smaller, independently reviewable branches. Its design
(`docs/splitters-design.md`) defines three commands: `map` decomposes a branch
into fragments and writes a manifest directory, `validate` proves that one
proposal (a named set of fragments) can be replayed onto the base and removed
from the current branch, and `extract` performs that split for real.

Today the binary is a stub. Running it prints a greeting and ignores every
argument:

```plaintext
$ splitters --help
Hello from Splitters!
```

After this work, `splitters` has a real, typed command surface. The command
line is declared with `clap` (the de facto Rust argument parser) and
`ortho_config` v0.11.0 (OrthoConfig, a layered-configuration library from
`leynos/ortho-config` that derives configuration-file keys, environment
variables, and defaults from the same Rust struct that declares the
command-line flags). Help text is routed through a Fluent message catalogue
(Fluent is Mozilla's localization format, which OrthoConfig uses), so it can be
translated without touching the declarations. A user can now observe:

```plaintext
$ splitters --help
Split one Git branch into smaller, independently reviewable branches.

Usage: splitters <COMMAND>

Commands:
  map       Decompose the current branch into fragments and write a manifest.
  validate  Prove that one proposal can be extracted cleanly.
  extract   Move one validated proposal onto a new branch.
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

(The exact wording is fixed by the catalogue written in `EP-M2`; the transcript
above is illustrative until then.) `splitters validate --help` lists the
proposal operand, `--manifest-dir`, `--base`, and `--check`;
`splitters extract --help` lists the proposal operand, `--manifest-dir`,
`--base`, `--branch`, and `--publish`; `splitters map --help` lists `--base` and
`--manifest-dir`. A malformed invocation such as `splitters validate` (missing
proposal) prints a usage error on standard error. A well-formed invocation such
as `splitters validate add-parser` parses successfully and then reports, on
standard error, that the command is not implemented yet, exiting with status
`2` (the design's "operational failure" class). No repository is read or
mutated by any command in this milestone.

This work closes the roadmap gap "Replace the stub binary with the `map`,
`validate`, and `extract` command skeletons using `ortho-config`" and its
success criteria: `splitters --help` documents the full command surface,
parsing has snapshot and behaviour coverage, and each option has a single
declared source. It deliberately does not wire configuration discovery or
precedence (roadmap 1.1.4), the domain error hierarchy and exit-code mapping
(1.1.2), or repository preflight (1.1.3).

## Constraints

- Implementation must not begin until the user explicitly approves this plan.
- The command surface must be declared with `clap` derive and
  `ortho_config::OrthoConfig` derive only: the root parser is a
  `clap::Parser`-derived type with a `clap::Subcommand` enum, and each of the
  three subcommand argument structs derives `clap::Parser` and
  `ortho_config::OrthoConfig` (plus `serde::Serialize` and
  `serde::Deserialize`, which OrthoConfig requires). No hand-rolled argument
  parsing, no `std::env::args` inspection beyond passing the iterator to the
  parser, and no configuration struct separate from the command declarations
  (`docs/splitters-design.md` §5, §12; `docs/developers-guide.md` §3.1).
- The root parser stays a plain `clap` type without `OrthoConfig`
  (`docs/developers-guide.md` §3.1: "a plain `clap` type that selects a
  subcommand"). It declares no options of its own beyond clap's generated
  `--help` and `--version`.
- No command performs repository discovery, cleanliness checks, manifest I/O,
  Git operations, or subprocess spawning. Those belong to roadmap items 1.1.3,
  1.2.x, 2.x, 3.x, and 4.x.
- No configuration-file discovery, environment-variable reading, or layer
  merging is wired in this item. `load_and_merge*`, `SelectedSubcommandMerge`,
  `SubcmdConfigMerge`, and `ConfigDiscovery` are not called; roadmap 1.1.4 owns
  them. Fields are nevertheless declared so that 1.1.4 needs no change to the
  declarations other than attributes it discovers it needs.
- Production code must not call `std::env::var`, `std::env::var_os`,
  `std::env::set_var`, `std::env::remove_var`, or `std::env::set_current_dir`
  (`AGENTS.md` "Testing"; `clippy.toml` `disallowed-methods`). In particular
  the locale is not detected from `LANG`/`LC_ALL` in this item.
- Standard output carries no prose. Help and version output go to standard
  output because that is clap's documented contract for display requests; every
  other diagnostic goes to standard error (`docs/splitters-design.md` §7;
  `docs/developers-guide.md` §3.1). `print_stdout` and `print_stderr` remain
  denied (`Cargo.toml` `[lints.clippy]`), so output uses explicit
  `std::io::Write` handles or clap's own `Error::exit`.
- Code files stay under 400 lines; every module starts with a `//!` comment;
  public items carry `///` documentation with examples where meaningful; every
  `assert!`-family call carries a diagnostic message; `expect` is confined to
  tests; lints are not silenced except by item-scoped
  `#[expect(..., reason = "...")]` as a last resort (`AGENTS.md`).
- Clippy thresholds in `clippy.toml` apply: cognitive complexity 9,
  4 arguments, 70 lines per function, nesting depth 4.
- The binary must remain named `splitters` (`Makefile` `TARGET ?= splitters`).
- `[profile.dev]` must keep `incremental = false` and `debug = 0`
  (`tests/dev_build_footprint.rs`), and no `codegen-backend` may be set in
  `[profile.dev]` (`tests/build_standard_backend.rs`).
- Dependencies use caret requirements only (`AGENTS.md` "Dependency
  management"). The authorized additions are listed in
  `Interfaces and dependencies`; nothing else may be added without escalation.
- Comments, documentation, and user-facing catalogue text use
  en-GB-oxendict spelling (`AGENTS.md`).
- Completed execplans are historical and must not be edited (none exist yet).
- No source-API compatibility machinery: `greeting()` is deleted, not
  deprecated, because the crate is pre-1.0 and has no external consumers.

## Tolerances (exception triggers)

Comparison base: `dc5c0b7` (`Point OrthoConfig references at v0.11.0`), the
inspected revision. Counting command, run from the repository root:

```bash
git diff --numstat dc5c0b7 -- . \
  ':(exclude)docs/execplans/' ':(exclude)Cargo.lock' \
  ':(exclude)tests/snapshots/'
```

Classification by whole-file rule:

- Code: `src/**`, `tests/**/*.rs`, `tests/features/**`, and
  `locales/**/*.ftl` (the catalogue is shipped, user-facing program text and is
  reviewed like code).
- Configuration: `Cargo.toml`, `Makefile`, `.github/**`, `clippy.toml`.
- Documentation: `docs/**/*.md` except `docs/execplans/`, `README.md`.
- Excluded generated artefacts: `Cargo.lock`, and `tests/snapshots/*.snap`
  (insta snapshot files, reviewed but generated). This ExecPlan file is
  excluded from every count.

Budgets (net lines, additions minus deletions, each enforced independently):

- Files: if more than 30 non-ExecPlan, non-generated files change, stop and
  escalate.
- Code: if code exceeds 1,500 net lines, stop and escalate. The estimate is
  about 1,150: roughly 450 production lines (`src/lib.rs`, `src/main.rs`,
  `src/cli/` with four declaration modules, localization, and dispatch), 40
  catalogue lines, and 650 test lines (unit tables, the property test, the
  end-to-end suite, the BDD (behaviour-driven development) harness, and the
  shared tested-example loader with its own tests). The contingency of about
  30% covers derive-driven boilerplate (for example `Default` impls) that the
  probe could not size.
- Configuration: if configuration exceeds 60 net lines, stop and escalate.
  The estimate is about 30 lines in `Cargo.toml` (four runtime and seven
  development dependencies plus feature lists).
- Documentation: if documentation exceeds 450 net lines, stop and escalate.
  The estimate is about 300: a new `docs/users-guide.md` (about 140), a
  `docs/splitters-design.md` §7.4 addition (about 50),
  `docs/developers-guide.md` updates (about 70), `docs/repository-layout.md` and
  `docs/contents.md` (about 20), and the roadmap tick.
- Interface: the public library API at the end of this item is exactly the
  surface listed in `Interfaces and dependencies`. Any further public item, or
  a change to a listed signature, requires escalation.
- Dependencies: any crate beyond the authorized list, or a different major
  version of a listed crate, requires escalation.
- Architecture: introducing an OrthoConfig derive on the root parser, calling
  any OrthoConfig loader, reading the process environment, or touching Git
  requires escalation.
- Iterations: if a focused test still fails after 4 fix attempts, stop and
  escalate.
- Time: if a milestone takes more than 4 hours of active work, stop and
  escalate.
- Ambiguity: the open questions listed in `Decision log` (D3, D4, D5, D6, D7)
  are proposed answers; if the user rejects any of them, update this plan
  before implementing.

Historical evidence: `docs/execplans/` does not exist at the base revision, so
no comparable plan history or prior budget revisions are available. Budgets are
therefore set from the planning-time probe (see `Artefacts and notes`) and the
file survey in `Context and orientation`.

## Risks

- Risk: OrthoConfig's derive on a struct that also derives `clap::Parser` might
  generate items that violate the repository's strict lint set (`missing_docs`,
  pedantic Clippy, Whitaker). Severity: medium. Likelihood: low. Mitigation:
  the planning probe compiled the exact pattern with the repository's `[lints]`
  table and `clippy.toml` and Clippy passed with `-D warnings` after one
  ordinary `doc_markdown` fix in hand-written text. Whitaker was not run on the
  probe; `EP-M1` runs `make lint` before anything else is built on top.
- Risk: Fluent wraps interpolated arguments in Unicode bidirectional isolation
  marks (U+2068 and U+2069), observed in localized clap errors such as
  `Missing required argument: ⁨<PROPOSAL>⁩`. Exact-text snapshots would embed
  invisible characters, and some terminals render them as boxes. Severity: low.
  Likelihood: high (observed). Mitigation: snapshot tests apply an insta filter
  that removes U+2068 and U+2069 and record that they do; help output (which
  has no interpolation) is unaffected. OrthoConfig v0.11.0 exposes no switch to
  disable isolation; the observation is recorded in `Surprises & discoveries`
  as upstream feedback rather than worked around.
- Risk: clap's help layout depends on terminal width and colour detection,
  making snapshots host-dependent. Severity: medium. Likelihood: low.
  Mitigation: `clap` is used without the `wrap_help` feature (no terminal-width
  probing), end-to-end tests run the binary through `assert_cmd` with a piped
  (non-terminal) standard output, and the test commands remove `CLICOLOR_FORCE`,
  `COLUMNS`, and `NO_COLOR` from the child environment with
  `Command::env_remove`.
- Risk: `rstest-bdd` 0.6.0 generates code against the caller's `rstest`; the
  repository pins `rstest = "0.27.0"` while a pending Dependabot branch moves
  to 0.28.0. Severity: low. Likelihood: low. Mitigation: `EP-M3` compiles the
  BDD harness first; if 0.27 is incompatible, escalate with the option of
  adopting the 0.28.0 bump in this branch.
- Risk: the user-guide tested-example contract from
  `docs/documentation-style-guide.md` ("make every fenced example …
  executable", with a shared loader) adds code that the roadmap item does not
  name. Severity: low. Likelihood: certain. Mitigation: scope the loader to
  what the first guide needs (marker parsing, fence extraction, and the four
  failure checks), document its reuse policy, and budget it explicitly above.
- Risk: later roadmap items may want different option spellings (for example
  `--base-ref` instead of `--base`, or `--check` on `extract`). Severity: low.
  Likelihood: medium. Mitigation: the crate is pre-1.0 with no released
  interface; spellings are recorded as decisions (D3–D7) and in design §7.4 so
  that later changes are deliberate.

## Progress

- [x] (2026-11-20 12:00Z) Branch renamed; OrthoConfig guide refreshed from the
  `v0.11.0` tag (`742ed58`); design and developers' guide references updated
  (`dc5c0b7`).
- [x] (2026-11-20 12:40Z) Reconnaissance (Wyvern team) and feasibility probe
  completed; initial draft written.
- [ ] Expert design review applied.
- [ ] Plan approved by the user.
- [ ] `EP-M1` typed command declarations parse in-process.
- [ ] `EP-M2` help text is localized through the en-GB catalogue.
- [ ] `EP-M3` the `splitters` binary uses the new surface end to end.
- [ ] `EP-M4` documentation, tested guide examples, and roadmap closure.

## Surprises & discoveries

- Observation: the vendored OrthoConfig guide described v0.10.0, while the
  roadmap item mandates v0.11.0. Evidence: `docs/ortho-config-users-guide.md`
  line 8 at `4c82d66`. Impact: refreshed from the `v0.11.0` tag before planning
  (`742ed58`), with relative links rewritten to tag-pinned absolute URLs.
- Observation: localized clap errors contain Unicode isolation marks around
  interpolated values, and OrthoConfig v0.11.0 offers no option to disable
  them. Evidence: probe output `error: Missing required argument: ⁨<PROPOSAL>⁩`;
  no `isolating` identifier in `ortho_config-0.11.0/src` or
  `ortho_l10n-0.11.0/src`. Impact: snapshot normalization (see `Risks`);
  candidate upstream feedback.
- Observation: OrthoConfig's built-in clap-error catalogue ships `en-US` and
  `ja` only, yet a `FluentLocalizer` built for `en-GB` with a consumer
  catalogue still localizes clap errors. Evidence: probe with
  `langid!("en-GB")` rendered the localized "Missing required argument" text;
  `ortho_l10n-0.11.0/locales/` lists `en-US` and `ja`. Impact: an `en-GB`
  consumer catalogue is viable without vendoring the error catalogue.

## Decision log

- D1. Decision: subcommand argument structs derive
  `clap::Parser + serde::Serialize + serde::Deserialize + ortho_config::OrthoConfig`
  and set
  `#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.<command>")]`;
  the root `Cli` derives only `clap::Parser`, and the `Command` enum derives
  only `clap::Subcommand`. Rationale: this is the shape mandated by the design
  and the developers' guide, and it is the shape used by OrthoConfig's own
  `hello_world` example at `v0.11.0`. Deriving `OrthoConfig` on the root was
  rejected because the root has no options and the developers' guide calls for
  a plain clap type; the root catalogue base is supplied by a crate constant
  instead. Deriving `SelectedSubcommandMerge` and `OrthoConfigSubcommandDocs`
  on the enum is deferred to 1.1.4, which owns merging, because nothing in this
  item consumes them. Date/Author: 2026-11-20, planning agent.
- D2. Decision: each subcommand's `localization_base` is the root base
  `splitters.cli` followed by the subcommand name, and this agreement is
  tested. Rationale: the probe showed that `LocalizeCmd::localize` looks up
  `splitters-cli-map-args-base-help` for the `map --base` argument, which is
  exactly the identifier OrthoConfig generates in `MapArgs::ARG_IDS` only when
  `localization_base = "splitters.cli.map"`. A mismatch silently falls back to
  the English doc comment, so it must be caught by a test, not by review.
  Date/Author: 2026-11-20, planning agent.
- D3. Decision (proposed; needs user confirmation): the option spellings are
  `--base <REF>` on all three commands, `--manifest-dir <DIR>` on all three
  commands, a required positional `<PROPOSAL>` operand on `validate` and
  `extract`, `--check <COMMAND>` on `validate` only, and `--branch <NAME>` and
  `--publish[=<BOOL>]` on `extract` only. Rationale: the design's Tables 3–5
  assign inputs per command, and placing a field on a command that does not use
  it would create a configuration key with no meaning. `--base` is the
  conventional spelling (Git hosting tools use it) and keeps the per-command
  configuration key short (`base`); the alternative `--base-ref` mirrors the
  manifest's `base_ref` field but would invite confusion between "the
  configured override" and "the value recorded in the manifest". The proposal
  is an operand rather than an option because it names the object the command
  acts on and is never a sensible configuration-file value; making it a
  required positional also gives a clear usage error when it is missing.
  `map`'s base is declared optional at the parser level so that 1.1.4 can
  source it from configuration (`[cmds.map] base = "origin/main"`); whether a
  missing base is an error is decided by roadmap 2.1.1. Date/Author:
  2026-11-20, planning agent.
- D4. Decision (proposed): `--manifest-dir` defaults to `splitters`, declared
  once as `cli::DEFAULT_MANIFEST_DIR` and used by both the clap default and the
  struct's `Default` implementation, with
  `#[ortho_config(cli_default_as_absent)]` so that 1.1.4's match-aware merge
  lets files and the environment override the clap default. Rationale: the
  design's §8 example layout uses `splitters/`, and a default keeps the common
  workflow (`splitters map --base origin/main`, then
  `splitters validate <proposal>`) free of repetition. Interpretation of the
  relative path (current directory or repository root) is owned by 1.1.3 and
  2.1.x and is not decided here. Date/Author: 2026-11-20, planning agent.
- D5. Decision (proposed): `--publish` is declared as `Option<bool>` with
  `num_args = 0..=1`, `require_equals = true`, and
  `default_missing_value = "true"`, so the accepted spellings are `--publish`,
  `--publish=true`, and `--publish=false`; `--publish false` is rejected.
  Rationale: this reproduces the boolean model documented in the OrthoConfig
  guide ("Boolean flag spellings", Table 4): omission defers to lower layers and
  `--publish=false` overrides a configured `true`. The `hello_world` example's
  `ArgAction::SetTrue` plus `skip_serializing_if = "is_false"` pattern was
  rejected because it makes a configured `publish = true` impossible to switch
  off from the command line. The probe confirmed all four spellings behave as
  stated. Date/Author: 2026-11-20, planning agent.
- D6. Decision (proposed): `--check` takes one string, `Option<String>`, and
  this item does not decide how it is executed. Rationale: the design writes
  `--check <cmd>`. Whether 3.2.1 splits the string into an argument vector
  (preferred by `AGENTS.md`'s "structured APIs instead of executable string
  construction") or runs it through a shell is a 3.2.1 decision; a single
  string is compatible with either. `--check` on `extract` (the design says
  extract "reruns the full validation sequence") is left to 3.2.1 or 4.1.1,
  where adding a field is an additive change. Date/Author: 2026-11-20, planning
  agent.
- D7. Decision (proposed): ship one consumer catalogue,
  `locales/en-GB/messages.ftl`, embedded with `include_str!`, and always build
  the localizer for `en-GB`; do not detect the locale from the process
  environment in this item. If the catalogue fails to build, fall back to
  `ortho_config::NoOpLocalizer` (clap's own text from doc comments) and report
  the failure on standard error. Rationale: the roadmap asks for "localized
  help support", meaning that help text flows through a catalogue and can be
  translated; it does not ask for a second language. Locale detection would
  require reading `LANG`/`LC_ALL`, which the repository forbids outside an
  injected environment, and that seam belongs with 1.1.4's injected
  configuration sources. `en-GB` matches the project's en-GB-oxendict writing
  standard. Date/Author: 2026-11-20, planning agent.
- D8. Decision: parsed commands dispatch to a skeleton that returns
  `CommandOutcome::NotYetImplemented(CommandName)`; the binary reports
  `splitters <command>: not implemented yet` on standard error and exits with
  status `2`. Display requests (`--help`, `--version`) and usage errors are
  rendered and exited by clap's `Error::exit`, which uses status `0` for
  display requests and `2` for usage errors. Rationale: exiting `0` for a
  command that did nothing would mislead automation, and `2` is the design's
  "operational failure" class. The final error hierarchy and the remapping of
  usage errors to the design's status `1` belong to 1.1.2; this item records
  the interim behaviour in the users' guide as "early-adopter" behaviour rather
  than inventing the hierarchy. Date/Author: 2026-11-20, planning agent.
- D9. Decision: the user-facing help strings live in the Fluent catalogue;
  the `///` doc comments on fields stay as developer documentation and as the
  `NoOpLocalizer` fallback, and must say the same thing in shorter form.
  Rationale: OrthoConfig v0.11.0 has no way to generate the catalogue from doc
  comments, and the catalogue completeness test (D2, `INV-catalogue-complete`)
  guarantees that the localized path never silently falls back. Date/Author:
  2026-11-20, planning agent.
- D10. Decision: no Verus proof, Kani harness, or state-machine model is
  planned for this item; verification is by `rstest` tables, one `proptest`
  property, `insta` snapshots, `rstest-bdd` scenarios, and `assert_cmd`
  end-to-end tests. See `Formal verification decision`. Date/Author:
  2026-11-20, planning agent.
- D11. Decision: record the command-surface conventions (D1–D9) in a new
  §7.4 of `docs/splitters-design.md` rather than in an ADR (architectural
  decision record). Rationale: the architectural decision (OrthoConfig for the
  command surface) is already recorded in design §5 and §12; this item refines
  it with spellings and conventions that are reversible before 1.0. If the
  expert review judges the localization choice (D7) substantive enough for an
  ADR, this decision is revisited before approval. Date/Author: 2026-11-20,
  planning agent.

## Outcomes & retrospective

Not started. Populate at each milestone boundary and at completion.

## Context and orientation

Inspected base revision: `dc5c0b7` on branch
`1-1-1-replace-the-stub-binary-with-command-skeletons`. Line numbers below are
for that revision; relocate symbols with `grep -n`.

The repository is a single Cargo package named `splitters` (edition 2024),
built with the pinned toolchain `nightly-2026-11-06` (`rust-toolchain.toml`).
`.cargo/config.toml` enables the parallel front end (`-Zthreads=8`) and, on
Linux, the `mold` linker. There is no Cargo workspace.

Current code:

- `src/lib.rs` (17 lines) exposes `pub const fn greeting() -> &'static str`
  returning `"Hello from Splitters!"`, with one unit test
  `tests::greeting_matches_stub_output`.
- `src/main.rs` (10 lines) prints `greeting()` with `println!` under an
  `#[expect(clippy::print_stdout, ...)]`.
- `tests/build_standard_contract.rs`, `tests/build_standard_backend.rs`, and
  `tests/dev_build_footprint.rs` (with helpers in `tests/build_standard/`)
  check build configuration only; none runs the binary or calls `greeting()`.
  `tests/workflow_contracts/cv005_wiring_test.py` checks the Makefile's CV-005
  wiring. `tests/build_standard_backend.rs` is about 548 lines, a pre-existing
  breach of the 400-line limit that this item does not touch.
- `Cargo.toml` has no runtime dependencies and the development dependencies
  `cap-std = "4.0.3"`, `rstest = "0.27.0"`, and `toml = "1.1.3"`. Its `[lints]`
  table denies, among others, `missing_docs`, `print_stdout`, `print_stderr`,
  `unwrap_used`, `expect_used`, `indexing_slicing`, `shadow_*`,
  `missing_const_for_fn`, `must_use_candidate`, `use_self`, `allow_attributes`,
  and `result_large_err`.

Gates (see `AGENTS.md` and the `Makefile`): `make check-fmt` (rustfmt check plus
`mdtablefix --check`), `make lint` (`interrogate` over `tests/` Python,
`cargo doc` with `-D warnings`, `cargo clippy --all-targets --all-features` with
`-D warnings`, and `whitaker --all`), and `make test`
(`cargo nextest run --all-targets --all-features`, then `cargo test --doc`).
`make markdownlint` and `make nixie` validate Markdown and Mermaid.
`cargo-nextest` 0.9.133, `whitaker`, `mold`, `mdtablefix`, `markdownlint-cli2`,
and `nixie` are installed on the planning host. CI (`.github/workflows/ci.yml`)
runs `make check-fmt`, markdownlint, `make lint`,
`make test-workflow-contracts`, and a coverage action that runs the tests.

Terms used in this plan:

- Subcommand: a verb after the program name (`map`, `validate`, `extract`).
- Operand: a positional argument (here, the proposal identifier).
- Catalogue: a Fluent `.ftl` file mapping message identifiers to text.
- Display request: `--help`, `help <command>`, or `--version`; clap reports
  these as `clap::Error` values whose kind is `DisplayHelp` or
  `DisplayVersion`, and OrthoConfig exposes `is_display_request` to detect them.
- Snapshot test: an `insta` assertion that compares output to a reviewed file
  under `tests/snapshots/`.

Budget survey: the production change replaces 27 lines of stub code; the
testing references (`docs/rust-testing-with-rstest-fixtures.md`,
`docs/rstest-bdd-users-guide.md`) and the OrthoConfig `hello_world` example
(about 2,000 lines including its tests) indicate that a three-command surface
with localization and full test coverage lands near 1,100 lines, which sets the
code budget. The documentation budget reflects that `docs/users-guide.md` does
not exist yet and must be created with tested examples. No earlier ExecPlans
exist to calibrate against.

Signposts. Read these before implementing:

- `docs/roadmap.md` §1.1 (items 1.1.1–1.1.4) for scope boundaries.
- `docs/splitters-design.md` §5 (constraints), §7 (command contracts, Tables
  3–5), §12 (implementation approach and module split), §13 (verification).
- `docs/ortho-config-users-guide.md`: "Give each subcommand its own settings"
  (subcommand prefixes and `[cmds.<name>]`), "Boolean flag spellings" (Table 4),
  `cli_default_as_absent`, "Localize help and errors", "Use the generated
  message identifiers".
- `docs/developers-guide.md` §2–§3.1 (module boundaries and the `cli`
  contract).
- `docs/reliable-testing-in-rust-via-dependency-injection.md` (no process
  environment mutation in tests).
- `docs/rust-testing-with-rstest-fixtures.md` (fixtures and `#[case]`
  tables), `docs/rstest-bdd-users-guide.md` (feature files, `#[scenario]`, step
  macros), `docs/rust-doctest-dry-guide.md` (doctest style), and
  `docs/complexity-antipatterns-and-refactoring-strategies.md` (keep the
  dispatcher flat).
- `docs/documentation-style-guide.md` "User's guide" (tested examples and the
  shared loader) and "Architectural decision records".
- Skills: `rust-router` (then `rust-types-and-apis` for the declarations,
  `rust-errors` for the dispatch outcome, `rust-unit-testing` for tables and
  assertions, `proptest` for the property), `domain-cli-and-daemons` (stream
  discipline and exit status), `execplans`, `en-gb-oxendict`, and
  `codegraph-mcp` for navigation.

## Conformance basis

Upstream artefacts and inspected revisions (all at `dc5c0b7` unless stated):

- `docs/roadmap.md` item 1.1.1 (`docs/roadmap.md` lines 23–34).
- `docs/splitters-design.md` §5, §6, §7 (Tables 3–5), §12, §13.
- `docs/developers-guide.md` §2 and §3.1.
- `docs/ortho-config-users-guide.md`, imported from `leynos/ortho-config` tag
  `v0.11.0` at `742ed58`.
- `AGENTS.md` and `docs/documentation-style-guide.md` as governing standards.
- No Terms of Reference document or ADR exists in this repository; none is
  invented.

Requirements derived for this item:

- `REQ-surface`: the binary exposes exactly the subcommands `map`,
  `validate`, and `extract`, each with the inputs of design Tables 3–5 (design
  §7; roadmap 1.1.1 bullet 1).
- `REQ-typed`: base override, manifest directory, proposal selection, branch
  naming, `--check`, and `--publish` are typed fields (roadmap 1.1.1 bullet 2).
- `REQ-single-declaration`: every command-line option is a field of an
  OrthoConfig-derived struct, and no other configuration schema or parser
  exists (design §5, §6; roadmap 1.1.1 bullet 3 and success criterion 3).
- `REQ-help`: `splitters --help` documents the full command surface, and
  each command's help documents its options (roadmap success criterion 1).
- `REQ-parse-coverage`: command parsing has snapshot and behaviour coverage
  (roadmap success criterion 2).
- `REQ-localized-help`: help text is resolved through a Fluent catalogue via
  OrthoConfig's localizer (user instruction for this item).
- `REQ-streams`: help and version on standard output, all other diagnostics on
  standard error (design §7; developers' guide §3.1).
- `REQ-no-effects`: no command reads the repository, the process environment,
  or the file system in this item (roadmap 1.1.3 and 1.1.4 own those).

Gaps at the base revision:

- `GAP-stub`: `src/main.rs` ignores arguments and prints a greeting.
- `GAP-deps`: no `clap`, `ortho_config`, `serde`, or `camino` dependency.
- `GAP-users-guide`: `docs/users-guide.md` does not exist.
- `GAP-design-conventions`: design §7 names inputs but not option spellings,
  types, or defaults.

Trace links:

```plaintext
roadmap-1.1.1 -> REQ-surface, REQ-typed, REQ-single-declaration -> EP-M1 -> cli::tests::parsing::*, cli::tests::declaration::*
roadmap-1.1.1 -> REQ-localized-help -> EP-M2 -> cli::tests::catalogue::*
roadmap-1.1.1 -> REQ-help, REQ-parse-coverage, REQ-streams, REQ-no-effects -> EP-M3
  -> tests/cli_end_to_end.rs, tests/command_surface_bdd.rs, tests/snapshots/*
GAP-users-guide, GAP-design-conventions -> EP-M4 -> docs/users-guide.md, docs/splitters-design.md §7.4, tests/users_guide_examples.rs
```

## Verification plan

### Abstract model

The command surface is a finite grammar. Let `A` be the argument vector after
the program name. The parser is a partial function `parse : A -> Cli + Error`
where `Cli` is one of `Map(MapArgs)`, `Validate(ValidateArgs)`, or
`Extract(ExtractArgs)` and `Error` is a clap error with a kind. The dispatcher
is a total function `dispatch : Cli -> CommandOutcome`. The binary composes
them: on `Error` with a display kind it prints to standard output and exits
`0`; on any other `Error` it prints to standard error and exits `2`; on `Cli`
it prints the not-implemented diagnostic to standard error and exits `2`. There
is no state and no ordering; every invocation is independent.

The declaration model is a set of fields per command:

- `map`: `base: Option<String>`, `manifest_dir: Utf8PathBuf`.
- `validate`: `proposal: String`, `manifest_dir: Utf8PathBuf`,
  `base: Option<String>`, `check: Option<String>`.
- `extract`: `proposal: String`, `manifest_dir: Utf8PathBuf`,
  `base: Option<String>`, `branch: Option<String>`, `publish: Option<bool>`.

### Invariants and lemmas

- `INV-single-declaration` (`REQ-single-declaration`): for each subcommand,
  the set of clap argument identifiers in its command tree, excluding clap's
  generated `help`, equals the set of `name` values in the struct's
  OrthoConfig-generated `ARG_IDS`; and the root command declares no arguments
  other than `help` and `version`. This shows that every option a user can type
  is an OrthoConfig field and that no option exists outside the declarations.
- `INV-surface` (`REQ-surface`, `REQ-typed`): the root command has exactly
  the subcommands `map`, `validate`, `extract` (plus clap's generated `help`),
  and each subcommand's argument set equals the model above, with the stated
  long names, value names, and operand positions.
- `INV-default-agreement` (D4): `MapArgs::default().manifest_dir`,
  `ValidateArgs::default().manifest_dir`,
  `ExtractArgs::default().manifest_dir`, and the value parsed when
  `--manifest-dir` is omitted are all equal to `DEFAULT_MANIFEST_DIR`.
- `INV-verbatim-values` (`REQ-typed`): for every string value `v` that does
  not begin with `-` and contains no NUL, parsing
  `["splitters", "extract", v, "--base", v, "--branch", v, "--manifest-dir", v]`
  yields fields equal to `v` (and `Utf8PathBuf::from(v)`): the declarations do
  not trim, case-fold, or otherwise normalize operator input.
- `INV-bool-spellings` (D5): `--publish` and `--publish=true` yield
  `Some(true)`, `--publish=false` yields `Some(false)`, omission yields `None`,
  and `--publish false` and `--publish=maybe` are rejected.
- `INV-catalogue-ids-agree` (D2): for each subcommand struct `S` with clap
  name `n`, `S::LOCALIZATION_BASE == format!("{ROOT}.{n}")` where `ROOT` is
  `cli::LOCALIZATION_BASE`.
- `INV-catalogue-complete` (`REQ-localized-help`, D9): every
  `ARG_IDS[i].help_id` and every `ABOUT_ID` of the three structs, plus the root
  about identifier, resolves to a message in the embedded en-GB catalogue
  (`Localizer::lookup(id, None).is_some()`), and the catalogue contains no
  message identifiers beyond those (no stale entries).
- `INV-streams` (`REQ-streams`, D8): for each display request, standard error
  is empty and the exit status is `0`; for each usage error and each
  well-formed command, standard output is empty and the exit status is `2`.
- `INV-no-effects` (`REQ-no-effects`): running each well-formed command in an
  empty temporary directory leaves that directory empty and does not require a
  Git repository.

Lemma: `L-dispatch-total`: `dispatch` maps each `Command` variant to the
`NotYetImplemented` outcome naming that variant; with `INV-surface` this gives
`INV-streams` for well-formed commands. It is discharged by an exhaustive
`match` (the compiler enforces totality) and an `rstest` table.

### Axioms (trusted, not verified here)

- `AXIOM-clap`: `clap` 4.6 derive parses arguments as its documentation
  states, including `num_args`, `require_equals`, `default_missing_value`,
  positional `required`, and `Error::exit` stream and status behaviour (status
  `0` for display requests on standard output, `2` for usage errors on standard
  error). The repository depends on this in `src/main.rs`.
- `AXIOM-ortho-derive`: `ortho_config` 0.11.0's derive generates
  `LOCALIZATION_BASE`, `ABOUT_ID`, and `ARG_IDS` as documented in "Use the
  generated message identifiers", and rejects duplicate normalized argument
  identifiers at compile time.
- `AXIOM-ortho-localize`: `LocalizeCmd::with_base(..).localize(..)` looks up
  `<base>-<subcommand>-about` and `<base>-<subcommand>-args-<arg>-help`, and
  falls back to clap's text when an identifier is missing. Observed in the
  planning probe; the repository depends on it through
  `INV-catalogue-ids-agree` and `INV-catalogue-complete`, which together make a
  fallback impossible for the shipped catalogue.
- `AXIOM-fluent-isolation`: Fluent inserts U+2068/U+2069 around interpolated
  values; this is cosmetic and not under repository control.

### Methods per obligation

- Obligation: `INV-surface`, `INV-single-declaration`.
  Method: `rstest` parameterized tests over the clap command tree
  (`Cli::command()` and `get_subcommands`/`get_arguments`) and `ARG_IDS`.
  Rationale: the domain is three finite declarations; enumeration is
  exhaustive. Domain: all three subcommands and the root. Artefact:
  `src/cli/tests/declaration.rs`. Evidence:
  `cargo nextest run -E 'test(declaration)'`; red before the declarations exist
  (compile failure on missing types, then assertion failures on missing
  arguments), green after. Non-vacuity: the expected sets are literal lists
  written in the test, not derived from the code under test, and each is
  asserted non-empty. Two seeded faults must be rejected and then reverted: a
  temporary root-level `#[arg(long)] verbose: bool` on `Cli` must fail the root
  assertion with a message naming `verbose`; and a temporary
  `#[ortho_config(skip_cli)]` field with `#[arg(long)]` on `MapArgs` (a flag
  that OrthoConfig does not list in `ARG_IDS`) must fail the equality with a
  message naming the field.
- Obligation: `INV-default-agreement`.
  Method: `rstest` table over the three structs. Artefact:
  `src/cli/tests/declaration.rs`. Evidence: as above. Non-vacuity: seeded fault
  changing the `Default` impl of `MapArgs` to `"manifest"` must fail; revert.
- Obligation: `INV-verbatim-values`.
  Method: `proptest` property, 256 cases, over strings generated by the regex
  `[^-\x00][^\x00]{0,40}` (not starting with `-`). Rationale: the input space
  is open-ended text, and the failure mode (a normalizing value parser) would
  affect arbitrary inputs rather than one example; a property is the cheapest
  adequate check. Artefact: `src/cli/tests/properties.rs`. Evidence:
  `cargo nextest run -E 'test(verbatim)'`. Oracle: the generated input itself,
  independent of the parser. Non-vacuity: the generator's first character class
  excludes only `-`; the test also asserts on a fixed witness containing
  spaces, Unicode, and `=` (`"feature/añadir = x"`). Seeded fault: temporarily
  add `value_parser = trim_value` to `--branch` and confirm a counterexample
  with leading or trailing whitespace is reported; revert and delete any
  `proptest-regressions/` file created by the fault.
- Obligation: `INV-bool-spellings`, `L-dispatch-total`.
  Method: `rstest` `#[case]` tables. Artefact: `src/cli/tests/parsing.rs`,
  `src/cli/dispatch.rs` unit tests. Non-vacuity: each table includes both
  accepting and rejecting rows; the rejecting rows assert the specific
  `clap::error::ErrorKind` (`UnknownArgument` for `--publish false`,
  `InvalidValue` for `--publish=maybe`, `MissingRequiredArgument` for a missing
  proposal).
- Obligation: `INV-catalogue-ids-agree`, `INV-catalogue-complete`.
  Method: `rstest` table plus one completeness test that parses the embedded
  catalogue's message identifiers (lines matching `^[a-z][a-z0-9_-]* =`) and
  compares them with the set computed from the derive constants. Artefact:
  `src/cli/tests/catalogue.rs`. Non-vacuity: seeded faults: delete one
  `-args-...-help` line from the catalogue (must fail naming the missing
  identifier) and add an unused `splitters-cli-map-args-colour-help` line (must
  fail naming the stale identifier); revert both.
- Obligation: `INV-streams`, `INV-no-effects`, `REQ-help`,
  `REQ-parse-coverage`. Method: `assert_cmd` end-to-end tests over the built
  binary, `insta` snapshots for `--help` of the root and each subcommand plus
  two usage errors, and `rstest-bdd` scenarios for the main user outcomes.
  Rationale: the contract is externally observable process behaviour; only
  running the binary proves it. Artefact: `tests/cli_end_to_end.rs`,
  `tests/command_surface_bdd.rs`, `tests/features/command_surface.feature`,
  `tests/snapshots/`. Evidence:
  `cargo nextest run --test cli_end_to_end --test command_surface_bdd`.
  Non-vacuity: before `EP-M3` changes `src/main.rs`, the end-to-end tests are
  run against the stub and must fail (the stub prints the greeting and exits
  `0` for every input). `INV-no-effects` uses a `tempfile`-free
  `cap_std::fs::Dir` over a directory created by the test harness and asserts
  it is empty after the run, after first asserting a witness write by the test
  itself is observable (proving the check can see files).

### Formal verification decision

No Verus proof, Kani harness, or model checker is warranted. This item
introduces no business logic: it declares data types and delegates parsing to
`clap` (an axiom). The only invariants are over finite declarations (handled
exhaustively by tables) and one open input space (handled by a property test
whose oracle is the input). A Verus proof would have to model clap's parser,
which is third-party behaviour outside the repository's verification remit, and
would leave the observable process contract unverified. Kani adds nothing
because there is no arithmetic, `unsafe` code, or bounded state machine. The
first roadmap items where Verus is plausibly warranted are 1.2.2 (proposal
graph acyclicity) and 2.2.x/4.1.3 (fragment-set subtraction); this plan makes
no proof obligations for them. Recorded as D10.

### Suggested BDD features

Feature file: `tests/features/command_surface.feature`, run by
`tests/command_surface_bdd.rs`
(`cargo nextest run --test command_surface_bdd`). Scenarios link to `REQ-help`,
`REQ-parse-coverage`, `REQ-streams`, and `EP-M3`; the documentation scenario
links to `GAP-users-guide` and `EP-M4`.

```gherkin
Feature: Splitters command surface
  The splitters binary declares map, validate, and extract and reports
  their status without touching the repository.

  Scenario: The top-level help lists every command
    When the operator runs "splitters --help"
    Then the exit status is 0
    And standard output lists the commands "map", "validate", and "extract"
    And standard error is empty

  Scenario Outline: Each command documents its options
    When the operator runs "splitters <command> --help"
    Then the exit status is 0
    And standard output mentions "<option>"

    Examples:
      | command  | option         |
      | map      | --base         |
      | map      | --manifest-dir |
      | validate | <PROPOSAL>     |
      | validate | --check        |
      | extract  | --branch       |
      | extract  | --publish      |

  Scenario: A well-formed command reports that it is not implemented yet
    When the operator runs "splitters validate add-parser --check 'make test'"
    Then the exit status is 2
    And standard output is empty
    And standard error says "splitters validate: not implemented yet"

  Scenario: A missing proposal is a usage error
    When the operator runs "splitters extract"
    Then the exit status is 2
    And standard output is empty
    And standard error mentions "<PROPOSAL>"

  Scenario: A boolean flag rejects a detached value
    When the operator runs "splitters extract add-parser --publish false"
    Then the exit status is 2
    And standard error mentions "false"

  Scenario: The documented help matches the binary
    Given the users' guide example "splitters-help"
    When the operator runs "splitters --help"
    Then standard output equals the documented example
```

The last scenario is added in `EP-M4`, when the users' guide exists; the others
are added in `EP-M3`. Steps split the quoted command with `shlex` semantics
implemented by a tiny test-local tokenizer over a fixed vocabulary (single
quotes only), so no extra dependency is needed.

## Plan of work

### `EP-M1` typed command declarations parse in-process

Assigned: `REQ-surface`, `REQ-typed`, `REQ-single-declaration`, `GAP-deps`, D1,
D3–D6.

1. Add runtime dependencies to `Cargo.toml`: `camino` (feature `serde1`),
   `clap` (feature `derive`), `ortho_config`, and `serde` (feature `derive`);
   add development dependencies `googletest`, `pretty_assertions`, and
   `proptest` (versions in `Interfaces and dependencies`).
2. Red: create `src/cli/tests/mod.rs`, `src/cli/tests/declaration.rs`,
   `src/cli/tests/parsing.rs`, and `src/cli/tests/properties.rs` with the tests
   listed under `Verification plan` for `INV-surface`, `INV-single-declaration`,
   `INV-default-agreement`, `INV-verbatim-values`, and `INV-bool-spellings`,
   plus parsing tables covering: each command with only required inputs; each
   command with every option; `--manifest-dir` omitted (default) and supplied;
   a proposal containing `/` and Unicode; a proposal beginning with `-`
   supplied after `--` (`splitters validate -- -odd`); and the rejecting rows.
   Add a minimal `src/cli/mod.rs` that declares only `pub mod` stubs so the
   failure is an unresolved-type compile error. Run
   `cargo nextest run --lib -E 'test(cli::)'` and record the failure.
3. Green: write `src/cli/mod.rs` (root `Cli`, `Command`, `LOCALIZATION_BASE`,
   `DEFAULT_MANIFEST_DIR`, `command`, and `parse_from`), and one module per
   command, `src/cli/map.rs`, `src/cli/validate.rs`, and `src/cli/extract.rs`,
   each declaring its struct per the model and D3–D5, with a manual or derived
   `Default` that uses `DEFAULT_MANIFEST_DIR`. `parse_from` takes
   `&dyn ortho_config::Localizer` from the start; unit tests pass
   `NoOpLocalizer`. Expose `pub mod cli;` from `src/lib.rs`, keeping
   `greeting()` until `EP-M3` so the binary still builds.
4. Refactor: keep each module under 150 lines; ensure doc comments are en-GB
   and that public functions have doctest examples (for `parse_from`, a
   `no_run`-free doctest that parses `["splitters", "map", "--base", "main"]`
   with `NoOpLocalizer`).
5. Run `make check-fmt`, `make lint`, and `make test` sequentially.

### `EP-M2` help text is localized through the en-GB catalogue

Assigned: `REQ-localized-help`, D2, D7, D9.

1. Red: add `src/cli/tests/catalogue.rs` with `INV-catalogue-ids-agree` and
   `INV-catalogue-complete`, referencing `cli::localization::help_localizer` and
   `cli::localization::CATALOGUE`. Run and record the compile failure.
2. Green: create `locales/en-GB/messages.ftl` with the root about and
   long-about text, one about line per subcommand, and one help line per
   argument (using the identifiers `splitters-cli-about`,
   `splitters-cli-map-about`, `splitters-cli-map-args-base-help`, and so on).
   Create `src/cli/localization.rs` with `CATALOGUE` (the `include_str!` of the
   file) and `help_localizer()`, which builds an
   `ortho_config::FluentLocalizer` for `en-GB` with
   `FluentLocalizerBuilder::new(langid!("en-GB")).with_consumer_resources(..)`.
   Update `cli::command` to apply
   `with_base(LOCALIZATION_BASE).localize(localizer)`.
3. Refactor: confirm with an in-process `insta` snapshot of
   `cli::command(&help_localizer()?).render_help()` for the root that the
   catalogue text, not the doc comment, is used (the doc comments are
   deliberately shorter, so the snapshot distinguishes them).
4. Run the three gates.

### `EP-M3` the `splitters` binary uses the new surface end to end

Assigned: `REQ-help`, `REQ-parse-coverage`, `REQ-streams`, `REQ-no-effects`,
`GAP-stub`, D8.

1. Add development dependencies `assert_cmd`, `insta` (feature `filters`),
   `rstest-bdd`, and `rstest-bdd-macros`.
2. Red: write `tests/cli_end_to_end.rs` (help snapshots for root and the three
   subcommands, `--version`, two usage-error snapshots with the isolation-mark
   filter, the not-implemented diagnostics table, and `INV-no-effects`) and
   `tests/features/command_surface.feature` with `tests/command_surface_bdd.rs`
   (all scenarios except the documentation scenario). Run them against the stub
   binary and record the failures (exit status `0` and the greeting on standard
   output).
3. Green: add `src/cli/dispatch.rs` with `CommandName`, `CommandOutcome`, and
   `dispatch`, and replace `src/main.rs`: build the localizer (falling back to
   `NoOpLocalizer` with a one-line standard-error warning if building fails),
   call `cli::parse_from(std::env::args_os(), &localizer)`, on error call
   `err.exit()`, on success `dispatch`, write the outcome's diagnostic to a
   locked standard-error handle, and return `std::process::ExitCode::from(2)`.
   Delete `greeting()` and its test from `src/lib.rs`, and give `src/lib.rs` a
   crate-level doc comment describing the library-first layout.
4. Refactor: keep `main` under 30 lines by extracting `report` and
   `localizer_or_fallback` helpers; review the snapshots and accept them with
   `cargo insta review` (or `INSTA_UPDATE=always` followed by a diff review).
5. Run the three gates.

### `EP-M4` documentation, tested guide examples, and roadmap closure

Assigned: `GAP-users-guide`, `GAP-design-conventions`, D11.

1. Red: write `tests/support/doc_examples.rs` (the shared tested-example
   loader) with unit tests for its failure modes, and
   `tests/users_guide_examples.rs`, which loads `docs/users-guide.md`, fails
   when the file is missing, and compares the `splitters-help`,
   `splitters-validate-help`, and `splitters-extract-help` examples with the
   binary's output. Add the documentation scenario to the feature file. Run and
   record the failures (missing guide).
2. Green: write `docs/users-guide.md`: an audience paragraph naming the
   release (`0.1.0`, early-adopter, commands not yet implemented); "Run the
   first command" with the tested help example; a command reference table per
   command (option, value, default, purpose); "Interpret the current exit
   status" describing D8 as interim behaviour; and a note that configuration
   files and `SPLITTERS_` environment variables arrive with roadmap 1.1.4.
   Every fence carries a `<!-- tested-example: <id> -->` marker.
3. Update `docs/splitters-design.md` with §7.4 "Command-line declaration
   conventions" recording D1–D9 (one paragraph each, plus a table of options);
   update `docs/developers-guide.md` §1 (current state no longer a stub), §3.1
   (module files under `src/cli/`, the catalogue, the localization-base rule,
   and how to add an option: field, catalogue line, tests), and a testing
   section documenting the tested-example loader's reuse policy; update
   `docs/repository-layout.md` (`locales/`, `tests/features/`,
   `tests/snapshots/`, `tests/support/`) and `docs/contents.md` (users' guide
   and the execplans directory).
4. Mark roadmap item 1.1.1 done (`- [x]`) in `docs/roadmap.md` only after the
   acceptance criteria below pass.
5. Run `make fmt`, then `make check-fmt`, `make lint`, `make test`,
   `make markdownlint`, and `make nixie`.

## Milestones and plateaus

- Identifier and outcome: `EP-M1`, the library exposes typed declarations and
  an in-process parser; the binary is unchanged. Requirements and gaps:
  `REQ-surface`, `REQ-typed`, `REQ-single-declaration`, `GAP-deps`. Acceptance
  evidence: `cargo nextest run --lib -E 'test(cli::)'` passes; three gates
  pass. Conformance check:
  - Requirements satisfied by in-process tests.
  - Design §12 module split followed (`cli` only).
  - Upstream assumptions (Tables 3–5) unchanged.
  - Only authorized dependencies added.
  - No trust boundary or persisted format introduced.
  - Trace links current.
  Recovery: revert the milestone commit; nothing depends on it. Remaining gaps:
  localization, binary, documentation. Compatibility decision: none.
- Identifier and outcome: `EP-M2`, help resolves through the en-GB catalogue
  with no fallback. Requirements and gaps: `REQ-localized-help`. Acceptance
  evidence: catalogue tests and the in-process help snapshot pass; three gates
  pass. Conformance check:
  - Catalogue complete and stale-free.
  - No process environment read.
  - No new dependency beyond the list.
  - Trace links current.
  Recovery: revert; `EP-M1` remains valid. Remaining gaps: binary and
  documentation. Compatibility decision: none.
- Identifier and outcome: `EP-M3`, `splitters` parses, reports, and exits as
  specified; the stub is gone. Requirements and gaps: `REQ-help`,
  `REQ-parse-coverage`, `REQ-streams`, `REQ-no-effects`, `GAP-stub`. Acceptance
  evidence: end-to-end and BDD suites pass; snapshots reviewed; three gates
  pass. Conformance check:
  - Stream discipline holds.
  - No repository access.
  - Exit statuses documented as interim (1.1.2 owns the final mapping).
  - Trace links current.
  Recovery: revert; `EP-M2` remains valid. Remaining gaps: documentation.
  Compatibility decision: none (`greeting()` deleted; pre-1.0, no consumers).
- Identifier and outcome: `EP-M4`, documentation matches behaviour and is
  tested; roadmap 1.1.1 is done. Requirements and gaps: `GAP-users-guide`,
  `GAP-design-conventions`. Acceptance evidence:
  `tests/users_guide_examples.rs` and the documentation scenario pass; all
  gates including Markdown pass. Conformance check:
  - Design §7.4 matches the code.
  - Developers' guide and layout current.
  - Roadmap ticked only after acceptance.
  Recovery: revert documentation commits independently. Remaining gaps: roadmap
  1.1.2–1.1.4. Compatibility decision: none.

## Concrete steps

All commands run from the repository root
(`/home/leynos/.lody/repos/github---leynos---splitters/worktrees/296519a9-983b-4984-a924-fc3edcb6d5dd`
on the planning host). Use `set -o pipefail` in the shell before any `tee`
pipeline so a failing gate cannot appear to pass.

```plaintext
Working directory: repository root
EP-M1 red:   cargo nextest run --lib -E 'test(cli::)'  -> error[E0412]/E0433 unresolved types in src/cli/tests
EP-M1 green: cargo nextest run --lib -E 'test(cli::)'  -> all cli tests pass (about 40 cases including the 256-case property)
EP-M2 red:   cargo nextest run --lib -E 'test(catalogue)' -> unresolved cli::localization
EP-M2 green: cargo nextest run --lib -E 'test(catalogue)' -> pass
EP-M3 red:   cargo nextest run --test cli_end_to_end --test command_surface_bdd
             -> failures: exit status 0, stdout "Hello from Splitters!"
EP-M3 green: same command -> pass; snapshots accepted after review
EP-M4 red:   cargo nextest run --test users_guide_examples -> failure: docs/users-guide.md not found
EP-M4 green: same command -> pass
```

Gate sequence after every milestone (sequential, never in parallel):

```bash
set -o pipefail
BR=$(git branch --show-current)
make check-fmt 2>&1 | tee "/tmp/check-fmt-splitters-${BR}.out"
make lint 2>&1 | tee "/tmp/lint-splitters-${BR}.out"
make test 2>&1 | tee "/tmp/test-splitters-${BR}.out"
```

`EP-M4` additionally runs `make markdownlint` and `make nixie` with the same
logging pattern. Builds queue for compile slots through `build-limits-rustc`; a
waiting build is not hung.

## Validation and acceptance

Acceptance is behavioural:

- `splitters --help` exits `0`, prints the catalogue's about text and the
  three commands on standard output, and prints nothing on standard error.
- `splitters map --help`, `splitters validate --help`, and
  `splitters extract --help` list exactly the options in design §7.4 with the
  catalogue's help text; the four help outputs match reviewed snapshots and the
  users' guide examples.
- `splitters validate add-parser` exits `2` with
  `splitters validate: not implemented yet` on standard error and nothing on
  standard output; likewise for `map` and `extract`.
- `splitters extract` exits `2` with a usage error naming `<PROPOSAL>` on
  standard error.
- `splitters extract add-parser --publish false` exits `2`;
  `splitters extract add-parser --publish=false` parses.
- Running any command in an empty directory outside a Git repository leaves the
  directory empty.

Red-Green-Refactor evidence is recorded per milestone in `Artefacts and notes`
as the work proceeds, including each seeded fault and its reversion.

Quality criteria:

- Tests: `make test` passes, including the new unit, property, end-to-end,
  BDD, and documentation tests.
- Verification: every `INV-*` obligation above is discharged by its named
  artefact with its non-vacuity check performed once and recorded.
- Lint and format: `make check-fmt` and `make lint` pass with no new
  `#[expect]` other than any justified in `Decision log`.
- Documentation: `make markdownlint` and `make nixie` pass.

## Idempotence and recovery

Every step is re-runnable: tests and gates are pure; `cargo insta review` can
be repeated; documentation edits are plain text. Each milestone is one or more
commits, so `git revert` of a milestone's commits restores the previous
plateau. Seeded faults are applied and reverted within the working tree and
must not be committed; after each, run `git status --short` to confirm a clean
tree, and delete any `proptest-regressions/` file the fault produced. The
planning probe lives under `target/plan-probe/` (ignored by Git) and is deleted
before the plan is submitted.

## Artefacts and notes

Planning probe (2026-11-20, scratch crate under `target/plan-probe/`, deleted
afterwards). It declared the three structs per D1–D5 with `clap` 4.6.9,
`ortho_config` 0.11.0, `serde` 1.0.229, and `camino` 1.2.6, and a catalogue
containing two candidate spellings for the `map --base` help identifier.
Observations:

```plaintext
map arg base help_id=splitters-cli-map-args-base-help
map arg manifest_dir help_id=splitters-cli-map-args-manifest_dir-help
map base=splitters.cli.map about=splitters-cli-map-about

$ probe map --help
FTL map about

Usage: splitters map [OPTIONS]

Options:
      --base <REF>          FTL map base help (args form)
      --manifest-dir <DIR>  Manifest directory [default: splitters]
  -h, --help                Print help

$ probe extract p --publish        -> publish: Some(true)
$ probe extract p --publish=false  -> publish: Some(false)
$ probe extract p --publish false  -> kind=UnknownArgument
$ probe validate                   -> kind=MissingRequiredArgument
                                      error: Missing required argument: ⁨<PROPOSAL>⁩
```

`cargo clippy --all-targets -- -D warnings` with the repository's `[lints]`
table and `clippy.toml` passed after one `doc_markdown` fix in hand-written
probe text, so the derive output is compatible with the lint set. The runtime
dependency closure is 121 unique crates.

## Interfaces and dependencies

Authorized dependencies (caret requirements):

- Runtime: `camino = { version = "1.2.6", features = ["serde1"] }`,
  `clap = { version = "4.6.9", features = ["derive"] }`,
  `ortho_config = "0.11.0"`,
  `serde = { version = "1.0.229", features = ["derive"] }`.
- Development: `assert_cmd = "2.2.2"`, `googletest = "0.14.4"`,
  `insta = { version = "1.49.0", features = ["filters"] }`,
  `pretty_assertions = "1.4.1"`, `proptest = "1.12.0"`, `rstest-bdd = "0.6.0"`,
  `rstest-bdd-macros = "0.6.0"`. The existing `cap-std`, `rstest`, and `toml`
  stay as they are.
- Not added: `thiserror` (no error type in this item), `mockable` (no
  environment access), `fluent-bundle` (re-exported by `ortho_config`),
  `tempfile` (the test creates directories with `cap-std` under the build's
  `target/` tmp dir provided by Cargo's `CARGO_TARGET_TMPDIR`).

Public library surface at the end of the item (all in `splitters::cli`):

```rust
pub const LOCALIZATION_BASE: &str = "splitters.cli";
pub const DEFAULT_MANIFEST_DIR: &str = "splitters";

#[derive(Debug, clap::Parser)]
pub struct Cli { pub command: Command }

#[derive(Debug, clap::Subcommand)]
pub enum Command { Map(MapArgs), Validate(ValidateArgs), Extract(ExtractArgs) }

pub struct MapArgs { pub base: Option<String>, pub manifest_dir: Utf8PathBuf }
pub struct ValidateArgs {
    pub proposal: String,
    pub manifest_dir: Utf8PathBuf,
    pub base: Option<String>,
    pub check: Option<String>,
}
pub struct ExtractArgs {
    pub proposal: String,
    pub manifest_dir: Utf8PathBuf,
    pub base: Option<String>,
    pub branch: Option<String>,
    pub publish: Option<bool>,
}

/// Builds the localized command tree.
pub fn command(localizer: &dyn ortho_config::Localizer) -> clap::Command;

/// Parses an argument vector (including the program name).
pub fn parse_from<I, T>(args: I, localizer: &dyn ortho_config::Localizer)
    -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;

pub mod localization {
    pub const CATALOGUE: &str;
    pub fn help_localizer()
        -> Result<ortho_config::FluentLocalizer, ortho_config::FluentLocalizerError>;
}

pub mod dispatch {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CommandName { Map, Validate, Extract }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CommandOutcome { NotYetImplemented(CommandName) }
    pub const fn dispatch(cli: &Cli) -> CommandOutcome;
}
```

The three argument structs derive `Debug`, `Clone`, `PartialEq`, `Eq`,
`clap::Parser`, `serde::Serialize`, `serde::Deserialize`, and
`ortho_config::OrthoConfig`; `Default` agrees with `DEFAULT_MANIFEST_DIR`.
`CommandName` implements `Display` as the lower-case subcommand name, which is
the only place the diagnostic text is assembled. Whether `FluentLocalizerError`
is boxed in any future error type is 1.1.2's decision; this item returns it
only from `help_localizer`.

Reuse policy:

- `splitters::cli` owns all command-line declarations. Later roadmap items add
  fields to the existing structs; they must not add a parallel options struct
  or read arguments elsewhere.
- `cli::command` is the only constructor of the localized command tree; tests,
  the binary, and any future documentation generator call it rather than
  `Cli::command()` directly, so localization is never skipped.
- `tests/support/doc_examples.rs` is the shared tested-example loader for
  every user-facing Markdown document. Integration tests and BDD steps include
  it with `#[path = "support/doc_examples.rs"] mod doc_examples;`; no test may
  copy example text into a fixture. Its scope is extraction and validation of
  marked fences; comparison logic stays in the calling test.

## Revision note

- 2026-11-20: initial draft from Wyvern reconnaissance and the planning probe.
