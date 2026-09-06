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

- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
