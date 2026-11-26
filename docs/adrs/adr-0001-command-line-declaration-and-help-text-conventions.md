# Architectural decision record (ADR) 0001: Command-line declaration and help-text conventions

## Status

Proposed.

## Date

2026-11-25.

## Context and problem statement

Splitters declares its command line with `clap` and `ortho_config`
(OrthoConfig) from `leynos/ortho-config`. One declaration supplies the
command-line flag, the configuration-file key, the environment variable, and
the default for every option, as design §5 and §12 require. That single-source
rule raises questions the declarations alone do not answer:

- Which derives belong on the root parser and on each subcommand struct?
- How are catalogue identifiers derived, and what keeps them aligned with the
  command names?
- How does a boolean flag express "absent", "explicitly true", and "explicitly
  false"?
- Which locale ships, and what happens when a translation is missing?
- Where does user-facing help text live, and what are Rust doc comments for?

The answer binds every later roadmap item that adds an option, so it belongs in
an ADR rather than in a comment beside one struct.

## Decision drivers

- Design §5 and §12 require one declaration per option and forbid a second
  configuration schema.
- `docs/developers-guide.md` §3.1 requires a plain `clap` root parser.
- Help text must be translatable without touching Rust declarations.
- Tests, not review, must catch drift between declarations and catalogues.
- The repository writes en-GB-oxendict English and denies silent fallbacks.

## Options considered

### Option A: OrthoConfig on the root and on every subcommand

Derive `OrthoConfig` on the root parser as well as on each subcommand struct.
The upstream `hello_world` example takes this route.

Consequence: the root gains configuration keys of its own and an
`OrthoConfig::load()` that reads un-namespaced keys such as `SPLITTERS_*`,
creating a second configuration surface beside the subcommand namespaces. The
root would also need a documented exception to developers' guide §3.1.

### Option B: Plain `clap` root, OrthoConfig on each subcommand

The root derives only `clap::Parser`; each subcommand struct derives
`clap::Parser`, `serde::Serialize`, `serde::Deserialize`, and
`ortho_config::OrthoConfig`. Subcommand configuration is namespaced as
`[cmds.<name>]` and `SPLITTERS_CMDS_<NAME>_*`.

Consequence: the root stays a dispatch table, and configuration layering is
owned by the struct that uses each option. Adding the root derives later, if a
real need appears, is additive.

| Topic                      | Option A           | Option B               |
| -------------------------- | ------------------ | ---------------------- |
| Root configuration surface | Present            | None                   |
| Developers' guide §3.1     | Needs an exception | Followed               |
| Subcommand keys            | `[cmds.<name>]`    | `[cmds.<name>]`        |
| Future global options      | Natural            | Needs a later decision |

_Table 1: Root-derive options._

### Option C: Hand-rolled parsing with a separate configuration struct

Rejected outright: design §5 and §12 forbid both a hand-rolled parser and a
second configuration schema.

## Decision outcome / proposed direction

Adopt Option B. The conventions that follow are part of the same decision.

### Declarations

- The root `Cli` derives `clap::Parser` only and declares no option beyond
  clap's generated `--help` and `--version`.
- `Commands` derives `clap::Subcommand`; each variant carries
  `#[command(name = "<command>")]`.
- Each of `MapArgs`, `ValidateArgs`, and `ExtractArgs` derives
  `clap::Parser`, `serde::Serialize`, `serde::Deserialize`,
  `ortho_config::OrthoConfig`, `Debug`, `Default`, `Clone`, `PartialEq`, and
  `Eq`, and carries both `#[command(name = "<command>")]` and
  `#[ortho_config(prefix = "SPLITTERS", localization_base = "splitters.cli.<command>")]`.

The explicit command names are required because OrthoConfig namespaces
subcommand configuration by the _struct's own_ clap command name, not by the
enum variant.

### Catalogue identifiers

The root catalogue base is `splitters.cli`. A subcommand's `localization_base`
is that base plus the command name, so the identifier the localizer looks up
(`splitters-cli-map-args-base-help`) equals the identifier the derive
generates. A test asserts the agreement, because a mismatch falls back silently
to clap's text.

### Boolean flags

A boolean option is `Option<bool>` with `num_args = 0..=1`,
`require_equals = true`, and `default_missing_value = "true"`. Omission defers
to lower configuration layers; `--publish` and `--publish=true` mean `true`;
`--publish=false` overrides a configured `true`; and `--publish false` is
rejected. `ArgAction::SetTrue` was rejected because a configured `true` could
then never be switched off on the command line.

### Catalogue and locale

One consumer catalogue, `locales/en-GB/messages.ftl`, is embedded with
`include_str!`, and the localizer is always built for `en-GB`. There is no
locale detection and no no-op fallback: the catalogue is fixed at compile time
and a unit test proves it builds, so a run-time construction failure is a code
defect that the binary reports on standard error with exit status `2`. Locale
detection needs an injected environment and belongs with roadmap item 1.1.4.

### Ownership of user-facing text

The catalogue is the only owner of user-facing help text. Rust doc comments
describe semantics for developers — for example "`None` defers to lower
configuration layers" — and must not restate help text. Because the catalogue
is always loaded, doc-comment help would never be shown, and keeping two copies
of the same prose would drift silently. Catalogue tests prove every generated
identifier resolves and that no stale identifier remains.

## Known risks and limitations

- OrthoConfig derives a configuration key and environment variable even for
  operands such as `proposal` and `branch`, because it has no
  "command-line-only" attribute. The required operand always wins at parse
  time; roadmap item 1.1.4 decides whether to document or suppress the keys.
- clap's own diagnostics use OrthoConfig's en-US catalogue even when the help
  text is en-GB, so error spellings are US English beside British help text.
  Roadmap item 5.2.2 owns the locale limitation.
- Fluent isolates interpolated values with Unicode marks, so tests assert on
  error text structurally rather than by exact snapshot.

## Outstanding decisions

- Roadmap item 1.1.4 chooses whether shared inputs become root-level global
  options, which would require the root derive this ADR defers.
- Roadmap item 3.2.1 settles the stored format of `--check`.
- Roadmap item 5.2.2 settles `cargo orthohelp`'s locale handling.

## Architectural rationale

The decision keeps one declaration per option, keeps the root a dispatch table,
and makes the catalogue the single source of user-facing prose. Each choice is
testable without a running repository: the command surface is a snapshot,
identifier agreement is a table test, and rendered help is compared against the
tested users'-guide examples.
