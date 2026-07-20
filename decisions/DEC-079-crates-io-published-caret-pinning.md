---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-079
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-19
supersedes: DEC-078
superseded_by: null

# Refines AGENTS.md §5 / DEC-011 / DEC-013's exact-pin convention: runtime deps
# are now caret; the committed Cargo.lock is the reproducibility mechanism.
affected_scope:
  - Cargo.toml

tags:
  - dependencies
  - semver
  - cargo-toml
  - crates-io
  - publish
  - pinning
---

# DEC-079: dependency pinning — caret for the (published) library, exact lock for reproducibility

## Decision

1. **Runtime (library-public) dependency requirements use caret** (`^x.y.z`,
   written as the bare version), so consumers can unify. This covers
   `[dependencies]` and both `[target.*.dependencies]` tables (native and
   wasm32).
2. **Reproducibility comes from the committed `Cargo.lock`**, not from
   manifest pins — `cargo install --locked` and our release/build flows
   resolve to the exact locked versions regardless of the caret ranges. The
   reproducible-build thesis (PROJ-007) is intact.
3. **`[dev-dependencies]` may stay exactly pinned** — they never constrain a
   consumer's resolution, and exact dev-deps keep test reproducibility crisp.
4. **This supersedes DEC-078** and refines AGENTS.md §5 / DEC-011 / DEC-013:
   exact manifest pins are no longer the policy for runtime deps of a
   published library; the lock carries reproducibility.

## Context

DEC-078 (shipped hours earlier, SPEC-098) assumed crustyimg was not yet on
crates.io and deferred relaxing the dependency pins to a future publish
(backlog #5). That premise was false, verified 2026-07-19 directly against
crates.io and `gh run list`:

- crustyimg **is published on crates.io — latest 0.4.0, `has_lib: true`**,
  every tag v0.1.0→0.4.0 auto-published by `.github/workflows/publish-crates.yml`
  (`cargo publish --locked` on every `v*` tag, all runs succeeded, first
  publish 2026-07-04).
- So the 30 exact `=` pins were live on a published library. Anyone running
  `cargo add crustyimg` got a lib with exact `=` requirements on ~23 runtime
  deps, forcing version-unification conflicts — the exact downstream harm
  DEC-078 called hypothetical, and it was real, on 0.4.0.

DEC-078's *direction* was right (binary reproducibility via the lock;
library-public deps should be caret) — only its premise ("no downstream Cargo
consumer exists") and its "defer, don't migrate" conclusion were inverted by
the facts.

**How the error slipped through:** the pre-launch audit's D4 asserted "not on
crates.io" citing stale docs (DEC-040/DEC-041) and DEC-078's verify checked
the decision against the audit, not against crates.io directly — graded
against the wrong oracle. The one-command check that would have caught it:
`gh run list --workflow=publish-crates.yml`.

## Alternatives Considered

- **Leave DEC-078 in place, ship the caret migration as a plain fix with no
  new decision.** Rejected: the false premise and the correction are
  themselves worth recording — a future reader hitting the same "check the
  audit, not the source" trap benefits from the paper trail DEC-078 →
  DEC-079 leaves.
- **Delete/rewrite DEC-078 in place.** Rejected: supersession preserves the
  error + correction as its own artifact, rather than erasing evidence of how
  a plausible-but-unchecked claim reached a shipped decision.
- **Caret-migrate now (chosen).** Matches the facts on the ground: the
  library is live, the harm is live, so the fix ships now instead of waiting
  for a publish event that already happened four times.

## Consequences

- **Positive:** the manifest that ships with the *next* release (e.g. 0.4.1 /
  0.5.0) is consumer-friendly. No resolved version changes now — the lock is
  unchanged, since caret is strictly looser than the prior exact requirements.
- **Negative:** none identified; this is a pure loosening with an unchanged
  lock.
- **Neutral:** `[dev-dependencies]` stay exactly pinned — they do not
  constrain a library consumer's resolution, so this decision does not reach
  them.

This refines, not contradicts, AGENTS.md §5 and DEC-011/DEC-013: caret is now
the policy for runtime deps of a published library; the committed lock
carries reproducibility. AGENTS.md §5 repoints to this decision.

## Validation

Right if: `Cargo.lock` stays byte-unchanged through the migration (proven by
`git diff --stat Cargo.lock` = empty), the full CI matrix (native, avif, lean,
MSRV, cargo-deny, wasm, clippy, fmt) stays green with no `--locked` flag in
CI, and `cargo publish --locked --dry-run` succeeds. Revisit if a future
runtime dependency needs an exact pin for a documented compatibility reason —
that would need its own decision, not a silent reversion.

## References

- Related specs: SPEC-099 (this decision), SPEC-098 (created DEC-078).
- Related decisions: DEC-078 (superseded by this decision), DEC-011 (`viuer`
  display, the exact-pin pattern's origin), DEC-013 (`kamadak-exif`, same
  pattern), DEC-040 (`cargo-dist` release pipeline), DEC-041 (release
  channels — corrected alongside this decision).
- External docs: `docs/research/proj-008-rust-directives-audit.md` (D4
  section — annotated with a correction pointing here).
