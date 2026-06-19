---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-035
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - src/sink/mod.rs
  - src/source/mod.rs

tags:
  - security
  - hardening
  - path-traversal
  - symlink
  - untrusted-input
---

# DEC-035: path/symlink traversal hardening — Source and Sink

## Decision

Close the two residual path/symlink traversal gaps on the I/O boundary
(STAGE-006, consolidating the `untrusted-input-hardening` constraint and DEC-010):

1. **Reject a symlinked output destination (Sink).** Before opening any file
   destination for write — `Sink::File` and `Sink::Dir`, in both `write` and
   `write_bytes` — reject the final path if it is itself a symlink
   (`std::fs::symlink_metadata(path).file_type().is_symlink()` →
   `SinkError::Traversal`). This is enforced **regardless of `--yes`**: `--yes`
   authorizes overwriting *the file you named*, not following a planted symlink
   out of `--out-dir`. Without this, `OpenOptions::write().create().truncate()`
   follows a pre-existing symlink (e.g. `out/photo.png` → `/etc/important`) and a
   `--yes` batch could truncate a file outside the target directory. `safe_join`
   already validates the file *name* (no `..`, separators, absolute, empty);
   this adds the missing check on a symlink *at* the destination.

2. **Never silently disable the Source glob escape-check.** `resolve_glob`'s
   symlink-escape guard is anchored to a canonicalized root; previously, if the
   glob base could not be canonicalized the root became `None` and the check was
   **skipped entirely** (the SPEC-004 defensive gap noted in DEC-010). Anchor the
   root robustly — fall back to the canonicalized current directory (where a
   relative glob resolves) when the base canonicalize fails — so the per-entry
   `canonicalize(entry).starts_with(root)` check **always runs**. (An absolute
   pattern whose base does not exist matches nothing, so the fallback only
   affects relative patterns, for which the cwd is the correct anchor.) The
   directory branch (`resolve_directory`) already anchors robustly and is the
   reference behavior; this brings glob to parity.

No new dependency. These are tightenings of existing code, recorded as a policy
because "reject-symlink-destination even under `--yes`" and "escape-check is
never bypassed" are non-obvious, security-relevant invariants future specs and
the threat-model pass (STAGE-006) must preserve.

## Context

STAGE-006 is the MVP exit gate: consolidate and verify the as-built
untrusted-input hardening. The Source/Sink path handling was built with traversal
defenses (DEC-010: directories non-recursive, symlink-escape entries skipped;
SPEC-005: `safe_join` rejects `..`/separators/absolute names + an overwrite
guard), but two residual gaps remained: the Sink followed a symlink *at* the
output path (a write-through-symlink escape, reachable under `--yes`), and the
glob branch silently disabled its escape guard when it could not anchor a root.
This DEC closes both with minimal, `std`-only changes and pins the invariants.

## Alternatives Considered

- **Allow a symlinked destination under `--yes`** — rejected: `--yes` is about
  clobbering a named file, not about following a link out of the tree; a planted
  symlink is exactly the attack. Refusing it is the safe default.
- **Canonicalize the final destination and re-check containment instead of
  rejecting symlinks** — heavier and racy (TOCTOU between check and open) for a
  non-existent output; a direct `symlink_metadata` reject is simpler and
  sufficient (we never want to write *through* a symlink).
- **Make the glob branch skip every symlinked entry when it cannot anchor** —
  rejected: changes legitimate behavior; anchoring to the cwd keeps normal globs
  working while never disabling the guard.
- **Add `walkdir`/a path crate** — unnecessary; `std::fs::{symlink_metadata,
  canonicalize}` cover it. (DEC-010 keeps the dependency surface minimal.)

## Consequences

- **Positive:** A planted symlink in `--out-dir` can no longer redirect a write
  outside it, even under `--yes`; a glob can no longer pull a symlinked file from
  outside the globbed tree into a batch. Source and Sink now have parity on the
  escape check. No new dependency.
- **Negative:** A user who deliberately points an output filename at a symlink
  must write to the real path instead (rare; the safe default). The glob cwd
  fallback assumes the cwd is canonicalizable (effectively always true).
- **Neutral:** `safe_join`'s name validation and the overwrite guard are
  unchanged; this is additive. Directory-source behavior is unchanged (already
  robust) and merely gains explicit tests.

## Validation

Right if: a `--out-dir` containing a symlink at the templated output name is
refused with `SinkError::Traversal` (target file untouched) even with `--yes`; a
glob/dir whose match would resolve (via symlink) outside the root skips that
entry; and normal in-tree writes/globs are unaffected (SPEC-034 tests +
`/security-review`). Revisit if: a `--follow-symlinks` opt-in is ever wanted
(its own DEC), or TOCTOU hardening (open with `O_NOFOLLOW`) becomes warranted.

## References

- Related specs: SPEC-034 (this hardening); SPEC-004 (Source), SPEC-005 (Sink —
  `safe_join`/overwrite guard this extends)
- Related decisions: DEC-010 (glob + non-recursive dirs; the escape-check origin),
  DEC-007 (typed errors), DEC-034 (decode limits — the prior STAGE-006 item)
- Constraints: `untrusted-input-hardening` (blocking), `no-unwrap-on-recoverable-paths`
- External docs: https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html
