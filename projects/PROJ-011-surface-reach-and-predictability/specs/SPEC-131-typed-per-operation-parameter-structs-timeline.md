# SPEC-131 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-131-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-131-frame.md`). **GO, M.** Premise corrected: a bad
      recipe already fails before any input on `apply`/`build`; the defect is silent acceptance —
      unknown keys ignored on every op (DEC-061 #1), and watermark's optional keys silently
      defaulted when wrong-typed (exit 0, output byte-identical to the default). Keep the map as
      the serialized form → `to_toml()` byte-stable. Sequential with SPEC-133 (`Watermark`).
- [x] **design** — 2026-10-24 (`prompts/SPEC-131-design.md`). M, no split: the registry publishes
      a per-op key/type/range schema checked before each constructor; the map stays the stored
      form (`to_toml()` pinned by a golden corpus). Widened by measurement: `nan`/`inf` and keys
      unused by the step's mode are refused too. 15 ACs, 6 revert conditions. No `src/wasm.rs` or
      `src/cli/**` edit → parallel with SPEC-132/134, before SPEC-133. DEC-102 written at build.
- [x] **build** — prompt: `prompts/SPEC-131-build.md`. **Opus**, branch
      `feat/spec-131-strict-recipe-step-params`. PR #200, $11.03, completed 2026-10-25. CI green
      on all 16 required legs at `c98b032`.
      ⚠ **Merged 2026-10-25 as `ba8b740` BEFORE verify ran** (head `48185da` = `c98b032` + an
      update-branch merge of docs only). Cause: the merge gate lived only inside prompts. Fixed:
      builds now open DRAFT PRs, marked ready only after verify approves.
- [x] **verify** — prompt: `prompts/SPEC-131-verify.md`. Opus, retroactive, $6.83, 2026-10-26.
      **⚠ PUNCH LIST**: (1) regression — `1e-50` passes the range check and reads as 0.0 in f32
      (`percent`/`size`/`scale`, silent wrong output); (2) no one-step keyed-marker test; (3)
      CHANGELOG wording. Everything else holds (see the spec's Verify Readout).
- [x] **fix** — prompt: `prompts/SPEC-131-fix.md`. Sonnet, draft PR #203 at `ee004f6`, ~$1.44,
      2026-10-26. All 3 punch items done; the 1e-50 rows RED before / GREEN after; both mutations
      (f32 revert, M10) flip exactly their new test; CI 16/16 green.
- [x] **re-verify** — prompt: `prompts/SPEC-131-reverify.md`. Opus, ~$1.82, 2026-10-26.
      **✅ APPROVED — merge PR #203 at `ee004f6`.** Bug reproduced on `main` (exit 0, 1×1 / unmarked)
      and gone on the PR (exit 1, names the key); edges right (1e-40 accepted, 1e-46 and
      opacity 1.00000001 refused); each mutation fails exactly its one test in the full suite
      (1007/1008); CI green on every leg. Marked ready for the maintainer to merge.
- [ ] **ship** — prompt: `prompts/SPEC-131-ship.md`. Runs after verify approves (or after its
      follow-up fix merges).
