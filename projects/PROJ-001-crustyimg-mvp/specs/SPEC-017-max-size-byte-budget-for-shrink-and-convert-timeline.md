# SPEC-017 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-017-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. Scope confirmed with the user: **quality-only v1** (dimension-reduction fallback deferred to a follow-up). Reuses the shipped SPEC-016 `src/quality` search via a generic `search_threshold` core (highest quality ≤ budget); generalizes the CLI `auto` hook to an `AutoQuality` enum. **No new DEC** (byte-budget is a DEC-019 dual). Build prompt at `prompts/SPEC-017-build.md`. Completed 2026-06-16.
- [x] **build** — PR #20 opened. `search_threshold` refactor (SPEC-016 `quality::tests` preserved) + `search_jpeg_under_size`/`auto_jpeg_under_size`/`jpeg_size_at` + `AutoQuality` enum + `parse_size`/`fmt_bytes` + `--max-size` on shrink/convert; 17 new tests, full suite 238 green, all 5 gates pass. Run by the orchestrator (Opus) directly (subagent Bash blocked). Branch `feat/spec-017-max-size-byte-budget-for-shrink-and-convert`. 2026-06-16.
- [ ] **verify** — confirm every named test exists + runs; the SPEC-016 `quality::tests` stay green through the refactor; `just deny` green (no new dep); all 5 gates + 3-OS CI + cost-capture audit.
- [ ] **ship** — orchestrator bookkeeping on `main` after merge (real cost numbers; PAUSE for the user before merge/ship).
