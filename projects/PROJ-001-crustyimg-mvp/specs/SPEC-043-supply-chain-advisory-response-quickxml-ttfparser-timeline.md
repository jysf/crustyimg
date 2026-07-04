# SPEC-043 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-043-<cycle>.md`.

## Instructions

- [x] **design** (2026-07-03, claude-opus-4-8) — Spec + DEC-042 (accept 3 ambient
  RustSec advisories with revisit triggers; reachability assessment for the quick-xml
  vulns) + Sonnet build prompt. Repairs the supply-chain gate red on `main`. Config
  only (3 documented `deny.toml` ignores); no code/dep change. Unblocks SPEC-042 + v0.1.0.
- [ ] **build** — `prompts/SPEC-043-build.md` (run on Sonnet, fresh session).
- [ ] **verify** — independent Explore subagent (Opus); scrutinize the reachability
  reasoning + confirm `cargo deny` green + no over-broad ignore / no `warn` downgrade.
- [ ] **ship** — pause for maintainer before merge. Then rebase SPEC-042 (#46) green.
