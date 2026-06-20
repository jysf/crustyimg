# SPEC-036 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-036-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-037**. Read ci.yml /
  deny.toml / justfile: CI runs cargo-deny `check licenses` only. Scope: extend to
  `check advisories bans sources licenses` (+ deny.toml sections + `just deny`);
  consolidate cargo-audit into cargo-deny's RUSTSEC advisories check (not a separate
  job). No new runtime dep; license policy (DEC-018) untouched. A CI/config chore —
  validation is the gate itself. Build prompt at `prompts/SPEC-036-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #40; CI cargo-deny `check licenses` →
  `check advisories bans sources licenses` + `deny.toml` `[advisories]`/`[bans]`/`[sources]`
  + `just deny`. One real finding handled per policy: RUSTSEC-2024-0436 (`paste`
  unmaintained, no safe upgrade) → dated narrow ignore. Fixed an in-flight YAML colon bug.
  16/16 CI green; 404 tests pass. subagent tokens=59326 (~$0.32).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED, no concerns; adversarially confirmed
  the genuine four-check gate (not weakened), `[licenses]` byte-for-byte unchanged, the
  single advisory ignore narrow + dated + justified, no check disabled, no new runtime dep.
  ~50k est.
- [x] **ship** (2026-06-19) — squash-merged PR #40 (6a45630); cost totals + ship reflection
  + archived to `specs/done/`. STAGE-006 backlog #4 complete.
