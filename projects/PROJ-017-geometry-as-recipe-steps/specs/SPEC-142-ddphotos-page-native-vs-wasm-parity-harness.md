---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-142
  type: task                       # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false                  # ruled (B) 2026-10-30: parity is bounded, DEC-110
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-060
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null
  created_at: 2026-10-30

references:
  decisions:
    - DEC-064   # the wasm cfg boundary: transform runs the SAME core the CLI runs; the harness tests that claim
    - DEC-065   # wasm encodes AVIF but cannot decode it; the harness decodes AVIF rows natively or independently
    - DEC-066   # wasm size levers; untouched (the harness is test-only), named so design confirms it
    - DEC-094   # AVIF bytes depend on tile count; SPEC-124 pinned it, which is why wasm AVIF matched here
    - DEC-059   # no cross-arch byte pins; the frame's evidence is aarch64 only, CI is x86_64
    - DEC-104   # watermark refuses a mark that does not fit; the page's size cap exists because of it
    - DEC-008   # resize backend fast_image_resize (SIMD): probable seat of the 1/255 native-vs-wasm finding
    - DEC-092   # resize in linear light, f32 + powf: the other probable seat
    - DEC-009   # CI matrix + single-runner extra jobs; the new job is advisory
    - DEC-018   # licence policy covers crates; Pillow is a CI-only test tool
    - DEC-110   # Rule B: dimensions exact; no-resample rows exact; resampled rows within 1/255 on at most K pixels (cap 78)
  constraints:
    - test-before-implementation
  related_specs:
    - SPEC-141  # no-bytes-changed guard over the recipe corpus (PROJ-017 line 1), framed beside this one
    - SPEC-124  # pinned the AVIF tile count (shipped)
    - SPEC-127  # transform's out_format/recipe.format precedence (shipped)
    - SPEC-128  # transform refuses asset-bearing steps (shipped)

