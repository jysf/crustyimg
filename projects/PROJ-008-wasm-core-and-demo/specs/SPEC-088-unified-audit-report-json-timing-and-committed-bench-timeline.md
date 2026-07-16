# SPEC-088 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-088-<cycle>.md`.

## Instructions

- [x] **design** — spec + failing tests + implementation context written to `main`.
- [x] **build** — audit report (`--json`/`--timing`) + committed bench; worktree `spec-088-audit-bench`, PR #92, ~$4.90 est, DEC-074. Gates green (731 default / 744 avif). 2026-07-16.
- [x] **verify** — independent adversarial pass, own worktree. ⚠ **PUNCH LIST** (4 items; byte-identity + privacy PROVEN clean against the pre-spec oracle). ~$3.40 est. 2026-07-16.
- [x] **fix** — punch list cleared (4/4) + the maintainer's corpus ruling (both halves): `--json`+`-o -` guarded at the shared writer (incl. the pre-existing `--explain=json` — DEC-074 §Corrections); real `docs/cli-reference.md` §"Audit surface" + DEC-074 #3 give the lint criterion actual evidence; DEC-028 de-staled; a real CC0 photo (`photo_forest_cc0.jpg`, classifier-verified `photograph`) makes the AVIF claim true — it is the only row reaching AVIF; `photo_*.jpg` → `gradient_*.jpg` (engine says `graphic-logo`); `print_table` caveat footer. Byte-identity re-proven vs the oracle (32/32); gates green (732 default / 745 avif). ~$4.05 est. 2026-07-16.
- [x] **re-verify** — independent pass over the fix commit (9add66b), own worktree. ⚠ **PUNCH LIST (2 minor, doc-accuracy only)**. **Licence + provenance: CONFIRMED, high confidence** — every element re-read from the Commons API (`Artist`=DimiTalen, `Credit`=Own work, `License`=cc0, `AttributionRequired`=false) and the committed file proven to BE that photograph by pixel diff vs the Commons original (ssim 53 vs −816 unrelated / 100 self). Bare JFIF confirmed by a raw segment walk (source GPS 50.4957/6.1034 absent). AVIF genuinely exercised (winner avif, 30%, ssim 81.4; confirmed by `file` + macOS `sips`). `-o -` guard fires exit 2 / 0 stdout bytes on every verb+spelling, and its test provably fails when the guard is removed. DEC-074 §Corrections reproduced EXACTLY (15,963 B, magic at offset 514). Byte-identity independently swept **32/32**. All docs values driven true; footer prints/suppresses; generator byte-for-byte. Gates green (732/745). **Defects:** (1) spec ~L385 cites `flat_ratio 0.02 / entropy 7.58` — those are the **960px intermediate's**; the committed file measures **0.04 / 7.37** (conclusion + thresholds still correct); (2) `Cargo.toml:303` still says criterion "runs via `just bench`" → `just bench-micro`. ~$3.30 est. 2026-07-16.
- [ ] **ship** — merge + bookkeeping (orchestrator). Two one-line doc corrections outstanding (re-verify punch list) — maintainer's call whether to fold in here.
