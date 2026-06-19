# SPEC-034 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-034-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Hardening policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-035**. Read Source + Sink:
  identified the two residual gaps — Sink follows a symlink AT the output destination
  (write-through escape, reachable under `--yes`) and the glob escape-check is skipped
  when the base can't canonicalize (`root_opt=None`, the SPEC-004 gap). Fix: a
  `reject_symlink_destination` helper on all four file-producing arms (reject regardless
  of `--yes`) + a one-line cwd-anchor fallback on the glob root. `std`-only, no new dep.
  Updated SECURITY.md + api-contract. Build prompt at `prompts/SPEC-034-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #38; `reject_symlink_destination` on all 4
  Sink file arms (enforced even with `--yes`) + glob root cwd-anchor fallback. 396 tests
  green (5 sink + 3 source new, Unix-gated); clippy/fmt/lean/deny clean. One test-only
  deviation (macOS `/var` canonicalization). subagent tokens=75724 (~$0.41).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED, no concerns; adversarial gap-hunt
  of every write/open path confirmed all 4 arms guarded + not gated behind `--yes`;
  surfaced the `--save-recipe` raw-write follow-up (deferred to backlog #5). Gates re-run
  green (396 tests). ~55k est.
- [x] **ship** (2026-06-19) — squash-merged PR #38 (21ec97b); cost totals + ship reflection
  + archived to `specs/done/`. STAGE-006 backlog #2 complete.
