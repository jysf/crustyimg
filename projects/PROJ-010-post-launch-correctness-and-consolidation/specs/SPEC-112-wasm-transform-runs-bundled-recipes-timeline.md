# SPEC-112 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-112-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-09. Found while scoping the 0.7.0 cut, not from the backlog.
      SPEC-111's DEC-087 named `wasm::transform` as carrying the same unstripped-terminal-
      `optimize` defect and filed it out of scope because *"the shipped demo never reaches
      it"* — **right about the demo, wrong about the README.** `README.md:34-36` tells readers
      to start from a bundled `web`/`gallery`/`product` recipe and says "the same recipe TOML
      runs in the browser demo too, via the wasm `transform()` binding." It does not, and that
      README renders on the crates.io crate page.
      **Driven** (throwaway native test over `transform`'s exact call chain — `bundled::resolve`
      → `Recipe::from_toml` → `build_pipeline`): all three bundled recipes return
      `unknown operation 'optimize'`. The demo escapes only because `demo/worker.js`'s
      `geometryRecipe()` hand-builds a different, terminal-step-free recipe — so the demo is
      fine and the **published `crustyimg-wasm` npm package is not**.
      **The design question SPEC-111 had does not exist here.** `build` needed something to
      choose the output format once the marker was stripped; `transform` takes `out_format`,
      and `parse_format` resolves it through `format_from_extension`, which cannot accept
      `"auto"` or empty — only a concrete format. So the caller has **always** pinned it. That
      is exactly DEC-087's pinned branch: strip the marker, run the pixel steps, encode as
      asked. `optimizeDetailed` remains the decide-path counterpart, untouched.
      Wrote 10 acceptance criteria and 7 failing tests plus a negative control. Two traps
      called out: **AC-3** (a strip that dropped the whole recipe passes AC-1 and AC-2 — only a
      dimension assertion catches it) and **AC-4** (markerless recipes must stay byte-identical,
      because that is the shape the live demo sends). Complexity **S** — one call site, the
      helper exists, the decision is already made.
      **Un-metered main-loop cycle** (AGENTS §4).

- [ ] **build** — run `prompts/SPEC-112-build.md` in a **fresh session**, own git worktree.
      Sonnet. Small, but it touches a published surface — `just wasm-check` is the fast gate
      before the full `just wasm-test`.

- [ ] **verify** — fresh session, **Opus**. Drive all three bundled recipes through the real
      wasm surface yourself rather than the native call chain; the native chain is what the
      design used to *find* the bug, not sufficient to confirm the fix. Confirm AC-4's
      byte-identity against `main`, since the live demo depends on it.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-112`, stage backlog. **STAGE-040 does not close here** — the
      0.7.0 cut is its second item, and it depends on this landing.
