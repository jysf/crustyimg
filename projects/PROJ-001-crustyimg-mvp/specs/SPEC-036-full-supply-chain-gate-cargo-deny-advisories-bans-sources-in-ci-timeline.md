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
- [ ] **build** (Sonnet) — extend the gate; run `cargo deny check …` clean; see prompt.
- [ ] **verify** (Opus, Explore) — confirm the CI command, deny.toml sections, the clean
  check, and that no whole check was disabled to pass.
- [ ] **ship** — pause for the user before merge.
