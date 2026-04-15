# Splitters roadmap

This roadmap sequences the implementation work for Splitters from the current
stub binary to a release-ready tool. It is intended for maintainers who need a
practical delivery plan that stays aligned with `docs/splitters-design.md`.
Tasks are written as concrete build increments with explicit dependencies and
measurable finish lines.

## 1. Core command and manifest foundation

### 1.1. Establish the executable command surface

This step delivers a real command-line interface (CLI) entry point with stable
command parsing, exit behaviour, and top-level repository preflight. It matters
first because every later workstream depends on predictable argument handling
and error reporting.

- [ ] 1.1.1. Replace the stub binary with the `map`, `validate`, and
  `extract` command skeletons. See `docs/splitters-design.md` §6 and §7.
  - Define the root `clap` parser and the three subcommands.
  - Add typed arguments for base override, manifest directory, proposal
    selection, branch naming, `--check`, and `--publish`.
  - Success criteria: `splitters --help` documents the full command surface and
    command parsing has snapshot or behaviour coverage.
- [ ] 1.1.2. Introduce top-level error and exit-code handling. Requires 1.1.1.
  See `docs/splitters-design.md` §7 and §12.
  - Define a domain error hierarchy for argument errors, repository-state
    rejection, validation rejection, and operational failure.
  - Map command outcomes to the documented exit codes without leaking raw
    library diagnostics to stdout.
  - Success criteria: integration tests prove exit code `0`, `1`, and `2`
    behaviour for at least one path each.
- [ ] 1.1.3. Enforce repository discovery and cleanliness checks. Requires
  1.1.1. See `docs/splitters-design.md` §5.
  - Reject commands outside a Git repository.
  - Reject dirty indexes and working trees before any diff materialization or
    mutation.
  - Reject unresolved base refs and detached states that violate the command
    contract.
  - Success criteria: command tests cover clean, dirty, and missing-repository
    cases.

### 1.2. Define the manifest and proposal schema

This step establishes the persisted workflow contract that all later commands
consume. It matters early because fragment generation, validation, and
extraction all depend on a stable on-disk format.

- [ ] 1.2.1. Implement manifest version 1 schema types and round-trip I/O. See
  `docs/splitters-design.md` §8.
  - Model top-level manifest fields, fragment records, and proposal records.
  - Serialize to Tom's Obvious Minimal Language (TOML) and mirror fragment
    metadata to JavaScript Object Notation (JSON).
  - Success criteria: round-trip tests prove no lossy field conversion for the
    normative schema.
- [ ] 1.2.2. Implement proposal graph validation. Requires 1.2.1. See
  `docs/splitters-design.md` §9.
  - Reject missing fragment references.
  - Reject duplicate fragment references inside a proposal.
  - Reject cycles in `depends_on`.
  - Success criteria: validation tests cover each rejected condition and one
    valid acyclic graph.
- [ ] 1.2.3. Define the machine-readable stdout envelope. Requires 1.1.2 and
  1.2.1. See `docs/splitters-design.md` §7.
  - Standardize summary objects for `map`, `validate`, and `extract`.
  - Keep human-readable progress reporting on stderr only.
  - Success criteria: command tests can parse stdout as JSON without filtering
    stderr.

## 2. Diff mapping and fragment identity

### 2.1. Materialize the change universe

This step delivers the `map` command’s core value: turning one branch diff into
addressable fragments and persisted artefacts. It matters before validation
because no proposal work can exist without a stable fragment set.

- [ ] 2.1.1. Resolve `merge-base(base, HEAD)` and traverse the change universe.
  Requires 1.1.3. See `docs/splitters-design.md` §1, §6, and §7.1.
  - Compute the merge base for the selected base ref.
  - Walk the diff between the merge base and `HEAD`.
  - Preserve enough per-file metadata to distinguish text, binary, rename, and
    mode-only deltas.
  - Success criteria: integration tests prove the reported range matches Git’s
    own merge-base and diff behaviour.
- [ ] 2.1.2. Emit textual, binary, rename, and metadata fragments. Requires
  2.1.1. See `docs/splitters-design.md` §7.1 and §8.
  - Split modified files into textual hunk fragments.
  - Emit file-level fragments for binary, empty-file, pure rename, and
    pure mode changes.
  - Persist unified diff payloads under `fragments/` and JSON mirrors under
    `metadata/`.
  - Success criteria: fixture repos cover each change kind listed in the
    design.
