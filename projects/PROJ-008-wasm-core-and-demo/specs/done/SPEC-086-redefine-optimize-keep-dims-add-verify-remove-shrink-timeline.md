# SPEC-086 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-086-<cycle>.md`.

## Cycles

- [x] **design** (2026-07-14) — Framed build-ready; mostly surface + deletion on the SPEC-084 engine:
  add `optimize --verify` (turn `score_winner_once` back on for one run), remove the `shrink` verb
  entirely, fix the stale `run_optimize` doc-comment. Grounded in the CLI verb wiring.
- [x] **build** (2026-07-15, ~35 min) — `optimize --verify` (JSON gains an `"ssim"` field gated on
  measured, non-verify byte-identical); hard-cut `shrink` (`Commands::Shrink`/`run_shrink`/
  `shrink_auto_config`/`DEFAULT_SHRINK_MAX`; neutral helper renames `shrink_params→resize_max_params`,
  `DEFAULT_SHRINK_QUALITY→DEFAULT_LOSSY_QUALITY`); stale doc-comment fixed; live-surface docs redirected
  to `web`/`optimize`. All 5 design tests + 2 supporting; drove the binary (self-check). Emitted DEC-071.
- [x] **verify** (2026-07-15, ~15 min) — **CLEAN**, independent adversarial pass, fresh worktree.
  Proved non-verify JSON byte-identical to main + pinned output byte-identical; `--verify` score matches
  `crustyimg diff` (80.75→80.7); `shrink` gone from `--help`/completions/live docs (historical records
  correctly preserved); renamed-helper regressions (`web` still scores, `optimize` keep-dims unchanged)
  + gates green. Non-defect: pinned `--verify -o x.avif` silently ignores `--verify` (by design).
- [x] **ship** (2026-07-15) — **SHIPPED.** Clean squash-merge **PR #90** (`f54aac9`) — mergeable/CLEAN,
  no rebase. DEC-071 recorded. Cost 300k tok / **$3.30** (build $2.00 + verify $1.30; design/ship null;
  session_count 4). Ship reflection (independent verify still worth it even when CLEAN; the historical-
  vs-live grep distinction). STAGE-030 backlog: SPEC-086 shipped (3/6). `just validate` + `just
  cost-audit` green.
