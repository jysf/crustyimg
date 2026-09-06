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

- [ ] **build** — prompt: `prompts/SPEC-128-build.md` (2026-09-05). **Sonnet**, own worktree.
      **DEC-100 reserved**, block-list `affected_scope` form required.
      ⚠ **Sized honestly this time: the code is M, the verification is heavier than the code.**
      Budget ~250 exchanges. AC-10's sweep must cover the verbs that reach `run_pixel_op` —
      `run_convert`/`run_optimize`/`run_web` do, which is what SPEC-127's narrowing got wrong.

- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
