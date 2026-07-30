# SPEC-107 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-107-<cycle>.md`.

## Instructions

- [x] **design** — 2026-07-28. Drove the STAGE-035 roster against a **release** and a **debug**
      binary at `f4c9d22` before writing anything, because the honest scope of a confirmation
      pass depends on what is already covered. Result table and coverage matrix are in the
      spec's Context. Headline: **no hang, no panic, no OOM on any input; nothing on release
      exceeded 0.25 s** — the launch board's "holds natively" is now driven, not assumed.
      Five findings, of which one is a live defect:
      **F1** a truncated JPEG succeeds **silently (exit 0, empty stderr)** on `info`/`web`/
      `resize` — the only finding that fails the stage's own "clear message on every input"
      bar, and it is on the flagship `web` path. Maintainer decision this cycle:
      **warn on stderr, still exit 0**, via a container-level EOI (`FF D9`) check —
      verified discriminating on the probe pair (`real.jpg` ends `ffd9`, `truncated.jpg`
      ends `2f69`). Rejected: exit 1 (breaks a workflow every viewer supports, changes a
      frozen exit-code surface) and document-only (closes a launch gate over a known
      silent-corruption path).
      **F2** exit 4's documented meaning (`api-contract.md:59`, "the message names the
      feature to rebuild with") does not cover the case it is actually used for — a
      zero-byte or text file, which names no feature. Code is deliberate
      (`src/image/mod.rs:417` → `src/cli/mod.rs:720`, pinned by `exit_code_mapping_is_total`)
      and the CLI is frozen; **fix the doc, not the code**.
      **F3** the **debug** profile leaks upstream `avif-parse` panic text on
      `meta_parser_state.avif`; **release is clean** (both exit 1 with a correct final
      message). Recorded and carved out by name, **not** fixed — but load-bearing, because
      `cargo test` *is* the debug profile, so a naive stderr-cleanliness assertion would
      assert the wrong thing.
      **F4** the 60 MP RAW gate is **wasm-only** (`src/wasm.rs:536`, DEC-082); native is
      DEC-063's 64 Mpix. The stage's "RAW either side of the 60 MP gate" is a wasm case —
      driving it natively would put a row in the results table that no code path claims.
      **F5** `oversize_preview.dng` exits 0 natively (0.25 s release / 6.14 s debug) — correct,
      given F4; recorded so the native/wasm split does not read as an inconsistency.
      Built the coverage matrix by reading the tests rather than assuming gaps: the **wasm
      half is in better shape than the stage assumed** (60 MP gate both sides, forged-header
      bomb, module-survival, message hygiene already driven; `just wasm-test` confirmed green
      **26/26 in 30.17 s** this cycle), so the wasm work is four named gaps, not a harness.
      Also established that the **empty-OBU AVIF has no committed file** — it is synthesised
      in-process by `build_avif_with_empty_alpha` inside `src/image/avif.rs`, so neither the
      CLI nor wasm can drive it; committing those bytes is a precondition, not an extra.
      Wrote 11 acceptance criteria, 13 failing tests, and two negative controls (revert the
      warning → RED; delete a fixture → RED).
      **Un-metered main-loop cycle** (AGENTS §4): one debug build, a 28-case probe corpus
      driven on both profiles, a CLI drive of the whole committed fuzz crash corpus, and one
      `just wasm-test` run.

