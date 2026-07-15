---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-085
  type: story
  cycle: ship
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
  decisions: [DEC-005, DEC-017, DEC-048, DEC-057, DEC-069, DEC-070]
  constraints: [pure-rust-codecs-default, ergonomic-defaults, untrusted-input-hardening,
                every-public-fn-tested, test-before-implementation, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-084, SPEC-086]
value_link: >
  Ships the measured flagship: the `web` flow (downscale → content-modernize to AVIF/lossless-WebP →
  never-bigger → strip → orient → prove the score) that got 98% median savings in 2.7 s, size-insensitive,
  vs today's default 24% in 16.5 s. The one verb a new user should reach for.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop (un-metered, §4), grounded in a probe of the
        CLI verb wiring + recipe system: `web` = `optimize` + a default downscale + always-score
        (reuse `optimize_pipeline`/`optimize_decide_one`), plus a bundled-recipe registry; flagged the
        `web == apply --recipe web` equivalence as the load-bearing design decision.
    - cycle: build
      interface: claude-code
      tokens_total: 165000
      estimated_usd: 1.80
      recorded_at: 2026-07-14
      note: >
        first build, ~30 min, own worktree. `web` verb reusing the SPEC-084 engine; bundled-recipe
        registry (`include_str!` + name resolver, file-path-wins precedence); DELIVERED (not descoped)
        the equivalence via a terminal-optimize recipe step routing `apply` through the fast decision.
        Order-of-magnitude estimate (main-loop, no metered subagent).
    - cycle: verify
      interface: claude-code
      tokens_total: 125000
      estimated_usd: 1.40
      recorded_at: 2026-07-14
      note: >
        first verify, ~14 min — near-CLEAN. Reproduced the equivalence byte-identically + the corpus
        flow; found the untested pinned corner (DEFECT: the terminal-optimize apply path ignored
        `-o <ext>`/`--format` → AVIF-bytes-in-a-.png) + a missing DEC for the recipe-model change.
    - cycle: build
      interface: claude-code
      tokens_total: 90000
      estimated_usd: 1.00
      recorded_at: 2026-07-14
      note: >
        fix pass, ~9 min. Honored the `-o`/`--format` pin in the terminal-optimize apply branch (mirror
        the verb's diversion; new pinned-equivalence test), and emitted DEC-070 (terminal-op semantics,
        precedence, build-manifest limitation).
    - cycle: verify
      interface: claude-code
      tokens_total: 95000
      estimated_usd: 1.05
      recorded_at: 2026-07-14
      note: >
        focused re-verify, ~10 min — CLEAN. Pinned + unpinned equivalence byte-identical; DEC-070
        present; the build-manifest limitation is a clean error (no panic); regressions + gates green.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      recorded_at: 2026-07-15
      note: >
        ship bookkeeping in the orchestrator main loop (un-metered, §4). PR #89 was BEHIND main →
        `gh pr update-branch` (re-ran CI) → clean squash-merge (f1e8ba7). Two DEC-070 follow-ups carried:
        `build` manifest doesn't run terminal-optimize recipes (UnknownOperation; run_build deferred) +
        a clearer unknown-recipe-name error.
  totals:
    tokens_total: 475000
    estimated_usd: 5.25
    session_count: 6
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
- **Branch:** `spec-085-web-verb`
- **PR:** (opened, do-not-merge — see PR body)
- **All acceptance criteria met?** **Yes.** `web <photo>` → downscale-to-2048 → AVIF via `Mode::Fast`,
  substantially smaller, never larger, metadata stripped, orientation baked, SSIMULACRA2 reported;
  `web <graphic>` stays lossless (content branch holds); size-insensitive on the real corpus (12.8 MP
  and 1.8 MB photos both ~3–4 s); `apply --recipe web` is **byte-identical** to `web`; bundled
  `gallery`/`product` run + a real file path still works (file-wins precedence); `web ./raw.nef` reads
  the embedded RAW preview end-to-end. All gates pass: `cargo test` (735 default / 749 `--features avif`,
  after the verify-fix pass added `web_pinned_format_equals_apply_recipe_web_pinned`),
  `cargo clippy --all-targets` (both feature sets), `cargo fmt --check`, `cargo build --no-default-features`.
- **Default long-edge validated (2048):** measured on the corpus with the release binary — DSC_2011.JPG
  (12.8 MB/12 MP → 98.6 KB AVIF, 2048×1367, **99% smaller, 3.9 s, ssim 80.2**); L1024678.JPG (14 MB →
  63.7 KB, 2.7 s, ssim 83.1); DSCF1154.JPG (1.8 MB → 83 KB, 4.0 s, ssim 78.9); DSC_0163.png (86%, 1.1 s).
  Size-insensitive confirmed (the 12 MP photo is no slower than the 1.8 MB one — it downscales first).
- **The equivalence WAS delivered (not descoped).** A reserved **terminal `optimize` recipe step** encodes
  via the fast decision instead of a plain sink write. The `web` verb builds the flow in memory
  (`optimize_pipeline(Some(2048))` + `Mode::Fast` + always-score); the bundled `web` recipe reaches the
  *same* `run_optimize_autodecide(..., always_score=true)` via `run_apply`'s terminal-`optimize` branch.
  Identical pixel pipeline (auto-orient + resize max 2048) → identical decision → identical bytes.
