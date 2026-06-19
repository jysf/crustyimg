---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-034
  type: story                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-035, DEC-010, DEC-007]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - clippy-fmt-clean
    - every-public-fn-tested
  related_specs: [SPEC-004, SPEC-005, SPEC-033]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  Second STAGE-006 hardening item: closes the write-through-symlink (Sink) and
  glob-escape-bypass (Source) traversal gaps so an untrusted directory/glob or a
  planted symlink can't redirect reads or writes outside the target tree.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-034: path and symlink traversal hardening across source and sink

## Context

**The second STAGE-006 hardening item.** SPEC-033 bounded decode; this spec
bounds the *path* surface — where untrusted directories, globs, and planted
symlinks could redirect reads or writes outside the intended tree. The I/O
boundary already has real defenses (DEC-010: non-recursive dirs, symlink-escape
entries skipped; SPEC-005: `safe_join` rejects `..`/separators/absolute output
names + an overwrite guard), but **two residual gaps** remain, both named by the
stage backlog:

1. **Sink — write-through a symlinked destination.** `Sink::write` /
   `write_bytes` open the final path with `OpenOptions::write().create().
   truncate()`, which **follows a pre-existing symlink at that path**.
   `safe_join` validates the file *name* but nothing rejects a symlink *at* the
   destination, so a planted `out-dir/photo.png` → `/etc/important` symlink lets a
   `--yes` batch truncate a file **outside** `--out-dir`.
2. **Source — glob escape-check bypass.** `resolve_glob` anchors its
   symlink-escape guard to a canonicalized root, but sets the root to `None` and
   **skips the check entirely** when the glob base can't be canonicalized — the
   SPEC-004 defensive gap DEC-010 explicitly flags. The directory branch is
   already robust; glob should match it.

