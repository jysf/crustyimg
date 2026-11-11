---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-159
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-017
  stage: STAGE-058
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-10

references:
  decisions:
    - DEC-095
    - DEC-106
    - DEC-110
    - DEC-112
    - DEC-113
    - DEC-114
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - single-image-library
    - every-public-fn-tested
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-141
    - SPEC-151
    - SPEC-152
    - SPEC-153

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-058's promised `pad` 'filled with a colour or transparent', and the rule for what a transparent fill does in a format that cannot store it"

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
      date: 2026-11-10
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5736088
      tokens_breakdown:
        input: 86
        cache_creation: 148478
        cache_read: 5547638
        output: 39886
      estimated_usd: 4.70
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "From the session JSONL, deduped by message.id (43 messages), measured after the frame push"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-159: pad with a transparent fill, and the no-alpha output rule

## Context

STAGE-058 promises a `pad` step "filled with a colour or transparent". SPEC-151 shipped `pad`
with an opaque fill only and refuses the rest when the recipe loads (`src/operation/pad.rs`,
`from_params`: "a fill with transparency is not supported yet"). SPEC-151's frame split
transparency out, and the maintainer ruled on 2026-11-06 (SPEC-151's Q1 and Q4) that it is its
own spec and that `transparent` into a format with no alpha is **refused per input, exit 4,
naming the step and the format**. On 2026-11-10 the maintainer ruled that it ships before
`edit --pad`/`--trim` (SPEC-160), so `--pad-color` takes `transparent` the day that flag ships.
Backlog item: STAGE-058, "`pad` with a transparent fill, and what `transparent` does in a format
with no alpha". Project: PROJ-017.

## Goal

Today a CLI user can't pad an image with a transparent or see-through border: `color =
"transparent"` and `"#RRGGBBAA"` below `ff` are refused at load (exit 1). When this ships, a
recipe `pad` takes either one, the new pixels carry that alpha (widening the image to an alpha
type when needed), and a recipe that would write a fill alpha the output format can't store is
refused for that input (exit 4, naming the step, the fill and the format) instead of being
flattened silently by the encoder.

## Frame

*Framed 2026-11-10 on `main` at `9bbdaf1` (0.8.2 plus SPEC-153). Every behaviour quoted below
was driven on a release build of that commit in a scratch `CARGO_TARGET_DIR`, deleted after;
fixtures made with ImageMagick 7 and read back with `magick … txt:-`, `file`, `tiffinfo`,
`sips` and `ffprobe`.*

### Verdict: GO at [M], one spec, one DEC (DEC-115)

The rulings hold against the code and nothing in the evidence argues for a split. The fill is
small; the weight is the format check on four surfaces and its DEC. Sized [M] (see "Sizing").

### Premise check (confirmed)

Recipe `version = "2"`, one `[[step]] op = "pad"`, `border = 0.25`, on a 4×4 RGB8 PNG,
`crustyimg apply --recipe r.toml -o out.png -y rgb.png`:

- `color = "transparent"` → exit 1: `error: recipe step 0 ('pad'): a fill with transparency is not supported yet; 'color' must be opaque, such as "#ffffff", got "transparent"`
- `color = "#ff000080"` → exit 1, same message with `got "#ff000080"`
- `color = "#ff000000"` → exit 1, same message
- `color = "#ff0000ff"` → exit 0 (opaque, as today)
- `color = "TRANSPARENT"` → exit 1: `'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "TRANSPARENT"`

So `transparent` is a keyword `pad` checks itself (lower case only); `parse_color`
(`src/text/mod.rs:51`) knows only hex and returns `[r, g, b, a]` straight from `RRGGBBAA`
(`ff000080` → `[255, 0, 0, 128]`, its own unit test). `trim` reads its `color` through
`parse_color` too, so `trim` does **not** accept `transparent` today (driven: `trim color = "transparent"` → exit 1, `recipe step 0 ('trim'): 'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "transparent"`); its docs spell a transparent
colour `"#00000000"` (`docs/recipes.md:139`).

### Today: source alpha into JPEG, with no `pad` (driven)

Source: 4×4 RGBA8 PNG, left half opaque red, top-right (0,255,0,0), bottom-right (0,0,255,128).
Recipe: one `flip` (so the pipeline runs and nothing is padded).

- `apply --recipe flip.toml -o o.jpg -y rgba.png` → **exit 0, nothing on stderr.** The fully
  transparent green came out `(0,255,7)` opaque, the half-transparent blue `(0,17,255)` opaque.
- `apply --recipe flip.toml --format jpeg --out-dir bout -y a.png b.png` → **exit 0, nothing on
  stderr**, `a.jpg` and `b.jpg` written, same pixels.

JPEG drops alpha and keeps whatever colour sat under it, on every path, silently. (SPEC-151's
frame saw the same on 0.8.2: transparent red came out `srgb(254,0,0)`.) So `#00000000` would give
a **black** border in JPEG, which is exactly why the ruling refuses it.

### The format table (driven, same RGBA source, `-o o.<ext>`, all exit 0, nothing on stderr)

The CLI writes eight formats (`sink::format_from_extension`, `src/sink/mod.rs:301`): png, jpeg,
gif, bmp, tiff, ico, webp, avif. Pixel (0,0) is the alpha-0 green, (0,2) the alpha-128 blue:

- **png**: keeps alpha. `(0,255,0,0)` and `(0,0,255,128)`, exact.
- **webp** (lossless, the default build): keeps alpha, exact.
- **bmp**: keeps alpha (32-bit BMP), exact.
- **ico**: keeps alpha (PNG inside), exact.
- **tiff**: keeps alpha for crustyimg (`info`: `rgba8`, `alpha: yes`), macOS (`sips`: `hasAlpha:
  yes`) and ffmpeg (`pix_fmt=rgba`, alpha bytes 00 and 80 read back). ⚠ libtiff and ImageMagick
  warn "Sum of Photometric type-related color channels and ExtraSamples doesn't match
  SamplesPerPixel" and show it opaque: `image`'s TIFF encoder writes no `ExtraSamples` tag
  (ImageMagick's own RGBA TIFF has `Extra Samples: 1<unassoc-alpha>`). Out of spec; filed in
  `guidance/ideas.yaml` (`tiff-alpha-extrasamples`). Classed as "keeps alpha" here.
- **avif**: keeps alpha, lossily: alpha 128 read back as 125, and the colour under alpha 0 is not
  kept (`(51,0,200,0)`). Alpha survives; that is all this rule needs.
- **gif**: **1-bit.** Alpha 0 stays fully transparent (`(0,255,0,0)`). Alpha 128 became **opaque
  blue** `(0,0,255,255)`. A ramp of alpha 1, 127, 254, 255 all came out opaque red, so the
  threshold is "alpha 0 is transparent, anything else is opaque with its colour".
- **jpeg**: **no alpha.** Both pixels opaque, as quoted above.

The wasm build carries `png`, `jpeg`, `gif`, `webp` and `avif` (`Cargo.toml:235`, plus the
`avif` feature its `just wasm-*` recipes pass), so wasm `transform` can write both no-alpha
formats and needs the check.

### Fill semantics (recommended; content for design)

- **`transparent` means `#00000000`**: zero alpha over black. Why black and not the edge colour:
  (1) `resize` premultiplies, so the colour under zero alpha never bleeds: an 8×8 image half red,
  half transparent, resized exact to 3×3 gave byte-identical output whether the transparent half was
  `(0,0,0,0)` or `(255,255,255,0)` (driven: `(255,0,0,255)`, `(255,0,0,128)`, `(255,0,0,0)` on both);
  (2) AVIF doesn't keep the colour under zero alpha anyway; (3) `trim` matches any alpha-0 pixel to
  an alpha-0 colour whatever is underneath (`src/operation/trim.rs:249`); (4) `(0,0,0,0)` is grey,
  so a grey image padded with `transparent` widens to grey+alpha, not RGB. An edge colour would make
  the fill depend on the image, which a pad that computes no pixel shouldn't.
- **`#RRGGBBAA` is straight alpha, stored as given.** `#ff000080` writes `(255,0,0,128)`;
  `#ff000000` writes `(255,0,0,0)` (not normalised to black). Nothing is blended: every input pixel
  is copied unchanged and every new pixel is the fill, as today.
- **16 bits:** every channel, alpha included, is `c × 257` (`wide()` in `pad.rs`), so `80` is
  32896 and `ff` 65535. `trim` scales its colour with the same `× 257` step (`trim.rs`, `step`), so
  the two agree. **Float** (`Rgb32F`/`Rgba32F`): `c / 255` (`unit()`), alpha included.
- **Keyword spelling:** `transparent`, lower case only, as today. Recommend one small helper shared
  by `pad` and `trim` (e.g. `operation::parse_fill_color`) that maps `transparent` to `[0,0,0,0]`
  and otherwise calls `parse_color`, **not** a change to `parse_color` itself. That keeps
  `watermark --color` untouched and leaves SPEC-160's sign fix inside `parse_color` with no
  conflict. `trim` accepting `transparent` is in scope because `docs/recipes.md:142` promises "`pad`
  then `trim` with the same colour gives back the original", which would be false for the keyword
  otherwise. (Maintainer question Q2.)

### Widening (DEC-095 stands; no amendment)

When the fill's alpha is below `ff` (call it "translucent"):

- `L8` → `LA8` for a grey fill (`r == g == b`, which includes `transparent`), `RGBA8` for a colour fill
- `L16` → `LA16` / `RGBA16`, same split
- `LA8` / `LA16` → unchanged for a grey fill, `RGBA8` / `RGBA16` for a colour fill (as today)
- `RGB8` → `RGBA8`; `RGB16` → `RGBA16`; `Rgb32F` → `Rgba32F`
- `RGBA8` / `RGBA16` / `Rgba32F` → unchanged
- Bit depth never changes.

When the fill is opaque: no change from today. `L8` stays `L8` for a grey fill and widens to `RGB8`
for a colour fill; `RGB8` stays `RGB8`; an `RGBA` source stays `RGBA` (it should, and does). What
pins it: `tests/pad.rs`'s colour-type rows (it names `rgb8`/`rgba8`/… per output), `pad.rs`'s
`no_op_returns_the_input`, and the SPEC-141 corpus guard's two `pad` recipes
(`tests/fixtures/recipe_corpus/v2_pad_aspect.toml`, `v2_pad_border.toml`).

An image already at the aspect is returned unchanged with its own type, as today, even with a
translucent fill: no fill pixel is written, so nothing widens.

DEC-095's alpha clause ("drop the channel only when the input had none *and* every alpha sample is
opaque") already implies "keep a channel that has non-opaque samples", and its chroma clause already
covers grey → RGB for a colour fill (a "colour watermark over a gray base genuinely adds chroma and
stays RGB"). So the widening is DEC-095 applied, not amended. `pad` doesn't call `narrow_rgba8` (it
builds the output type directly); design states the equivalence in `pad`'s doc comment rather than
routing through the narrow helpers.

### The no-alpha rule, made exact (recommend; content for DEC-115)

**Each format's alpha capacity**, from the drives above: *none* = JPEG; *binary* (alpha 0 or
opaque only) = GIF; *full* = PNG, WebP, AVIF, TIFF, BMP, ICO.

**The rule:** a recipe whose `pad` step has a fill alpha the resolved output format can't store
exactly is refused for that input, before its pipeline runs:

- JPEG: any fill alpha below `ff` is refused.
- GIF: a fill alpha strictly between `00` and `ff` is refused; `transparent` and `#RRGGBB00` are
  **allowed**, because GIF stores alpha 0 exactly (driven), and an opaque fill is allowed.
  Recommended over "refuse every non-opaque fill to GIF" because the pixel written is exactly the
  one asked for; GIF today turns `#ff000080` into opaque red with exit 0, which is the case to
  refuse. (Maintainer question Q1.)
- Every other format: allowed.

**Static, not pixel-based.** The check reads the recipe and the format, not the output pixels. A
later step that makes the image opaque again doesn't lift it: nothing does that today except a
`crop` or `trim` that cuts the border back off, and that recipe is contrived. A static check is
predictable (the same recipe is refused for every JPEG input, including an image already at the
aspect where no fill pixel would be written), cheap (before the decode-heavy work), and doesn't
touch source-alpha images, so no existing output changes. A pixel check would also catch the
source-alpha case below and change today's exit 0 into a refusal, which STAGE-058 forbids.

**The asymmetry, and why it is acceptable here.** With the static rule, an RGBA PNG to JPEG with
no `pad` still drops alpha silently (exit 0); only alpha that `pad` brought is refused. That is the
ruling's scope and keeps every existing output identical. The docs must say so in one line. The
source-alpha case is real but sink-wide (every op, every verb, `convert` too), not geometry, so it
is filed as its own entry, not folded: `guidance/ideas.yaml`, `alpha-dropped-silently-on-encode`
(a warning on stderr, like JPEG's existing 16→8-bit warning `eight_bit_downgrade_warning`, would
change no bytes).

**The message** (exit 4, design words it; it names the step index, `'pad'`, the fill and the
format, and the way out):

```
error: recipe step 2 ('pad'): the fill "transparent" has alpha, and JPEG output can't store it; write PNG, WebP, AVIF or TIFF instead (-o out.png or --format png)
error: recipe step 0 ('pad'): the fill "#ff000080" is partly transparent, and GIF output stores only fully transparent or opaque pixels; use an alpha of 00 or ff, or write PNG
```

### Is it a DEC? Yes: DEC-115 (recommended content; design writes it)

It is the first step whose validity depends on the output format, it binds every surface (CLI,
`build`, wasm) and every future op that can introduce alpha, and it fixes the asymmetry on purpose.
Content:

- A recipe step that can write a sample the resolved output format can't store exactly is refused
  for that input before the pipeline runs: exit 4 for one input, the per-input failure in a batch
  (exit 6), a thrown error in wasm. The message names the step index, the op, the value and the
  format.
- The check is static (recipe × format), never a scan of the output pixels.
- Alpha capacity per format: none (JPEG), binary (GIF: alpha 0 or opaque), full (PNG, WebP, AVIF,
  TIFF, BMP, ICO). AVIF is lossy in alpha but keeps it.
- Scope: alpha a step introduces. Alpha the source carried is out (today's silent drop stands, and
  the docs say so); the `alpha-dropped-silently-on-encode` idea owns it.
- Flattening against a colour is rejected (ruled 2026-11-06).
- Where the format is decided after the pipeline (the auto-decide path), no check is needed because
  that decision already refuses JPEG for any alpha type and never offers GIF.

DEC-095 stands unamended (see "Widening").

### Where the format is known, per surface (read from the code)

- **`apply`, one input** (`src/cli/optimize.rs` ~:150-195): decodes, runs the pipeline, *then*
  calls `output_format_for` (`src/cli/ops.rs:214`). Every input to that call (flags, `-o`,
  template, `recipe.format`, `img.source_format()`) is known right after decode, so the build moves
  the call before `pipeline.run` and checks there. Known in time: yes.
- **`apply`, a batch** (`apply_one` → `encode_one`, `src/cli/common.rs:72`): `format_override`
  (`--format` > template extension > `recipe.format`) is resolved once before any input, and the
  per-input fallback is `img.source_format()` after decode, before `run_with`. Known in time: yes.
  Recommend: when `format_override` is `Some`, check once before any input is read (exit 4, nothing
  written, like a bad `--format`); otherwise check per input in `encode_one` (the preserved-source
  rung, e.g. a folder of JPEGs with no `--format`, which gives exit 6 with one error line per input).
- **`apply` with a terminal `optimize` step, pinned format** (`--format`, `-o` ext or
  `recipe.format`): goes to `run_pixel_op` with `Some(fmt)` forced (`optimize.rs` ~:95). Known in
  time: yes; check in `run_pixel_op` (`src/cli/ops.rs` ~:390/:494, where it calls
  `output_format_for`).
- **`apply` / `build` with a terminal `optimize` step, auto-decided format**
  (`run_optimize_autodecide`, `encode_one_optimize_decided`): **the format is not known when the
  check would run**; it is chosen after the pipeline. Not needed: the shortlist reads `has_alpha`
  from the colour type (`src/image/mod.rs:445`), a translucent pad always yields an alpha type, and
  the shortlist excludes JPEG for alpha (`decide.rs` test `shortlist_alpha_photo_excludes_jpeg`) and
  never lists GIF. Design confirms the "passthrough (source kept)" branch can't fire after a pad,
  and pins the whole path with one test row.
- **`build`** (`src/cli/build.rs:579/584`, `encode_one` with `None` or `Some(fmt)`): same as the
  batch. Known in time: yes.
- **wasm `transform`** (`src/wasm.rs:206`): the format (`out_format`, else `recipe.format`) is
  resolved before decode-side work and before `pipeline.run`. Known in time: yes; check before the
  run and throw the same line.
- `edit` gains a pad only through SPEC-160's `--pad`, which goes through `run_pixel_op`, so the
  check there covers it when that ships.

### Where each failure happens, and its exit code

- Bad `color` syntax, unknown keyword, other key errors: **load**, exit 1, as today (DEC-102).
  `transparent` and `#RRGGBBAA` stop being load errors.
- Fill alpha the format can't store: **after the format is resolved, before the pipeline**.
  Exit **4** for one input; a batch with one uniform format (`--format`, a template extension,
  `recipe.format`) refuses once before any input is read, exit **4**, nothing written; a batch
  whose format is preserved per input fails that input and goes on, exit **6**; wasm throws.
- Border that rounds to 0 px, canvas past the caps: **apply**, exit 1 / 6, unchanged.
- Not through the refusal hook (`Operation::refusal`): skipping the step would write an unpadded
  image, so this is never a skip (SPEC-151's rule, now in `Operation::refusal`'s doc comment). So
  the "watermark skipped" batch-summary entry is not triggered by this spec (see Fold-ins).

### `pad` then `trim` with `transparent`

Pixels round-trip, colour type doesn't: an RGB8 image padded with `transparent` becomes RGBA8, and
trimming `#00000000` (or `transparent`, with Q2) removes exactly the alpha-0 border, giving back
the original pixels in an RGBA8 buffer whose alpha is all `ff`. It is the same caveat
`docs/recipes.md:143` already states for a grey image padded with a colour, and needs the same
sentence. The existing caveat also holds: an original edge row that is already fully transparent
is trimmed too.

### `watermark`: a model for reading the colour, a trap for writing it

Its colour reading is the model: `parse_color`, straight `RRGGBBAA`. Its writing is the trap: it
composites source-over in `f32`, where `255 × 0.99999994` truncates to 254 (DEC-095's
`restore_opaque_alpha` fix). `pad` must keep copying the fill value exactly and never blend. Also
`watermark` refuses a fully transparent colour (`InvisibleMark::TransparentColor`); `pad` must not.

### What doesn't change (and what pins it)

Every opaque `pad` and every recipe with no `pad` gives byte-identical output and the same exit
code. Pinned by the SPEC-141 corpus guard (`tests/recipe_corpus_output.rs`, including the two
`v2_pad_*` recipes) and `tests/pad.rs`'s byte and colour-type rows. Needed on top: one corpus-style
row that an RGBA PNG to JPEG with no `pad` still exits 0 and writes (the asymmetry, pinned so a
later pixel-based check can't sneak in), and one that an opaque `pad` to JPEG still exits 0.

### wasm (DEC-110)

`transform` must refuse the same recipe × format pairs with the same line; padded pixels are
copied, not computed, so native and wasm agree byte for byte on PNG. Rows that change: the wasm
size guard's `WASM_BROTLI_BASELINE` (`scripts/lib/wasm-artifact.mjs`) may move by a few bytes
(design measures it); `tests/ddphotos_parity.mjs` gains no row unless design adds a transparent
`pad` row (recommended: one PNG row, one refused JPEG row).

### Verification, sized (AGENTS §2)

- Fill × colour type (unit, `pad.rs`): 4 fill forms (`transparent`, `#ff000000`, `#ff000080`,
  opaque `#ff0000`) × 10 colour types (L8, LA8, L16, LA16, RGB8, RGBA8, RGB16, RGBA16, Rgb32F,
  Rgba32F) = 40 cells, one table, plus a grey-fill column for the L→LA vs L→RGBA split
  (HANDOFF trap 10: an input where the derived and naive types differ).
- Fill alpha × format (unit, a pure `fn` over the format's capacity): 3 alpha classes (00, 01-fe,
  ff) × 8 formats = 24 cells.
- Surface × format source (integration, `tests/pad.rs`): `apply` one input × {`-o` ext, `--format`,
  `recipe.format`, preserved source} = 4; batch × {`--format`, template ext, `recipe.format`,
  preserved per input} = 4; `build` × {pinned, preserved} = 2; terminal `optimize` × {pinned,
  auto} = 2; wasm × {`out_format`, `recipe.format`} = 2. 14 rows, plus GIF allowed-at-00 and
  refused-at-80, plus the two "unchanged" rows above, plus the pad-then-trim round trip.
- Every refusal message asserted as text (trap 13), each mutation row naming its exact edit and
  every test expected red (traps 12 and 14), and the GIF threshold driven at 01 and fe, not argued
  (trap 17).

### Sizing: [M], and why

The code is small (a fill alpha, four widening arms, one capacity table, one check called from
four places, a keyword in `trim`), but the verification above is ~80 unit cells and ~20
integration rows across four surfaces, plus a DEC and seven doc files. That is SPEC-145/153
territory, not SPEC-151/152. To keep it [S] you could (a) refuse every translucent fill to GIF,
(b) drop the once-before-any-input batch check and check per input only, (c) leave `trim`'s
keyword to SPEC-160. Recommendation: **don't cut**; (a) and (c) are each one table row and a
promise the docs already make, and the size is the surfaces, which are the point.

### Scope

In:
- `pad` accepts `transparent` and `#RRGGBBAA` below `ff`; fill semantics and widening as above.
- `trim` accepts `transparent` as `#00000000` (shared helper), if Q2 is yes.
- The static recipe × format check on `apply` (one input, batch, terminal `optimize` pinned),
  `build` and wasm `transform`; the move of `output_format_for` before `pipeline.run` in
  single-input `apply`.
- DEC-115.
- Docs (below).

Out (and where each lives):
- `edit --pad` / `--pad-color` / `--trim`: SPEC-160, which then takes `transparent` for free.
- `parse_color`'s `+f+f+f` sign bug: SPEC-160 (ruled). This spec doesn't touch `parse_color`, so
  no collision.
- Flattening a transparent fill against a colour: ruled out 2026-11-06.
- Source alpha dropped silently into JPEG (no `pad`): `guidance/ideas.yaml`,
  `alpha-dropped-silently-on-encode`.
- TIFF RGBA written without `ExtraSamples`: `guidance/ideas.yaml`, `tiff-alpha-extrasamples`.
- `crop`/`pad` aspect following orientation: its own STAGE-058 entry.

Fold-ins (every open STAGE-058 entry read):
- `edit --pad` and `--trim` (SPEC-160): not folded; ruled a separate spec after this one.
- Recipe-output guard reports both failures: not folded; test-only, optional, unrelated.
- `crop`'s aspect follows orientation: not folded; a new key on two ops, its own design.
- Batch summary says "watermark skipped" for any refusing op: **not folded.** This refusal is a
  failure, not a refusal-hook skip, so it never reaches `report_skipped_images`; the entry's
  trigger stays "before `trim` or another hook user ships".
- `apply`/`build` warn on a missing `auto-orient`: not folded; unrelated.
- Pin `watermark`'s nine gravities with a margin: not folded; test-only, unrelated surface.
- `parse_color` sign: not folded; ruled into SPEC-160.

### Docs to change (each states the opaque-only rule or the fill)

- `src/operation/pad.rs` module and `Pad` doc comments: "`color` is required and must be opaque",
  "Its alpha is always opaque", "A new pixel's alpha is the type's maximum" → the fill's alpha, the
  widening, and the format rule.
- `docs/recipes.md:116`: "A transparent fill is not supported yet." → `transparent` / `#RRGGBBAA`,
  what it widens to, the JPEG/GIF refusal and the source-alpha asymmetry; `:142-143` the round-trip
  sentence gains the RGB → RGBA caveat.
- `docs/api-contract.md:477-478`: "a required opaque `color`… a translucent colour and
  `transparent` are refused when the recipe loads" → the exit 4 per input and where it is checked.
- `docs/data-model.md:399`: `color` row "`RRGGBBAA` with alpha `ff`" → any alpha, or
  `transparent`; and the `trim` row if Q2.
- `recipes/README.md:296`: "hex only, and opaque. A transparent fill is not supported yet." →
  the new rule; `:297` the grey-becomes-RGB line gains grey → grey+alpha.
- `skills/crustyimg/SKILL.md:237`: "a required opaque `color`" → the new rule.
- `docs/cli-reference.md:54` and `docs/api-contract.md:59`: the exit-code tables say what exit 4
  covers (unsupported format, codec not built); each gains "a recipe step writes alpha the output
  format can't store".
- `README.md` doesn't mention `pad`'s fill (grepped). Build drives every doc sentence that states
  behaviour (SPEC-153's reflection).

### Rough build file list

- `src/operation/pad.rs` — fill alpha, widening arms, doc comments, unit tables
- `src/operation/mod.rs` — the shared `transparent` helper, and the format-capacity check (or a
  new `src/operation/alpha.rs`)
- `src/operation/trim.rs` — accept `transparent` (Q2)
- `src/cli/optimize.rs` — single-input `apply`: resolve the format before `pipeline.run` and check;
  the batch's once-before-any-input check
- `src/cli/common.rs` — `encode_one`: per-input check
- `src/cli/ops.rs` — `run_pixel_op`: check where it resolves the format
- `src/cli/build.rs` — per-target uniform check, if design takes it
- `src/wasm.rs` — `transform`: check before the run
- `tests/pad.rs`, `tests/trim.rs`, `tests/recipe_corpus_output.rs` (if a row is added),
  `tests/ddphotos_parity.mjs` (one row each way)
- `scripts/lib/wasm-artifact.mjs` — baseline, if it moves
- `decisions/DEC-115-…md` (design writes it)
- `docs/recipes.md`, `docs/api-contract.md`, `docs/data-model.md`, `docs/cli-reference.md`,
  `recipes/README.md`, `skills/crustyimg/SKILL.md`

### Questions only the maintainer can answer

- **Q1. GIF:** allow `transparent` / `#RRGGBB00` (stored exactly) and refuse only partial alpha, or
  refuse every translucent fill? Recommend: allow alpha 00, refuse 01-fe.
- **Q2. `trim` accepts `transparent`:** in this spec, through a helper shared with `pad` (not in
  `parse_color`)? Recommend: yes, so the documented pad-then-trim round trip holds for the keyword.
- **Q3. Source alpha into JPEG with no `pad`:** accept the asymmetry here (static check, only
  pad-introduced alpha is refused) and leave the silent drop to the filed idea? Recommend: yes.
- **Q4. Size:** [M] uncut, or cut to [S] with (a)-(c) above? Recommend: [M] uncut.

### Maintainer rulings (2026-11-10)

The maintainer accepted all four recommendations:
- **Q1, GIF:** allow fill alpha `00` (stored exactly) and an opaque fill; refuse `01`–`fe`.
- **Q2, `trim`:** accepts `transparent` as `#00000000` in this spec, through a helper shared with
  `pad`. `parse_color` is not changed.
- **Q3, source alpha:** accepted. Only alpha a `pad` brings is refused; an RGBA source written to
  JPEG still drops alpha silently, and the docs say so. `alpha-dropped-silently-on-encode` owns it.
- **Q4, size:** [M], uncut.

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

- `edit --pad`, `--pad-color`, `--trim` and their flags: SPEC-160.
- `parse_color`'s sign bug (`+f+f+f`): SPEC-160 (ruled 2026-11-10). This spec doesn't edit `parse_color`.
- Flattening a transparent fill against a stated colour: ruled out 2026-11-06 (SPEC-151 Q4).
- Source alpha dropped silently on a JPEG write with no `pad`: `guidance/ideas.yaml`, `alpha-dropped-silently-on-encode`.
- TIFF RGBA written with no `ExtraSamples` tag: `guidance/ideas.yaml`, `tiff-alpha-extrasamples`.
- A pixel-based (rather than recipe-based) alpha check: rejected at frame; it would change today's exit 0 for source-alpha images.
- `crop`/`pad` aspect following each image's orientation: its own STAGE-058 entry.
- The "watermark skipped" batch-summary wording: its own STAGE-058 entry; this refusal never goes through the refusal hook.

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
