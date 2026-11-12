# SPEC-159 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-159-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-10, Opus) — GO at [M], one DEC (DEC-115 recommended). Premise confirmed
  (`transparent`/`#ff000080` refused at load, exit 1). Driven: JPEG drops alpha silently (exit 0),
  GIF keeps only alpha 0, the other six formats keep it. Recommends `transparent` = `#00000000`, a
  static recipe × format check on four surfaces, and GIF allowed at alpha 00. Four maintainer questions.
- [x] **design** (2026-11-11, Opus, $12.84) — [M] holds; DEC-115 written. Frame corrected: the auto-decided
  format CAN be JPEG (the shortlist read the source's alpha; driven: black border, exit 0), fixed by
  OR-ing `Pipeline::introduces_alpha`. Check lives on `Operation::introduced_alpha` + `alpha.rs` +
  `Pipeline::unstorable_alpha`, six call sites; 18 mutation rows, 17 driven on a prototype; byte pin recorded.
- [x] **build** (2026-11-11, Sonnet, $6.92) — draft PR #262 at `920b5ef`, CI green (18 pass, 8 release
  jobs skipped). FT-6 byte pin green on `main` before the first edit and after the last; 22/22 ACs
  claimed. 17 mutations driven (M18 by search); three reasoned cells confirmed (M3 FT-11, M9 single
  input, M11 wasm via the parity harness). Table gaps: FT-2 also reds under M1–M3; M13's
  `pad_then_trim_is_the_identity` lives in `tests/trim.rs`. Outside the file list: `tests/wasm_roundtrip.rs`
  (P30 row removed) and `scripts/wasm-ci-check.sh` (counts 55→56, 53→54), both caught by CI.
  Open: DEC-115's `affected_scope` lacks `src/cli/mod.rs` and `src/operation/trim.rs` (AC-1).
- [ ] **verify** — `prompts/SPEC-159-verify.md`
- [ ] **ship**
