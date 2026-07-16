# SPEC-088 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-088-<cycle>.md`.

## Instructions

- [x] **design** — spec + failing tests + implementation context written to `main`.
- [x] **build** — audit report (`--json`/`--timing`) + committed bench; worktree `spec-088-audit-bench`, PR #92, ~$4.90 est, DEC-074. Gates green (731 default / 744 avif). 2026-07-16.
- [x] **verify** — independent adversarial pass, own worktree. ⚠ **PUNCH LIST** (4 items; byte-identity + privacy PROVEN clean against the pre-spec oracle). ~$3.40 est. 2026-07-16.
- [x] **fix** — punch list cleared (4/4) + the maintainer's corpus ruling (both halves): `--json`+`-o -` guarded at the shared writer (incl. the pre-existing `--explain=json` — DEC-074 §Corrections); real `docs/cli-reference.md` §"Audit surface" + DEC-074 #3 give the lint criterion actual evidence; DEC-028 de-staled; a real CC0 photo (`photo_forest_cc0.jpg`, classifier-verified `photograph`) makes the AVIF claim true — it is the only row reaching AVIF; `photo_*.jpg` → `gradient_*.jpg` (engine says `graphic-logo`); `print_table` caveat footer. Byte-identity re-proven vs the oracle (32/32); gates green (732 default / 745 avif). ~$4.05 est. 2026-07-16.
- [ ] **ship** — merge + bookkeeping (orchestrator).