- [ ] 2.1.3. Write the `map` manifest directory atomically. Requires 2.1.2.
  See `docs/splitters-design.md` §6 and §8.
  - Write outputs to a temporary directory and move them into place only after
    success.
  - Refuse to overwrite an existing manifest directory without an explicit
    replace policy.
  - Success criteria: interrupted writes do not leave a half-populated manifest
    directory behind.

### 2.2. Implement stable fragment identity

This step converts fragments from simple numbered items into reliable handles
that survive normal branch evolution. It matters because stale or ambiguous
fragment matching would make every later command unsafe.

- [ ] 2.2.1. Implement fragment fingerprints and anchors. Requires 2.1.2. See
  `docs/splitters-design.md` §9.
  - Derive fingerprints from change kind, paths, changed content, and anchor
    context.
  - Store blob identifiers when Git provides them.
  - Success criteria: identical fragment content mapped twice yields the same
    fingerprint.
- [ ] 2.2.2. Detect overlap inside one mapped change universe. Requires 2.1.2.
  See `docs/splitters-design.md` §5 and §9.
  - Reject overlapping changed-line ownership across fragments.
  - Preserve byte-identical unchanged context only as anchors, not as
    selectable fragment content.
  - Success criteria: overlap tests reject double-owned lines and accept
    adjacent non-overlapping fragments.
- [ ] 2.2.3. Add stale-fragment rematching support. Requires 2.2.1. See
  `docs/splitters-design.md` §9 and §10.
  - Match by blob identifiers first, then by fingerprint, then by path plus
    anchor context.
  - Reject ambiguous matches instead of guessing.
  - Success criteria: validation fixtures cover successful rematch, no match,
    and ambiguous match outcomes.

## 3. Validation engine and policy hooks

### 3.1. Build isolated replay environments

This step delivers the safety proof that the design requires before any branch
rewrite. It matters because extraction must never proceed on a proposal that
cannot be replayed and removed cleanly.

- [ ] 3.1.1. Create candidate and residual linked worktrees for validation.
  Requires 1.1.3 and 2.2.3. See `docs/splitters-design.md` §10 and §12.
  - Materialize one throwaway worktree at the chosen base ref.
  - Materialize one throwaway worktree at `HEAD`.
  - Guarantee cleanup on success and failure.
  - Success criteria: repeated validation runs leave no orphaned worktrees.
- [ ] 3.1.2. Apply proposal patchsets forward and in reverse. Requires 3.1.1.
  See `docs/splitters-design.md` §10 and §11.
  - Replay the selected fragment set onto the candidate worktree.
  - Replay the same fragment set in reverse onto the residual worktree.
  - Fail validation if either side leaves a dirty or conflicted result.
  - Success criteria: fixtures prove the candidate branch plus residual branch
    reconstruct the original change universe.
- [ ] 3.1.3. Implement the `validate` command report. Requires 3.1.2. See
  `docs/splitters-design.md` §7.2 and §10.
  - Report rematch, candidate replay, residual replay, and optional-check
    results separately.
  - Include enough fragment and proposal identifiers for an agent to act on the
    failure.
  - Success criteria: failed validation produces structured diagnostics without
    mutating the main worktree.

### 3.2. Add policy-driven checks

This step makes validation useful as a delivery guard rather than only a patch
applicability test. It matters because semantic correctness is intentionally
delegated to caller-supplied commands.

- [ ] 3.2.1. Add `--check` execution in both validation environments. Requires
  3.1.3. See `docs/splitters-design.md` §7.2 and §10.
  - Run the supplied command once in the candidate worktree and once in the
    residual worktree.
  - Capture exit status and stderr without contaminating structured stdout.
  - Success criteria: tests prove non-zero command exit causes validation
    rejection with exit code `1`.
- [ ] 3.2.2. Distinguish policy failure from operational failure. Requires
  3.2.1. See `docs/splitters-design.md` §7.2.
  - Treat a failing check command as proposal rejection.
  - Treat spawn failures, missing executables, and worktree setup failures as
    operational errors.
  - Success criteria: integration tests cover both classes of failure and the
    expected exit-code mapping.

## 4. Extraction, subtraction, and publication

### 4.1. Materialize validated proposals as branches