- [x] **build** — 2026-07-30, `feat/spec-107-hostile-input-pass`, PR #127 opened (not merged).
      All 11 acceptance criteria met. Fixed F1 (`src/image/mod.rs`'s container-level
      `jpeg_missing_eoi`, wired at three CLI call sites: `report.rs::run_info`,
      `ops.rs::run_pixel_op`, `optimize.rs::optimize_decide_one`) — a truncated JPEG now
      warns on stderr (unconditionally, not `--quiet`-gated) and still exits 0; recorded as
      **DEC-085**, with the two rejected alternatives carried over from design. Committed
      the 8-file `tests/fixtures/hostile/` corpus + `tests/hostile_inputs.rs` (enumeration
      + exit-code + stderr-cleanliness harness, AC-2/AC-3), re-pointed the SPEC-094
      empty-alpha-OBU unit test at the committed fixture (AC-4), pinned F3's debug-only
      panic-leak with a by-name carve-out proven against the existing
      `meta_parser_state.avif` fuzz fixture (AC-6), and closed the four named wasm gaps in
      `tests/wasm_roundtrip.rs` (AC-7; `just wasm-test` 30/30). Fixed the doc (F2/AC-8) and
      closed the launch-readiness hostile-input blocker with the driven outcome, naming the
      still-open browser-specific remainder (AC-9); corrected the stale Mobile line while in
      the file (SPEC-101's actual outcome). Ran both AC-10 negative controls for real
      (reverting the warning → 2 tests RED; deleting a fixture → the enumeration RED) and
      restored before the final run. Full matrix clean from fresh per-leg
      `CARGO_TARGET_DIR`s: lean 797 (+13), default 816 (+13), webp-lossy 823 (+13) — the
      delta reconciles exactly against the tests added; `clippy -D warnings` and `fmt
      --check` clean on all legs. Two build-time findings not anticipated by the design's
      Notes: the referenced `png_header_declaring` shape needed an appended empty
      `IDAT`+`IEND` to actually reach `check_pixel_budget` on the native path (extended in a
      new `tests/common` copy, `wasm_roundtrip.rs`'s original left untouched), and the
      design's ~⅓ JPEG-truncation ratio doesn't transfer across images (this session's
      96×96 fixture's actual decodes-OK boundary is ~50%, so `truncated.jpg` uses 60%; the
      wasm AC-7 test deliberately uses a MORE aggressive truncation instead, since AC-7
      requires an `Err`). Identified but out-of-scope-for-this-spec (corrected on the
      punch-list pass — the original list here was wrong in both directions): the F1 warning
      is not wired on `diff` (both inputs), `responsive`, `apply`/`build` with a plain pixel
      recipe, `watermark --image` (overlay only), `lint`, or `meta strip` — each also decodes
      JPEGs directly without going through `run_pixel_op`. (`edit`, `watermark`'s primary
      input, and `apply --recipe web` DO warn — all three route through `run_pixel_op`, so
      they were wrongly listed here before.) Filed as a follow-up candidate, not fixed here
      (`one-spec-per-pr`).
      **Punch-list pass (second build session, same day):** verify returned ⚠ PUNCH LIST on
      PR #127 — fixed the one unmet criterion (AC-6's carve-out now screens the debug-profile
      extra stderr lines by exact 5-line match, not `contains()`; drove both mutations
      myself — replace RED, coexist RED — confirming the coexist case was GREEN under the old
      assertion first); corrected the headline deviation (design's shape was right; a second,
      unnamed cap — `MAX_IMAGE_DIMENSION`/DEC-034 — explains the wasm test's real
      `LimitsExceeded`) and the follow-up verb list (above); strengthened two
      `wasm_roundtrip.rs` assertions; fixed doc overclaims (corpus coverage claim,
      `--quiet` row, verb-count framing) and DEC-085's rationale (decision unchanged); drove
      `build`/release-profile-stderr/`view`'s non-tty path from "what verify could not check".
      Re-ran the full matrix clean, same counts (lean 797 / default 816 / webp-lossy 823,
      `just wasm-test` 30/30 — no tests added/removed, only bodies changed). Full detail in
      the spec's `## Build Completion`.

- [ ] **verify** — fresh session. Re-derive the findings independently rather than
      inheriting them; drive the corpus yourself on your own builds of branch **and** `main`.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-107`, stage backlog, and the STAGE-035 close-out (this is the
      stage's only spec, so shipping it closes the stage).
