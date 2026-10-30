---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-142
  type: task                       # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
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

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- **Replay parity (b)**, `cliRecipe` on a 4x original vs `pixelRecipe` on the published copy:
  STAGE-060's crop spec, which owns the 4x replay and is the first spec that can break it.
- **Crop and straighten rows:** added by STAGE-060's crop and straighten specs.
- **Parity of the page's final encode with the CLI's**: the page's `optimizeDetailed` search and
  the CLI's terminal `optimize` differ by design (README "Limits"). Not a promised property.
- **Running `just wasm-test` / `wasm-check` in CI:** STAGE-053's "A wasm32 CI leg" chore. Shape
  the new job so that chore is one more step.
- **The three-server test-bed browser round:** STAGE-060's last item. This spec needs one
  browser load check of the page after the `recipe.js` move, no more.
- **Any change to `src/`:** the premise held, so there is nothing to fix. If the x86_64 run finds
  a divergence, file it as a bug; do not fix it here.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
