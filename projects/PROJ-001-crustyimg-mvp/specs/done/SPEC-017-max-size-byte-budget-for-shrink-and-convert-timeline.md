# SPEC-017 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-017-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. Scope confirmed with the user: **quality-only v1** (dimension-reduction fallback deferred to a follow-up). Reuses the shipped SPEC-016 `src/quality` search via a generic `search_threshold` core (highest quality ≤ budget); generalizes the CLI `auto` hook to an `AutoQuality` enum. **No new DEC** (byte-budget is a DEC-019 dual). Build prompt at `prompts/SPEC-017-build.md`. Completed 2026-06-16.
- [x] **build** — PR #20 opened. `search_threshold` refactor (SPEC-016 `quality::tests` preserved) + `search_jpeg_under_size`/`auto_jpeg_under_size`/`jpeg_size_at` + `AutoQuality` enum + `parse_size`/`fmt_bytes` + `--max-size` on shrink/convert; 17 new tests, full suite 238 green, all 5 gates pass. Run by the orchestrator (Opus) directly (subagent Bash blocked). Branch `feat/spec-017-max-size-byte-budget-for-shrink-and-convert`. 2026-06-16.
- [x] **verify** — orchestrator-run independent review (7-agent `/code-review` medium, ~511k metered tokens). ✅ No correctness bugs (the `search_threshold` refactor provably preserves SPEC-016 behavior; encoder parity exact). Applied 4 cleanups; the user then landed the format-agnostic generalization (`LossyFormat` trait) the review recommended. 238 tests; all 5 gates + 3-OS CI + cost-capture audit green. 2026-06-16.
- [x] **ship** — PR #20 squash-merged to `main` (4b64063). Orchestrator bookkeeping: cost sessions (real verify figure + estimated build), ship reflection, archived to `specs/done/`, STAGE-008 backlog updated, brag added. 2026-06-16.
