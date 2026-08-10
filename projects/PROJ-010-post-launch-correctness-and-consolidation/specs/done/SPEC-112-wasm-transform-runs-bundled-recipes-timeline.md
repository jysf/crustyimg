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

- [x] **build** — 2026-08-09/10. PR [#144](https://github.com/jysf/crustyimg/pull/144),
      branch `feat/spec-112-wasm-bundled-recipes`. All 10 ACs met. `transform` strips the
      terminal marker via `split_terminal_optimize`, moved (not copied) from `cli::optimize`
      to `src/recipe/mod.rs` — `cli` and `wasm` are mutually exclusive `#[cfg(target_arch)]`
      module trees (SPEC-072), so a `cli`-hosted `pub(crate)` helper could not have reached
      `wasm::transform` at any visibility; `recipe` is one of the modules compiled for both.
      DEC-087 amended (AC-7). Full matrix green, fresh per-leg `CARGO_TARGET_DIR`s,
      sequential, through `rtk proxy`: lean 821/821, default 841/841, webp-lossy 847/847,
      `just wasm-test` 37/37 (30 pre-existing + 7 new) — every log confirmed
      `Compiling crustyimg`. Note: the prompt's stated lean/webp-lossy reference numbers
      (818/844) undercounted this branch's actual base commit (462b829) by 3 each, confirmed
      via a git-stash positive control against that exact commit — a pre-existing drift, not
      caused by this spec; default's reference (841) matched exactly. (Verify later settled
      this more cheaply: all 7 new tests sit inside `tests/wasm_roundtrip.rs`'s
      `#[cfg(target_arch = "wasm32")]` block, so the native delta is 0 by construction.)
      AC-9 negative control driven and recorded (see spec).
      **Cost correction, made by the orchestrator at ship:** the build reported running on
      `claude-opus-5` and flagged a model mismatch. There was none — it ran on
      **claude-sonnet-5**, exactly as `agents.implementer` pinned. It had resolved "its own
      transcript" as the newest `.jsonl` in the project directory, which was the parent
      orchestrator's session. Corrected figures are in the spec's `cost.sessions`
      (81,750,802 tokens / $39.46 at Sonnet anchors, 320 messages).
      **CI legs on PR #144 not read by the build** — handed to verify to confirm the full
      required matrix rather than a local macOS pass.

- [x] **verify** — 2026-08-10, fresh Opus session, own worktree. **⚠ PUNCH LIST → both items
      closed on the branch.** All 10 ACs met. **BEFORE was measured, not inherited:** a `main`
      (462b829) worktree driven through the real wasm-bindgen surface in Node returns
      `unknown operation 'optimize'` for `web`, `gallery` AND `product`; on the branch all
      three succeed and their output decodes as PNG. **AC-4 got the real byte diff**, not only
      the in-process reconstruction: `transform`'s output for `geometryRecipe(900)`'s exact
      shape fingerprints identically on both sides (len 12549, FNV-1a-64 `86e971b1a8845667`,
      CRC32 `60384d86`), with a cap-800 negative control proving the fingerprint
      discriminates. **AC-9 re-run rather than read:** 37/0 → 34/3 → 37/0, artifact SHA-256
      `355a2f4f…` → `929d8615…` → `355a2f4f…`, the three RED being exactly AC-1/AC-2/AC-3.
      AC-3's 1600×1067 independently re-derived on `main` from `product`'s pixel half.
      **The structural finding holds:** `src/lib.rs` gates `cli` and `wasm` on mutually
      exclusive `cfg(target_arch)`, so the spec's "just widen `pub(super)`" option was never
      available — the design named an impossible option, and DEC-087's amendment records it
      as a finding, not an aside. Full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      through `rtk proxy`: lean 821/821, default 841/841, webp-lossy 847/847, each log
      carrying `Compiling crustyimg`; clippy `-D warnings` and `fmt --check` clean;
      `just wasm-test` 37/37. Two punch-list items, both **claims about the code rather than
      the code**, fixed on the branch in `741fd16`: `src/recipe/mod.rs`'s doc block claimed
      `build` reaches the helper "via `cli::optimize`'s re-export" (it imports it from
      `crate::recipe`; the only `pub use` in `cli/mod.rs` is `WEB_DEFAULT_LONG_EDGE`), and the
      two AC-5 tests asserted only `!msg.is_empty()` where the rest of that file pins the
      message — both now pin the driven strings. **Named, not fixed:** no CI leg runs
      `just wasm-test`, so the seven tests that pin this spec never execute in CI.

- [x] **ship** — 2026-08-10. PR [#144](https://github.com/jysf/crustyimg/pull/144) merged as
      `3bd26b5` with all 27 required CI legs green and 6 release-only jobs skipped, read
      individually off `gh pr checks` at head `741fd16` — not inferred from a local pass.
      **Cost corrected at ship, not accepted as reported:** the build's entry named
      `claude-opus-5` / 8,119,424 / $6.75, measured against the parent orchestrator's
      transcript; its own is 320 messages of `claude-sonnet-5` → 81,750,802 / $39.46. Verify's
      own entry was re-read after completion (165 messages, not the 156 it could see while
      still writing) → 27,320,821 / $36.70. Totals: **109,071,623 tokens / $76.16 over 2
      metered sessions**, components summing exactly and re-priced at each session's own
      anchors (DEC-083). Reflection appended; two follow-ups filed, the load-bearing one being
      that **no CI leg runs `just wasm-test`**. **STAGE-040 does not close here** — the 0.7.0
      cut is its second item, and it depended on this landing.
