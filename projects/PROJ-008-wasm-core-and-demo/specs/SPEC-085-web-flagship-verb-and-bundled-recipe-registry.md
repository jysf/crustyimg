---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-085
  type: story
  cycle: design
  blocked: false
  priority: high
  complexity: M
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-14
references:
  decisions: [DEC-005, DEC-017, DEC-048, DEC-057, DEC-069]
  constraints: [pure-rust-codecs-default, ergonomic-defaults, untrusted-input-hardening,
                every-public-fn-tested, test-before-implementation, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-084, SPEC-086]
value_link: >
  Ships the measured flagship: the `web` flow (downscale → content-modernize to AVIF/lossless-WebP →
  never-bigger → strip → orient → prove the score) that got 98% median savings in 2.7 s, size-insensitive,
  vs today's default 24% in 16.5 s. The one verb a new user should reach for.
---

# SPEC-085: `web` flagship verb + bundled-recipe registry

## Context

STAGE-030's benchmark found the hero flow is **downscale-then-modernize**: `web` (downscale to ~2048 →
AVIF for photos / lossless-WebP for graphics) hits **98% median in 2.7 s and is size-insensitive**,
where the full-dimension default gets 24% in 16.5 s. SPEC-084 shipped the engine that makes the format
call fast and AVIF-aware (`Mode::Fast`, never-bigger, `score_winner_once`); this spec puts a **verb** on
the flow and gives crustyimg a **bundled-recipe registry** so `web` is also `apply --recipe web`.

Key grounding (probed): the existing `optimize` already composes the pieces — `optimize_pipeline`
(`src/cli/mod.rs`) does auto-orient (bakes EXIF orientation + drops the metadata bundle, DEC-017) + an
optional `--max` long-edge resize, and `optimize_decide_one` runs the `Mode::Fast` decision +
never-bigger. So **`web` is `optimize` with a default downscale + an always-on score** — not a new
engine. The recipe system (`Recipe::from_toml`, `run_apply`, `src/recipe/`) loads recipes **only from a
file path** today; there is no bundled registry and no "encode via the optimize decision" recipe step —
those are this spec's new surface.

## Goal

Ship a **`web <inputs>`** verb delivering the measured flow (default downscale to a web long-edge +
`Mode::Fast` content-modernize + never-bigger + strip + orient + an always-on SSIMULACRA2 score on the
downscaled output), and a **bundled-recipe registry** (`include_str!` + a name resolver) so
`apply --recipe <name>` runs a shipped recipe and **`web` == `apply --recipe web`**. Ship the
`web`/`gallery`/`product` recipes; feature RAW input (`web ./raws/`) as a highlight.

## Inputs — files to read

- `src/cli/mod.rs` — the `Commands` enum (`Optimize` ~311, `Apply` ~443, `Shrink` ~278 to be removed by
  SPEC-086), `run_apply` (~1065), `load_recipe` (~1041), `optimize_pipeline` (~4050),
  `optimize_decide_one` / `run_optimize` (~4004), `AutoQuality`/`Mode::Fast` (SPEC-084),
  `score_winner_once` (`src/quality/mod.rs`).
- `src/recipe/mod.rs` — `Recipe`, `Recipe::from_toml`, the step/op model, the size cap.
- `src/operation/` + `OperationRegistry::with_builtins` — the ops a recipe can carry (resize, auto-orient, …).
- STAGE-030 design notes (the score cost / always-on-for-web rationale) + DEC-069.

## Outputs

- **`src/cli/mod.rs`**
  - A new **`Web { inputs, max?, format?, /* opt-in overrides */ }`** command: the flagship. Default
    behavior = downscale the long edge to a **web default (recommend 2048; validate/record)** unless the
    source is already smaller (never upscale), then `Mode::Fast` modernize + never-bigger + strip +
    orient, and **always score the (downscaled) winner** via `score_winner_once` (cheap — ~0.2–0.35 s at
    2–3 MP, STAGE-030 notes) and report it. Reuse `optimize_pipeline` + `optimize_decide_one`; do **not**
    duplicate the engine. `-o`/`--format`/`--out-dir`/`--name-template`/`-j` behave as elsewhere.
  - `run_web(...)` wiring; batch fan-out via the shared path (like `optimize`).
- **Bundled-recipe registry** (new small module, e.g. `src/recipe/bundled.rs`)
  - `include_str!` the shipped recipe TOMLs (a new `recipes/` dir: `web.toml`, `gallery.toml`,
    `product.toml`), a `name → &'static str` resolver, and a `list()` for help.
  - Extend `apply --recipe <arg>`: if `<arg>` resolves to a bundled name, use it; else treat it as a path
    (a real file always wins / is unambiguous — decide + document the precedence).