This step converts a validated proposal into a real branch and commit. It
matters because this is the first irreversible local action in the roadmap.

- [ ] 4.1.1. Create the target branch from the selected base ref. Requires
  3.1.3. See `docs/splitters-design.md` §7.3 and §11.
  - Honour explicit base overrides and `base_ref` from the manifest.
  - Honour optional explicit branch names and deterministic defaults.
  - Success criteria: extraction tests prove the new branch tip is rooted at
    the intended base.
- [ ] 4.1.2. Commit the candidate proposal content onto the target branch.
  Requires 4.1.1. See `docs/splitters-design.md` §11.
  - Reuse the validated candidate state rather than recomputing the proposal
    from scratch.
  - Populate commit metadata from the proposal title and body.
  - Success criteria: extracted branch trees match the validated candidate
    state byte-for-byte.
- [ ] 4.1.3. Subtract the exact validated patchset from the current branch.
  Requires 4.1.2. See `docs/splitters-design.md` §11.
  - Apply the proposal patchset in reverse to the current branch.
  - Refuse partial subtraction.
  - Success criteria: extraction tests prove unrelated fragments remain on the
    current branch after subtraction.

### 4.2. Add manifest lifecycle and optional publication

This step closes the operational loop after extraction. It matters because the
tool needs to prevent double extraction and support optional pull request
creation without making the local transaction depend on the network.

- [ ] 4.2.1. Mark extracted proposals in the manifest lifecycle. Requires
  4.1.3. See `docs/splitters-design.md` §11 and §15.
  - Record that a proposal has been extracted.
  - Prevent re-extraction of the same proposal from an unchanged manifest.
  - Success criteria: repeated extraction attempts fail with a clear diagnostic.
- [ ] 4.2.2. Implement optional GitHub publication through `gh`. Requires
  4.1.2. See `docs/splitters-design.md` §6, §7.3, and §12.
  - Call `gh pr create` only after the local extraction succeeds.
  - Pass `--base`, `--head`, `--title`, and `--body` explicitly.
  - Report remote publication failures without rolling back successful local
    extraction.
  - Success criteria: tests or harnesses cover missing `gh`, unauthenticated
    `gh`, and successful publication paths.

## 5. Quality, documentation, and release readiness

### 5.1. Build the regression safety net

This step makes the implementation safe to extend and refactor. It matters
because Splitters manipulates Git state directly, so edge-case regressions
would be costly.

- [ ] 5.1.1. Add unit coverage for core invariants. Requires 2.2.2 and 3.2.2.
  See `docs/splitters-design.md` §13.
  - Cover fingerprint generation, proposal graph validation, overlap
    detection, and manifest round-trips.
  - Add property-style checks where the invariant is more important than one
    fixture layout.
  - Success criteria: each core invariant has a direct, isolated test target.
- [ ] 5.1.2. Add end-to-end fixture repositories for risky change kinds.
  Requires 4.2.2. See `docs/splitters-design.md` §7.1, §10, and §13.
  - Cover pure renames, binary files, mode-only changes, empty-file changes,
    stale manifests, and ambiguous rematches.
  - Cover both successful extraction and rejected validation.
  - Success criteria: fixture names map directly to the design risks they
    protect.

### 5.2. Ship operator-facing and maintainer-facing assets

This step makes the tool usable and releasable once the core behaviour is in
place. It matters last because the guide material and release plumbing must
reflect the actual implemented contracts.

- [ ] 5.2.1. Write the user and maintainer documentation set. Requires 4.2.2.
  See `docs/splitters-design.md` §7, §8, and §11.
  - Add `docs/users-guide.md` covering the `map` to `validate` to `extract`
    workflow.
  - Add `docs/developers-guide.md` covering architecture references, test
    policy, and release workflow.
  - Update `README.md` with installation and quick-start guidance.
  - Success criteria: documentation examples match the implemented command
    surface.
- [ ] 5.2.2. Add CI, release, and packaging support. Requires 5.1.2. See
  `docs/splitters-design.md` §12 and §13.
  - Add GitHub Actions jobs for format, lint, test, and release builds.
  - Ensure release artefacts cover Linux, macOS, and Windows.
  - Add shell completion generation if the final CLI surface warrants it.
  - Success criteria: one tagged release path can build the documented binary
    set without manual patching.
