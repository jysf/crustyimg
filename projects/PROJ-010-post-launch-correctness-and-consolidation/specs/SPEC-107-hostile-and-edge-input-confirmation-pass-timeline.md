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

- [ ] **build** — run `prompts/SPEC-107-build.md` in a **fresh session**, in its own git
      worktree. Engine-adjacent: needs the clean full matrix from fresh per-leg
      `CARGO_TARGET_DIR`s (AC-11).

- [ ] **verify** — fresh session. Re-derive the findings independently rather than
      inheriting them; drive the corpus yourself on your own builds of branch **and** `main`.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-107`, stage backlog, and the STAGE-035 close-out (this is the
      stage's only spec, so shipping it closes the stage).
