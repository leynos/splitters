# Splitters user's guide

This guide is for operators who run the `splitters` command. It covers release
`0.1.0`, an early-adopter release: the `map`, `validate`, and `extract`
commands are declared and documented here, but no command changes a repository
yet. Interfaces may change before the first stable release, so pin the version
if you depend on the command surface.

## See the command surface

Run `splitters --help` to list the commands this release declares.

<!-- tested-example: splitters-help -->
```console
$ splitters --help
Split one Git branch into smaller, independently reviewable branches.

Usage: splitters <COMMAND>

Commands:
  map       Decompose the current branch into fragments and write a manifest.
  validate  Prove that one proposal can be extracted cleanly.
  extract   Move one validated proposal onto a new branch.
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

Each command documents its own options. `map` decomposes the current branch
into fragments and writes a manifest:

<!-- tested-example: splitters-map-help -->
```console
$ splitters map --help
Decompose the current branch into fragments and write a manifest.

Usage: splitters map [OPTIONS]

Options:
      --base <REF>          Base ref to compare against
      --manifest-dir <DIR>  Directory that receives the manifest, fragments, and metadata
  -h, --help                Print help
```

`validate` proves that one proposal can be extracted cleanly:

<!-- tested-example: splitters-validate-help -->
```console
$ splitters validate --help
Prove that one proposal can be extracted cleanly.

Usage: splitters validate [OPTIONS] <PROPOSAL>

Arguments:
  <PROPOSAL>  Identifier of the proposal to validate

Options:
      --manifest-dir <DIR>  Directory holding the manifest to read
      --base <REF>          Base ref to compare against
      --check <COMMAND>     Command to run in both the candidate and residual worktrees
  -h, --help                Print help
```

`extract` moves one validated proposal onto a new branch:

<!-- tested-example: splitters-extract-help -->
```console
$ splitters extract --help
Move one validated proposal onto a new branch.

Usage: splitters extract [OPTIONS] <PROPOSAL>

Arguments:
  <PROPOSAL>  Identifier of the proposal to extract

Options:
      --manifest-dir <DIR>  Directory holding the manifest to read
      --base <REF>          Base ref to compare against
      --branch <NAME>       Name of the branch that receives the proposal
      --publish[=<BOOL>]    Publish the extracted branch as a pull request after local extraction
  -h, --help                Print help
```

## Command reference

Every command accepts `--base` and `--manifest-dir`, and `validate` and
`extract` also require a proposal identifier. The option names below are the
primary operator surface; roadmap item 1.1.4 adds configuration files and
`SPLITTERS_` environment variables generated from the same declarations.

### `map`

| Option           | Value | Purpose                                            |
| ---------------- | ----- | -------------------------------------------------- |
| `--base`         | `REF` | Base ref to compare against                        |
| `--manifest-dir` | `DIR` | Directory that receives the manifest and artefacts |

### `validate`

| Option           | Value     | Purpose                                                     |
| ---------------- | --------- | ----------------------------------------------------------- |
| `<PROPOSAL>`     | `STRING`  | Identifier of the proposal to validate                      |
| `--manifest-dir` | `DIR`     | Directory holding the manifest to read                      |
| `--base`         | `REF`     | Base ref to compare against                                 |
| `--check`        | `COMMAND` | Command to run in both the candidate and residual worktrees |

`--check` is a trusted operator hook, not a sandbox: Splitters runs the command
as a normal subprocess in worktrees that contain repository contents, with your
ambient operating-system permissions.

### `extract`

| Option           | Value    | Purpose                                                         |
| ---------------- | -------- | --------------------------------------------------------------- |
| `<PROPOSAL>`     | `STRING` | Identifier of the proposal to extract                           |
| `--manifest-dir` | `DIR`    | Directory holding the manifest to read                          |
| `--base`         | `REF`    | Base ref to compare against                                     |
| `--branch`       | `NAME`   | Name of the branch that receives the proposal                   |
| `--publish`      | `[BOOL]` | Publish the extracted branch as a pull request after extraction |

Write `--publish` or `--publish=true` to request publication, and
`--publish=false` to override a configured `true`. The detached form,
`--publish false`, is rejected so the flag cannot swallow the next operand.

## Interpret the current exit status

This release declares the commands without implementing them, so the exit
status is interim behaviour that roadmap item 1.1.2 replaces with the design's
full mapping.

| Situation                               | Standard output | Standard error                             | Exit status |
| --------------------------------------- | --------------- | ------------------------------------------ | ----------- |
| `--help`, `help <command>`, `--version` | Help or version | Empty                                      | `0`         |
| A well-formed command                   | Empty           | `splitters <command>: not implemented yet` | `2`         |
| A usage error                           | Empty           | A diagnostic naming the problem            | `2`         |

Usage errors include a missing `<PROPOSAL>`, an unknown option, a detached
boolean value, a non-UTF-8 operand, and a bare `splitters` invocation. Help and
version text always goes to standard output so redirection and assistive
technology behave predictably; every diagnostic goes to standard error.

## Where configuration comes from

This release reads options from the command line only. Configuration files and
`SPLITTERS_` environment variables are generated from the same declarations and
arrive with roadmap item 1.1.4. Until then, run every command with explicit
options.

## See also

- [Splitters design](splitters-design.md) records the command contracts and the
  on-disk manifest format.
- [Architectural decision record 0001](adrs/adr-0001-command-line-declaration-and-help-text-conventions.md)
  records the declaration and help-text conventions this guide relies on.
