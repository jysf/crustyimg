# SPEC-040 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-040-<cycle>.md`.

## Instructions

- [x] **design** (2026-06-19, claude-opus-4-8) — Spec + DEC-039 (clap_complete
  =4.6.5, probe-verified against pinned clap =4.6.1; subcommand-over-build.rs;
  deny green) + build prompt authored. STAGE-007 #6: user-facing README rewrite
  (install/usage/completions/License-corrected) + `completions <shell>` subcommand.
  Safe (no outward-facing action).
- [ ] **build** — `prompts/SPEC-040-build.md` (run on Sonnet, fresh session).
- [ ] **verify** — independent Explore subagent (Opus); incl. lean build + deny.
- [ ] **ship** — pause for maintainer before merge.
