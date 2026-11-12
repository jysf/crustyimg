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
- [x] **verify** (2026-11-11, Opus, $7.47) — ⚠ PUNCH LIST at `920b5ef`, wording only, no re-verify.
  Drove every message (plus upper-case alpha, no-`#`, three-step rows), 12 widening cells and the four
  byte-pin hashes with its own PNG codec and FNV-1a, every format rung, GIF 00/01/80/fe/ff, batch and
  `build` exit 4/6, auto-decide via `apply` and a fresh `build`, six mutations (M1's FT-2 red
  confirmed). Default 1334 and lean 1270 passed. Punch items: (1) DEC-115 `affected_scope` lacks
  `src/cli/mod.rs`, `src/operation/trim.rs`; (2) DEC-115 part 4 and its Neutral consequence misstate
  `run_pixel_op` (only a forced format is checked; the format is still resolved after the run) —
  both on `main`, at ship; (3) `docs/recipes.md`, `recipes/README.md`, `CHANGELOG.md` say exit 4
  but a kept-format batch gives exit 6 — on the PR. Cleanups: A/B wasm asserts (declined), C AVIF
  "approximately" too loose (taken into the punch list). Table cells for ship: M1–M3 add FT-2; M13
  file; M9, M11 now driven; M16 adds `introduces_alpha_is_any_translucent_step`, FT-5/FT-7 driven.
- [x] **punch list** (2026-11-11, Sonnet, $0.56) — `ad8b76d`, 4 files (+56/−4): `docs/recipes.md`,
  `recipes/README.md` and `CHANGELOG.md` now give exit 4 for one input or a one-format batch and exit 6
  for the failing input when each keeps its own format; the AVIF sentence names the 5/255 small-image
  error. Driven: mixed batch exit 6 with `rgb.png` only; `--format jpeg` exit 4, no out-dir; 4×4 AVIF
  corner alpha 0.0196. `docs_ops` green. DEC-115 and the mutation table are left for ship.
- [x] **merge** (2026-11-12) — PR #262 merged as `a090a75` (`mergedAt` 2026-11-12T05:24:24Z), CI green at `004a66d` (18 pass, 8 release jobs skipped).
- [x] **ship** (2026-11-12, Sonnet, $1.70) — PR #262 merged as `a090a75`. DEC-115 reworded (two
  `affected_scope` files, `run_pixel_op` forced-format-only, only single-input `apply` resolves the
  format first); mutation table folded (M1-M3, M9, M11, M13, M16); reflection written; cost
  re-measured for all five earlier cycles, spec total $39.69 over six sessions. Archived.
