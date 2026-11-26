# 1.1.1 Replace the stub binary with the `map`, `validate`, and `extract` command skeletons

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: COMPLETE (implemented 2026-11-26; PR delivery continues in
pr-babysitting)

Branch: `1-1-1-replace-the-stub-binary-with-command-skeletons`

Roadmap item: `1.1.1` in `docs/roadmap.md`.

## Purpose / big picture

Splitters is a planned Rust command-line tool that splits one Git branch into
smaller, independently reviewable branches. Its design
(`docs/splitters-design.md`) defines three commands: `map` decomposes a branch
into fragments and writes a manifest directory, `validate` proves that one
proposal (a named set of fragments) can be replayed onto the base and removed
from the current branch, and `extract` performs that split for real.

Today the binary is a stub. It prints a greeting and ignores every argument:

```plaintext
$ splitters --help
Hello from Splitters!
```

After this work, `splitters` has a real, typed command surface. The command
line is declared with `clap` (the de facto Rust argument parser) and
`ortho_config` v0.11.0 (OrthoConfig, a layered-configuration library from
`leynos/ortho-config` that derives configuration-file keys, environment
variables, and defaults from the same Rust struct that declares the
command-line flags). Help text is resolved through a Fluent message catalogue
(Fluent is Mozilla's localization format, which OrthoConfig uses), so it can be
translated without touching the declarations. A user can observe:

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

The exact wording is fixed by the catalogue written in `EP-M2`, and the exact
rendered help is recorded once, in `docs/users-guide.md`, where a test compares
it with the binary. `splitters map --help` lists `--base` and `--manifest-dir`;
`splitters validate --help` lists the `<PROPOSAL>` operand, `--manifest-dir`,
`--base`, and `--check`; `splitters extract --help` lists the `<PROPOSAL>`
operand, `--manifest-dir`, `--base`, `--branch`, and `--publish`.

A malformed invocation such as `splitters validate` (missing proposal) prints a
usage error on standard error and exits with status `2`. A well-formed
invocation such as `splitters validate add-parser` parses and then reports
`splitters validate: not implemented yet` on standard error, exiting with status
`2` (the design's "operational failure" class). No command reads or writes the
repository, the file system, or configuration in this item.

This work closes roadmap item 1.1.1: `splitters --help` documents the full
command surface, parsing has snapshot and behaviour coverage, and each option
has a single declared source. It deliberately leaves configuration discovery
and precedence to 1.1.4, the domain error hierarchy and final exit-code mapping
to 1.1.2, and repository preflight to 1.1.3.

## Constraints

- Implementation must not begin until the user explicitly approves this plan.
- The command surface is declared with derives only. The root `Cli` derives
  `clap::Parser`; the `Commands` enum derives `clap::Subcommand`; each of
  `MapArgs`, `ValidateArgs`, and `ExtractArgs` derives `clap::Parser`,
  `serde::Serialize`, `serde::Deserialize`, and `ortho_config::OrthoConfig`.
  There is no hand-rolled argument parsing and no configuration struct separate
  from these declarations (`docs/splitters-design.md` §5 and §12;
  `docs/developers-guide.md` §3.1).
- The root parser stays a plain `clap` type with no `OrthoConfig` derive and no
  options of its own beyond clap's generated `--help` and `--version`
  (`docs/developers-guide.md` §3.1).
- No command performs repository discovery, cleanliness checks, manifest I/O,
  Git operations, file-system writes, or subprocess spawning.
- No configuration discovery or merging is wired: `load`, `load_and_merge*`,
  `SubcmdConfigMerge`, `SelectedSubcommandMerge`, and `ConfigDiscovery` are not
  called. The root-level `T::load()` that the `OrthoConfig` derive generates on
  each struct must never be called, because it would read un-namespaced keys
  such as `SPLITTERS_BASE` and become a second configuration surface.
- Production code must not call `std::env::var`, `std::env::var_os`,
  `std::env::set_var`, `std::env::remove_var`, or `std::env::set_current_dir`.
  `std::env::args_os()` is read once, in `src/main.rs`, and passed to the
  library. The locale is not detected from `LANG` or `LC_ALL` in this item.
- Help and version output go to standard output (clap's contract for display
  requests); every other diagnostic goes to standard error. `print_stdout` and
  `print_stderr` stay denied, so output uses explicit `std::io::Write` handles
  or clap's `Error::exit`.
- Code files stay under 400 lines; every module starts with `//!`; public items
  carry `///` documentation; every `assert!`-family call carries a message;
  `expect` appears only in tests; lints are silenced only by item-scoped
  `#[expect(..., reason = "...")]` as a last resort (`AGENTS.md`). Clippy
  thresholds in `clippy.toml` apply (cognitive complexity 9, 4 arguments, 70
  lines per function, nesting depth 4). `self_named_module_files` is denied, so
  directories use `mod.rs`.
- Tests never mutate the process environment or working directory. End-to-end
  tests configure only the child process (`Command::env_clear`, `env`,
  `current_dir`), which is the sole exemption in `AGENTS.md`.
- The binary stays named `splitters`; `[profile.dev]` keeps
  `incremental = false` and `debug = 0` with no `codegen-backend`.
- Dependencies use caret requirements; only the crates listed in
  `Interfaces and dependencies` may be added.
- Comments, documentation, and catalogue text use en-GB-oxendict spelling.
- No compatibility machinery: `greeting()` is deleted, because the crate is
  pre-1.0 and has no external consumers.

## Tolerances (exception triggers)

Comparison base: `dc5c0b7`. Counting command, from the repository root:

```bash
git diff --numstat dc5c0b7 -- . \
  ':(exclude)docs/execplans/' ':(exclude)Cargo.lock' \
  ':(exclude)**/snapshots/**'
```

Classification by whole-file rule:

- Code: `src/**`, `tests/**/*.rs`, `tests/features/**`, and
  `locales/**/*.ftl` (shipped user-facing program text, reviewed like code).
- Configuration: `Cargo.toml`, `Makefile`, `.github/**`, `clippy.toml`.
- Documentation: `docs/**/*.md` except `docs/execplans/`, and `README.md`.
- Excluded generated artefacts: `Cargo.lock` and insta `snapshots/*.snap`
  files. This ExecPlan is excluded from every count.

Budgets (net lines, each enforced independently):

- Files: stop and escalate if more than 32 non-ExecPlan, non-generated files
  change. About 26 are planned. The plan-base comparison now measures 35, and
  D14 records a 36-file budget for it; the feature delta is 32.
- Code: stop and escalate above 2,600 net lines (revised from 1,700, then 2,300
  and 2,400 during the review rounds; see D14). The measured change is 2,520.
  Estimate about 1,300: production 350 (`src/lib.rs`, `src/main.rs`, `src/cli/`
  with the argument modules, localization, and dispatch), catalogue 35, and
  tests 900 (unit tables and the surface snapshot driver, the end-to-end and
  BDD (behaviour-driven development) harness with its sandbox fixture, and the
  shared tested-example loader with its own tests). The test estimate follows
  the review's comparison with OrthoConfig's `hello_world` example, whose
  localized-help tests alone are 472 lines.
- Configuration: stop and escalate above 60 net lines (estimate 35 in
  `Cargo.toml`).
- Documentation: stop and escalate above 550 net lines. Estimate about 420:
  `docs/users-guide.md` (160), `docs/adrs/adr-0001-…` (120), design Tables 3–5
  and §7 (30), `docs/developers-guide.md` (80), `docs/repository-layout.md` and
  `docs/contents.md` (25), roadmap tick (1).
- Interface: the public library surface is exactly the list in
  `Interfaces and dependencies`; anything more requires escalation.
- Dependencies: any crate beyond the authorized list, or another major version
  of a listed crate, requires escalation.
- Architecture: an `OrthoConfig` derive on the root, any OrthoConfig loader
  call, any process-environment read in production code, or any Git or
  file-system access requires escalation.
- CI cost: record the `build-test` job duration of the latest green `main` run
  before `EP-M1` and of the branch after `EP-M3`
  (`gh run list --workflow ci.yml --branch <branch> --json databaseId,createdAt,updatedAt`);
  escalate if it grows by more than 5 minutes.
- Iterations: stop and escalate if a focused test still fails after 4 fix
  attempts.
- Time: stop and escalate if a milestone takes more than 4 hours of active
  work.
- Ambiguity: decisions D3–D7 and D12 are proposals that the user may reject;
  revise this plan before implementing if they do.

Historical evidence: `docs/execplans/` did not exist before this plan, so no
earlier budgets or revisions are available. The initial draft's budgets were
revised once during design review (code 1,500 to 1,700 after re-estimating
tests; documentation 450 to 550 after adding the ADR; files 30 to 32); that
revision is recorded in `Revision note`.

## Risks

- Risk: OrthoConfig's derive might generate items that violate the
  repository's lint set, or Whitaker might reject them. Severity: medium.
  Likelihood: low. Mitigation: the planning probe compiled the exact derive
  pattern with the repository's `[lints]` table and `clippy.toml`, and Clippy
  passed with `-D warnings`. Whitaker was not run on the probe, so `EP-M1` runs
  `make lint` before building on the declarations.
- Risk: Fluent wraps interpolated values in Unicode isolation marks (U+2068 and
  U+2069), as in `Missing required argument: ⁨<PROPOSAL>⁩`. Severity: low.
  Likelihood: certain (observed). Mitigation: error output is checked
  structurally (exit status, empty standard output, standard error containing
  the operand name after removing the two marks), never by exact snapshot. Help
  output has no interpolation and is unaffected.
- Risk: help layout depends on terminal width, colour, and dependency versions,
  and the grouped daily Dependabot bumps could break exact-text tests.
  Severity: medium. Likelihood: medium. Mitigation: `localized_command` fixes
  the width with `Command::term_width(100)`, so a future dev-dependency
  enabling clap's `wrap_help` feature cannot make test help differ from release
  help; end-to-end tests clear the child environment; each rendered help text
  is pinned in exactly one place (the users' guide); and the developers' guide
  documents the update procedure.
- Risk: the end-to-end suite becomes host-dependent once 1.1.4 adds
  configuration discovery (a developer's `SPLITTERS_*` variables or
  `~/.config/splitters` would leak in), or coverage instrumentation breaks it
  (`env_clear` drops `LLVM_PROFILE_FILE`, so the instrumented binary writes
  `default_*.profraw` into the temporary directory). Severity: medium.
  Likelihood: high without mitigation. Mitigation: one `splitters_cmd` fixture
  builds every child command with `env_clear`, an allowlist (`PATH`,
  `LLVM_PROFILE_FILE`, and `CARGO_LLVM_COV*`), `HOME` and `XDG_CONFIG_HOME`
  pointing at a sandbox directory, and `GIT_CEILING_DIRECTORIES` set to the
  sandbox's parent.
- Risk: `rstest-bdd` 0.6.0 generates code against the caller's `rstest`
  (0.27.0 here; a Dependabot branch proposes 0.28.0). Severity: low.
  Likelihood: low. Mitigation: `EP-M3` compiles the BDD harness first; on
  incompatibility, escalate with the option of adopting 0.28.0.
- Risk: the development-dependency closure (rstest-bdd brings in Tokio,
  `gherkin`, and `i18n-embed`; googletest brings in `regex`) lengthens CI.
  Severity: low. Likelihood: medium. Mitigation: the CI-cost tolerance above.
- Risk: OrthoConfig's documentation metadata misdescribes subcommand fields
  (see `Surprises & discoveries`), so later generated documentation could
  advertise a `--proposal` flag and un-namespaced environment variables.
  Severity: low for this item. Likelihood: certain. Mitigation: this item does
  not consume that metadata; the issue is recorded for 1.1.4 and 5.2.2 and
  upstream as
  [leynos/ortho-config#717](https://github.com/leynos/ortho-config/issues/717).

## Progress

- [x] (2026-11-20 12:00Z) Branch renamed; OrthoConfig guide refreshed from the
  `v0.11.0` tag (`742ed58`); design and developers' guide references updated
  (`dc5c0b7`).
- [x] (2026-11-20 12:40Z) Reconnaissance (Wyvern team) and feasibility probe
  completed; initial draft committed (`8019edf`).
- [x] (2026-11-20 13:40Z) Expert design review (structure and contracts;
  failure modes and test cost; alternatives and viability) applied; second
  probe of documentation metadata completed.
- [x] (2026-11-25) Plan approved by the user; implementation started.
- [x] (2026-11-26) `EP-M1` typed command declarations parse in-process; the
  focused suite reports 31 passed, 1 skipped, and the surface snapshot is
  reviewed and accepted.
- [x] (2026-11-26) `EP-M2` help text is localized through the en-GB catalogue;
  the focused suite reports 40 passed, 1 skipped, and the re-reviewed surface
  snapshot carries the catalogue text.
- [x] (2026-11-26) `EP-M3` the `splitters` binary uses the new surface end to
  end; the focused `cli` suite reports 32 passed, 0 skipped, after the red run
  against the stub reported 9 passed and 11 failed.
- [x] (2026-11-26) `EP-M4` documentation, tested guide examples, and roadmap
  closure; the `cli` target reports 45 passed, 0 skipped, and markdownlint and
  nixie pass.

## Surprises & discoveries

- Observation: the vendored OrthoConfig guide described v0.10.0, while the
  roadmap item mandates v0.11.0. Evidence: `docs/ortho-config-users-guide.md`
  line 8 at `4c82d66`. Impact: refreshed from the `v0.11.0` tag before planning
  (`742ed58`).
- Observation: localized clap errors contain Unicode isolation marks around
  interpolated values, and OrthoConfig v0.11.0 has no option to disable them.
  Evidence: probe output; no `isolating` identifier in the
  `ortho_config-0.11.0` or `ortho_l10n-0.11.0` sources. Impact: structural
  error assertions (see `Risks`); raised upstream as
  [leynos/ortho-config#718](https://github.com/leynos/ortho-config/issues/718).
- Observation: OrthoConfig's built-in clap-error catalogue ships `en-US` and
  `ja`; a `FluentLocalizer` built for `en-GB` still localizes clap errors,
  because `ortho_l10n` maps any `en` locale to its en-US catalogue. Evidence:
  probe with `langid!("en-GB")`; `ortho_l10n-0.11.0/locales/`. Impact: an en-GB
  consumer catalogue works at run time, but clap errors are spelt in US English
  beside British help text (accepted; see D7).
- Observation: a `#[derive(Parser)]` struct without `#[command(name = ...)]`
  takes the package name, and OrthoConfig derives the `[cmds.<name>]` table and
  `SPLITTERS_CMDS_<NAME>_*` variables from the struct's own command name, not
  from the enum variant. Evidence: `ortho_config-0.11.0/src/subcommand/mod.rs`
  lines 122–129 (reported by the structure review). Impact: each struct must
  carry an explicit `#[command(name = "...")]`, and a test pins it (D1,
  `INV-surface`).
- Observation: `googletest`'s `unordered_elements_are!` requires a statically
  known matcher list, while the expected identifier sets are table data.
  Evidence: the macro expands `$($matcher:expr),*` into a fixed-arity tuple.
  Impact: `INV-single-declaration` compares the two runtime `BTreeSet` values
  with `container_eq`, which is order-independent and names missing and
  unexpected elements. The obligation and its non-vacuity checks are unchanged.
- Observation: clap renders positionals under `Arguments:` and options under
  `Options:`. Evidence: `splitters validate --help` places `<PROPOSAL>` in an
  `Arguments:` section. Impact: the BDD step that checks an entry selects the
  section from the entry's spelling, so the planned `<PROPOSAL>` row is checked
  where clap prints it.
- Observation: once the root has a `long_about`, clap renders the root's help
  flag as `Print help (see more with '--help')`, because `-h` and `--help` now
  differ. Evidence: the EP-M2 surface snapshot. Impact: none for the contract;
  the users' guide records the rendered text the binary actually produces.
- Observation: clap renders `--publish` as `--publish[=<BOOL>]`, so the BDD row
  matcher accepts a bracketed suffix after the option name while still rejecting
  `--base-ref` for `--base`. Evidence: the EP-M3 focused run.
- Observation: clap appends `[possible values: true, false]` to a boolean's
  help, which made the rendered `--publish` line 128 columns and so
  unrecordable under the repository's 120-column code-block limit. Impact:
  `publish` sets `hide_possible_values = true`; the listing was redundant for a
  boolean documented by the boolean model, and the line is now 99 columns.
- Observation: `rstest-bdd` 0.6.0's Scenario Outline expansion adds a hidden
  case-index parameter, taking the scenario function to five arguments and
  tripping the repository's four-argument limit on generated code. Evidence:
  `clippy::too_many_arguments` at the outline binding. Impact: the feature uses
  six explicit scenarios named for the command and option; the coverage is
  unchanged.
- Observation: CodeRabbit's completeness assessment found the public interface
  differed from the plan: the argument structs sat behind a public `args`
  module and `CommandName::as_str` was public. Evidence: the assessment at head
  `578739e`. Impact: `cli` re-exports `MapArgs`, `ValidateArgs`, and
  `ExtractArgs`, `args` became private, and `as_str` is now `pub(crate)`, so
  the surface is exactly the plan's list.
- Observation: the plan's seeded `--branch` trimming fault does not
  discriminate, because the witness has no leading or trailing whitespace.
  Evidence: the seeded trimming parser left `option_values_are_stored_verbatim`
  green. Impact: the recorded fault normalizes `/` to `-`, which fails the
  witness, and `Artefacts and notes` records both results.
- Observation: the users' guide records clap's next-line option layout for the
  root `--help`, where `-h, --help` and its description occupy separate lines.
  Evidence: the guide's `splitters-help` example. Impact: the guide test
  compares that layout exactly, so a clap layout change shows up as a reviewed
  guide diff rather than a silent test change.
- Observation: `rstest-bdd-macros` 0.6.0's `ScenarioState` derive does not
  generate `Default` (the vendored guide says it does). Evidence: compile error
  `SurfaceState: Default is not satisfied`. Impact: the state struct derives
  `Default` explicitly, and the fixture constructs `Slot::new()` in a statement
  so rstest's re-emitted body stays a non-trivial block.
- Observation: the red `EP-M1` run failed with 13 compile errors
  (`E0432`/`E0433`/`E0425` unresolved command-surface items, plus `E0599` and
  `E0382` in the new tests). Impact: the two test-authoring defects were fixed
  before the green run; the rest were the intended missing declarations.
- Observation: OrthoConfig's documentation metadata (`OrthoConfigDocs` and
  `OrthoConfigSubcommandDocs`) describes subcommand fields from the OrthoConfig
  attributes rather than the clap declaration. For `ValidateArgs` it reports
  the positional operand as `--proposal`, drops value names, and gives
  environment variables `SPLITTERS_PROPOSAL` and `SPLITTERS_CHECK` instead of
  `SPLITTERS_CMDS_VALIDATE_*`; through the enum derive it reports
  `app_name = "validate"` and `about_id = "validate-about"`, losing the
  localization base. Evidence: second planning probe (see
  `Artefacts and notes`). Impact: this item does not derive or consume
  documentation metadata; 1.1.4 and 5.2.2 must not rely on it for subcommands
  until upstream fixes it
  ([leynos/ortho-config#717](https://github.com/leynos/ortho-config/issues/717)).

## Decision log

- D1. Decision: each argument struct derives
  `clap::Parser + serde::Serialize + serde::Deserialize + ortho_config::OrthoConfig`
  and carries both `#[command(name = "<command>")]` and
  `#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.<command>")]`;
  each `Commands` variant also carries `#[command(name = "<command>")]`. The
  root `Cli` derives only `clap::Parser`, and `Commands` derives only
  `clap::Subcommand`. Rationale: the developers' guide (§3.1) calls for a
  plain-clap root, and the design (§5, §12) for OrthoConfig on each subcommand
  struct. OrthoConfig's own `hello_world` example differs (its root derives
  `OrthoConfig` and its enum derives `SelectedSubcommandMerge` and
  `OrthoConfigSubcommandDocs`); those derives are deferred because nothing in
  this item consumes them, adding them later is purely additive, and the
  documentation derive currently produces misleading metadata. A plain-clap
  root also means no `[package.metadata.ortho_config] root_type` for
  `cargo orthohelp` yet. The explicit command names are required because
  OrthoConfig namespaces subcommand configuration by the struct's own command
  name. Date/Author: 2026-11-20, planning agent (revised after structure
  review).
- D2. Decision: each struct's `localization_base` is the root base
  `splitters.cli` plus the command name, and a test asserts the agreement.
  Rationale: the probe showed that `LocalizeCmd::localize` looks up
  `splitters-cli-map-args-base-help` for `map --base`, which equals the
  identifier in `MapArgs::ARG_IDS` only under that base. A mismatch silently
  falls back to clap's text, so a test, not review, must catch it. Date/Author:
  2026-11-20, planning agent.
- D3. Decision (proposed; needs user confirmation): spellings are `--base <REF>`
  and `--manifest-dir <DIR>` on all three commands; a required positional
  `<PROPOSAL>` operand on `validate` and `extract`; `--check <COMMAND>` on
  `validate` only; `--branch <NAME>` and `--publish[=<BOOL>]` on `extract`
  only. Field names are `base`, `manifest_dir`, `proposal`, `check`, `branch`,
  and `publish`. Rationale: design Tables 3–5 assign inputs per command; a
  field on a command that does not use it would create a meaningless
  configuration key. `--base` is the conventional spelling and keeps the key
  short; `--base-ref` was rejected because it would be confused with the
  manifest's recorded `base_ref`. The proposal names the object acted upon, so
  it is an operand with a clear usage error when missing. Because OrthoConfig
  has no attribute for "command line only", the derive still generates a
  configuration key and environment variable for `proposal` (and for `branch`);
  they are harmless because the required operand always wins, and 1.1.4 decides
  whether to document or suppress them. `map`'s base is optional at parse time
  so that 1.1.4 can source it from configuration; whether a missing base is an
  error is roadmap 2.1.1's decision. Date/Author: 2026-11-20, planning agent.
- D4. Decision (proposed; revised after review): `manifest_dir` is
  `Option<Utf8PathBuf>` with no default. Rationale: the draft's default of
  `splitters` was withdrawn. Design §8 shows `splitters/` as an example layout,
  not a default; a relative default inside the work tree creates an untracked
  directory that 1.1.3's cleanliness check could then reject; and adding a
  default later is compatible whereas removing one is breaking. Roadmap 2.1.3
  (which writes manifests) owns any default. This also removes
  `cli_default_as_absent` and a duplicated default constant. Date/Author:
  2026-11-20, planning agent.
- D5. Decision (proposed): `publish` is `Option<bool>` with `num_args = 0..=1`,
  `require_equals = true`, and `default_missing_value = "true"`, accepting
  `--publish`, `--publish=true`, and `--publish=false`, and rejecting
  `--publish false`. Rationale: this is the boolean model in the OrthoConfig
  guide ("Boolean flag spellings", Table 4): omission defers to lower layers and
  `--publish=false` overrides a configured `true`. The `ArgAction::SetTrue`
  pattern from `hello_world` was rejected because a configured `true` could
  then never be switched off on the command line. The probe confirmed all four
  spellings. Date/Author: 2026-11-20, planning agent.
- D6. Decision (proposed): `check` is `Option<String>`; execution semantics are
  not decided here. Rationale: the design writes `--check <cmd>`. The stored
  configuration format, not the Rust type, is the real contract: if 3.2.1 later
  wants an argument vector (preferred by `AGENTS.md`'s "structured APIs instead
  of executable string construction"), changing a string key into an array is a
  breaking configuration change. Roadmap 3.2.1 must therefore settle
  shell-string versus argument-vector before 1.1.4 documents the key. `--check`
  on `extract` (which "reruns the full validation sequence") is left to 3.2.1
  or 4.1.1 as an additive change. Date/Author: 2026-11-20, planning agent.
- D7. Decision (proposed): ship one consumer catalogue,
  `locales/en-GB/messages.ftl`, embedded with `include_str!`, and always build
  the localizer for `en-GB`; no locale detection; no `NoOpLocalizer` fallback.
  If the embedded catalogue fails to build at run time, `main` reports the
  failure on standard error and exits `2`. Rationale: the user asked for
  localized help support, meaning help text flows through a catalogue and can
  be translated; locale detection needs an injected environment, which belongs
  with 1.1.4. `en-GB` matches the project's writing standard. A fallback was
  rejected after review: the catalogue is fixed at compile time and a unit test
  proves it builds, so a run-time failure is a code defect, and a fallback
  warning would break the stream contract for `--help`. Known limitation:
  `cargo orthohelp` takes agent-context summaries only from `en-US` resources,
  and clap errors use OrthoConfig's en-US catalogue; roadmap 5.2.2 must choose
  between `--locale en-GB` with `locales = ["en-GB"]` metadata and an upstream
  fix
  ([leynos/ortho-config#719](https://github.com/leynos/ortho-config/issues/719)).
  Date/Author: 2026-11-20, planning agent (revised after review).
- D8. Decision: parsed commands dispatch to
  `CommandOutcome::NotYetImplemented(CommandName)`; the binary writes
  `splitters <command>: not implemented yet` to standard error and exits `2`.
  Display requests and usage errors are rendered and exited by clap's
  `Error::exit` (status `0` on standard output for display requests, `2` on
  standard error otherwise; broken pipes are ignored). Rationale: exiting `0`
  for a command that did nothing would mislead automation, and `2` is the
  design's operational-failure class. Keeping `Error::exit` in `main` preserves
  clap's colour and terminal handling; the in-library `run(args, out, err)`
  suggested in review was rejected because it would lose that handling, and the
  stream contract is owned by the executable, where the end-to-end tests check
  it. The final error hierarchy and the remapping of usage errors to the
  design's status `1` belong to 1.1.2. Date/Author: 2026-11-20, planning agent.
- D9. Decision (revised after review): the Fluent catalogue is the only owner
  of user-facing help text. `///` comments on fields and structs describe the
  Rust semantics for developers (for example "`None` defers to lower
  configuration layers") and must not restate the help text. Rationale: with
  the catalogue always loaded, doc-comment help is never shown, so keeping two
  copies of the same prose would drift silently. The catalogue tests prove
  every identifier exists and none is stale. Date/Author: 2026-11-20, planning
  agent.
- D10. Decision (revised after review): no Verus proof, Kani harness, model
  checker, or `proptest` property is planned. Verification is by `rstest`
  tables (including fixed-witness value cases), one `insta` snapshot of the
  structured command surface, `rstest-bdd` scenarios, `assert_cmd` end-to-end
  tests, and tested users'-guide examples. Rationale: see
  `Formal verification decision`. The draft's verbatim-values property tested
  clap behaviour that the plan already treats as an axiom; fixed witnesses plus
  a seeded fault give the same protection. Date/Author: 2026-11-20, planning
  agent.
- D11. Decision (revised after review): record the help-text and locale policy
  and the declaration conventions (D1, D2, D5, D7, D9) in
  `docs/adrs/adr-0001-command-line-declaration-and-help-text-conventions.md`,
  and add a "Flag" column to design Tables 3–5 instead of a new design
  subsection. Rationale: the policy binds every later roadmap item and cannot
  be guessed by a contributor, which is what `AGENTS.md` reserves ADRs for; the
  option spellings belong next to the inputs they spell, so a reader has one
  place to look. Date/Author: 2026-11-20, planning agent.
- D12. Decision (proposed): shared inputs (`base`, `manifest_dir`) are declared
  separately on each command, so 1.1.4 will expose them per command
  (`[cmds.map] manifest_dir`, `[cmds.validate] manifest_dir`, …). Rationale:
  lifting them to root-level globals would need an `OrthoConfig` derive on the
  root and an amendment to developers' guide §3.1, which this item may not
  make. The duplication is recorded as an open question for 1.1.4, which owns
  configuration layering and may propose globals. Date/Author: 2026-11-20,
  planning agent.
- D14. Decision (tolerance outcome, 2026-11-26): the budget is revised to
  2,600 net code lines rather than trimming coverage. The plan estimated 1,700
  and the estimate under-counted the verification the acceptance criteria
  require: the BDD harness, the tested-example loader with its CommonMark fence
  rules, the sandbox comparison with entry kinds, and the regression tests the
  review rounds demanded. At the final head the plan-base comparison measures
  35 files, 2,520 net code lines, 20 configuration lines, and 425 documentation
  lines against `dc5c0b7`. Four of those files (`clippy.toml`, `dylint.toml`,
  `.github/workflows/ci.yml`, and `docs/documentation-style-guide.md`) changed
  only on `main`, so the comparison against the assessed PR base is 32 files.
  The 32-file budget applies to the feature delta; the plan-base comparison
  needs a 36-file budget. Configuration and documentation stay within their
  budgets. Upstream changes and required coverage are not removed to fit a
  budget. Escalation was raised with the user, and each revision is recorded
  here rather than applied silently. Earlier measurements at intermediate heads
  were 2,163 lines (before the review repairs) and 2,363 (after the first
  review round); the revisions to 2,300 and 2,400 are superseded by this one.
- D13. Decision: the users' guide is the single exact copy of rendered help.
  Each documented example begins with a `$ splitters …` command line; the guide
  test runs that command and compares its output with the fence, and fails on
  any marked example it does not handle. `insta` pins the structured command
  surface instead, and `--version` is compared with
  `env!("CARGO_PKG_VERSION")`. Rationale: the review found the draft pinned
  root help in four places, so every clap or catalogue change would have needed
  four coordinated edits. The tested-example loader is required by
  `docs/documentation-style-guide.md` for any fenced users'-guide example, and
  the user asked for the users' guide to be updated; it is scoped to what this
  guide needs. Date/Author: 2026-11-20, planning agent.

## Outcomes & retrospective

- `EP-M1` (2026-11-26): `cli::args` declares `MapArgs`, `ValidateArgs`, and
  `ExtractArgs` with the field model, explicit command names, and per-command
  localization bases; `cli::localized_command` and `cli::try_parse_from`
  localize the tree at a fixed width and preserve `ArgMatches`. The focused
  suite passed (31 passed, 1 skipped), the surface snapshot was reviewed
  against design Tables 3–5 and D3–D5 before acceptance, and the two
  test-authoring defects found in the red run were fixed.
- Second review round (2026-11-26): CodeRabbit's re-assessment of `c158a56`
  found that the sandbox recorded paths but not entry kinds, that fence
  detection stripped every leading space instead of CommonMark's three, that
  four assertions lacked diagnostics, and that `src/main.rs` needed direct
  module links. Fixed: snapshots now record each entry's kind with a
  directory-to-file regression test; fences allow at most three leading spaces
  with opening and closing regression tests, a closer may be followed only by
  spaces or tabs, and a mismatch is returned before the scan probe writes
  anything (with a probe-collision regression test); every assertion in this
  change carries a message; and the binary links `splitters::cli` and
  `splitters::cli::dispatch`. The code budget moved to 2,400 for the 2,363
  measured lines.
- Post-implementation review (2026-11-26): CodeRabbit's formal review of
  `fb0042b` found four in-scope harness weaknesses and one module-documentation
  check. Fixed: the detached-boolean case now requires an unknown-argument
  diagnostic and rejects the interim dispatch report (end-to-end and BDD); the
  tested-example loader recognises backtick and tilde fences and validates a
  closing fence against its opener, with regression cases; `assert_untouched`
  compares the post-command snapshot before writing its probe, so a
  child-created entry cannot be masked; and `src/main.rs` plus the three
  argument modules carry complete module documentation. Three findings against
  `docs/ortho-config-users-guide.md` were dispositioned out of scope: that file
  is vendored verbatim from the OrthoConfig `v0.11.0` tag apart from absolute
  link targets, so correcting upstream prose belongs upstream.
- Open questions for later items, recorded here as EP-M4 requires: 1.1.4 owns
  whether shared inputs become globals (D12), how the generated keys for
  `proposal` and `branch` are treated (D3), and whether risky fields such as
  `check` and `publish` may come from a repository-committed file; 3.2.1 owns
  the stored format of `--check` (D6); 5.2.2 owns the `cargo orthohelp` locale
  limitation (D7) and the documentation-metadata issue in
  `Surprises & discoveries`.
- `EP-M4` (2026-11-26): `docs/users-guide.md` records the rendered help as
  four tested examples, the option tables, the interim exit status, and the
  configuration gap; ADR-0001 records the declaration and help-text
  conventions; design Tables 3–5 gained a Flag column; the developer's guide,
  repository layout, and documentation contents describe the new files; and
  roadmap item 1.1.1 is ticked. The guide's tested-example loader rejects an
  unmarked fence, an unterminated fence, a missing identifier, and a duplicate
  identifier. The `cli` target passed 45 tests, markdownlint and nixie passed,
  and all three commit gates passed.
- `EP-M3` (2026-11-26): `cli::dispatch` names the selected command and reports
  the interim outcome, and `main` builds the en-GB localizer, parses
  `std::env::args_os()` once, lets clap exit display requests and usage errors,
  and otherwise writes `splitters <command>: not implemented yet` to standard
  error with exit status `2`. The stub greeting is deleted. The `cli`
  integration target covers stream discipline, exit status, non-UTF-8 operands,
  and the no-effects sandbox, and the BDD scenarios cover the documented
  surface. The focused suite passed (32 passed, 0 skipped).
- `EP-M2` (2026-11-26): `locales/en-GB/messages.ftl` owns all help text and
  `cli::localization::help_localizer` builds the en-GB localizer with no
  fallback. The catalogue tests prove the base, about, and help identifiers
  agree with the derive, that the catalogue declares exactly the generated set
  and nothing stale, and that rendered help contains the catalogue text and no
  documentation text. The focused suite passed (40 passed, 1 skipped), and the
  snapshot was re-reviewed with the catalogue text before acceptance.
  `EP-M3`–`EP-M4` remain.

## Context and orientation

Inspected base revision: `dc5c0b7` on branch
`1-1-1-replace-the-stub-binary-with-command-skeletons`. Relocate symbols with
`grep -n`.

The repository is a single Cargo package named `splitters` (edition 2024, no
workspace), built with the pinned toolchain `nightly-2026-11-06`.
`.cargo/config.toml` enables the parallel front end (`-Zthreads=8`) and, on
Linux, the `mold` linker.

Current code:

- `src/lib.rs` (17 lines) exposes `pub const fn greeting() -> &'static str`
  and its unit test `tests::greeting_matches_stub_output`.
- `src/main.rs` (10 lines) prints `greeting()` under an
  `#[expect(clippy::print_stdout, ...)]`.
- `tests/build_standard_contract.rs`, `tests/build_standard_backend.rs`, and
  `tests/dev_build_footprint.rs` (helpers in `tests/build_standard/`) check
  build configuration only; none runs the binary. Their scratch files live under
  `CARGO_TARGET_TMPDIR`. `tests/build_standard_backend.rs` is about 548 lines,
  a pre-existing breach of the 400-line limit outside this item's scope.
- `Cargo.toml` has no runtime dependencies; development dependencies are
  `cap-std = "4.0.3"`, `rstest = "0.27.0"`, and `toml = "1.1.3"`. Its `[lints]`
  table denies, among others, `missing_docs`, `print_stdout`, `print_stderr`,
  `unwrap_used`, `expect_used`, `indexing_slicing`, `shadow_*`,
  `missing_const_for_fn`, `must_use_candidate`, `let_underscore_must_use`,
  `self_named_module_files`, and `result_large_err`.

Gates: `make check-fmt` (rustfmt check and `mdtablefix --check`), `make lint`
(`interrogate` over Python in `tests/`, `cargo doc` with `-D warnings`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`whitaker --all`), `make test`
(`cargo nextest run --all-targets --all-features`, then `cargo test --doc`),
`make markdownlint`, and `make nixie`. Installed on the planning host:
`cargo-nextest` 0.9.133, `whitaker`, `mold`, `mdtablefix`, `markdownlint-cli2`,
and `nixie`. CI (`.github/workflows/ci.yml`) runs `make check-fmt`,
markdownlint, `make lint`, `make test-workflow-contracts`, and a coverage
action that runs the tests; Dependabot opens grouped daily updates.

Terms:

- Subcommand: a verb after the program name (`map`, `validate`, `extract`).
- Operand: a positional argument, here the proposal identifier.
- Catalogue: a Fluent `.ftl` file mapping message identifiers to text.
- Display request: `--help`, `help <command>`, or `--version`, which clap
  reports as a `clap::Error` of kind `DisplayHelp` or `DisplayVersion`.
- `ArgMatches`: clap's record of which arguments the user actually supplied.
  1.1.4's match-aware merge needs it, so this item preserves it.
- Snapshot test: an `insta` assertion comparing output with a reviewed file
  in a `snapshots/` directory.

Budget survey: the change replaces 27 lines of stub code. OrthoConfig's
`hello_world` example (about 2,000 lines with tests, of which 472 are
localized-help tests) indicates the size of a three-command surface with
localization and full coverage. `docs/users-guide.md` and `docs/adrs/` do not
exist and are created here. No earlier ExecPlans exist.

Signposts. Read these before implementing:

- `docs/roadmap.md` §1.1 for the scope boundaries between 1.1.1 and 1.1.2–1.1.4.
- `docs/splitters-design.md` §5, §7 (Tables 3–5), §8, §12, and §13.
- `docs/ortho-config-users-guide.md`: "Give each subcommand its own
  settings", "Boolean flag spellings" (Table 4), "Localize help and errors",
  and "Use the generated message identifiers".
- `docs/developers-guide.md` §2 and §3.1.
- `docs/reliable-testing-in-rust-via-dependency-injection.md`,
  `docs/rust-testing-with-rstest-fixtures.md`, `docs/rstest-bdd-users-guide.md`
  (feature files, `#[scenario]`, step placeholders such as `{cmd:string}`),
  `docs/rust-doctest-dry-guide.md`, and
  `docs/complexity-antipatterns-and-refactoring-strategies.md`.
- `docs/documentation-style-guide.md`: "User's guide" (tested examples) and
  "Architectural decision records".
- Skills: `rust-router`, then `rust-types-and-apis` (declarations),
  `rust-errors` (dispatch outcome), `rust-unit-testing` (tables, googletest,
  pretty_assertions, insta), `domain-cli-and-daemons` (streams and exit status),
  `execplans`, `en-gb-oxendict`, and `codegraph-mcp`.

## Conformance basis

Upstream artefacts (at `dc5c0b7` unless stated): `docs/roadmap.md` item 1.1.1
(lines 23–34); `docs/splitters-design.md` §5, §6, §7 (Tables 3–5), §12, §13;
`docs/developers-guide.md` §2 and §3.1; `docs/ortho-config-users-guide.md`
imported from tag `v0.11.0` at `742ed58`; `AGENTS.md` and
`docs/documentation-style-guide.md` as governing standards. No Terms of
Reference or ADR exists; ADR-0001 is created by this item.

Requirements:

- `REQ-surface`: exactly the subcommands `map`, `validate`, and `extract`,
  each with the inputs of design Tables 3–5.
- `REQ-typed`: base override, manifest directory, proposal selection, branch
  naming, `--check`, and `--publish` are typed fields.
- `REQ-single-declaration`: every option is a field of an OrthoConfig-derived
  struct; no other configuration schema or parser exists.
- `REQ-help`: `splitters --help` documents the full command surface, and each
  command's help documents its options.
- `REQ-parse-coverage`: parsing has snapshot and behaviour coverage.
- `REQ-localized-help`: help text resolves through a Fluent catalogue via
  OrthoConfig's localizer.
- `REQ-streams`: help and version on standard output; other diagnostics on
  standard error.
- `REQ-no-effects`: no command touches the repository, the file system, or
  configuration.
- `REQ-merge-ready`: the declarations and parser output let 1.1.4 add
  match-aware merging without reshaping fields (preserved `ArgMatches`,
  explicit command names, `Default` on each struct).

Gaps at the base revision: `GAP-stub` (the binary ignores arguments),
`GAP-deps` (no `clap`, `ortho_config`, `serde`, or `camino`), `GAP-users-guide`
(no `docs/users-guide.md`), and `GAP-conventions` (design §7 names inputs but
no spellings, types, or help-text policy).

Trace links:

```plaintext
roadmap-1.1.1 -> REQ-surface, REQ-typed, REQ-single-declaration, REQ-merge-ready
  -> EP-M1 -> src/cli/tests/{declaration,parsing}.rs, src/cli/tests/snapshots/
roadmap-1.1.1 -> REQ-localized-help -> EP-M2 -> src/cli/tests/catalogue.rs
roadmap-1.1.1 -> REQ-help, REQ-parse-coverage, REQ-streams, REQ-no-effects
  -> EP-M3 -> tests/cli/{end_to_end,bdd}.rs, tests/features/command_surface.feature
GAP-users-guide, GAP-conventions -> EP-M4
  -> docs/users-guide.md, docs/adrs/adr-0001-…, tests/cli/guide_examples.rs
```

## Verification plan

### Abstract model

The command surface is a finite grammar. For an argument vector `A` (after the
program name), `parse : A -> Invocation + clap::Error`, where an `Invocation`
holds a `Cli` (one of `Map(MapArgs)`, `Validate(ValidateArgs)`, or
`Extract(ExtractArgs)`) and the `ArgMatches`.
`dispatch : Cli -> CommandOutcome` is total. The binary composes them: on a
display-request error it prints to standard output and exits `0`; on any other
error it prints to standard error and exits `2`; on success it writes the
not-implemented diagnostic to standard error and exits `2`. There is no state
and no ordering.

Field model:

- `map`: `base: Option<String>`, `manifest_dir: Option<Utf8PathBuf>`.
- `validate`: `proposal: String`, `manifest_dir: Option<Utf8PathBuf>`,
  `base: Option<String>`, `check: Option<String>`.
- `extract`: `proposal: String`, `manifest_dir: Option<Utf8PathBuf>`,
  `base: Option<String>`, `branch: Option<String>`, `publish: Option<bool>`.

### Invariants and lemmas

- `INV-surface` (`REQ-surface`, `REQ-typed`): the localized command tree has
  exactly the subcommands `map`, `validate`, and `extract` (plus clap's
  `help`); each subcommand's arguments match the field model, with the long
  names, value names, operand positions, `required`, `num_args`, and
  `require_equals` settings of D3 and D5; and each struct's own
  `command().get_name()` equals its subcommand name in the tree.
- `INV-single-declaration` (`REQ-single-declaration`): for each subcommand,
  the set of clap argument identifiers (excluding `help`) equals the set of
  `name` values in the struct's OrthoConfig-generated `ARG_IDS`, and the root
  declares no argument other than `help` and `version`.
- `INV-values-verbatim` (`REQ-typed`): fixed witnesses containing `/`, `=`,
  spaces, and non-ASCII text (`"feature/añadir = x"`) are stored unchanged in
  `base`, `branch`, `check`, `proposal`, and `manifest_dir`; an operand
  beginning with `-` is accepted after `--`.
- `INV-bool-spellings` (D5): `--publish` and `--publish=true` give
  `Some(true)`, `--publish=false` gives `Some(false)`, omission gives `None`,
  and `--publish false` and `--publish=maybe` are rejected.
- `INV-matches-preserved` (`REQ-merge-ready`): the returned `ArgMatches`
  records exactly the subcommand chosen and the arguments supplied (for example
  `matches.subcommand_matches("extract")` reports `branch` as present only when
  `--branch` was given).
- `INV-catalogue-ids-agree` (D2): for each struct `S` named `n`,
  `S::LOCALIZATION_BASE == "splitters.cli." + n`.
- `INV-catalogue-complete` (`REQ-localized-help`, D9): every `ABOUT_ID` and
  every `ARG_IDS[i].help_id` of the three structs, plus the root about and
  long-about identifiers, resolve in the en-GB localizer; and the catalogue
  declares no other identifiers (stale-entry check, with `.` normalized to `-`
  as OrthoConfig does).
- `INV-help-localized` (`REQ-localized-help`): rendered help for each command
  contains the catalogue text for that command and none of the doc-comment text.
- `INV-command-names-agree`: `CommandName`'s `Display` for each variant equals
  the clap subcommand name, so the diagnostic and the command line cannot
  diverge.
- `INV-streams` (`REQ-streams`, D8): display requests (`--help`, each
  `<command> --help`, `help validate`, `--version`) exit `0` with empty
  standard error; usage errors (missing proposal, unknown option,
  `--publish false`, bare `splitters`, a non-UTF-8 operand) and well-formed
  commands exit `2` with empty standard output. No input exits `101` (panic).
- `INV-no-effects` (`REQ-no-effects`): each well-formed command run in an
  empty sandbox directory outside any Git repository leaves the sandbox,
  including its `HOME` and `XDG_CONFIG_HOME` subdirectories, empty.
- `INV-guide-matches-binary` (`REQ-help`, D13): every tested example in
  `docs/users-guide.md` runs its documented command and the output equals the
  fence; every marked example is handled.

Lemma `L-dispatch-total`: `dispatch` maps each `Commands` variant to
`NotYetImplemented` naming that variant. With `INV-surface` it yields the
well-formed-command half of `INV-streams`. The exhaustive `match` and an
`rstest` table discharge it.

### Axioms (trusted, not verified here)

- `AXIOM-clap`: `clap` 4.6 derive parses as documented (including
  `num_args`, `require_equals`, `default_missing_value`, required positionals,
  rejection of invalid UTF-8 for `String` and `Utf8PathBuf` fields, and
  `term_width`); `Error::exit` prints display requests to standard output with
  status `0` and other errors to standard error with status `2`, ignoring print
  failures such as broken pipes.
- `AXIOM-ortho-derive`: `ortho_config` 0.11.0 generates `LOCALIZATION_BASE`,
  `ABOUT_ID`, and `ARG_IDS` as documented and rejects duplicate normalized
  argument identifiers at compile time.
- `AXIOM-ortho-localize`: `parse_localized_command` and
  `LocalizeCmd::with_base(..).localize(..)` look up `<base>-<command>-about` and
  `<base>-<command>-args-<arg>-help` and fall back to clap's text when an
  identifier is missing (observed in the probe). `INV-catalogue-ids-agree` and
  `INV-catalogue-complete` make the fallback unreachable for the shipped
  catalogue.
- `AXIOM-fluent-isolation`: Fluent inserts U+2068 and U+2069 around
  interpolated values.

### Methods per obligation

- Obligation: `INV-surface`.
  Method: one `insta` snapshot of a structured, deterministic description of
  the localized command tree (for each command: name, about text, and per
  argument: identifier, long name, value name, positional index, required,
  value arity, `require_equals`, and help text), rendered as indented plain
  text by a small test helper, plus an `rstest` table asserting each struct's
  `command().get_name()`. Rationale: the surface is multi-variant structured
  output; one reviewed snapshot pins it without pinning clap's layout. Artefact:
  `src/cli/tests/declaration.rs` and its `snapshots/` directory. Evidence:
  `cargo nextest run --lib -E 'test(/declaration/)'`. Red: the types do not
  compile and no snapshot exists. Green: snapshot reviewed and accepted. Run
  `cargo insta reject` after the red run so nothing stale can be accepted.
  Non-vacuity: the snapshot is reviewed against design Tables 3–5 and D3–D5
  before acceptance; seeded fault: removing `require_equals` from `publish`
  must change the snapshot.
- Obligation: `INV-single-declaration`.
  Method: `rstest` table over the three structs comparing the two identifier
  sets with `googletest`'s `unordered_elements_are!`, and a root test. Artefact:
  `src/cli/tests/declaration.rs`. Non-vacuity: both sets come from independent
  sources (clap's tree and OrthoConfig's derive) and are asserted non-empty.
  Seeded faults, each reverted: a root-level `#[arg(long)] verbose: bool` on
  `Cli` must fail the root test naming `verbose`; a `#[ortho_config(skip_cli)]`
  field with `#[arg(long)]` on `MapArgs` must fail the equality naming the
  field. Before relying on the second fault, confirm at implementation time
  that it compiles and appears in clap's argument list; if it does not, record
  that in `Surprises & discoveries` and substitute a `#[command(flatten)]`
  group of clap-only arguments.
- Obligation: `INV-values-verbatim`, `INV-bool-spellings`,
  `INV-matches-preserved`, `L-dispatch-total`, `INV-command-names-agree`.
  Method: `rstest` `#[case]` tables with `pretty_assertions::assert_eq!` on
  whole structs. Artefact: `src/cli/tests/parsing.rs`, unit tests in
  `src/cli/dispatch.rs`. Non-vacuity: every table has accepting and rejecting
  rows; rejecting rows assert the specific `clap::error::ErrorKind`
  (`UnknownArgument` for `--publish false`, `InvalidValue` for
  `--publish=maybe`, `MissingRequiredArgument` for a missing proposal). Seeded
  fault: a temporary normalizing `value_parser` on `--branch` (`/` to `-`) must
  fail the witness row. A trimming parser was tried first and recorded as
  non-discriminating, because the witness has no outer whitespace.
- Obligation: `INV-catalogue-ids-agree`, `INV-catalogue-complete`,
  `INV-help-localized`. Method: `rstest` tables; the completeness test reads
  identifiers from the embedded catalogue text with the pattern
  `^[A-Za-z][A-Za-z0-9_.-]* *=` and normalizes `.` to `-` before comparing with
  the set computed from the derive constants. Artefact:
  `src/cli/tests/catalogue.rs`. Non-vacuity: seeded faults, each reverted:
  delete one `-args-…-help` line (fails naming it); add
  `splitters.cli.map-args-colour-help = x` (fails naming the stale identifier);
  change one struct's `localization_base` (fails `INV-catalogue-ids-agree`).
- Obligation: `INV-streams`, `INV-no-effects`.
  Method: `assert_cmd` end-to-end tests through the shared `splitters_cmd`
  fixture. Rationale: the stream and exit-status contract belongs to the
  executable. Artefact: `tests/cli/end_to_end.rs`, `tests/cli/support/mod.rs`.
  Evidence: `cargo nextest run --test cli`. Red against the stub: every display
  and error case fails (status `0`, greeting on standard output). Non-vacuity:
  the sandbox is a `tempfile::TempDir` in the system temporary directory with
  `GIT_CEILING_DIRECTORIES` set to its parent, so Git cannot discover the
  Splitters checkout; the test first writes and removes a witness file to prove
  the emptiness check can see files. Seeded fault: a temporary
  `std::fs::create_dir_all("splitters")` in `main` must fail `INV-no-effects`.
  The non-UTF-8 row (Unix only, `OsStr::from_bytes(b"\xff")`) must exit `2`;
  seeded fault: switching `main` to `std::env::args()` must make it exit `101`.
- Obligation: `INV-guide-matches-binary` and the BDD scenarios.
  Method: the shared tested-example loader plus `rstest-bdd` scenarios using
  `{cmd:string}` placeholders split with `shlex::split` (assert
  `argv[0] == "splitters"`). Artefact: `tests/cli/guide_examples.rs`,
  `tests/cli/bdd.rs`, `tests/cli/support/doc_examples/`,
  `tests/features/command_surface.feature`. Non-vacuity: the loader's own tests
  cover an unmarked fence, an unterminated fence, a marker without an
  identifier, and a duplicate identifier; the guide test fails if any loaded
  identifier is unhandled; seeded fault: editing one word of a documented help
  example must fail with a diff.

### Formal verification decision

No Verus proof, Kani harness, model checker, or property test is warranted.
This item introduces no business logic: it declares data types and delegates
parsing to `clap` (an axiom). Its invariants range over finite declarations,
which tables cover exhaustively, and over the process contract, which only
end-to-end tests can observe. A Verus proof would have to model clap's parser
(third-party behaviour) and would still leave the executable unverified; Kani
adds nothing without arithmetic, `unsafe` code, or a bounded state machine. The
first items where Verus is plausibly warranted are 1.2.2 (proposal-graph
acyclicity) and 4.1.3 (exact fragment-set subtraction); no proof obligations
for them are made here. Recorded as D10.

### Suggested BDD features

Feature file `tests/features/command_surface.feature`, run by `tests/cli/bdd.rs`
(`cargo nextest run --test cli -E 'test(/bdd/)'`). Scenarios link to
`REQ-help`, `REQ-parse-coverage`, and `REQ-streams` in `EP-M3`; the last
scenario links to `GAP-users-guide` in `EP-M4`.

```gherkin
Feature: Splitters command surface
  The splitters binary declares map, validate, and extract and reports
  their status without touching the repository.

  Scenario: The top-level help lists every command
    When the operator runs "splitters --help"
    Then the exit status is 0
    And the commands section lists "map", "validate", and "extract"
    And standard error is empty

  Scenario Outline: Each command documents its options
    When the operator runs "splitters <command> --help"
    Then the exit status is 0
    And the options section lists "<option>"

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
    And standard error names "<PROPOSAL>"

  Scenario: A boolean flag rejects a detached value
    When the operator runs "splitters extract add-parser --publish false"
    Then the exit status is 2
    And standard output is empty

  Scenario: The documented help matches the binary
    Given the users' guide example "splitters-help"
    When the operator runs the documented command
    Then standard output equals the documented output
```

"Lists" steps match an anchored row (`^\s+<name>\s` within the `Commands:` or
`Options:` section), not a substring, so `extract` cannot match "extracted".

## Plan of work

### `EP-M1` typed command declarations parse in-process

Assigned: `REQ-surface`, `REQ-typed`, `REQ-single-declaration`,
`REQ-merge-ready`, `GAP-deps`; D1, D3–D6, D12.

1. Add to `Cargo.toml` the runtime dependencies `camino` (feature `serde1`),
   `clap` (feature `derive`), `ortho_config`, and `serde` (feature `derive`),
   and the development dependencies `googletest`, `insta`, and
   `pretty_assertions`.
2. Red: create `src/cli/tests/mod.rs`, `src/cli/tests/declaration.rs`, and
   `src/cli/tests/parsing.rs` with the obligations above, and a minimal
   `src/cli/mod.rs` declaring the module tree, so the failure is an
   unresolved-type compile error. Run
   `cargo nextest run --lib -E 'test(/cli::/)'` and record it.
3. Green: write `src/cli/mod.rs` (`Cli`, `Commands`, `Invocation`,
   `localized_command`, and `try_parse_from`, implemented with
   `ortho_config::parse_localized_command`), `src/cli/args/mod.rs`, and
   `src/cli/args/{map,validate,extract}.rs`, each struct per the field model,
   D1, and D3–D5, deriving `Default`. `localized_command` applies
   `term_width(100)` and `with_base(LOCALIZATION_BASE).localize(localizer)`;
   tests in this milestone pass `ortho_config::NoOpLocalizer`. Expose
   `pub mod cli;` from `src/lib.rs`, keeping `greeting()` until `EP-M3` so the
   binary still builds.
4. Review the surface snapshot against design Tables 3–5 and accept it.
5. Refactor: keep each file under 150 lines; give `try_parse_from` a doctest
   that parses `["splitters", "map", "--base", "main"]` with `NoOpLocalizer`.
6. Run `make check-fmt`, `make lint`, and `make test` in sequence.

### `EP-M2` help text is localized through the en-GB catalogue

Assigned: `REQ-localized-help`; D2, D7, D9.

1. Red: add `src/cli/tests/catalogue.rs` (`INV-catalogue-ids-agree`,
   `INV-catalogue-complete`, `INV-help-localized`, and a test that
   `help_localizer()` builds). Run and record the compile failure.
2. Green: write `locales/en-GB/messages.ftl` (root about and long-about, one
   about line per command, one help line per argument) and
   `src/cli/localization.rs` (`pub(crate) const CATALOGUE`, the `include_str!`
   of the file, and `pub fn help_localizer()`, which builds a `FluentLocalizer`
   for `en-GB` with
   `FluentLocalizerBuilder::new(langid!("en-GB")).with_consumer_resources(..)`).
   Switch the surface snapshot test to the en-GB localizer, and shorten the
   `///` comments to developer-facing semantics per D9.
3. Re-review the surface snapshot, which now carries the catalogue text.
4. Run the three gates.

### `EP-M3` the `splitters` binary uses the new surface end to end

Assigned: `REQ-help`, `REQ-parse-coverage`, `REQ-streams`, `REQ-no-effects`,
`GAP-stub`; D8.

1. Add the development dependencies `assert_cmd`, `rstest-bdd`,
   `rstest-bdd-macros`, `shlex`, and `tempfile`, and register one test target in
   `Cargo.toml` (`[[test]] name = "cli"`, `path = "tests/cli/main.rs"`), so
   the end-to-end tests, the BDD scenarios, the guide test, and their support
   code link once.
2. Red: write `tests/cli/main.rs` (module root), `tests/cli/support/mod.rs`
   (the `splitters_cmd` fixture and sandbox), `tests/cli/end_to_end.rs`
   (`INV-streams` and `INV-no-effects` tables, `--version` against
   `env!("CARGO_PKG_VERSION")`), `tests/cli/bdd.rs`, and
   `tests/features/command_surface.feature` (all scenarios except the
   documentation scenario). Run against the stub and record the failures.
3. Green: add `src/cli/dispatch.rs` (`CommandName`, `CommandOutcome`,
   `dispatch`, and `report`, which writes the diagnostic to a supplied
   `std::io::Write`) and replace `src/main.rs`: build `help_localizer()` (on
   failure, report to standard error and exit `2`), call
   `cli::try_parse_from(std::env::args_os(), &localizer)`, call `err.exit()` on
   error, otherwise `dispatch` and `report` to a locked standard-error handle,
   handling the write result explicitly, and return `ExitCode::from(2)`. Delete
   `greeting()` and its test, and give `src/lib.rs` a crate-level comment
   describing the library-first layout.
4. Refactor: keep `main` under 30 lines.
5. Run the three gates.

### `EP-M4` documentation, tested guide examples, and roadmap closure

Assigned: `GAP-users-guide`, `GAP-conventions`; D11, D13.

1. Red: write `tests/cli/support/doc_examples/` (marker parsing, fence
   extraction, and the four structural failures, with its own `rstest` table)
   and `tests/cli/guide_examples.rs` (load `docs/users-guide.md`, run each
   example's `$ splitters …` line through `splitters_cmd`, compare the rest of
   the fence with standard output using `pretty_assertions`, and fail on
   unhandled identifiers); add the documentation scenario. Run and record the
   failure (guide missing).
2. Green: write `docs/users-guide.md`: an audience paragraph naming release
   `0.1.0` as an early-adopter release in which the commands are declared but
   not implemented; "See the command surface" with tested examples
   `splitters-help`, `splitters-map-help`, `splitters-validate-help`, and
   `splitters-extract-help`; an option table per command (option, value,
   purpose); "Interpret the current exit status" describing D8 as interim
   behaviour; and a note that configuration files and `SPLITTERS_` environment
   variables arrive with roadmap 1.1.4. Every fence carries a
   `<!-- tested-example: <id> -->` marker.
3. Write
   `docs/adrs/adr-0001-command-line-declaration-and-help-text-conventions.md`
   (status `Proposed` until the PR merges, then `Accepted`) recording D1, D2,
   D5, D7, and D9 with options considered. Add a "Flag" column to design Tables
   3–5 and a short paragraph in design §7 linking the ADR. Update
   `docs/developers-guide.md` §1 (no longer a stub), §3.1 (files under
   `src/cli/`, the catalogue, the localization-base rule, how to add an option
   in four steps, and the never-call-`load` rule), and a testing subsection (the
   `splitters_cmd` fixture and allowlist, the tested-example loader and its
   reuse policy, and the snapshot and guide update procedure). Update
   `docs/repository-layout.md` (`locales/`, `tests/cli/`, `tests/features/`,
   `docs/adrs/`, `docs/execplans/`) and `docs/contents.md` (users' guide, ADRs,
   execplans).
4. Record the open questions for later items (see
   `Interfaces and dependencies`) in the roadmap's 1.1.4, 3.2.1, and 5.2.2
   entries only if the user agrees; otherwise keep them in this plan's
   `Outcomes & retrospective`.
5. Mark roadmap item 1.1.1 done (`- [x]`) only after the acceptance criteria
   pass.
6. Run `make fmt`, then `make check-fmt`, `make lint`, `make test`,
   `make markdownlint`, and `make nixie`.

## Milestones and plateaus

- `EP-M1`: the library exposes typed declarations and an in-process parser; the
  binary is unchanged. Requirements: `REQ-surface`, `REQ-typed`,
  `REQ-single-declaration`, `REQ-merge-ready`, `GAP-deps`. Evidence: the `cli`
  unit tests pass and the surface snapshot is accepted; three gates pass.
  Conformance check:
  - Tables 3–5 inputs present per command; design §12 module split followed.
  - Only authorized dependencies; no trust boundary or persisted format.
  - Trace links current.
  Recovery: revert the milestone commits. Remaining: localization, binary,
  documentation. Compatibility: none.
- `EP-M2`: help resolves through the en-GB catalogue with no fallback.
  Requirements: `REQ-localized-help`. Evidence: catalogue tests pass; three
  gates pass. Conformance check:
  - Catalogue complete and stale-free; no environment read.
  - Trace links current.
  Recovery: revert; `EP-M1` stands. Compatibility: none.
- `EP-M3`: `splitters` parses, reports, and exits as specified; the stub is
  gone. Requirements: `REQ-help`, `REQ-parse-coverage`, `REQ-streams`,
  `REQ-no-effects`, `GAP-stub`. Evidence: `cargo nextest run --test cli`
  passes; three gates pass; CI duration recorded. Conformance check:
  - Stream discipline and no effects proven at the process boundary.
  - Exit statuses documented as interim (1.1.2 owns the mapping).
  - Trace links current.
  Recovery: revert; `EP-M2` stands. Compatibility: none (`greeting()` deleted).
- `EP-M4`: documentation matches behaviour and is tested; roadmap 1.1.1 done.
  Requirements: `GAP-users-guide`, `GAP-conventions`. Evidence: guide test and
  documentation scenario pass; all gates including Markdown pass. Conformance
  check:
  - ADR-0001 and design Tables 3–5 match the code; developers' guide and
    layout current; roadmap ticked only after acceptance.
  Recovery: revert documentation commits independently. Remaining: roadmap
  1.1.2–1.1.4. Compatibility: none.

## Concrete steps

Run every command from the repository root. Set `set -o pipefail` before any
`tee` pipeline.

```plaintext
EP-M1 red:   cargo nextest run --lib -E 'test(/cli::/)'
             -> E0412/E0433 unresolved types in src/cli/tests
EP-M1 green: same command -> pass; cargo insta review accepts the surface snapshot
EP-M2 red:   cargo nextest run --lib -E 'test(/catalogue/)'
             -> unresolved cli::localization
EP-M2 green: same command -> pass
EP-M3 red:   cargo nextest run --test cli
             -> failures: exit status 0 and "Hello from Splitters!" on stdout
EP-M3 green: same command -> pass
EP-M4 red:   cargo nextest run --test cli -E 'test(/guide/)'
             -> failure: docs/users-guide.md not found
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

`EP-M4` also runs `make markdownlint` and `make nixie` with the same logging.
Builds queue for compile slots through `build-limits-rustc`; a waiting build is
not hung.

## Validation and acceptance

- `splitters --help` exits `0`, prints the catalogue's about text and the three
  commands on standard output, and nothing on standard error; its output equals
  the `splitters-help` example in `docs/users-guide.md`.
- Each `splitters <command> --help` lists exactly the options of design Tables
  3–5 with catalogue text, and equals its users'-guide example.
- `splitters validate add-parser` exits `2` with
  `splitters validate: not implemented yet` on standard error and nothing on
  standard output; likewise `map` and `extract add-parser`.
- `splitters extract` exits `2` with a usage error naming `<PROPOSAL>` on
  standard error; `splitters extract add-parser --publish false` exits `2`;
  `splitters extract add-parser --publish=false` reaches the not-implemented
  report.
- Bare `splitters` exits `2` with help on standard error; a non-UTF-8 operand
  exits `2`, never `101`.
- Running any command in an empty sandbox outside a Git repository leaves the
  sandbox empty.

Quality criteria: `make test` passes with the new unit, snapshot, end-to-end,
BDD, and guide tests; every `INV-*` obligation is discharged by its artefact,
with each seeded fault applied, observed, reverted, and recorded in
`Artefacts and notes`; `make check-fmt`, `make lint`, `make markdownlint`, and
`make nixie` pass with no new `#[expect]` unless justified in `Decision log`.

## Idempotence and recovery

Every step is re-runnable; tests and gates are pure. Each milestone is one or
more commits, so reverting a milestone's commits restores the previous plateau.
Seeded faults are applied and reverted in the working tree and never committed;
after each, `git status --short` must show only intended changes. Snapshot
updates go through `cargo insta review`; never run `cargo insta accept` without
reviewing the diff. The planning probe under `target/plan-probe/` (ignored by
Git) is deleted before the plan is submitted.

## Artefacts and notes

`EP-M1` evidence (logs under `/tmp`, named by branch):

```plaintext
red:   cargo nextest run --lib -E 'test(/cli::/)'
       -> 13 errors: unresolved args/Invocation/localized_command/Commands/
          try_parse_from, plus E0599 and E0382 in the new tests
green: same command -> 31 passed, 1 skipped
       surface snapshot: src/cli/tests/snapshots/
         splitters__cli__tests__declaration__surface.snap
EP-M2 red:   cargo nextest run --lib -E 'test(/catalogue/)'
             -> E0432 unresolved import crate::cli::localization
EP-M2 green: same command -> 40 passed, 1 skipped; snapshot re-accepted
EP-M3 red:   cargo nextest run --test cli
             -> against the stub: 9 passed, 11 failed ("Hello from Splitters!"
                on stdout, exit status 0 where 2 was required)
EP-M3 green: same command -> 32 passed, 0 skipped
EP-M4 red:   cargo nextest run --test cli -E 'test(/guide/)'
             -> E0583-style include_str failure: docs/users-guide.md missing
EP-M4 green: cargo nextest run --test cli -> 45 passed, 0 skipped
```

Seeded faults (each applied, observed, reverted; `git status --short` showed
only intended changes afterwards, and no `.snap.new` remained):

| Fault                                               | Observed failure                                                                                                        |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Remove `require_equals` from `publish`              | `command_surface_is_pinned` snapshot diff                                                                               |
| Add `#[arg(long)] verbose` to the root `Cli`        | `root_declares_only_help_and_version` names `verbose`; snapshot diff                                                    |
| Add `skip_cli` + `#[arg(long)] colour` to `MapArgs` | `clap_arguments_match_ortho_config_ids` reports `colour` unexpected; snapshot diff                                      |
| Normalize `--branch` values (`/` to `-`)            | `option_values_are_stored_verbatim::case_2_branch` fails the verbatim witness                                           |
| Trim `--branch` values                              | not discriminating: the witness has no outer whitespace (recorded above)                                                |
| Delete one `-args-check-help` catalogue line        | `catalogue_declares_exactly_the_generated_identifiers` names it missing; `rendered_help_uses_catalogue_text_only` fails |
| Add a stale `splitters.cli.map-args-colour-help`    | same test names it unexpected                                                                                           |
| Change `MapArgs` `localization_base`                | `localization_base_agrees_with_the_command_name` and the catalogue set fail                                             |
| `std::fs::create_dir_all("splitters")` in `main`    | every `commands_leave_the_sandbox_untouched` row reports the stray directory                                            |
| `std::env::args()` in `main`                        | `a_non_utf8_operand_is_a_usage_error` fails                                                                             |
| Edit one word of a guide example                    | `the_documented_output_matches_the_binary` fails with a diff                                                            |

CI durations for the tolerance check (`build-test` job, successful runs).
`ci.yml` has no `push` trigger, so no literal `main`-branch run exists. The
agreed substitute baseline is the latest green pull request targeting `main`
before the `EP-M1` commit: run 45610333088 (PR #91), 3m50s. The earlier
`deec445` and `2da5797` durations were pull-request runs on this branch, not
`main` runs, and are superseded.

```plaintext
main-targeting PR baseline  445e4f1  12:05:32Z -> 12:09:22Z  (3m50s)
branch after EP-M4          9f454ef  07:44:42Z -> 07:51:54Z  (7m12s)
branch after repair         578739e  08:06:17Z -> 08:12:31Z  (6m14s)
rebased head                709d4cd  09:57:54Z -> 10:05:06Z  (7m12s)
```

Growth against the baseline is 2m24s to 3m22s, inside the five-minute
tolerance. Run 45742732950 is later than the branch's first push and is not a
usable pre-change baseline.

Planning probe (2026-11-20, scratch crate under `target/plan-probe/`, now
deleted) declared the three structs with `clap` 4.6.9, `ortho_config` 0.11.0,
`serde` 1.0.229, and `camino` 1.2.6, and a catalogue with two candidate
spellings for the `map --base` help identifier. (The probe still used the
draft's `manifest_dir` default, since withdrawn by D4.)

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
table and `clippy.toml` passed. The runtime dependency closure is 121 unique
crates.

Second probe, documentation metadata for `ValidateArgs`:

```plaintext
OrthoConfigDocs::get_doc_metadata():
field proposal required=true cli=(Some("proposal"), None) env=SPLITTERS_PROPOSAL file=proposal
field check required=false cli=(Some("check"), None) env=SPLITTERS_CHECK file=check
app="splitters" about=splitters-cli-validate-about

OrthoConfigSubcommandDocs::get_subcommand_doc_metadata() on the enum:
(same fields) app="validate" about=validate-about
```

## Interfaces and dependencies

Authorized dependencies (caret requirements):

- Runtime: `camino = { version = "1.2.6", features = ["serde1"] }`,
  `clap = { version = "4.6.9", features = ["derive"] }`,
  `ortho_config = "0.11.0"`, and
  `serde = { version = "1.0.229", features = ["derive"] }`.
- Development: `assert_cmd = "2.2.2"`, `googletest = "0.14.4"` (set-equality
  and string matchers such as `unordered_elements_are!` and
  `contains_substring`), `insta = "1.49.0"` (the surface snapshot),
  `pretty_assertions = "1.4.1"` (readable diffs for whole-struct and
  guide-output comparisons), `rstest-bdd = "0.6.0"`,
  `rstest-bdd-macros = "0.6.0"`, `shlex = "2.0.1"`, and `tempfile = "3.27.0"`
  (already in the closure through `insta`). Existing `cap-std`, `rstest`, and
  `toml` stay as they are.
- Not added: `thiserror` (no error type), `mockable` (no environment access),
  `proptest` (D10), and `fluent-bundle` (OrthoConfig re-exports the Fluent
  types it needs).

Public library surface (all in `splitters::cli`):

```rust
#[derive(Debug, clap::Parser)]
pub struct Cli { pub command: Commands }

#[derive(Debug, clap::Subcommand)]
pub enum Commands { Map(MapArgs), Validate(ValidateArgs), Extract(ExtractArgs) }

#[derive(Debug, Default, Clone, PartialEq, Eq,
         clap::Parser, serde::Serialize, serde::Deserialize, ortho_config::OrthoConfig)]
pub struct MapArgs { pub base: Option<String>, pub manifest_dir: Option<Utf8PathBuf> }
// ValidateArgs and ExtractArgs derive the same and hold the fields in the
// field model.

/// A parsed command line together with clap's record of supplied arguments.
pub struct Invocation { pub cli: Cli, pub matches: clap::ArgMatches }

/// Builds the localized command tree at a fixed help width of 100 columns.
pub fn localized_command(localizer: &dyn ortho_config::Localizer) -> clap::Command;

/// Parses an argument vector (including the program name).
pub fn try_parse_from<I, T>(args: I, localizer: &dyn ortho_config::Localizer)
    -> Result<Invocation, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;

pub mod localization {
    /// Builds the en-GB help localizer from the embedded catalogue.
    pub fn help_localizer()
        -> Result<ortho_config::FluentLocalizer, ortho_config::FluentLocalizerError>;
}

/// Provisional: reshaped by 1.1.2-1.1.4.
pub mod dispatch {
    pub enum CommandName { Map, Validate, Extract }   // Display = clap name
    pub enum CommandOutcome { NotYetImplemented(CommandName) }
    pub fn dispatch(cli: &Cli) -> CommandOutcome;     // const while Clippy requires it
    pub fn report(outcome: CommandOutcome, err: &mut dyn std::io::Write)
        -> std::io::Result<()>;
}
```

`LOCALIZATION_BASE` and the catalogue text are `pub(crate)`. The `dispatch`
module is provisional: 1.1.2 adds the error hierarchy, 1.1.3 injects repository
context, and 1.1.4 passes merged configuration and `ArgMatches`, so no trait or
derive beyond `Debug`, `Clone`, `PartialEq`, and `Eq` is promised. Whether
`FluentLocalizerError` is boxed inside a future error type is 1.1.2's decision.

Reuse policy:

- `splitters::cli` owns every command-line declaration. Later items add fields
  to the existing structs; they must not add a parallel options struct, read
  arguments elsewhere, or call an OrthoConfig `load` on a struct.
- `localized_command` is the only constructor of the command tree used for
  parsing or help, so localization and the fixed width are never skipped.
- `tests/cli/support/doc_examples/mod.rs` is the shared tested-example
  loader for user-facing Markdown documents. Tests in the `cli` test target use
  it directly; no test may copy example text into a fixture. It only extracts
  and validates marked fences; comparison stays in the calling test.
- `tests/cli/support/mod.rs::splitters_cmd` is the only way tests start the
  binary, so the environment allowlist and sandbox apply everywhere.

Notes for later roadmap items (open questions, not decided here): 1.1.4 must
derive `SelectedSubcommandMerge` on `Commands` (with
`#[ortho_subcommand(with_matches)]` where a field uses
`cli_default_as_absent`), choose whether shared inputs become globals (D12),
decide how to treat the generated keys for `proposal` and `branch` (D3), and
decide whether risky fields (`check`, which runs a command, and `publish`,
which reaches the network) may be set from a repository-committed configuration
file. 3.2.1 must settle the stored format of `check` (D6). 5.2.2 must resolve
the `cargo orthohelp` locale limitation (D7) and the documentation-metadata
issue in `Surprises & discoveries`.

## Revision note

- 2026-11-20: initial draft from Wyvern reconnaissance and the planning probe.
- 2026-11-20: expert design review applied. Changed: explicit
  `#[command(name)]` on every struct (D1); the parser now returns `Invocation`
  with `ArgMatches` through `parse_localized_command` (`REQ-merge-ready`);
  `manifest_dir` lost its default (D4); the catalogue is the only owner of help
  text and the fallback localizer was removed (D7, D9); the proptest property
  was replaced by fixed witnesses (D10); conventions move to ADR-0001 and a
  "Flag" column in design Tables 3–5 (D11); shared inputs stay per command
  (D12); rendered help is pinned once, in the tested users' guide, with `insta`
  pinning only the structured surface (D13); the test suite became one `cli`
  target with a sandboxed `splitters_cmd` fixture, `GIT_CEILING_DIRECTORIES`, a
  coverage-safe environment allowlist, anchored BDD matching, `shlex`
  splitting, and new failure rows (bare `splitters`, `help validate`, non-UTF-8
  operand); the `EP-M2` sequencing bug (`insta` used before being added) was
  fixed; module paths moved to `src/cli/args/`; budgets were re-estimated (code
  1,700, documentation 550, files 32) and a CI-cost tolerance was added.
  Remaining work and milestone order are otherwise unchanged.
- 2026-11-20: linked the three OrthoConfig findings to the upstream issues
  filed at the user's request (#717 documentation metadata, #718 isolation
  marks, #719 `cargo orthohelp` locale). No change to scope, decisions, or
  milestones.