- **New decisions:** **DEC-070** records the recipe-model change (the terminal `optimize` recipe step, the
  bundled-vs-file precedence, the pinned-format bypass on the apply path, and the `build`-manifest
  limitation) — the follow-through DEC-069 had deferred. Two design choices captured there: (1) the
  terminal `optimize` marker lives in the CLI apply path, **not** the operation registry (it produces
  bytes + a format choice, not an `Image`, so it can't be an `Operation`); (2) **precedence = a real file
  always wins** — `--recipe <arg>` is a path first, bundled name only on fallback, so a local `web.toml`
  unambiguously shadows the bundle and every existing file recipe is unchanged.
- **Deviations:** (a) `gallery`/`product` also use the terminal-`optimize` step (they modernize format
  like `web`, at 2560/1600 px) rather than being fixed-format "non-optimize flows" as the descope note
  imagined — cleaner and more consistent now that the mechanism exists. (b) A pinned format (`-o x.png` /
  `--format`) bypasses the auto-decision (and the score), mirroring `optimize`'s pin — **on both the `web`
  verb AND the terminal-`optimize` apply path** (`apply --recipe web hero.jpg -o hero.png` writes a real
  PNG, byte-identical to `web hero.jpg -o hero.png`). *(The apply-path pin was Defect 1 in verify — the
  first build honored the pin on the verb but not the `run_apply` terminal-`optimize` branch, which wrote
  AVIF-in-a-`.png`; the fix mirrors the verb's `pinned`→`run_pixel_op` diversion. See DEC-070 §2.)* (c) The
  terminal-`optimize` apply path is **sequential** (like `optimize`/`web`), not the rayon batch the plain
  apply path uses. (d) AVIF-producing tests use a small `--max`/small sources because the debug-build AVIF
  encoder is far too slow to encode a 2048 px image inside a unit test; the 2048 default is validated on
  the release corpus above.
- **Follow-ups:** (1) `build` binding a terminal-`optimize` recipe would hit `UnknownOperation("optimize")`
  at `build_pipeline` (a typed error, not a panic) — wire the same terminal-optimize split into `run_build`
  if DEC-057 build-manifest use of bundled flows is wanted (recorded as a known limitation in DEC-070 §4).
  (2) `apply` unknown-recipe-name error still
  prints the generic "could not read recipe file" (exit 3 is correct); surface the bundled-names hint by
  giving the not-found case its own `CliError` message. (3) An `optimize` step mid-recipe (not terminal) is
  left to fail as `UnknownOperation`; a dedicated "must be terminal" error would read better. (4) Reframe
  SPEC-080 (demo redesign) onto the `web` flow, per the STAGE-030 sequencing.
### Build-phase reflection
1. **The equivalence was the whole risk, and it resolved cleanly because `optimize_decide_one` already
   took a `&Pipeline`.** That seam meant `web` and the bundled recipe could share the identical fan-out by
   just handing it different pipelines — the verb builds one in memory, the recipe strips its terminal step
   and builds the rest. No engine was re-implemented; byte-identical output fell out for free.
2. **SPEC-084 left the exact seam it promised.** `optimize_decide_one` already returned the third
   `Option<f64>` score value and `emit_optimize_report` already appended `· ssim N` — wiring `web`'s
   always-score was adding one `bool` param and a best-effort decode-and-score, exactly the "one-liner" the
   SPEC-084 comment predicted. Landing that seam paid off a spec later.
3. **The debug AVIF encoder is the real test-design constraint.** Encoding a 2048 px AVIF in a debug build
   is >2 min; every AVIF-producing test had to keep the *encoded* image tiny (small `--max` or sub-2048
   sources). The 2048 default itself can only be honestly validated on a release binary against the corpus,
   which the build report now records.

---

## Reflection (Ship)
1. **What would I do differently next time?** — Name the *pinned corner* explicitly in the spec. The
   equivalence was delivered impressively (a terminal-optimize recipe step, not a descope), but the
   `-o`/`--format` pin path through `apply --recipe web` was the one case neither the spec's acceptance
   nor the build's tests covered — verify found AVIF-bytes-in-a-`.png`. When two paths are claimed
   "equivalent," the spec should enumerate the *flag combinations* that must agree, not just the happy
   path. Also: don't drop the `cost:` block when Writing a spec over its scaffold (caught at ship).
2. **Does any template, constraint, or decision need updating?** — DEC-070 now carries the recipe-model
   change (a terminal-optimize "magic op" that isn't a registry operation) — a genuine DEC-005 extension
   worth its own entry, not a code comment. Follow-ups it records: the `build` manifest can't run
   terminal-optimize recipes (`UnknownOperation`; `run_build` wiring deferred) and a clearer
   unknown-recipe-name error.
3. **Is there a follow-up spec I should write now before I forget?** — SPEC-086 (`optimize --verify` +
   remove `shrink`) is next and unblocked. The `build`-manifest-runs-terminal-optimize gap (DEC-070) is
   a small future spec if pulled. The reframed SPEC-080 (demo → `web` hero + recipe presets) now has its
   dependency (the bundled recipes) shipped.
