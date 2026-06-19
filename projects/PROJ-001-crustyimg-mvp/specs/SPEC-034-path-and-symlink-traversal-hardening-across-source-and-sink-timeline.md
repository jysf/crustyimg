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
- [ ] **build** (Sonnet) — make the failing tests pass; see `prompts/SPEC-034-build.md`.
- [ ] **verify** (Opus, Explore) — independent read-only review + gate re-run (incl. lean).
- [ ] **ship** — pause for the user before merge.