value_link: >
  Infrastructure enabling STAGE-060's "a recipe from the page that the CLI replays on the
  full-size original, proven by a native-vs-wasm parity test": the test exists and is green
  on today's resize + watermark recipes before crop or straighten touch the page.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 7002258
      tokens_breakdown:
        input: 116
        output: 43937
        cache_creation: 135171
        cache_read: 6823034
      estimated_usd: 5.36
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive frame session in its own worktree (docs/spec-142-frame). Measured from the
        session's transcript (1726bd18-….jsonl, identified by content), deduped by `.message.id`
        (58 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25
        ($6.25), cache_read x0.10 ($0.50): $0.0006 + $0.84 + $3.41 + $1.10. Measured before
        the commit/push turns, so it slightly under-counts.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 7670372
      tokens_breakdown:
        input: 106
        output: 63021
        cache_creation: 175792
        cache_read: 7431453
      estimated_usd: 6.39
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive design session in its own worktree (docs/spec-142-design), ended BLOCKED on a
        ruling. Measured from the session's transcript (c346905b-….jsonl, identified by content),
        deduped by `.message.id` (53 unique ids, all claude-opus-5-5), input/cache_creation/
        cache_read once per id, MAX output_tokens per id. Priced by component at Opus $5/$25 per
        MTok, cache_creation x1.25 ($6.25), cache_read x0.10 ($0.50). Measured before the
        commit/push turns, so it slightly under-counts.
    - cycle: design-amend
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 3505009
      tokens_breakdown:
        input: 56
        output: 26403
        cache_creation: 129195
        cache_read: 3349355
      estimated_usd: 3.14
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive design-amend session in its own worktree (docs/spec-142-design-amend): wrote
        DEC-110 (Rule B), amended AC-3/FT-3/FT-4/Comparator/x86 outcomes, filed two findings.
        Measured from the session's transcript (03bc18f1-….jsonl, identified by content), deduped
        by `.message.id` (28 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Priced at Opus $5/$25 per MTok, cache_creation x1.25,
        cache_read x0.10: $0.0003 + $0.66 + $0.81 + $1.67. Measured before the commit/push turns,
        so it slightly under-counts.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 11187823
      tokens_breakdown:
        input: 140
        output: 48161
        cache_creation: 170111
        cache_read: 10969411
      estimated_usd: 4.65
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Build session in its own worktree (test/spec-142-ddphotos-parity), Sonnet. Measured from the
        session's transcript (8cdf0742-....jsonl, identified by content), deduped by `.message.id`
        (70 unique ids, all claude-sonnet-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced at Sonnet $3/$15 per MTok, cache_creation x1.25 ($3.75),
        cache_read x0.10 ($0.30): $0.0004 + $0.64 + $3.29 + $0.72. Measured before the final
        commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-142: ddphotos page native-vs-wasm parity harness

## Context

PROJ-017's thesis is that an edit made on the ddphotos edit page (`integrations/ddphotos/edit/`)
is a recipe, and the native CLI replays it with the same result. The page runs crustyimg's
**slim** wasm build (`just wasm-build-slim`: `--no-default-features --features avif`, installed by
`just ddphotos-edit-install`). The CLI is the native build. So "the same recipe gives the same
pixels" is a claim about two binaries built with different feature sets.

**What is checked today: nothing that compares the two.**

- `tests/wasm_roundtrip.rs` (1,668 lines of `#[wasm_bindgen_test]`) checks formats, dimensions
  and errors on the real wasm surface. No test in it compares against a native output: the
  word "native" appears only in doc comments about the native suite's fixtures.
- `just wasm-test` runs in **no** workflow. `ci.yml` has no wasm job (`grep -i wasm` finds
  nothing in it); only `pages.yml` builds wasm, for the demo, and its smoke drives the demo,
  not the ddphotos page. Already filed as a STAGE-053 chore ("A wasm32 CI leg").
- `ci.yml`'s `changes` filter (`^(src/|tests/|examples/|benches/|Cargo…|\.github/workflows/|justfile|scripts/)`)
  does not include `integrations/`. A PR that changes only `edit.js` skips every heavy job.

**The page builds two different recipes** (`edit.js:266–293`). `pixelRecipe(edit)` is what
wasm `transform` runs on the published 1600 px WebP (`auto-orient`, an optional `resize`
`mode = "max"`, an optional text `watermark`). `cliRecipe(edit)` is what the user is told to run
on the original: the same steps, always resized to the page's output long edge, plus `format`
and (for lossy formats) a terminal `optimize`. The page's final copy is `transform(…, "png")`
then `optimizeDetailed`; the CLI's is `apply` with the terminal `optimize`. Those encoders search
differently by design (README "Limits"), so encoded-file equality between the page's download
and the CLI's output is not a property anyone promises.

This is STAGE-060's first backlog item and PROJ-017 build-order line 3. It has no dependency.
Crop on the page (line 7) depends on it, so that when crop lands a red harness means crop broke
parity, not that the harness was never right.

## Goal

A harness that takes the recipes the page's own `pixelRecipe` / `cliRecipe` emit for a table of
edits, runs each through wasm `transform` (the slim package, in Node) and native `crustyimg apply`
on the same input bytes, and fails naming the edit, format and difference when the decoded pixels
(or, per format, the bytes) are not identical. It is green on today's resize + watermark
recipes, runs on every PR that touches the page or the engine, and is proven able to go red.

When it ships, a STAGE-060 builder runs `just ddphotos-parity` (name for design to confirm) and
sees one line per row; a broken row reads like
`FAIL both/webp-input/png: native 1024x685 vs wasm 1024x685, 312 pixels differ, max channel diff 3`
with the recipe text that produced it.

## Frame

### Verdict: **GO**, [S]

The premise holds: native and wasm agree **exactly** on today's recipes (48/48 rows
pixel-identical, 47/48 byte-identical, on both the full and the slim wasm build). So the harness
can be green from day one and assert exact pixel equality, which is the strongest useful bound.
Scope is (a) engine parity; (b) replay parity belongs to the crop spec, which is the first spec
that can break it.

### Premise check

Evidence from `main` @ `8ccc2d1` on the maintainer's Mac (aarch64-apple-darwin, 14 cores),
fresh `CARGO_TARGET_DIR`s in a scratch directory, deleted after. Scripts stayed in scratch.

**Builds.**
- Native: `CARGO_TARGET_DIR=<scratch>/tgt-native cargo build --release` → `crustyimg 0.8.1`,
  31.6 s.
- Wasm full: `CARGO_TARGET_DIR=<scratch>/tgt-wasm just --set _wasm_out <scratch>/pkg wasm-build`
  → 52.4 s, 1.23 MB brotli.
- Wasm slim (what the page ships): the same with
  `--set _wasm_features " --no-default-features --features avif"` → 0.78 MB brotli.

**Inputs, made independently of crustyimg** (ImageMagick 7.1.2-34 and exiftool, from
`tests/fixtures/classify/color_photo_fuji.png`, cropped to 448×300 so the image is not square
and `resize` rounding is exercised):
- `photo.webp`: 1600×1071 lossy WebP q82 (the shape ddphotos publishes)
- `photo.jpg`: 1600×1071 JPEG q90
- `photo_rot.jpg`: the same JPEG with EXIF Orientation = 6 (`exiftool -n -Orientation=6`), so
  `auto-orient` does real work

**Edits**, as `edit` objects built the way `readEdit` builds them (long edge 1600, Medium
watermark `0.04`, margin 2 % of the output long edge, gravity `southeast`, white, opacity 0.6):
`resize` (to 1024), `wm` (watermark "© Jane Doe 2026", no resize), `both`, `none`
(auto-orient only).

**Recipes came from the page, not a copy.** A Node script read `edit.js`, sliced the text
between the `// ── the recipe` and `// ── the editor` comments, and evaluated it with a stub
`$` returning the one form value the section reads (`wm-size`). It called the page's
`pixelRecipe`, `cliRecipe` and `watermarkSize` (which computed `size = 41` for the `both` edit).
It then ran `transform(input, pixelRecipe(edit), fmt)` through the wasm package
(`initSync` + `transform` from `pkg/crustyimg.js`), for fmt ∈ {png, jpeg, webp, avif}, and wrote
the recipe text to disk for the native side. Native ran
`crustyimg apply --recipe <that file> <input> -o <out>.<ext> -Q`.

**Comparison** (ImageMagick, not crustyimg): `cmp -s` on the encoded files,
`magick identify -format %wx%h` on both, `magick compare -metric AE` (differing pixels) and
`-metric PAE` (largest channel difference).

**Result, 3 inputs × 4 edits × 4 formats = 48 rows, identical for the full and the slim build:**

| | rows | dims equal | pixels equal (AE = 0, PAE = 0) | bytes equal |
|---|---|---|---|---|
| PNG | 12 | 12 | 12 | 12 |
| JPEG (default quality) | 12 | 12 | 12 | 12 |
| WebP (lossless) | 12 | 12 | 12 | **11** |
| AVIF | 12 | 12 | 12 | 12 |

One edit per line, WebP input, PNG out (the page's own `make` path):
- `resize`: `photo_webp__resize png bytes=same dims=1024x685/1024x685 AE=0 (0) PAE=0 (0)`
- `wm`: `photo_webp__wm png bytes=same dims=1600x1071/1600x1071 AE=0 (0) PAE=0 (0)`
- `both`: `photo_webp__both png bytes=same dims=1024x685/1024x685 AE=0 (0) PAE=0 (0)`
- `none`: `photo_webp__none png bytes=same dims=1600x1071/1600x1071 AE=0 (0) PAE=0 (0)`
- auto-orient did run: `photo_rot_jpg__resize png … dims=685x1024/685x1024 AE=0`

**The one byte difference.** `photo_rot_jpg__wm` as lossless WebP: both files are 678,726 B,
32,201 bytes differ (first at offset 103, then from 624,314), and the decoded RGBA is identical
(`magick … rgba:- | shasum -a 256` → `72c37a9e49ed7c49` for both). Each side is deterministic:
three native runs gave one SHA-256 (`af6a18a2…`), and a second full wasm run reproduced all 48
wasm files byte for byte. So the VP8L encoder makes a different (equally valid) choice on
wasm32 for this image. It is not a pixel difference, and the harness rule for WebP must not be
byte equality.

**Negative controls, driven** (the comparer can say "different"):
- One parameter changed on the native side: `both` with `opacity = 0.6` → `0.61`. Result
  `AE=23.26, PAE=771` (771/65535 ≈ 3/255). Red.
- One pixel of geometry: `resize` with `width = 1024` → `1023`. Native output 1023×685 vs wasm
  1024×685; `magick compare` exits 1 on the size mismatch. Red, but only because of the dims
  check: the harness must compare dimensions first and report them, not rely on the comparer
  failing.

⚠ `magick compare -metric AE` in ImageMagick 7.1.2 printed a fractional count (23.26) for a
change that touches far more than 23 pixels, so AE is not a pixel count here. Design should not
use AE as its metric. Compare decoded RGBA buffers directly (count differing pixels and the max
channel difference itself).

**A first read of property (b), not asserted by this spec.** An "original" was made at 6400×4286
(ImageMagick, q95 JPEG) and a "published copy" at 1600×1072 from it (ImageMagick, q82 WebP).
`pixelRecipe` on the copy vs `cliRecipe` (format png) on the original, both native:
`resize` → 1024×686 both sides, PSNR 46.24 dB, PAE 2570 (10/255); `both` → 1024×686, PSNR
46.25 dB, PAE 10/255. The watermark lands in the same place because `cliRecipe` resizes to the
page's long edge before watermarking. Two cautions for whoever asserts (b): the real published
copy comes from ddphotos' photogen, not ImageMagick, so the bound depends on photogen's resizer;
and the original's and the copy's aspect ratios round independently, so the output can differ
by one pixel in one dimension on some photos (6400×4286 and 1600×1072 both gave 686 here, by
luck of the numbers).

⚠ **All of this is aarch64.** The CI runner is x86_64. DEC-059 records that nothing establishes
cross-arch byte identity, and zune-jpeg (JPEG input) and rav1e (AVIF output) have x86 SIMD paths
that the wasm build cannot use. The harness only ever compares native and wasm **on the same
runner**, so the question is "does x86_64 native agree with wasm32", and it is unmeasured.
Design must run the harness on an x86_64 runner (a draft PR's CI is enough) before it fixes the
per-format rules below. The page's real input is WebP (`image-webp`, pure Rust), so the rows that
matter most are the least exposed.

### The property

**This spec asserts (a), engine parity, exactly:** for each edit, the recipe the page emits,
run through wasm `transform` and through native `apply` on the **same input bytes**, gives
outputs with equal dimensions and identical decoded pixels (zero differing pixels). No
tolerance. The evidence above says exact equality holds today, and any tolerance would hide
the first real divergence.

It covers both recipes the page emits: `pixelRecipe` (what wasm runs) and `cliRecipe` (what the
CLI runs). For `cliRecipe` with a lossy format, the terminal `optimize` makes the native and
wasm encoders search differently, so `cliRecipe` rows run with format `png` (no `optimize`
step), which checks every pixel step of the CLI recipe.

**(b), replay parity, lives in STAGE-060's crop spec** (the second backlog item), which already
owns the 4× replay. That is where (b) first can fail: today `cliRecipe` and `pixelRecipe` share
every pixel parameter, and the only difference is the resize source size. Recommended bound
for that spec: dimensions equal (or a stated ±1 px rule, see above) and a PSNR floor set from
measurement; 46 dB was measured here for the non-crop case.

### Per-format rule

The page's own pixel paths are `transform(…, "png")` (the copy) and `transform(…, "jpeg")` with
`quality = 88` (the preview, `worker.js:56`). Those two are the required rows.

| format | assert | why |
|---|---|---|
| PNG | bytes equal **and** dimensions | 12/12 byte-equal; byte equality implies pixel equality and needs no decoder in the harness. Report pixels too when it fails |
| JPEG | decoded pixels equal; bytes reported, not asserted | 12/12 byte-equal on aarch64, but the encoder may have an x86 SIMD path; assert bytes only if the x86 run proves them |
| WebP (lossless) | decoded pixels equal; never bytes | measured: same pixels, different bytes on one image, deterministic both sides |
| AVIF | decoded pixels equal (decode natively or with an independent decoder: wasm cannot decode AVIF, DEC-065) | 12/12 byte-equal here because SPEC-124 pinned the tile count. AVIF is lossy, so pixel equality follows from byte equality; if x86 bytes differ, design states a bound from measurement. One AVIF row is enough: they cost 1–4 s each in wasm |

### Where it runs

**Recommend a CI job**, not only a `just` recipe. A parity check that runs only when someone
remembers is a claim. One new job in `ci.yml`, ubuntu-latest, gated on the `changes` filter
widened to include `integrations/ddphotos/`: install the wasm target and `wasm-pack`, build the
slim package (the same `wasm-build-slim` recipe, so DEC-066's size profile is the one tested),
build native release, run the harness. Plus a `just` recipe that runs the same thing locally.

Cost estimate: `pages.yml`'s job (a full wasm build + demo assembly + Chrome smoke) took 3 m 49 s
warm on run 41104358361. This job adds a native release build, so estimate **6–8 min warm,
~12 min cold**. It runs in parallel with the Windows-bound matrix (~17.8 min median), so time to
green does not move; runner-minutes grow by that amount on code PRs. The harness itself ran
48 rows in about 35 s of wasm time, most of it the AVIF rows.

**The negative control design must require:** a self-test mode (or a test case) that feeds the
native side a recipe with one parameter changed (`opacity 0.6 → 0.61`, measured red above) and
asserts the harness reports red, naming the edit, the format and the difference. The STAGE-060
item's "two recipes that differ by one pixel of crop" waits for crop; until then `width 1024 →
1023` is the geometry version (measured red above, via the dimensions check).

**The positive control:** assert each output's dimensions equal what the page's own arithmetic
predicts (`outW × outH` from `readEdit`), that a `resize` row's output differs from the input's
size, and that a watermark row's pixels differ from the same edit without the watermark. Then
a harness where both sides do nothing, or both sides fail identically, cannot pass. Count the
rows that ran and fail if the count is not the table's size.

Per the repo's lesson that wasm32 compiling is not wasm32 running: the wasm side must run the
real `.wasm` in Node (as the probe did), never a native call to the same Rust function.

### Use the page's own recipes

**Recommend factoring the recipe section of `edit.js` into `edit/recipe.js`**, an ES module with
no DOM access that `edit.js` imports and the harness imports. It holds `tomlString`, the font
advance table, `textEms`, `watermarkSize` (taking the size fraction as a parameter instead of
reading `$("wm-size")`), `pixelRecipe`, `cliRecipe`, and the arithmetic half of `readEdit`
(long edge, `outW`/`outH`, margin) as a pure function of the photo size and the form values.

The last point matters: the probe had to re-type `readEdit`'s arithmetic to build its `edit`
objects. That copy is a drift point the factor removes.

Cost to the page: one more static file (the 114 lines, 5.5 KB, moved out of `edit.js`'s
19 KB), one import line, one more module request on first load. No build step:
`<script type="module">` already loads `edit.js`, and a relative `import` works on every server
the page was tested on. `scripts/ddphotos-edit-install.mjs` copies every file in `edit/`
(`readdirSync`), so the installer needs no change. It does need one browser check that the
page still loads and previews after the move (one server, headless Chromium, through the test
bed). The full three-server round stays with STAGE-060's last item.

The alternative is the probe's own method: slice `edit.js` between the two section comments and
evaluate it with a stub `$`. It works today with no page change, and it fails loudly if a function
moves (a `ReferenceError`). But it still depends on comment markers and a stub, and it leaves
the `readEdit` arithmetic copied. It is the fallback if the browser check proves costly.

### Does this need a DEC?

**No.** DEC-066 governs the `.wasm` size profile, and the harness changes no shipped binary; the
CI job builds through the same `wasm-build-slim` recipe, so it tests the profile rather than
changing it. DEC-064's cfg boundary is untouched. Moving pure functions into a second page file
is an internal change with no new dependency or build step, so `constraints.yaml` has nothing
it binds against. A new CI job has precedent as ordinary spec work (the `avif`, `webp-lossy` and `heic`
jobs came with their specs). The per-format rule (pixels, not bytes, for WebP) is a test rule and lives in this spec.
Design should write one only if it finds the x86 run forces a tolerance, because a tolerance on
"same recipe, same pixels" would be a decision about the thesis.

### Sizing: [S]

Verification matrix: 4 edits (`none`, `resize`, `wm`, `both`) × 2 recipes (`pixelRecipe`,
`cliRecipe` as png) × the PNG rule, on one WebP input, is 8 core rows. Add the JPEG preview path
(4 rows), one lossless-WebP row, one AVIF row, the orientation-6 JPEG input for the `resize` row,
2 negative controls, and the positive control. About 16 rows, all automated, one fixture pair.
Plus one browser load check for the `recipe.js` move and one x86 CI run.

Code: `recipe.js` (moved, not new), a Node harness of roughly 150–250 lines, a `just` recipe,
a CI job of about 30 lines, and the filter change. Top of S. If design finds it heavier, the cuts
in order are: (1) the `recipe.js` factor goes to the crop spec, which edits `edit.js` anyway, and
this spec uses the section-slice method; (2) the CI job becomes STAGE-053's wasm-leg chore, and
this spec ships the `just` recipe that the STAGE-060 specs must run and paste.

### Scope

In:
- The harness (Node), reading recipes from the page's own functions, running the slim wasm
  package and native `apply` on the same bytes, and comparing per the table above
- Committed fixtures made outside crustyimg: a lossy WebP (the published-copy shape) and an
  EXIF-orientation-6 JPEG, both non-square, small enough to keep the repo lean
- The negative and positive controls
- `edit/recipe.js` and the matching `edit.js` import, with one browser load check
- A `just` recipe and a `ci.yml` job, with `integrations/ddphotos/` added to the `changes` filter

Out (and where it lives):
- (b) replay parity at 4×: STAGE-060's crop spec
- Crop and straighten rows: the crop and straighten specs add their rows
- Parity of the page's final encode (`optimizeDetailed`) with the CLI's terminal `optimize`: not a
  promised property (README "Limits"), not filed
- Running `just wasm-test` in CI: STAGE-053's "A wasm32 CI leg" chore. The new job should be
  shaped so that chore is one more step in it
- The three-server test-bed round: STAGE-060's last item

### Rough build file list

- `integrations/ddphotos/edit/recipe.js` (new: the recipe section, pure)
- `integrations/ddphotos/edit/edit.js` (import from `recipe.js`; `readEdit` keeps only the DOM reads)
- `tests/ddphotos_parity.mjs` (new: the harness; `tests/demo_smoke.mjs` is the precedent for a
  Node test in `tests/`)
- `tests/fixtures/ddphotos/` (new: the WebP and orientation-6 JPEG, and a note on how they were
  made)
- `justfile` (a `ddphotos-parity` recipe)
- `.github/workflows/ci.yml` (the job and the `changes` filter)
- `integrations/ddphotos/README.md` ("How it was tested": one line naming the harness)

### Open maintainer questions

1. **CI job or `just` recipe only?** Recommend a CI job (6–8 min warm, parallel, so no change to
   time to green).
2. **Factor `recipe.js` now, or slice `edit.js` and leave the page alone until crop?** Recommend
   factoring now: it removes the copied `readEdit` arithmetic, and the page cost is one file.
3. **Fold STAGE-053's `wasm-test` CI leg into the same job?** Recommend no, to keep this [S];
   shape the job so that chore is one added step.

### Maintainer rulings (2026-10-30)
All three recommendations above were accepted:
1. A new `ci.yml` job, plus a `just` recipe.
2. Move the recipe code into `integrations/ddphotos/edit/recipe.js` now.
3. Keep STAGE-053's wasm-test leg out of this job. Shape the job so that leg becomes one added step.

### Maintainer rulings (2026-10-30, continued)
4. **Rule B, bounded, through a DEC (DEC-110).** Dimensions are always exact. Rows with no
   resampling step (1, 3, 9, 11) are exact. Resampling rows may differ by at most 1/255 per
   channel on at most K pixels, where K comes from the build's runs and is capped at 0.01 % of the
   row's pixels (78 at 1080×723). (A), the exact engine fix, is filed against PROJ-015's
   working-type stage, not dropped.

## Design finding: blocked on a ruling (2026-10-30)

**Ruled (B), 2026-10-30, DEC-110.** This section is kept as the record of why.

⛔ **The frame's premise does not hold at the page's own sizes.** On aarch64, native `apply` and
the slim wasm `transform` differ by **one pixel, by 1/255 in one channel**, after a `resize` to
1200 px, which is the page's first option below "as published". The frame drove only 1024 px,
and 1024 is not a page option (`fillSizes`, `edit.js:386–392`, offers 1200, 1080, 800, 400).

Driven 2026-10-30 at `9fbfbe1`. The native release build (23 s) and the slim wasm (48 s, 0.78 MB
brotli) were built in fresh scratch `CARGO_TARGET_DIR`s, deleted after. The inputs are the
fixtures below. Decoding is Pillow 12.3.0, which uses no crustyimg code. The recipe is
`auto-orient` + `resize mode = "max"`, PNG out, and each case ran twice:

| input | 1200 | 1080 | 800 | 400 |
|---|---|---|---|---|
| landscape WebP 1600×1071 | **1 px, max 1** (both runs) | 0 | 0 | 0 |
| portrait WebP 1071×1600 (no EXIF tag) | **1 px, max 1** | 0 | **2 px, max 1** | 0 |
| JPEG 1600×1071, EXIF orientation 6 | **1 px, max 1** | **1 px, max 1** | 0 | 0 |
| the same JPEG, tag stripped | 0 | 0 | 0 | 0 |

- **It is deterministic.** Both runs gave the same result. The full wasm build (the frame's
  package) gives the same table, so the slim build is not the cause.
- **The pixel:** landscape at 1200 differs at (507, 417): native `(66, 56, 37, 255)`, wasm
  `(66, 55, 37, 255)`. That is a rounding boundary.
- **The JPEG decode is not the cause.** The stripped JPEG matches at every size. The JPEG
  rows' differences come from `auto-orient` changing the shape the resize sees.
- **Probable cause, not proven:** `resize` works in linear light in `f32` (`src/operation/mod.rs`,
  the linear-light block around `:500–575`, DEC-092): `powf` for the sRGB curve, then Lanczos3
  on `fast_image_resize`'s per-architecture SIMD backend (DEC-008). Either can round
  differently on native and wasm32. x86_64 (AVX2, glibc `powf`) is unmeasured and may differ
  more.

So "zero differing pixels" is false today on a recipe the page makes for any photo at 1200 px.
The maintainer's ruling ("the property is engine parity, **exactly**") can't be met without a
change to `src/`, which the frame ruled out. Per the design prompt, this design does **not**
loosen the rule and does **not** write the tolerance DEC. It stops for a ruling:

- **(A) Exact: fix the engine first.** Make resampling bit-identical across targets: one `powf`
  implementation on every target, and an `f32` convolution with the same evaluation order on
  every backend (or the portable backend everywhere). This changes native output bytes for every
  resize (SPEC-141's guard would flag it). It may cost native resize speed, the reason for
  DEC-008, and needs its own spec and a DEC-008 amendment. SPEC-142 then ships as designed below
  with **Rule E**. Straighten, which interpolates too, inherits exactness for free.
- **(B) Bounded (recommended): a DEC that defines parity as a bound, and SPEC-142 asserts it.**
  **Rule B:** dimensions are always exact. Decoded pixels may differ by **at most 1/255 per
  channel**, on **at most K pixels**. A row whose recipe has no resampling step (rows 1, 3, 9 and
  11 below) stays exact. K is set from the build's first x86_64 run. The cap is 0.01 % of the
  row's pixels (77 at 1080×723), and above the cap the build stops. The driven negative control
  still fails under Rule B: opacity 0.6 → 0.61 differs by 3/255 on lossless rows and 13–22 on
  lossy ones. A 1/255 step on a few pixels in a million is invisible on the page and well
  inside the native-vs-wasm noise DEC-059 already declines to pin. (A) is filed as a bug either
  way, so exactness stays a target and not a forgotten claim.

Everything else in this design is the same under (A) and (B). Only the comparator rule (AC-3)
and, for (B), the DEC id in `references` change. The resuming session (a short design amend, or
the build, once ruled) edits AC-3, FT-3 and the "Comparator" note, and nothing else.

A second finding, not blocking: **wasm `info()` reports the stored size of an EXIF-rotated
JPEG**, 1600×1071 for an orientation-6 file whose output is 1071×1600. `readEdit`
(`edit.js:235–264`) takes the photo's size from `info` (`edit.js:420–421`). So for such an input
the page predicts landscape output dimensions, and it sizes the watermark for a landscape image
(`size = 43` for a portrait output). This is harmless today, because ddphotos publishes WebP
copies photogen has already oriented. **The crop spec must know about it**: crop coordinates on
a rotated original are in the space `auto-orient` produces, not the space `info` reports.
Rows 15–16 assert the observed sizes, and P1 records the transposed prediction (see below).

## Inputs

- **Files to read:** `integrations/ddphotos/edit/edit.js` (the recipe section `:181–293`,
  `readEdit` `:235–264`, `fillSizes` `:386–392`), `integrations/ddphotos/edit/worker.js`
  (`withQuality` `:40–42`, `PREVIEW_QUALITY` `:22`, the preview call `:56`, the copy `:66`),
  `integrations/ddphotos/edit/index.html:16` (`<script type="module" src="./edit.js">`) and the
  form defaults (`:84–108`: size `0.04` selected, colour `#ffffff`, opacity `0.6`),
  `scripts/ddphotos-edit-install.mjs:42` (copies every file in `edit/` with `readdirSync`), `.github/workflows/ci.yml:31–61`
  (the `changes` job) and `:125–151` (`avif`, the shape to copy), `.github/workflows/pages.yml`
  (toolchain + `taiki-e/install-action@wasm-pack` + `Swatinem/rust-cache@v2`), `justfile:137–231`
  (`_wasm_bin`, `wasm-build`, `wasm-build-slim`, `wasm-test`), `tests/demo_smoke.mjs` (the Node-test
  precedent in `tests/`).
- **External tools:** Pillow **12.3.0** (PyPI, 2026-07-01; its manylinux x86_64 cp312 wheel
  bundles libjpeg-turbo, libwebp and libavif; `features.check` gives jpg, webp, avif and zlib all
  True, checked locally). `ubuntu-latest` = Ubuntu 24.04, image 20261026.354.1. Its runner-images
  manifest lists Python 3.12.3, Pip 24.0 and Node.js 22.23.4, and **no ImageMagick and no
  Pillow** (`grep -i "imagemagick\|pillow"` finds nothing in `Ubuntu2404-Readme.md`).
- **Related code paths:** `src/wasm.rs` (`transform`, `info`, `initSync`), read only.
  `src/operation/mod.rs` (resize), read only. **No change to `src/`.**

## Outputs

- **Files created:**
  - `integrations/ddphotos/edit/recipe.js`: the page's recipe code as a DOM-free ES module.
  - `tests/ddphotos_parity.mjs`: the harness.
  - `tests/ddphotos_decode.py`: the independent decoder (Pillow), about 15 lines.
  - `tests/fixtures/ddphotos/photo.webp` (23,424 B) and `tests/fixtures/ddphotos/photo_rot.jpg`
    (≈128,700 B after exiftool).
  - `tests/fixtures/ddphotos/README.md`: how the fixtures were made, and their provenance.
  - `scripts/ci-code-paths.sh`: the `changes` classifier with a `--self-test`.
- **Files modified:**
  - `integrations/ddphotos/edit/edit.js`: imports from `recipe.js`; `readEdit` keeps only the
    DOM reads.
  - `integrations/ddphotos/edit/worker.js`: imports `previewRecipe` from `./recipe.js`, and its
    own `withQuality` and `PREVIEW_QUALITY` go.
  - `justfile`: a `ddphotos-parity` recipe.
  - `.github/workflows/ci.yml`: the `changes` step calls the script and runs its self-test, plus
    a new `ddphotos-parity` job.
  - `integrations/ddphotos/README.md`: "How it was tested" names the harness. The page-size
    line ("about 35 KB") gets re-measured.
- **New exports** (`recipe.js`, all pure; nothing reads `document`):
  - `tomlString(s)`
  - `textEms(text)`
  - `watermarkSize(text, outW, outH, margin, sizeFraction)`: takes the size fraction as a
    parameter, where today it reads `$("wm-size")`.
  - `planEdit(photo, form)`: the arithmetic half of `readEdit`, line for line.
    - `photo`: `{width, height}`.
    - `form`: `{size, wmOn, wmText, wmGravity, wmSize, wmColor, wmOpacity, format}`, the raw
      form values.
    - It returns today's `edit` object **plus** `outW` and `outH`.
  - `pixelRecipe(edit)`
  - `cliRecipe(edit)`
  - `previewRecipe(recipe)`: `withQuality(recipe, PREVIEW_QUALITY)`, with `PREVIEW_QUALITY = 88`.
  - `PREVIEW_QUALITY`
- **Database changes:** none.

## Acceptance Criteria

- [ ] **AC-1 `recipe.js` is the only copy of the recipe code.** `edit.js` and `worker.js`
  import from it. Neither defines `tomlString`, `textEms`, `watermarkSize`, `pixelRecipe`,
  `cliRecipe`, `withQuality` or `PREVIEW_QUALITY`, and the harness defines none of them either.
  `recipe.js` contains no `document`, `$(` or `window`.
- [ ] **AC-2 The harness runs the 16-row table below and asserts the count.** Each row builds
  its recipe with `planEdit` plus `pixelRecipe`, `cliRecipe` or `previewRecipe`, all imported
  from `recipe.js`, and nothing re-typed. It runs that recipe through the slim wasm package's
  `transform` (the real `.wasm` in Node, `initSync`) and through native `crustyimg apply
  --recipe <file> <input> -o <out> -Q` on the same input bytes. It prints one line per row and
  fails unless exactly **16** rows ran.
- [ ] **AC-3 Comparator: Rule B (DEC-110).**
  - **Dimensions first, always exact.** A mismatch prints both sizes and fails the row. The pixel
    comparison never runs on that row.
  - **Which rows are exact.** A row is *resampling* when its recipe text has an `op = "resize"`
    step. The harness derives this from the recipe text, not from the edit: `cliRecipe` always
    resizes, so rows 5 and 7 resample although their edits ask for no new size. It then asserts
    that the derived no-resample set equals the literal `{1, 3, 9, 11}`.
  - **No-resample rows:** 0 differing pixels.
  - **Resampling rows:** every channel difference ≤ 1, and at most **K** differing pixels.
    - K is one literal in the harness. The build sets it to the largest count observed on any
      resampling row, over both targets it has run (aarch64 locally, then the first x86_64 CI
      run). It records K in Build Completion and in DEC-110's Validation section.
    - K must be ≤ 78, the cap (0.01 % of 1080×723, the smallest resampling row). A row above the
      cap is BLOCKED, not a reason to raise it.
  - **Per format, on top of the pixel rule:**
    - PNG: bytes asserted on the no-resample rows (1, 3). On resampling rows bytes are reported
      only, because a 1/255 pixel within the bound changes the bytes.
    - JPEG: decoded pixels. Bytes are reported, and asserted only on the no-resample JPEG rows
      (9, 11), and only under x86 outcome (i). Rows 10 and 12 resample, so their bytes are
      reported only, for the same reason as PNG (orchestrator ruling, 2026-10-30).
    - WebP: decoded pixels, never bytes.
    - AVIF: decoded pixels.
  - Pixels are decoded RGBA from `tests/ddphotos_decode.py`, never from crustyimg. The harness
    counts the differing pixels and the largest channel difference itself. It does not use
    ImageMagick's AE.
  - N1 (opacity 0.6 → 0.61, 3/255 lossless, 13–22/255 lossy) is above the 1/255 limb, so it stays
    red under this rule.
- [ ] **AC-4 A failing row is readable.** The line names the row id, both sizes, the
  differing-pixel count, the largest channel difference and whether the bytes match. The recipe
  text that produced it follows, indented. The exit status is non-zero.
- [ ] **AC-5 Positive controls** (each a separate assertion, each red under a named mutation):
  - **P1:** each row's output size equals a per-row literal.
    - For rows 1–14, the literal also equals `planEdit`'s `outW × outH`.
    - For rows 15–16, it equals that prediction **transposed** (the `info` finding).
  - **P2:** row 2's output (1200×803) is not the input's size (1600×1071).
  - **P3:** row 3's decoded pixels (the watermark) differ from row 1's (none) on both the native
    and the wasm side.
  - **P4:** the row count is 16.
  - **P5:** the recipe text for row 4 (`pixelRecipe`, edit `both`) and row 8 (`cliRecipe`, png)
    each equal a literal string in the harness, so a change to `recipe.js` that moves both sides
    together still goes red.
- [ ] **AC-6 Negative controls, run by the harness itself.** `--self-test` runs the table twice
  more:
  - with N1, native recipe `opacity = 0.6` → `0.61`;
  - with N2, native recipe `width = 1080` → `1079`.

  Each must report exactly the rows the mutation table lists as red. Before those two runs,
  `--self-test` feeds the comparator the synthetic cases B1–B7 (FT-3), which need no binary.
  The self-test exits 0 only if every case and both runs give their listed verdicts. CI runs the
  plain run and `--self-test`.
- [ ] **AC-7 Nothing silently passes.** The harness exits non-zero and names the cause when:
  - the wasm package is missing or fails `initSync`;
  - `recipe.js` fails to import, or lacks an export that `edit.js` or `worker.js` imports;
  - the native binary is missing;
  - `tests/ddphotos_decode.py` fails, or Pillow is absent;
  - any row throws.

  There is no skip path anywhere.
- [ ] **AC-8 The page still loads (module check).** The harness:
  - imports `recipe.js` in Node;
  - parses `edit.js` and `worker.js` as ES modules (copy each to a temp `.mjs` and run
    `node --check`);
  - asserts that every name each one imports from `./recipe.js` is an export of `recipe.js`.

  **Browser check (one-time, build):** run the test bed's `just retest <build worktree path>`
  (headless Chromium, its own servers, never the maintainer's browser). Paste the browser-check
  pass/fail counts per server into the PR. A missing `recipe.js` would fail every editor check
  there.
- [ ] **AC-9 `just ddphotos-parity` runs the whole thing locally.**
  - It builds the slim package into its own output dir, through `wasm-build-slim`'s feature set
    and DEC-066's profile, never a bare cargo call.
  - It builds native release.
  - It makes a venv with the pinned Pillow under `target/ddphotos-parity/`.
  - It runs the harness, then `--self-test`.
- [ ] **AC-10 CI job `ddphotos-parity`** in `ci.yml`.
  - It runs on `ubuntu-latest`, `needs: changes`, `if: needs.changes.outputs.code == 'true'`.
  - It runs in parallel with the matrix and is not a required check.
  - It runs `just ddphotos-parity`.
- [ ] **AC-11 The path filter counts the page.** `scripts/ci-code-paths.sh` holds the single
  regex, the old one plus `integrations/ddphotos/`. The `changes` job calls it, and first runs
  `scripts/ci-code-paths.sh --self-test`, which checks fixed file lists:
  - `integrations/ddphotos/edit/edit.js` → code;
  - `docs/x.md` → not code;
  - `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-x.md` → not code;
  - `src/lib.rs` → code.
- [ ] **AC-12 Fixtures are committed bytes.** They were made outside crustyimg, with the exact
  commands and provenance in `tests/fixtures/ddphotos/README.md`. CI needs no generator.
- [ ] **AC-13 The README names the harness** under "How it was tested", and the install list is
  still right (the installer copies `edit/` whole, so `recipe.js` travels with no change).

## Failing Tests

All in `tests/ddphotos_parity.mjs` unless named. "FT" ids are what the build's completion table
cites.

- **FT-1 `rows`.** The 16 rows below run. It asserts the count literal 16 (AC-2, P4).
- **FT-2 `dims`.** Per row, native and wasm sizes are equal, and equal the row's literal (AC-3
  dims, P1).
- **FT-3 `pixels`.** Per row, Rule B (AC-3, DEC-110): exact on rows 1, 3, 9 and 11; at most
  1/255 on at most K pixels on the other twelve. It also asserts that the no-resample set derived
  from the recipe texts is `{1, 3, 9, 11}`. Each limb of the rule has a case that must go red.
  These are comparator self-tests, run by `--self-test` (AC-6) on synthetic RGBA buffers, with
  no binary involved. Every red case has a green twin one step inside the limit (HANDOFF trap 10):

  | case | input to the comparator | row kind | expected |
  |---|---|---|---|
  | B1 | one pixel differs by **2/255** | resampling | **red** (max limb) |
  | B2 | one pixel differs by 1/255 | resampling | green |
  | B3 | **K+1** pixels differ by 1/255 | resampling | **red** (count limb) |
  | B4 | exactly K pixels differ by 1/255 | resampling | green |
  | B5 | one pixel differs by 1/255 | **no-resample** | **red** (exact limb) |
  | B6 | identical buffers | no-resample | green |
  | B7 | sizes 1080×723 vs 1080×722, with buffers that would throw if compared | resampling | **red, reason `dims`**, and the pixel comparison never ran |

  For B3 and B4 at K = 0, B4 is "0 pixels" and B3 is "1 pixel", which B2 also covers. The cases
  still run.
- **FT-4 `bytes`.** PNG bytes are equal on the no-resample PNG rows (1, 3). On the resampling PNG
  rows bytes are printed, not asserted. JPEG rows' byte equality is printed on all four, and
  asserted on rows 9 and 11 only, under x86 outcome (i) (AC-3).
- **FT-5 `prediction`.** `planEdit`'s `outW × outH` equals the literal on rows 1–14, and its
  transpose on rows 15–16 (P1).
- **FT-6 `resize-moves`.** P2.
- **FT-7 `watermark-draws`.** P3, on both sides.
- **FT-8 `golden-recipes`.** P5. The literals are the texts below.
- **FT-9 `self-test`.** `--self-test`: N1 and N2 each give exactly their listed red rows
  (AC-6).
- **FT-10 `module-imports`.** AC-8's Node checks: the import, the `.mjs` parse and the export
  names.
- **FT-11 `no-silent-pass`.** Run by the build by hand, and kept as driven evidence in the PR,
  not as a CI step. Each case must exit non-zero with a message naming the cause:
  - the harness pointed at an empty package dir;
  - a `recipe.js` with an export renamed;
  - `PATH` without the venv's python;
  - a row table with row 16 deleted (needs P4, which goes red).
- **FT-12 `scripts/ci-code-paths.sh --self-test`.** AC-11's four cases. This one runs in the
  `changes` job.
- **FT-13 the browser round.** The test bed's browser-check counts (AC-8), pasted. This one is
  manual.

### Row table (one fixture pair; form values are the page defaults)

`W` = `photo.webp`, 1600×1071 lossy, the shape ddphotos publishes. `R` = `photo_rot.jpg`, the same
picture as a JPEG with EXIF orientation 6. The edits use `planEdit` with the page defaults:

- watermark text `© Jane Doe 2026`, gravity `southeast`, size `0.04`, colour `#ffffff`,
  opacity `0.6`;
- `none` = as published, no watermark;
- `resize` = size 1200;
- `wm` = as published, with the watermark;
- `both` = size 1080, with the watermark.

1080 is there because 1071 × 0.675 = 722.9 rounds to 723 but floors to 722 (HANDOFF trap 10).
1200 is there because it is the page's first size and it diverges today.

| # | id | input | edit | recipe | out | asserts | size literal | aarch64 today |
|---|---|---|---|---|---|---|---|---|
| 1 | `W/none/pixel/png` | W | none | pixelRecipe | png | bytes + dims | 1600×1071 | equal |
| 2 | `W/resize/pixel/png` | W | resize | pixelRecipe | png | dims + bound; bytes reported | 1200×803 | **1 px, 1/255** |
| 3 | `W/wm/pixel/png` | W | wm | pixelRecipe | png | bytes + dims | 1600×1071 | equal (`size = 64`) |
| 4 | `W/both/pixel/png` | W | both | pixelRecipe | png | as row 2 | 1080×723 | equal (`size = 43`) |
| 5 | `W/none/cli/png` | W | none | cliRecipe (png) | png | as row 2 (cliRecipe resizes) | 1600×1071 | equal |
| 6 | `W/resize/cli/png` | W | resize | cliRecipe (png) | png | as row 2 | 1200×803 | **1 px, 1/255** |
| 7 | `W/wm/cli/png` | W | wm | cliRecipe (png) | png | as row 2 (cliRecipe resizes) | 1600×1071 | equal |
| 8 | `W/both/cli/png` | W | both | cliRecipe (png) | png | as row 2 | 1080×723 | equal |
| 9 | `W/none/preview/jpeg` | W | none | previewRecipe | jpeg | pixels; bytes reported | 1600×1071 | equal, bytes too |
| 10 | `W/resize/preview/jpeg` | W | resize | previewRecipe | jpeg | pixels | 1200×803 | equal, bytes too |
| 11 | `W/wm/preview/jpeg` | W | wm | previewRecipe | jpeg | pixels | 1600×1071 | equal, bytes too |
| 12 | `W/both/preview/jpeg` | W | both | previewRecipe | jpeg | pixels | 1080×723 | equal, bytes too |
| 13 | `W/both/pixel/webp` | W | both | pixelRecipe | webp (lossless) | pixels, never bytes | 1080×723 | equal |
| 14 | `W/both/pixel/avif` | W | both | pixelRecipe | avif | pixels | 1080×723 | equal (wasm 1.1 s) |
| 15 | `R/resize/pixel/png` | R | resize | pixelRecipe | png | as row 2 | 803×1200 | **1 px, 1/255** |
| 16 | `R/both/pixel/png` | R | both | pixelRecipe | png | as row 2 | 723×1080 | **1 px, 1/255** |

- **Rows 2, 6, 15 and 16 are the design finding.** Under Rule B (DEC-110) they are green at
  K ≥ 1.
- **Rows 1, 3, 9 and 11 are the only rows with no `resize` step**, so they stay exact. Every
  other row resamples and is held to the bound.
- **Rows 1, 3, 5, 7 and 9–14 matched exactly on aarch64**, so the frame's exact claim holds on
  them. Row 10 is the JPEG preview of row 2: the 1/255 pixel vanished in quantisation, and the
  bytes were equal.
- **cliRecipe rows run as PNG**, so no terminal `optimize`.
- **The full wasm build gets no row.** The page ships slim, and the full build gave identical
  results to the slim one here. The full build belongs to STAGE-053's wasm-test leg.
- **Measured runtime:** the 16 rows took about 3.5 s of wasm time and 1.7 s of native time,
  plus 0.4 s of Pillow decoding, on the Mac. The AVIF row is the slow one.

### Golden recipe texts (FT-8)

Row 4, `pixelRecipe(both)`:
```
version = "2"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 1080

[[step]]
op = "watermark"
text = "© Jane Doe 2026"
gravity = "southeast"
size = 43
margin = 22
color = "#ffffff"
opacity = 0.6
```

Row 8 is `cliRecipe({...both, format: "png"})`: the same text with `format = "png"` as the
second line. Both texts were driven in the design probe (`size = 43`, `margin = 22` at 1080). The
build pastes the literals from its own first run of `recipe.js`, then checks them against these.

### Mutation table (HANDOFF trap 12: where it is applied, every row red and green)

All rows below were driven on aarch64 in the design probe, except the S-, H- and C-rows, which
are the build's to drive. "Red" means FT-3 or FT-2 fails on that row, unless another FT is
named.

| id | mutation | applied where | expected red | expected green |
|---|---|---|---|---|
| N1 | native `opacity = 0.6` → `0.61` | `tests/ddphotos_parity.mjs`, in the native input only: the string replace on the recipe text **after** it leaves `recipe.js` and **before** it is written to the `.toml` that `apply` reads. Never in `recipe.js`, or both sides move together | FT-3 on 3, 4, 7, 8, 11, 12, 13, 14, 16 (lossless max 3/255; JPEG 13–16; AVIF 22) | 1, 2, 5, 6, 9, 10, 15 (2, 6 and 15 are green under Rule B at K ≥ 1) |
| N2 | native `width = 1080` → `1079` | the same place as N1 | FT-2 on 4, 8, 12, 13, 14, 16 (native 1079×722, rotated 722×1079) | 1, 2, 3, 5, 6, 7, 9, 10, 11, 15 |
| S1 | `pixelRecipe` emits `width = ${edit.resize - 1}` | `recipe.js` (both sides move) | FT-2 literal on 2, 4, 5, 6, 7, 8, 10, 12, 13, 14, 15, 16 (cliRecipe always resizes, so 5 and 7 too), plus FT-8 | FT-3 parity on all 16; FT-2 on 1, 3, 9, 11 |
| S2 | margin `0.02` → `0.03` in `planEdit` | `recipe.js` | FT-8 only | FT-2, FT-3 and FT-5 on all 16 |
| S3 | `outH = Math.floor(…)` for `Math.round(…)` in `planEdit` | `recipe.js` | FT-5 on 4, 8, 12, 13, 14, 16 (722 vs 723) | FT-5 on 1–3, 5–7, 9–11, 15 (803.25 floors to 803); FT-2, FT-3 on all; FT-8 green (fitsHeight floors to 72 either way; the build confirms) |
| S4 | `pixelRecipe` never emits `resize` | `recipe.js` | FT-2 literal on every resize row, FT-6, FT-8 | FT-3 parity on all 16 |
| S5 | `pixelRecipe` never emits `watermark` | `recipe.js` | FT-7, FT-8 | FT-3 parity on all 16 |
| H1 | `--pkg` points at an empty directory | the harness invocation | the whole run (exit ≠ 0, "wasm package" in the message) | — |
| H2 | rename the `pixelRecipe` export | `recipe.js` | FT-10 and the run (import fails) | — |
| H3 | delete row 16 from the table | `tests/ddphotos_parity.mjs` | FT-1 (15 ≠ 16) | the other 15 rows |
| H4 | no Pillow (system `python3` on `PATH`) | the harness environment | the run (exit ≠ 0, names the decoder) | — |
| C1 | drop `integrations/ddphotos/` from the regex | `scripts/ci-code-paths.sh` | FT-12, the `edit.js` case | the other three cases |
| C2 | the regex becomes `.` | `scripts/ci-code-paths.sh` | FT-12, the two docs and spec cases | the code cases |
| B-max | drop the max limb: a resampling row passes on count alone | the comparator function in `tests/ddphotos_parity.mjs` | FT-3 self-test case B1 (so FT-9) | B2–B7; the plain run's 16 rows |
| B-count | count limb `n <= K` → `n <= K + 1` | the comparator function | FT-3 case B3 (so FT-9) | B1, B2, B4–B7; the plain run |
| B-edge | count limb `n <= K` → `n < K` | the comparator function | FT-3 case B4 (so FT-9); the plain run, on the target where K was observed, on each row where n = K | B1–B3, B5–B7 |
| B-exact | no-resample rows use the bound too | the comparator function | FT-3 case B5 (so FT-9) | B1–B4, B6, B7; the plain run (rows 1, 3, 9, 11 are equal anyway, which is why B5 exists) |
| B-dims | compare pixels before dims, or skip the dims check | the comparator function | FT-3 case B7 (throws, or a reason other than `dims`); N2's rows in FT-9 | B1–B6 |
| B-derive | derive "resamples" from the edit (`edit.resize != null`), not the recipe text | the row classifier in `tests/ddphotos_parity.mjs` | FT-3's set assert (`{1, 3, 5, 7, 9, 11}` ≠ `{1, 3, 9, 11}`) | B1–B7 (synthetic, classifier not used) |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### The ruling: Rule B (DEC-110)

Ruled 2026-10-30. Read DEC-110 before the comparator. It holds the rule, the exact rows, K's
provenance and the cap (78). The build sets K and writes it back into DEC-110's Validation
section. It never edits the rule or the cap.

### The independent decoder: Pillow 12.3.0

- **Why Pillow:**
  - It decodes all four output formats with libraries that are not crustyimg's: libjpeg-turbo,
    libwebp, libavif with dav1d, and zlib.
  - It installs from one pinned wheel in about 5 s.
  - It works on both the Mac and the runner.
- **Why not the alternatives:**
  - ImageMagick is not on `ubuntu-latest`, and apt's 6.9 is old and slow to install.
  - A Node package would mean the repo's first `package.json` for tests; there is none.
  - An in-harness PNG reader can't decode the JPEG, WebP or AVIF rows.
- **`tests/ddphotos_decode.py`** takes paths. For each one it writes `<path>.rgba` (raw RGBA8,
  `Image.open(p).convert("RGBA").tobytes()`) and prints one JSON line `{path, width, height}`.
  It runs once per harness run, with every output file, under `python -I`.
- **CI:**
  - `actions/setup-python@v5` with `python-version: "3.12"`. Ubuntu's system pip is externally
    managed, so it is not used.
  - Then the just recipe: `python3 -m venv target/ddphotos-parity/venv` and
    `venv/bin/pip install pillow==12.3.0`. The pin is one `justfile` variable.
  - Locally the same recipe makes the same venv; it needs `python3`.

### The `recipe.js` import shape

- `edit.js` is already loaded as a module (`index.html:16`), so it can just add
  `import { planEdit, pixelRecipe, cliRecipe } from "./recipe.js";`. It keeps its other
  functions, and `readEdit()` becomes the DOM reads plus `planEdit(photo, form)`.
- `worker.js` is a module worker (`edit.js:25`, `{ type: "module" }`), so it can add
  `import { previewRecipe } from "./recipe.js";`. Its preview becomes
  `transform(source, previewRecipe(req.recipe), "jpeg")`.
- There is no build step, and the page stays a static file set.
- The installer copies `edit/` whole (`scripts/ddphotos-edit-install.mjs:42`). The README's
  "replace the whole `edit` folder" and the DD Photos app steps already move every file. Nothing
  in the README or #224/#227 assumed one file.
- ⚠ **One test-bed gap, theirs to fix, not ours:** the test bed's HTTP check lists `edit.js`,
  `worker.js`, `edit.css` and `vendor/*` by name (e.g. `results/2026-10-29-71b4f38/check.md:7–10`),
  so it will not report `recipe.js`'s MIME type. Its browser check still fails if `recipe.js` is
  missing. Tell the maintainer; never edit the test bed.

### The harness

- **Invocation:**
  `node tests/ddphotos_parity.mjs --pkg <slim pkg dir> --bin <crustyimg> --python <venv python> [--self-test]`.
  Every argument is required, with no defaults that could point at a stale build.
- **It writes to a fresh `fs.mkdtempSync` dir per run.** `apply` refuses to overwrite: driven, it
  exits 5 with "output file already exists". Remove the dir on success; keep it and print its
  path on failure.
- **Comparator (Rule B, DEC-110):**
  - Dimensions first. On a mismatch, fail with reason `dims` and return without touching the
    buffers.
  - Then bytes (`Buffer.compare`), reported on every row and asserted only where AC-3 says so.
  - Then the RGBA buffers: count the pixels where any channel differs, and take the largest
    absolute channel difference.
  - The verdict:
    - a no-resample row (no `op = "resize"` in its recipe text) needs `px == 0`;
    - a resampling row needs `max <= 1 && px <= K`.
  - `K` and the cap `78` are named constants at the top of the file, each with a comment citing
    DEC-110, and the harness asserts `K <= 78` at start.
  - The verdict is one pure function of `(dims, bytesEqual, px, max, resamples, format)`, so
    `--self-test` can drive cases B1–B7 through it without a binary.
- **The one-line format:** `PASS|FAIL <id> native WxH wasm WxH bytes same|differ px=<n> max=<d>`.
  The output reaches the CI log because it is plain stdout from a `run:` step, not libtest
  (HANDOFF trap 11). Put the self-test in its own step so its lines are separate.

### The CI job (`ci.yml`)

```yaml
  ddphotos-parity:
    needs: changes
    if: needs.changes.outputs.code == 'true'
    name: ddphotos page parity (native vs wasm)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: taiki-e/install-action@just
      - uses: taiki-e/install-action@wasm-pack   # as pages.yml; wasm-pack fetches wasm-bindgen 0.2.126 from Cargo.lock
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - uses: Swatinem/rust-cache@v2
        with:
          key: ddphotos-parity
      - name: Parity harness
        run: just ddphotos-parity
      # STAGE-053's wasm32 leg adds ONE step here:
      #   - run: cargo install wasm-bindgen-cli --version 0.2.126 --locked && just wasm-check && just wasm-test
      #     (or taiki-e/install-action with tool: wasm-bindgen@0.2.126)
```

- **Minutes, estimated (not measured on CI).** Measured on the Mac: native release 23 s, and
  slim wasm with fat LTO 48 s, from fresh target dirs. On a 4-vCPU runner without cache, expect
  about 3–4 min native and 3 min wasm. Setup takes about 1 min, and the harness under 30 s.
  - Cold: **about 8–9 min**.
  - With `rust-cache` warm: **about 4–5 min**.
  - Comparisons:
    - `pages.yml`'s full-wasm job took 178 s on its last green run.
    - The Windows check leg, which sets time to green, took 649–1129 s on recent code runs.
  - So the job runs in parallel and time to green does not move. Runner-minutes grow by about
    5–9 per code PR.
- **The build's first run gives the real number.** Record it in the PR.
- **STAGE-053's chore** becomes one added step in this job: the `wasm-bindgen-cli 0.2.126`
  install, then `just wasm-check && just wasm-test`. The toolchain, the wasm target and the
  cache are already there. The comment in the YAML above marks the spot.

### Fixtures (`tests/fixtures/ddphotos/`)

Made with ImageMagick 7.1.2-34 and exiftool from `tests/fixtures/classify/color_photo_fuji.png`,
which is maintainer-owned and EXIF-stripped (`tests/fixtures/classify/RECIPES.md:91–92`). They
carry the repo's licence (MIT or Apache-2.0).

```
magick tests/fixtures/classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 82 -define webp:lossless=false photo.webp
magick tests/fixtures/classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 90 photo_rot.jpg
exiftool -q -n -Orientation=6 -overwrite_original photo_rot.jpg
```

- **Sizes:** `photo.webp` is 23,424 B at 1600×1071. `photo_rot.jpg` is about 128,700 B, also
  stored as 1600×1071, with Orientation 6. Together that is about 150 KB, against 1.1 MB of
  fixtures today.
- **Commit the bytes and record the commands** in the fixture README. CI needs no ImageMagick.
- **Rebuilding them does not reproduce the bytes.** ImageMagick's version shows in the encoder
  output, so a rebuild changes the row literals' *pixels*, not their sizes. Don't regenerate
  them casually.

### x86_64: both outcomes pre-specified

The build's **first checkpoint**: a WIP **draft PR** with the harness, the fixtures,
`recipe.js` and the CI job, pushed as soon as `just ddphotos-parity` passes locally. Read its
first run with a backgrounded `gh pr checks <n> --watch`, never a poll loop. That run decides:

Set K on the first local run to the aarch64 maximum (1 today, from rows 2, 6, 15 and 16), and
push. Then:

- **(i) x86 agrees** (every row within Rule B, and JPEG rows 9 and 11 byte-equal on both targets):
  - assert JPEG bytes on rows 9 and 11 (rows 10 and 12 resample, so their bytes are reported only);
  - set K to the largest differing-pixel count on any resampling row over **both** targets
    (≤ 78);
  - record K, the targets and the CI run id in Build Completion and in DEC-110.
- **(ii) JPEG bytes differ, but every row's pixels are within the bound:**
  - assert pixels only on rows 9–12, which is the designed rule; this is not a finding;
  - set and record K as in (i). WebP bytes are never asserted anyway.
- **Anything over the bound is BLOCKED.** That is:
  - any difference on a no-resample row (1, 3, 9, 11);
  - a channel difference above 1/255 on any row, including a lossy row where a 1/255
    pre-encode difference grew through JPEG or AVIF;
  - a differing-pixel count above 78 on any row;
  - a dimension difference.

  **Don't loosen the rule and don't raise the cap.** Push the WIP and report BLOCKED with the
  row, its numbers and whether its PNG twin also moved. AVIF row 14 is held to the same rule:
  don't drop the row.

### Decisions that apply

- `DEC-110`: Rule B, the comparator. Dimensions exact; rows 1, 3, 9 and 11 exact; resampling rows
  within 1/255 on at most K pixels, K ≤ 78. The build sets K and records it there.
- `DEC-064`: `transform` runs the same core as `apply`. This spec measures that claim.
- `DEC-065`: wasm can't decode AVIF, so row 14 is decoded by Pillow.
- `DEC-066`: the size profile. The job builds through `wasm-build-slim`'s path, so it tests the
  shipped profile and moves no size gate.
- `DEC-008`: resizing goes through `fast_image_resize`'s SIMD backend. That is the probable seat
  of the finding (Rule A would amend it).
- `DEC-092`: resizing is done in linear light, in `f32` (the `powf` half of the finding).
- `DEC-059`: there are no cross-arch byte pins. The harness only ever compares native and wasm on
  one runner.
- `DEC-094`: the AVIF tile count is pinned, which is why AVIF bytes matched.
- `DEC-104`: the watermark refuses a mark that doesn't fit; `watermarkSize`'s cap exists because
  of it.
- `DEC-009`: CI is the three-OS matrix plus single-runner extra jobs. This job is one more
  advisory job, like `avif` and `lean`, and the required checks don't change.
- `DEC-018`: the licence policy for Rust crates. Pillow (MIT-CMU, permissive) is a CI-only
  test tool, never shipped or linked, so cargo-deny doesn't see it and needs no exception.

### Constraints that apply

- `test-before-implementation`: the harness and its self-test land before, or with, the
  `recipe.js` move. The move is proven by FT-10 and the row table.
- **No change to `src/` and no shipped-binary change.** This was confirmed at design: the harness
  is test-only, and `recipe.js` is page code.

### Prior related work

- `SPEC-124` (shipped): pinned the AVIF tile count.
- `SPEC-127` (shipped): set `transform`'s `out_format` / `recipe.format` precedence (the cli
  rows' `format = "png"` meets `transform(…, "png")`).
- `SPEC-128` (shipped): `transform` refuses steps that carry assets; there are none here.
- PR #218: the page. PRs #224 and #227: the test-bed rounds.

### Size

S stays S under (B). There is about 40 lines more than the frame's estimate: `planEdit`, the
self-test and the classifier script. Under (A), SPEC-142 stays S, but it **waits** on a new engine
spec, probably [M], that is not in PROJ-017's build order. Neither frame cut is needed, and
neither is applied.

### Out of scope (for this spec specifically)

- **Replay parity (b)**, `cliRecipe` on a 4x original vs `pixelRecipe` on the published copy:
  STAGE-060's crop spec, which owns the 4x replay and is the first spec that can break it. Two
  cautions to record there:
  - ddphotos' real photogen resizer sets the bound, not ImageMagick's.
  - The original's and the copy's aspect ratios round independently, so the output can be off
    by ±1 px in one dimension.
- **Crop and straighten rows:** added by STAGE-060's crop and straighten specs. The crop spec
  must handle the `info`-reports-stored-size finding.
- **Parity of the page's final encode with the CLI's:** the page's `optimizeDetailed` search and
  the CLI's terminal `optimize` differ by design (README "Limits"). Not a promised property.
- **Running `just wasm-test` / `wasm-check` in CI:** STAGE-053's "A wasm32 CI leg" chore, which
  becomes one step in this job.
- **The three-server test-bed browser round:** STAGE-060's last item. This spec needs one run,
  for AC-8, no more.
- **Any change to `src/`:** under (A), that is a separate engine spec. If the x86_64 run finds a
  divergence beyond the ruled comparator, file it as a bug; don't fix it here.

## Notes for the Implementer

- **`node --check` on a `.js` file without `"type": "module"`** may parse it as CommonJS. Copy it
  to a temp `.mjs` first; the `.mjs` extension forces an ES module.
- **Point `--bin` at a release build of this checkout** made in a target dir the harness owns
  (`target/ddphotos-parity/`), never a `crustyimg` off `PATH`. A released binary and a `main`
  build report the same version.
- **Build the wasm package into `target/ddphotos-parity/pkg-slim`** through
  `just --set _wasm_out … wasm-build-slim`'s feature set. Don't overwrite the repo's `pkg-slim/`
  that `ddphotos-edit-install` uses.
- **Prettier-ignore** stays on the advance table when it moves.
- **`recipe.js` comments describe behaviour**, with no spec or DEC ids.
- **Commit with `git commit -s`.** Push a WIP once the harness compiles and runs (the first
  checkpoint above).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `test/spec-142-ddphotos-parity`
- **PR (if applicable):** #234 (draft), head `f204d43` at the last CI check
- **All acceptance criteria met?** yes (table below)
- **New decisions emitted:** none. DEC-110 gains its measured K (Validation section).
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`, plus this spec file):
  - `.github/workflows/ci.yml`: `changes` step calls the classifier and its self-test; new advisory `ddphotos-parity` job (first step logs the runner CPU).
  - `decisions/DEC-110-native-and-wasm-agree-on-dimensions-exactly-and-on-resampled-pixels-within-1-255.md`: Validation section records K = 1.
  - `integrations/ddphotos/README.md`: names the harness under "How it was tested"; page size line re-measured to about 38 KB (38,228 B across the five page files).
  - `integrations/ddphotos/edit/edit.js`: imports `cliRecipe`, `pixelRecipe`, `planEdit` from `recipe.js`; `readEdit` is the DOM reads plus `planEdit`.
  - `integrations/ddphotos/edit/recipe.js`: new, the page's recipe code as a DOM-free module.
  - `integrations/ddphotos/edit/worker.js`: imports `previewRecipe`; its own `withQuality` and `PREVIEW_QUALITY` are gone.
  - `justfile`: `ddphotos-parity` recipe and the Pillow pin.
  - `scripts/ci-code-paths.sh`: new, the one path regex with `--self-test`.
  - `tests/ddphotos_decode.py`: new, the Pillow decoder.
  - `tests/ddphotos_parity.mjs`: new, the harness, comparator and `--self-test`.
  - `tests/fixtures/ddphotos/README.md`, `photo.webp` (23,424 B), `photo_rot.jpg` (128,696 B): new fixtures and provenance.
  - this spec file: cost entry, this section.
  - **`git diff --stat origin/main...HEAD -- src/ Cargo.toml Cargo.lock` is empty.**
- **Outcome: (i).** Every row is within Rule B on both targets and JPEG rows 9 and 11 are byte-equal, so JPEG bytes are asserted on those two rows (`JPEG_BYTES_ASSERTED = true`).
  - **K = 1** (rows 2, 6, 15, 16: 1 pixel, max 1; every other row `px=0 max=0`), identical on both targets.
  - Targets: aarch64-apple-darwin locally, and x86_64 `ubuntu-latest`, `AMD EPYC 9V45 96-Core Processor`, flags avx2, avx512f, sse4_1. CI run 41144102452 (first), 41147305687 (head `f204d43`).
  - **The 16 wasm output SHA-256 hashes are identical on the Mac and on the runner** (diffed line by line), as are the per-row `px`/`max`.
  - Job wall time: 3m22s cold (first run; 3m07s of that is `just ddphotos-parity`), 1m45s warm (`f204d43`).
- **Golden recipe texts:** the literals in the harness are the spec's two texts; `recipe.js` output equals them (P5 green on the first run), so there is no difference to report.
- **Mutation rows** (red/green sets are as the table lists unless noted; each was applied with a non-empty `git diff`, run, then restored and shown green):
  - N1 red {3,4,7,8,11,12,13,14,16}; N2 red {4,8,12,13,14,16}. Both are asserted by `--self-test` and pass.
  - S1 red (literal size) {2,4,5,6,7,8,10,12,13,14,15,16} + P5; parity green on all 16.
  - S2 red P5 only. S3 red {4,8,12,13,14,16} (size prediction), P5 green (confirmed). S4 red on every resize row, P2, P5, plus the no-resample-set assert. S5 red P3 (both sides) and P5.
  - H1 exit 1 "wasm package missing or empty". H2 exit 1 "edit.js imports pixelRecipe, which recipe.js lacks". H3 exit 1 "P4: 15 rows ran, expected 16" (the other 15 pass). H4 exit 1 "the Pillow decoder failed".
  - C1 red on the `edit.js` case only. C2 red on the two docs/spec cases only.
  - B-max red B1. B-count red B3. B-exact red B5. B-dims red B7 ("pixels were read"). B-derive red on the set assert ({1,3,5,7,9,11}). B-edge red B4 and, in the plain run, rows 2, 6, 15, 16 (n = K = 1).
  - **Note on B-edge:** at K = 1, B2 ("one pixel off by 1") is the same input as B4, so B2 goes red too. The table lists B2 as green. At K = 0 they would differ.
  - **Added (not in the table):** B8, B9, B10 for the byte limbs. `JPEG_BYTES_ASSERTED = false` turns B8 red; dropping PNG from the byte check turns B10 red.
- **FT-11, by hand (exit 1 each):**
  - empty package dir: `FATAL: wasm package missing or empty: expected …/crustyimg.js and …/crustyimg_bg.wasm`
  - renamed export: `FAIL modules: edit.js imports pixelRecipe, which recipe.js lacks`
  - system python without Pillow: `FATAL: the Pillow decoder failed (/usr/bin/python3): … No module named 'PIL'`
  - row 16 deleted: `FAIL P4: 15 rows ran, expected 16`
- **FT-13, the browser round:** `just retest <this worktree>`, headless, on the test bed's own servers. Browser checks: dev 40 pass / 0 fail, static 40 / 0, apache 40 / 0 (21 `editor:` checks each, none skipped). **The test bed's automated stage then crashed**: `scripts/check.py:247: RuntimeError: edit.js has no GO_ADVANCE / GO_WIDEST`. It reads the font table out of `edit.js`, and the table now lives in `recipe.js`. That is the test bed's gap (it also names files one by one, so it will not list `recipe.js`'s MIME type); not edited here.
- **CI:** the matrix legs ran (not skipped): ubuntu, macos, windows, avif, webp-lossy, heic, lean, msrv all green on the second run, `changes` job included.
- **Deviations from spec:**
  - `JPEG_BYTES_ASSERTED` flag and comparator cases B8-B10 added (outcome (i) needs a test that goes red).
  - The recipe always recreates its venv. The warm CI run failed with `venv/bin/pip: No such file`: `rust-cache` restored `target/` with `venv/bin/python` but without pip, and the "is there a python" guard passed.
  - The justfile recipe runs the plain run and `--self-test` in one step, so the self-test is not in its own CI step as the spec suggested; both print into the same log.
- **Follow-up work identified:**
  - The test bed's `check.py` must read the font table from `recipe.js` (and list `recipe.js` in its HTTP check).
  - S4 shows a recipe that never resizes also trips the resample-set assert; harmless.
  - STAGE-060 crop/straighten specs add rows to `TABLE`; K and the cap are DEC-110's.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Very little. The row table, comparator and mutation table were complete. No design call turned out wrong; the x86_64 run agreed with aarch64 on every row.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The test bed reads `edit.js` for the font table, so moving the recipe code breaks its automated stage; and `rust-cache` restores `target/` partially, which breaks any venv kept there.

3. **If you did this task again, what would you do differently?**
   — Commit before running mutation scripts that restore with `git checkout -- file` (it reverted uncommitted edits once), and recreate the venv from the start.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
