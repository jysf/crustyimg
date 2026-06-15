# SPEC-011 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-011-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus architect; spec + DEC-015 + build prompt authored; CLI half of split `resize`, building on shipped SPEC-010)
- [x] **build** — PR #12 opened 2026-06-15. `run_resize` + fan-out + ArgGroup mode-exclusivity + new CliError::{PartialBatch→6, Usage→2}; all 146 tests pass; 4 gates green. NOTE: the Sonnet build session dropped (API socket error) after writing the code but before gates/commit; orchestrator (Opus) finished it — fixed a clippy too_many_arguments (6 mode flags → ResizeModes struct), ran fmt, verified gates, did bookkeeping + PR.
