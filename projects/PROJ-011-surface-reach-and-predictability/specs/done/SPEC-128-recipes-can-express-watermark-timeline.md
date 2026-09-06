# SPEC-128 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-128-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-05. 11 ACs, 4 settled design calls, 8 failing tests.
      ⚡ **The backlog item's own title was wrong, and reading the code is what found it.** It said
      "`watermark` becomes a registry operation." `Watermark` **is already an `Operation`**
      (`operation/mod.rs:1118`, `impl` at `:1161`, with a `params()` that already serialises the
      overlay path). It is **deliberately unregistered**, and DEC-031 says why in terms: the
      registry constructor is a pure `fn(&OperationParams) -> Result<Box<dyn Operation>>` that
      "cannot (and must not) load the overlay file."
      So the real spec is one level down: **widen the seam so an op whose params name a FILE can be
      constructed from a recipe** — with `src/operation/**` staying filesystem-free, because it
      compiles for wasm32 (DEC-064). Verified that premise holds today: a grep for
      `std::fs`/`Image::load`/`File::open` under `src/operation/` returns nothing, which is what
      makes AC-8 a guard rather than a fix.
      📌 **Three customers, not one** — image watermark (overlay pixels), text watermark (font
      bytes), and the decided `.cube` LUT op. That is why it is a seam. **Call 4 keeps LUT a design
      constraint, not a deliverable**, and AC-7 enforces it: a second asset-bearing op must register
      in a test *without changing the seam*.
      ⚡ **Call 3b was found by asking what the TOML would actually look like** — text mode's
      `params()` is wrong today. `watermark_overlay` returns the rendered pixels plus a label, and
      for `--text` the label **is the text**, which `params()` then writes under the key `image`.
      A text watermark serialises as `image = "© crustyimg"` — the text in the field that means
      file path — so a round-trip would try to load a file by that name. It has never mattered
      because watermark is unregistered; this spec is what makes it matter. The two modes now get
      distinct, non-overlapping keys, and `Watermark`'s single `overlay_path` slot has to widen.
      ⚠ **Highest-consequence line: the loaded bytes must never round-trip into the params.**
      `to_toml` emits the path, never the bytes — same shape as SPEC-127's
      `to_toml`-must-emit-`"1"` guard, and it has its own test.

- [x] **build** — prompt: `prompts/SPEC-128-build.md` (2026-09-05). **Sonnet**, own worktree.
      All 11 ACs met; `DEC-100` created (block-list `affected_scope`). Nine failing tests written
      (the build prompt said eight — the spec's own `## Failing Tests` list has nine; built all of
      them). Native suite green across default/`--no-default-features`/`webp-lossy` (fresh
      `CARGO_TARGET_DIR` each), `just wasm-check` + `just wasm-test` green (43/43, incl. 2 new).
      AC-9's three negative controls (Calls 1/2/3) each reverted alone and confirmed to flip only
      their own tests — Call 3's needed a stronger assertion to actually discriminate from the
      constructor's own fallback error (see spec's Build Completion reflection). AC-10's sweep: built
      `main` (`ed48efa`) and this branch as separate release binaries, 8 verbs × 4 files × 2 formats
      + a plain-recipe `apply`/`build` pair — 39/39 byte-identical, plus a positive control
      (`main` rejects a watermark recipe, branch accepts it) proving the methodology can detect a
      real difference. One follow-up filed to STAGE-050's backlog: `build`'s cache key doesn't hash
      a watermark asset's own file content, only the recipe's declared path. See PR for the link.

- [x] **verify** — 2026-09-06, Opus, read-only. **$26.31** / 44.2M tokens / 50 min.
      ⚠ **PUNCH LIST, 13 items — no behavioural defect.** "I exercised more of it than the test
      suite does and found no behavioural defect in what it claims."
      ⚡ **It BUILT the cheaper alternative rather than arguing for one:** a `#[cfg]`-gated
      registration recovering **117,173 B (9.26 % of the bundle)**, inside the ±5 % window so no
      baseline move, AC-6's refusal still passing. **Maintainer ruled to keep the +121,614 B** —
      the wasm artifact exists to demo the tool and a watermark is worth demoing. Recorded in
      DEC-100's alternatives and as the only `+` row in DEC-066's ledger.
      ⚡ Caught **AC-5 half-met** (the `font` key had no test — half the mechanism) and **AC-9's
      Call 2 control flipping ZERO tests** (the upfront probe already catches it; the claimed flip
      needs two conditions). Also the stale floor note, a circular corpus reference, an 18-of-19
      file list, and that **`build` serves a stale watermark from cache** — reproduced, filed.

- [x] **ship** — 2026-09-06. PR [#189](https://github.com/jysf/crustyimg/pull/189) merged as
      **`7f7fed0`**; punch list applied at `b72db1f` (records) and `2e1eae9` (three tests closing
      AC-5, both driven with controls). `cost.totals` **$72.07** / 179,135,883 — build **$45.76**
      (corrected from $42.29: right method, stopped 20 ids early) + verify **$26.31**.
      ⛔ **No tag, version still 0.7.1** — batches with the rest of STAGE-050.