This spec closes both (DEC-035), with `std`-only changes and traversal tests
across Source and Sink. Parent: `STAGE-006` (backlog item #2). Governing:
**DEC-035** (the hardening policy), **DEC-010** (the escape-check origin),
**DEC-007** (typed errors). No new dependency.

## Goal

Reject a symlink at any file output destination (`Sink::File`/`Dir`, `write` and
`write_bytes`) **regardless of `--yes`**, and make the Source glob symlink-escape
check **always run** (never silently disabled) — so an untrusted glob/directory
or a planted symlink cannot redirect reads or writes outside the target tree,
each surfaced as a typed error.

## Inputs

- **Files to read:**
  - `src/sink/mod.rs` — `safe_join`, `guard_overwrite`, `Sink::write` (File/Dir
    arms), `Sink::write_bytes` (File/Dir arms), `SinkError::Traversal`.
  - `src/source/mod.rs` — `resolve_glob` (the `root_opt = None` bypass),
    `resolve_directory` (the robust reference behavior), `glob_base_dir`.
  - `decisions/DEC-035` (the policy this implements), `DEC-010`, `DEC-007`.
- **External APIs:** `std::fs::symlink_metadata` (does NOT follow the final
  symlink; `.file_type().is_symlink()`), `std::fs::canonicalize`. No new crate.
  Docs: https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html
- **Related code paths:** `src/sink/`, `src/source/`, `tests/`.

## Outputs

- **Files modified:**
  - `src/sink/mod.rs` — add `fn reject_symlink_destination(path: &Path) ->
    Result<(), SinkError>` (returns `Traversal` if the path is a symlink, else
    `Ok`). Call it in the File and Dir arms of BOTH `write` and `write_bytes`,
    before opening the file (for Dir, after `safe_join`). Unit-tested.
  - `src/source/mod.rs` — in `resolve_glob`, replace
    `let root_opt = std::fs::canonicalize(&base).ok();` with a robust anchor:
    `std::fs::canonicalize(&base).or_else(|_| std::fs::canonicalize("."))` so the
    per-entry `starts_with(root)` escape check always runs (the `None` bypass is
    removed). Unit/integration-tested.
  - `docs/api-contract.md` — note the symlinked-destination rejection (exit 5).
    (Done at design.)
  - `SECURITY.md` — mark the path-traversal / symlink rows as closed (SPEC-034 /
    DEC-035). (Done at design.)
- **New exports:** none required (helpers internal; `reject_symlink_destination`
  may be `pub(crate)` or private + in-module unit-tested).
- **Database changes:** none.

## Hardening policy (PINNED — DEC-035)

- **Symlinked destination (Sink):** before opening a `Sink::File`/`Sink::Dir`
  destination for write, `reject_symlink_destination(path)` returns
  `SinkError::Traversal(path)` if `std::fs::symlink_metadata(path)` reports the
  path is a symlink. **Enforced regardless of `Overwrite`** (`--yes` overwrites a
  named file; it does not authorize following a link out of the dir). A
  non-existent destination (`symlink_metadata` errors) is fine → proceed. Applies
  to all four file-producing arms (`write` File, `write` Dir, `write_bytes` File,
  `write_bytes` Dir). Order: for Dir, `safe_join` → `reject_symlink_destination`
  → `guard_overwrite` → open. Maps to **exit 5** (output write refused — same as
  the other `Traversal`/`AlreadyExists` cases).
- **Glob escape-check (Source):** `resolve_glob`'s root is anchored with a cwd
  fallback so it is effectively always `Some`, and the existing per-entry
  `canonicalize(entry).starts_with(root)` check runs for every entry — the
  guard is **never bypassed**. An entry whose real (canonicalized) path escapes
  the root, or that dangles, is skipped (unchanged behavior; now always applied).
- **No behavior change for legitimate paths:** in-tree writes and normal
  globs/dirs are unaffected; only symlinked destinations and escaping glob
  entries change outcome.

## Acceptance Criteria

- [ ] Writing a `Sink::File` whose path is a symlink → `SinkError::Traversal`,
  even with `Overwrite::Allow` (`--yes`); the symlink's target file is NOT
  modified.
- [ ] Writing a `Sink::Dir` whose templated output name resolves to a pre-existing
  symlink in `dir` → `SinkError::Traversal` (both `write` and `write_bytes`),
  even with `--yes`; the target outside `dir` is untouched.
- [ ] A normal (non-symlink) in-dir destination still writes successfully (no
  regression) for `write` and `write_bytes`.
- [ ] `reject_symlink_destination` returns `Ok` for a non-existent path and a
  regular file, and `Traversal` for a symlink (unit-tested directly).
- [ ] A glob that matches a symlink pointing OUTSIDE the glob root skips that
  entry (it is not yielded), and the escape check runs even when the base would
  previously have yielded `root_opt = None`.
- [ ] A directory source containing a symlink to a file outside the dir skips it
  (reference behavior, now explicitly tested).
- [ ] A glob/dir of normal in-tree images still resolves all of them (no
  regression).
- [ ] The Sink traversal rejection maps to exit **5** at the CLI
  (`CliError::Sink(SinkError::Traversal(_)).code() == 5` — already mapped; confirm).
- [ ] `cargo deny` green; the **lean build** compiles; no new dependency; no
  `unwrap`/`expect`/`panic!` on the new non-test paths.

## Failing Tests

Written during **design**, BEFORE build. Use real symlinks via
`std::os::unix::fs::symlink` — gate the symlink tests with
`#[cfg(unix)]` (Windows symlink creation needs privileges; the hardening logic is
cross-platform but the *test* uses Unix symlinks). Fixtures: tiny PNGs via the
`image` crate into `tempfile::tempdir()`.

- **`src/sink/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"reject_symlink_destination_ok_for_regular_and_missing"` — returns `Ok` for a
    non-existent path and for a regular file in a tempdir.
  - `#[cfg(unix)] "reject_symlink_destination_rejects_symlink"` — create a symlink
    in a tempdir; `reject_symlink_destination(link)` → `Err(SinkError::Traversal(_))`.
  - `#[cfg(unix)] "write_file_through_symlink_is_rejected_even_with_yes"` — create
    an outside-target file + a symlink `link.png` → it inside a tempdir; a
    `Sink::File { path: link.png }` `write(..., Overwrite::Allow, ...)` →
    `Err(Traversal)`; assert the outside target's bytes are unchanged.
  - `#[cfg(unix)] "write_bytes_dir_through_symlink_is_rejected"` — `Sink::Dir`
    whose templated name is a symlink in `dir` → `write_bytes(..., Allow, ...)` →
    `Err(Traversal)`; outside target unchanged.
  - `"write_dir_normal_destination_still_succeeds"` — a `Sink::Dir` to a plain name
    writes the file (no regression; `Overwrite::Forbid`).
- **`src/source/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `#[cfg(unix)] "glob_skips_symlink_escaping_root"` — tempdir `root/` with an
    in-tree image and a symlink `root/escape.png` → an image OUTSIDE `root/`;
    `resolve("root/*.png", …)` yields only the in-tree image (the escaping symlink
    is skipped). (Proves the guard runs.)
  - `#[cfg(unix)] "directory_skips_symlink_escaping_root"` — same shape via
    `resolve("root", …)` (directory branch); the escaping symlink is skipped.
  - `"glob_resolves_all_in_tree_images"` — a tempdir of 2–3 plain PNGs +
    `resolve("dir/*.png", …)` yields all of them sorted (no regression).
- **`tests/source.rs` / `tests/sink.rs` (integration, if the in-module unit tests
  don't already drive the public API)** — add an end-to-end symlink-escape case
  only if the unit coverage above leaves a gap (avoid duplicate coverage).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-035` — the hardening policy: reject symlinked output destinations even
  under `--yes`; the Source glob escape-check is always anchored (never bypassed).
  **Implement exactly this.**
- `DEC-010` — the escape-check origin (glob + non-recursive dirs, symlink-escape
  entries skipped); this spec brings glob to parity with the robust dir branch.
- `DEC-007` — typed errors; no `unwrap`/`expect`/`panic!` on the new non-test
  paths. Use the existing `SinkError::Traversal` / `SourceError` variants.

### Constraints that apply

- `untrusted-input-hardening` (blocking) — the path/symlink half of the
  constraint (decode limits were SPEC-033).
- `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, `every-public-fn-tested`.

### Prior related work

- `SPEC-004` (shipped) — `resolve`/`resolve_glob`/`resolve_directory` + the
  symlink-escape skip; `resolve_glob`'s `root_opt = None` bypass is the gap.
- `SPEC-005` (shipped) — `safe_join` (name validation) + `guard_overwrite`; this
  spec adds the missing symlink-AT-destination check alongside them.
- `SPEC-033` (shipped) — the prior STAGE-006 item (decode limits); same
  reject-with-a-typed-error, hardening-at-a-choke-point posture.

### Out of scope (for this spec specifically)

- A `--follow-symlinks` opt-in (its own DEC if ever wanted).
- TOCTOU hardening via `O_NOFOLLOW`/`openat` — `symlink_metadata`-then-open is
  sufficient for the threat model here; deeper TOCTOU work is not in scope.
- Recursive directory traversal / `walkdir` (DEC-010 keeps dirs non-recursive).
- Security-grade recipe validation, `cargo audit`/`deny` in CI, the threat-model
  verification pass — later STAGE-006 backlog items.

## Notes for the Implementer

- **`symlink_metadata` does NOT follow the final component** (unlike `metadata`/
  `exists`), so `symlink_metadata(path).map(|m| m.file_type().is_symlink())` is the
  correct symlink-at-destination test. A missing path → `Err` → treat as "not a
  symlink" (`Ok(())`), letting the normal create proceed.
- **Apply the check in all four arms:** `Sink::write` File + Dir, and
  `Sink::write_bytes` File + Dir. For File the path is `path`; for Dir it is the
  `full_path` returned by `safe_join`. Put it right before `guard_overwrite` (or
  immediately after it — either order is fine since both precede `open`).
  **Enforce regardless of `overwrite`** — do NOT gate it behind `Overwrite::Allow`.
- **Glob anchor fix is one line:** `let root_opt = std::fs::canonicalize(&base)
  .or_else(|_| std::fs::canonicalize(".")).ok();`. Keep the existing `if let
  Some(ref root) = root_opt { … }` per-entry check; with the fallback it is
  effectively always taken. Do NOT change `resolve_directory` (already robust) —
  just add its test.
- **Honest note on the glob fix (for the reviewer):** the `root_opt = None`
  bypass is *nearly unreachable behaviorally* — if the glob base can't be
  canonicalized it usually doesn't exist, so the pattern matches nothing anyway.
  The anchor fix is **defense-in-depth** that removes the only branch by which the
  escape guard could be skipped. The glob/dir symlink-escape tests therefore pin
  the security *property* (an entry resolving outside the root is skipped) — they
  are valuable regression guards and may already pass against the canonicalizable
  base, which is expected; do not contrive an unreachable failing case for them.
- **Symlink tests are Unix-gated** (`#[cfg(unix)]` + `std::os::unix::fs::symlink`).
  The production hardening code itself is cross-platform (`symlink_metadata` /
  `canonicalize` exist on Windows); only the test fixtures need Unix symlinks.
- Reuse `SinkError::Traversal` (exit 5, already mapped in `CliError::code`) — no
  new error variant or exit code. Confirm the existing mapping with a CLI unit
  test if one isn't already present.
- Run clippy right after the doc comments (the SPEC-031 `doc_lazy_continuation`
  lesson) and **run the lean build** (`cargo build --no-default-features`).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
