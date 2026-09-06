# SPEC-127 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-127-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-04. 9 ACs, 4 settled design calls, 7 failing tests. **STAGE-050's
      thesis item**, unblocked by STAGE-049 shipping the day before.
      ⚡ **Call 1 was settled by measurement, not preference.** The naive answer — an optional
      field on `version = "1"` — is wrong, and driving it on `main` at `7181eed` shows why:
      `Recipe` is `deny_unknown_fields`, so an old binary handed a v1-plus-`format` recipe fails
      with a **TOML parse error at line 2**, while the same binary handed `version = "2"` says
      *"unsupported recipe version '2' (supported: 1)"*. A recipe is a file people commit and
      share, so a forward recipe **will** meet an old binary; the version gate is what makes that
      meeting produce an actionable sentence. v1 stays valid and unchanged.
      📌 **Call 4 SPLIT the backlog item rather than accepting it.** "Typed per-operation param
      structs" was bundled with this work; it touches every op in the registry, has no dependency
      in either direction, and turns an M into an L. Filed back to STAGE-050 as its own item.
      ⚠ **The most likely thing to get wrong is `to_toml` emitting `"2"` unconditionally** — it
      would strand every existing recipe on the next `--save-recipe`, and it would *look* like it
      worked because the new binary reads both. That is `v1_still_round_trips_and_stays_v1`.
      ⚠ Driving the Context table through `| head` first reported `exit=0` on all three rows —
      the pipe's code, not the binary's. Redirected and re-read; the prompt carries the warning.

- [x] **build** — 2026-09-04. PR #188 (`feat/spec-127-recipe-format-quality` → `main`), head
      `b4a7d63`, CI 16/16 green, mergeable/clean. All 9 ACs met (AC-7's per-condition reverts and
      AC-8's two-binary byte-identical corpus both driven, not assumed); DEC-099 filed with
      block-list `affected_scope`. $38.49, 120,553,651 tokens, 53.4 min (measured post-hoc by the
      orchestrator from the subagent's saved transcript — the dispatch prompt omitted the
      self-measurement snippet). Cycle advanced to `verify`.
      prompt: `prompts/SPEC-127-build.md` (2026-09-04). **Sonnet**, own worktree,
      branch `feat/spec-127-recipe-format-quality`. **DEC-099 reserved in the prompt** —
      `next_id` scans only the working tree and has collided here before, and the prompt requires
      the **block-list** `affected_scope` form because the audit silently drops inline arrays.
      ⛔ **Byte-changing on the surface: the prompt says do not bump the version, do not cut a
      release.** Batches with the rest of STAGE-050.
      Carries the traps this repo has paid for most recently: resolve the recipe's values at the
      call site rather than inside `encode_one` (reading them below the CLI flags is how the two
      paths silently diverge again — the exact defect SPEC-126 existed to fix); write
      `docs/api-contract.md` in the same change, because the decisions audit **cannot** warn you
      when it is missed; and `just wasm-test` runs in no CI job, so a wasm assertion is not
      covered by the required matrix and Build Completion must say so.

- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