- **The `web == apply --recipe web` equivalence** — the load-bearing design decision (see Notes): the
  bundled `web` recipe must produce the same result as the `web` verb, which means recipes need a way to
  **encode via the `Mode::Fast` optimize decision** (a terminal "optimize"/auto-format step), not a fixed
  format. Deliver the equivalence; if the recipe-step extension proves heavy, descope to "`web` verb +
  the bundled recipes for the non-optimize flows (gallery/product)" and document the gap honestly — flag
  it, don't fake the equivalence.
- **RAW highlight:** confirm `web ./raws/` reads RAW (embedded preview, PROJ-009) end-to-end — a
  "sharp can't do this" demo path; add a test/fixture.
- **DEC** (if the equivalence needs a recipe-model change) or fold into DEC-069's follow-through.

## Acceptance Criteria

- [ ] `web <photo>` downscales to the web default (never upscales a smaller source), produces **AVIF**
      (via `Mode::Fast`), is **substantially smaller**, **never larger than the source**, strips
      metadata, bakes orientation, and **reports an SSIMULACRA2 score** on the output.
- [ ] `web <graphic>` stays **lossless** (WebP/PNG) — the content branch holds through the `web` flow.
- [ ] `web` is **size-insensitive** (a 0.7 MP and a 24 MP photo both finish in a few seconds — it
      downscales first). Drive real corpus photos.
- [ ] `apply --recipe web <inputs>` produces a result **equivalent to `web <inputs>`** (or, if descoped,
      the divergence is documented and `web` the verb still ships the full flow).
- [ ] `apply --recipe gallery` / `product` run their shipped recipes; a **real file path** still works
      (bundled-vs-path precedence is defined + documented).
- [ ] `web ./raws/` (or a RAW fixture) reads a RAW input end-to-end.
- [ ] Batch + `--out-dir` + never-upscale + hostile-input handling all hold; `cargo test`
      (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, lean build pass.

## Failing Tests (written at design)

- **`src/cli/mod.rs` / integration (`--features avif`)**
  - `web_photo_downscales_modernizes_scores` — real photo → AVIF, ≤ source, downscaled to the default,
    metadata gone, orientation baked, a score reported.
  - `web_graphic_stays_lossless` — graphic → lossless, never AVIF.
  - `web_never_upscales_small_source` — a sub-default image keeps its dimensions.
  - `web_equals_apply_recipe_web` — the two paths agree on bytes/format/dims (or, if descoped, a test
    pinning the documented behavior).
  - `web_reads_raw_input` — a RAW fixture goes through `web`.
- **`src/recipe/bundled.rs`**
  - `bundled_recipe_names_resolve` — `web`/`gallery`/`product` resolve to parseable recipes.
  - `apply_prefers_real_path_over_bundled_name` (or the chosen precedence) — the resolution rule holds.

## Implementation Context

### Decisions that apply
- `DEC-069` (SPEC-084) — the `Mode::Fast` decision + never-bigger + `score_winner_once` this verb drives.
  `web` scores **always** (cheap on the downscaled output — STAGE-030 design notes).
- `DEC-005` — the recipe TOML format (`apply` reads the same recipes the wasm build does); bundled
  recipes are the same format, just shipped in-binary.
- `DEC-017` — auto-orient bakes EXIF orientation + drops the metadata bundle (the strip); reuse it.
- `DEC-048` — the content branch (graphic → lossless) the `web` flow must preserve.
- `DEC-057` — the build/manifest recipe binding; bundled recipes should be usable there too.

### Constraints
- `pure-rust-codecs-default` — no new dep. `ergonomic-defaults` — `web` is the thing a new user should
  reach for; the default downscale is `web`'s opinion (NOT `optimize`'s — SPEC-086 keeps dims).
- `untrusted-input-hardening` — the decode caps carry; RAW/AVIF/hostile inputs stay typed-error/no-panic.

### Out of scope (this spec)
- `optimize` redefinition, `--verify`, `shrink` removal (SPEC-086) — though `web` and the `shrink`
  removal are conceptually paired (web absorbs shrink's downscale-re-encode, upgraded to AVIF).
- The `meta` group (SPEC-087); the unified audit report / committed bench (SPEC-088); `convert --to`
  and social/archive recipes (SPEC-089).
- Any wasm change.

## Notes for the Implementer
- **`web` is `optimize` + a default downscale + always-score** — build it by reusing `optimize_pipeline`
  (pass a default `max`) and `optimize_decide_one`, plus an unconditional `score_winner_once` on the
  winner. Resist re-implementing the engine.
- **The equivalence is the hard part.** `apply --recipe web` must reach the `Mode::Fast` encode, which
  today's recipe model can't express (it encodes to a fixed format). Either add a terminal
  `optimize`/`auto-format` recipe step (cleanest — makes `web` genuinely recipe-driven) or descope and
  **document** honestly. Decide early; it drives the shape.
- **Never upscale.** The downscale is a max-bound (resize mode "max"), like `optimize --max`.
- **Validate the default long-edge** (2048 recommended) against the corpus + the q-sweep; record it.
- **Verify will drive the real corpus + a RAW file**; keep the flow honest (report the real score, never
  claim visually-lossless at q85).

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
