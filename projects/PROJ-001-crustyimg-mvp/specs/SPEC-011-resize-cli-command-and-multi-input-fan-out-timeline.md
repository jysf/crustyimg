# SPEC-011 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-011-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15 (Opus architect; spec + DEC-015 + build prompt authored; CLI half of split `resize`, building on shipped SPEC-010)
- [x] **build** — PR #12 opened 2026-06-15. `run_resize` + fan-out + ArgGroup mode-exclusivity + new CliError::{PartialBatch→6, Usage→2}; all 146 tests pass; 4 gates green. NOTE: the Sonnet build session dropped (API socket error) after writing the code but before gates/commit; orchestrator (Opus) finished it — fixed a clippy too_many_arguments (6 mode flags → ResizeModes struct), ran fmt, verified gates, did bookkeeping + PR.
  - **verify punch-list follow-up** (2026-06-15): the original build session dropped before adding the SPEC-011 integration tests to `tests/cli.rs`; a Sonnet verify-punch-list session added all 11 integration tests (`resize_max_single_input_writes_scaled`, `resize_exact_single_input_exact_dims`, `resize_multi_input_fan_out_preserves_format`, `resize_format_override_changes_format`, `resize_no_mode_is_usage_error`, `resize_two_modes_is_usage_error`, `resize_bad_wxh_is_usage_error`, `resize_missing_input_exits_3`, `resize_partial_batch_exits_6`, `resize_stdout_keeps_stdout_clean`, `resize_multi_without_out_dir_is_usage_error`). All 4 gates green at 157 tests (up from 146). PR #12 updated.
