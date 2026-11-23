---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-116
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-11-22
# Stands beside DEC-035 (symlinked destinations: the guard still runs first),
# DEC-044 (the out-dir is created first) and DEC-059 (the lockfile). Amends none.
supersedes: null
superseded_by: null

affected_scope:
  - src/sink/mod.rs
  - src/sink/publish.rs
  - src/cli/ops.rs
  - src/cli/build.rs
  - SECURITY.md
  - docs/cli-reference.md
  - docs/api-contract.md
  - tests/atomic_write.rs

tags:
  - sink
  - writes
  - durability
---

# DEC-116: A file write is all or nothing

## Decision

- Every file crustyimg writes for the user is staged in a new file in the destination's own
  directory, created exclusively (`create_new`, which refuses an existing path and never follows a
  symlink), and renamed over the destination only after every byte is written. That covers image
  outputs on every verb, `build` outputs, `meta` writing in place, `edit --save-recipe` and the
  build lockfile. One helper does it, `src/sink/publish.rs`.
- A failed write exits non-zero and leaves the destination's old bytes untouched, or leaves no file
  if there was none. No staged file survives an error the process returns. A killed process can
  leave one, named `.<name>.crustyimg-<pid>-<n>.tmp`; its `.tmp` extension keeps it out of
  crustyimg's folder and glob inputs.
- The symlink and overwrite guards (DEC-035) run before anything is created. An existing
  destination this user cannot open for writing is refused, as it was before writes were staged,
  so `-y` never replaces a read-only file.
- `-y` replaces the file at that name with a new file. Its permission bits carry over. Its owner
  (unless crustyimg runs as root), ACLs, extended attributes (on macOS, Finder tags) and hard links
  do not.
- A destination whose directory can't take a new file fails with exit 5 and names the directory.
  There is no in-place fallback: it would bring back the write that destroys the old file.
- Replacing a file needs room for the old and the new file at once. On a full disk that fails
  loudly and keeps the old file.
- The guarantee covers errors and killed processes. It does not cover power loss: nothing calls
  `fsync` on the file or its directory. (`sync_all` is `F_FULLFSYNC` on macOS, about 5 ms a file.)
- stdout is a stream: its errors exit non-zero, and there is nothing to stage. The build cache
  keeps its own staged write (`src/build/cache.rs`), whose entries are hash-checked on read.

## Context

Every sink arm opened the destination with `truncate(true)` and wrote through an 8 KiB
`BufWriter` that nothing flushed. An output under 8 KiB stayed in the buffer and its flush error
was dropped on `Drop`, so a failed write exited 0 over a short file. A larger output returned its
error, but the old file had been truncated before the first byte, so a failure destroyed it. That
includes `-y` over the input itself and `meta` writing in place. `edit --save-recipe` truncated the
same way and had no overwrite guard. The design and the driven premise check are in the spec.

## Alternatives Considered

- **Flush the `BufWriter` and keep writing in place.**
  - What it is: report the flush error.
  - Why rejected: fixes the silent half only; the old file is still gone on any failure.
- **The `tempfile` crate.**
  - What it is: `NamedTempFile::persist`.
  - Why rejected: its default mode is `0o600`, it is a native-only dev-dependency and would need a
    target-gated normal dependency, and its API hides the steps a fault-injection seam fails one by
    one.
- **Fall back to writing in place when the directory is read-only.**
  - What it is: keep today's behaviour for that case.
  - Why rejected: it brings back the destroy-on-failure path.
- **Option C (chosen): hand-rolled stage and rename on `std`.**
  - What it is: about 170 lines, behind a `StageFs` seam so a test can fail each step.
  - Why selected: no new dependency, the mode rule is std's own default, and each failure point is
    testable.

## Consequences

- **Positive:** a failed write never reports success and never costs the user the file they had.
- **Negative:** `-y` makes a new file (owner, ACLs, xattrs and hard links are not kept); replacing
  needs room for both files; a read-only directory refuses `-y`; a killed process can leave a
  `.crustyimg-*.tmp` file.
- **Neutral:** `edit --save-recipe` now needs `-y` to replace an existing recipe.

## Validation

`tests/atomic_write.rs` and the unit tests in `src/sink/publish.rs`, with the mutation table in the
spec. Revisit if users ask for power-loss durability (add `fsync`, and measure the macOS cost
first) or for stale staged files to be swept.

## References

- Related specs: SPEC-155, SPEC-157 (destination identity builds on this helper), SPEC-158
- Related decisions: DEC-035, DEC-044, DEC-059, DEC-061
