---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-117
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-11-25
# Stands beside DEC-035 (per-file link refusal stays the last layer), DEC-116 (the write
# helper) and amends DEC-061's symlinked-out-dir residual (closed here). Supersedes none.
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/build.rs
  - src/cli/mod.rs
  - src/build/mod.rs
  - tests/build_destinations.rs
  - tests/build_watch.rs
  - docs/cli-reference.md
  - SECURITY.md
  - skills/crustyimg/SKILL.md

tags:
  - build
  - destinations
  - security
  - lockfile
  - watch
---

# DEC-117: A build destination has one identity, resolved before anything is written

## Decision

A `build` destination has one identity, computed once at planning, before the lockfile is read, the cache
is opened or anything is written. Each target's `out` is resolved against the build root (the working
directory, canonicalized): `.` components are dropped, the longest leading part that exists is
canonicalized (links followed, `..` after a link applied as the kernel applies it, on-disk case taken), and
the part that does not exist yet is appended with `..` applied lexically. The output's file name is joined
onto it as written and is never followed: planning compares it as written (so a name `src/a.{ext}` is
checked as `<out>/src/a.png` and can be refused as an overlap, exit 2), and `safe_join` refuses a name
with a separator at write time (exit 6). Planning, the collision check, the overlap check, the sink, the
lockfile and the `--watch` exclusion all use this identity.

1. Collisions and overlaps compare the identity after `str::to_lowercase`, on every OS. Two outputs that
   differ only in letter case are one output, even on a case-sensitive filesystem. `to_lowercase` does not
   merge `ß` and `SS`; see "Not covered".
2. A destination may not be a file the build reads: any target's resolved source, any target's recipe
   file, any file a recipe step names through an asset key (a watermark's `image` or `font`), the manifest,
   or the lockfile. So one target's output may not be another target's source. A destination may not lie
   under `.crustyimg/`. An output name ending in `{ext}` is compared under every extension the build can
   write, unless the target's format is pinned. An output that already exists and is the same file as one of
   these (device and inode, links followed; Unix) is refused whatever its name, so `ß` for `SS`, NFC for
   NFD and a hard link are caught for a file that exists. A target's glob or folder source may not
   list one of its planned outputs on the next build.
3. An `out` that resolves outside the build root is refused. A link to a directory inside the tree is
   allowed and is de-aliased by the identity. There is no opt-in for an out-of-tree link; build inside the
   tree and copy.
4. Every refusal above exits 2 and writes nothing, the same class as a malformed manifest.
5. The lockfile records each output by its root-relative identity with `/` separators. A lockfile
   written before this decision with a `..`, link or off-case spelling drifts once, on paths only.
6. The sink creates the resolved directory, resolves it again, and refuses the write if it moved (exit 6
   from `build`, which counts it as a failed input; exit 5 is the sink's code on `apply`).
   DEC-035's per-file link refusal stays the last layer. A swap after that re-check is not covered.
7. `--watch` excludes the resolved `out` of every target, and an event on a file the build reads always
   triggers a rebuild, even inside an excluded directory (rule 2 guarantees the build's own writes are never
   such a file).

Not covered: two outputs whose names differ only in Unicode normalization (NFC and NFD) or in a case
`to_lowercase` does not fold (`ß` and `SS`) are one file on macOS and two identities here, on any build, new
file or not; that needs normalization tables `std` does not have. The check compares outputs with the files
the build reads, never two outputs with each other by file identity, so the output-to-output collision is
all that is left: an output over a file the build reads is caught by file identity. Destinations of `apply` and the
single-file verbs (`-o`, `--out-dir`) are not resolved this way.

## Context

The 2026-11-10 external code review (`docs/reviews/2026-11-10-external-code-review.md`) raised two P1
findings. SPEC-157's frame reproduced both on `main`: `out = "src"` replaced the source at exit 0, and
`alias/../dist` gave one file, two lockfile records and a `--check` that passed. DEC-061 had accepted the
symlinked-out-dir case as a residual. The design and the driven premise check are in the spec.

## Alternatives Considered

- **A lexical-only identity.**
  - Why rejected: wrong, driven. `link/..` is the link target's parent, not the current folder.
- **Refuse any link in `out`.**
  - Why rejected: breaks in-tree conveniences that work today.
- **Keep out-of-tree links.**
  - Why rejected: keeps the reviewer's write outside the tree.
- **Full Unicode case folding or normalization.**
  - Why rejected: needs tables `std` does not have, so a new dependency. Filed in STAGE-063's backlog.
- **Option chosen:** one resolver and one planning pass on `std`, plus a same-file (device and inode)
  check for an output that already exists.

## Consequences

- **Positive:** a build can no longer overwrite a file it reads, write two outputs to one file, or write
  outside the tree through a link. A source that would list its own output is refused on the first build.
  `--watch` no longer loops through a linked `out` and no longer misses an edit to a source inside `out`.
  DEC-061's symlinked-out-dir residual is closed.
- **Negative:** an `out` that resolves outside the tree through a link is refused, with no opt-in. A
  lockfile written before this with a `..`, link or off-case spelling reports path drift once under
  `build --check`; one `crustyimg build` refreshes it.
- **Neutral:** `apply` and the single-file verbs keep their own destination rules.

## Validation

`tests/build_destinations.rs`, `tests/build_watch.rs` and the unit rows in `src/cli/build.rs`, with the
mutation table (M1 to M19) in the spec. Revisit if NFC/NFD or `ß`/`SS` output-to-output collisions are
reported on macOS (STAGE-063 backlog), or if a user needs an out-of-tree `out`.

## References

- Related specs: SPEC-157 (PR #270), SPEC-155 (the write helper this sits beside)
- Related decisions: DEC-035, DEC-057, DEC-059, DEC-060, DEC-061, DEC-116
