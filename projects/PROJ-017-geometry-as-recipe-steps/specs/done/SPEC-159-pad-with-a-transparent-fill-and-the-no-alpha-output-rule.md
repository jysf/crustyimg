---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-159
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - DEC-015
    - DEC-095
    - DEC-098
    - DEC-102
    - DEC-106
    - DEC-110
    - DEC-112
    - DEC-113
    - DEC-114
    - DEC-115
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
      tokens_total: 6096960
      tokens_breakdown:
        input: 90
        cache_creation: 149736
        cache_read: 5903859
        output: 43275
      estimated_usd: 4.97
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Re-measured at ship from the session JSONL (was 43 messages, $4.70, measured after the frame push); deduped by message.id (45 messages, max output_tokens per id), no subagents"
    - cycle: design
      date: 2026-11-11
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 18442532
      tokens_breakdown:
        input: 198
        cache_creation: 257237
        cache_read: 18072784
        output: 112313
      estimated_usd: 13.45
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Re-measured at ship from the session JSONL (was 94 messages, $12.84, measured after the design push); deduped by message.id (97 messages, max output_tokens per id), no subagents"
    - cycle: build
      date: 2026-11-11
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 28312523
      tokens_breakdown:
        input: 266
        cache_creation: 277373
        cache_read: 27930962
        output: 103922
      estimated_usd: 10.98
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Re-measured at ship from the session JSONL (was 81 messages, $6.92, measured before the final push; the later turns are now counted); deduped by message.id (122 messages, max output_tokens per id), no subagents"
    - cycle: verify
      date: 2026-11-11
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 10881118
      tokens_breakdown:
        input: 130
        cache_creation: 194893
        cache_read: 10636908
        output: 49187
      estimated_usd: 7.77
      rates: "Opus $5/$25 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured at ship from the session JSONL (the verify report gave $7.47, measured before its last message); deduped by message.id (62 messages, max output_tokens per id), no subagents"
    - cycle: punchlist
      date: 2026-11-11
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1595879
      tokens_breakdown:
        input: 44
        cache_creation: 55920
        cache_read: 1530056
        output: 9859
      estimated_usd: 0.82
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Re-measured at ship from the session JSONL (was 14 messages, $0.56, measured before the final commit); deduped by message.id (21 messages, max output_tokens per id), no subagents"
    - cycle: ship
      date: 2026-11-12
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 3720139
      tokens_breakdown:
        input: 74
        cache_creation: 98248
        cache_read: 3605247
        output: 16570
      estimated_usd: 1.70
      rates: "Sonnet $3/$15 per MTok; cache_creation x1.25, cache_read x0.10 of input"
      note: "Measured from the session JSONL just before the ship commit; the last few turns are not counted; deduped by message.id (37 messages, max output_tokens per id), no subagents"
  totals:
    tokens_total: 69049151
    estimated_usd: 39.69
    session_count: 6
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

## Design

*Designed 2026-11-11 on `main` at `532db03` (code identical to `9bbdaf1` plus docs). Every
behaviour below was driven on a release build of `main` and on a prototype of this design (a
detached worktree of `origin/main`, never pushed), each in its own scratch `CARGO_TARGET_DIR`,
deleted after. Fixtures: ImageMagick 7, a Python xorshift with a hand-written PNG encoder, and
`image`'s encoder in the prototype tests. Read back with crustyimg's own decoder (`info`,
`convert --format png`), `magick … txt:`/`%[pixel:]` and a Python PNG reader.*

### Design corrections to the frame

1. **The auto-decide path DOES need a change: the frame's "no check is needed there" is wrong.**
   The shortlist's `has_alpha` is the *source's* (`src/cli/optimize.rs:1161`,
   `let has_alpha = source_info.has_alpha;`, captured before the pipeline), not the padded
   image's. Driven on the prototype before the fix: a 64×48 JPEG photo, recipe
   `[pad border = 0.1 color = "transparent", optimize]`, no format pinned:
   - `apply --recipe padopt.toml --out-dir ao -y photo.jpg` → exit 0,
     `photo.jpg: jpeg → jpeg · 2688 → 2786 B (4% larger)`, `ao/photo.jpg` JPEG 76×60 `rgb8`, corner
     pixel `srgb(0,5,1)`: a **black border, silently**.
   - `build` with that recipe (`source = "bsrc/*.jpg"`, default `{stem}.{ext}`) → exit 0,
     `bout/photo.jpg` JPEG, corner `srgb(0,5,1)`.
   The fix is one line: `let has_alpha = source_info.has_alpha || pipeline.introduces_alpha();`.
   After it (driven): `apply` → `photo.jpg: jpeg → webp`, `ao/photo.webp` `srgba`, corner
   `srgba(0,0,0,0)`; `build` (cache cleared) → `bout/photo.webp`, corner `srgba(0,0,0,0)`. The same
   variable feeds `fast_fallback_lossy_entry` (`optimize.rs:982-996`, JPEG only `if !has_alpha`),
   so the size fallback can't add JPEG either. The other ways the decided format could still be
   JPEG or GIF, and what stops each:
   - **Shortlist:** `decide::format_shortlist` never offers GIF (only PNG, WebP, JPEG, AVIF:
     `src/analysis/decide.rs:131-190`) and, with the fix, never JPEG after a translucent pad.
   - **"Nothing beats the source" passthrough** (`optimize.rs:1278`): fires only when
     `!pipeline_altered` (`pipeline_altered_source`, `optimize.rs:940`: dimensions, EXIF, ICC). A pad
     that writes a fill pixel always changes the dimensions, so it can't fire then. It fires for an
     image already at the aspect (driven: `ataspect.jpg: kept jpeg (2688 B, already optimal)`), and
     the source shipped then has no fill pixel and no introduced alpha: correct, not a leak.
   - **Degenerate-analysis passthrough** (`optimize.rs:1176-1192`): fires before the pipeline,
     only when `Analysis::compute` fails on a zero-area image, and ships the untouched source.
   - **Cache:** a `build` cache hit replays bytes an earlier run encoded. No released binary could
     load a translucent `pad` (exit 1 at load), so no cache holds such an output. (Driven as a
     trap: re-running `build` with the pre-fix prototype's cache in place served the stale JPEG,
     `(1 cached)`. A build that tests this path must use a fresh project dir.)
2. **`run_pixel_op` needs a once-up-front check, not a per-input one.** Its only caller that can
   carry a `pad` is `apply`'s terminal-`optimize` pinned path (`optimize.rs:95`), which always
   passes `Some(fmt)`; the other eight callers build flag pipelines with no `pad`
   (`/usr/bin/grep -n "run_pixel_op(" src/cli/*.rs`: `ops.rs:299,1393,1495,1519,1783`,
   `optimize.rs:607,738,859`). So the check goes at the top of `run_pixel_op`, on `forced_format`,
   before any input is read: exit 4 for one input and for a batch, like the plain batch. A
   per-input check for the unforced case is unreachable today (mutation M18: removing it left all
   58 tests green), so it is **SPEC-160's** to add with `edit --pad`, which is the first caller that
   reaches it. The frame's "move `output_format_for` before the run" is therefore needed only in
   single-input `apply`, not in `run_pixel_op`.
3. **`build` has two shapes, not "same as the batch" verbatim.** A `Pinned(fmt)` target (literal
   template extension or `recipe.format`) is checked once in `prepare_target`, before any input:
   exit 4, with `build`'s existing `error: target #0 (recipe …, out …)` line first and no out-dir.
   A `Preserve` target fails that input in `encode_one`, exit 6. `Decide` gets fix 1.
4. **Moving `output_format_for` changes one precedence, driven.** Its inputs are `global`
   (`--format`, `-o`, `--out-dir`, `--name-template`), `recipe.format` and `img.source_format()`
   (`optimize.rs:153-176`): none reads the processed image (no output-dimension template, no
   colour-type choice). The one change: an input that is wrong both ways. On `main`,
   `apply --recipe zero.toml --format bogus -o z.png -y rgb.png` (border rounds to 0 px) → exit 1,
   `error: operation 'pad' failed: the border rounds to 0 px …`; after the move → exit 4,
   `error: unsupported output extension: bogus`. A batch already reports the format first (driven on
   `main`: exit 4, no out-dir). Accepted and pinned (AC-17).
5. **The message can't name `-o`/`--format`.** The frame's draft way out (`-o out.png or --format
   png`) is wrong on `build` (no `-o`/`--format`, DEC-098) and wasm. One surface-neutral line serves
   every surface (DEC-110: same line).
6. **More old tests change than the frame listed.** `tests/pad.rs` `every_pad_refusal_names_its_reason`
   has **four** translucent rows, P29-P32 (`transparent`, `#ff000080`, `ff000000`, `#ffffffFE`),
   not two; its count goes 37 → 33 (37 − 4). `apply_refuses_a_translucent_fill_and_writes_nothing`
   (`tests/pad.rs:1185-1199`) is replaced. `tests/trim.rs` row T19 (`color = "transparent"` refused)
   moves to the accepted list: refusal count 23 → 22, accepted 7 → 8.
7. **AVIF's alpha is lossy at the edges of tiny images.** On a 4×4 RGB8 image padded to 6×6, a
   `transparent` border read back as alpha 5/255 (`srgba(0,1,7,0.0196)`); on a 64×48 photo it read
   back as exactly 0, and `#ff000080` as 128. AVIF stays *full* (it keeps alpha); the docs say
   "approximately".
8. **Docs the frame missed:** `CHANGELOG.md:19-21` (Unreleased: "a required opaque `color`"),
   `docs/backlog.md:194` ("opaque fill; a transparent fill is a later STAGE-058 entry"),
   `recipes/README.md:322` (trim: "`color` is hex and may be translucent"), `docs/data-model.md:428`
   (trim's `color` row), and `skills/crustyimg/SKILL.md:334` (its exit-code table's `4` row).

### Drives the prompt asked for

- **GIF at the threshold, on the prototype binary** (`apply --recipe g.toml -o gAA.gif -y rgb.png`,
  `color = "#ff0000AA"`): `00` exit 0; `01` exit 4; `80` exit 4; `fe` exit 4; `ff` exit 0;
  `transparent` exit 0. `#ff000000` into GIF: ImageMagick shows the border as `srgba(0,0,0,0)`, but
  **crustyimg's decoder reads it back as `(255,0,0,0)`** (`convert --format png`, PNG bytes
  `[255, 0, 0, 0]`): GIF keeps the palette colour under alpha 0. `trim color = "transparent"` and
  `trim color = "#ff000000"` on that GIF both exit 0 and give back the 4×4 original.
- **TIFF:** crustyimg reads its own `#ff000080` TIFF border back as `(255,0,0,128)` exactly (PNG
  bytes). ImageMagick shows `srgb(255,0,0)` (opaque), as the frame found. TIFF stays *full*; the
  docs line says other readers may show it opaque. No maintainer question needed: the rule is
  about crustyimg writing a value it can't read back, and it reads it back.
- **16-bit and float fills** (prototype, FT-4): `#ff000080` on RGB16 → RGBA16 fill
  `[65535, 0, 0, 32896]`; on Rgb32F → Rgba32F fill `[1.0, 0.0, 0.0, 128/255]` (bytes compared
  exactly). `#80808080` on L16 → LA16 `[32896, 32896]`. Pad-then-trim at 16 bits: RGB16 noise
  (every channel odd, so no sample equals the fill) padded `#ff000080` then trimmed `#ff000080`
  gives RGBA16 whose RGB equals the source: `trim`'s `× 257` (`trim.rs:186-213`, `step` 257) and
  `pad`'s `wide()` agree.
- **The other formats** (prototype, RGB8 4×4, `-o f.EXT`, `transparent` and `#ff000080`): png,
  webp, bmp, ico all exit 0 and keep `(0,0,0,0)` / `(255,0,0,128)` exactly; tiff and avif as above.
- **Static means static:** an RGB8 4×4 with `aspect = "1:1"` (already square) + `transparent` → `-o
  a.jpg` exit 4; `-o a.png` exit 0, written `rgb8` (unchanged type, no fill pixel). Two pads:
  `[#ffffff, transparent]` to JPEG names `recipe step 1`; `[#ff000000, #ff000080]` to GIF names
  `recipe step 1` (step 0's alpha 00 GIF stores), and the same recipe to JPEG names `recipe step 0`.
- **Batch shapes on `main`:** `apply --recipe flip.toml --format bogus --out-dir bo -y src.jpg
  rgb.png` → exit 4, `error: unsupported output extension: bogus`, `bo` not created; the same with
  `format = "bogus"` in the recipe → identical. A per-input failure (`bad.png` undecodable) → exit 6,
  `error: bad.png: unsupported or undetectable image format` then `error: 1 of 2 inputs failed`,
  `src.jpg` written. The new refusals match these two shapes exactly (FT-10, FT-11).
- **The unchanged rows on the prototype:** RGBA PNG + `flip` → `-o f.jpg` exit 0, stderr empty;
  opaque `pad` → `-o o.jpg` exit 0, stderr empty.

### The design

**The question "can this step write alpha?" lives on the step; the capacity on the format; the
comparison on the pipeline.**

- **`Operation::introduced_alpha(&self) -> Option<IntroducedAlpha>`** (`src/operation/mod.rs`,
  beside `refusal`, default `None`): the alpha of every pixel the step adds, the key that set it
  and that key's value as written. `Pad` answers `Some` when its fill alpha is below `ff`
  (`key: "color"`, `written`: the `color` param as written, e.g. `"transparent"`, `"#FF000080"`).
  A future op that can introduce alpha implements this one method; every surface then checks it.
- **`src/operation/alpha.rs`** (new, `pub mod alpha` in `src/operation/mod.rs`, with
  `pub use alpha::{AlphaCapacity, IntroducedAlpha, UnstorableAlpha}`):
  - `pub enum AlphaCapacity { None, Binary, Full }`;
  - `pub fn alpha_capacity(format: ImageFormat) -> AlphaCapacity`: `Jpeg => None`,
    `Gif => Binary`, everything else `Full`;
  - `pub fn stores(capacity: AlphaCapacity, alpha: u8) -> bool`: `None => alpha == 0xff`,
    `Binary => alpha == 0 || alpha == 0xff`, `Full => true`;
  - `pub struct IntroducedAlpha { pub alpha: u8, pub key: &'static str, pub written: String }`;
  - `pub struct UnstorableAlpha { pub step: usize, pub op: &'static str, pub introduced:
    IntroducedAlpha, pub format: ImageFormat }` with `Display` (the messages below) and
    `std::error::Error`.
- **`Pipeline::unstorable_alpha(&self, format) -> Option<UnstorableAlpha>`** and
  **`Pipeline::introduces_alpha(&self) -> bool`** (`src/pipeline/mod.rs`): the first step, in
  order, whose introduced alpha `format` can't store; and whether any step introduces alpha. The
  pipeline's index is the recipe step index (`Recipe::build_pipeline` pushes one op per step, in
  order, `src/recipe/mod.rs:391-400`; the terminal `optimize` is the last step and is stripped).
- **`CliError::UnstorableAlpha(#[from] crate::operation::UnstorableAlpha)`**, `#[error(transparent)]`,
  exit **4** (`src/cli/mod.rs`, beside `MetaFormatMismatch => 4`).
- **Call sites** (each `if let Some(e) = … { return Err(e.into()) }`):
  1. single-input `apply` (`src/cli/optimize.rs` ~:150-195): build the pipeline, resolve `fmt`
     with `output_format_for` (moved up from after `pipeline.run`), check, then run;
  2. batch `apply` (`optimize.rs` ~:200-220): keep step 4's probe pipeline
     (`let probe = recipe.build_pipeline(&registry)?;`) and, right after `format_override` is
     resolved, check `format_override.and_then(|f| probe.unstorable_alpha(f))`, before the
     progress bar and any decode;
  3. `encode_one` (`src/cli/common.rs:72`): compute `fmt` before `run_with` and check (the batch's
     and `build`'s kept-format inputs);
  4. `run_pixel_op` (`src/cli/ops.rs:326`): first statement, on `forced_format`;
  5. `prepare_target` (`src/cli/build.rs:279`): keep the probe pipeline, check when
     `format_plan` is `Pinned(fmt)`;
  6. wasm `transform` (`src/wasm.rs:206`): after `build_pipeline`, before `run`:
     `return Err(JsError::new(&e.to_string()))`;
  7. `optimize_decide_one` (`optimize.rs:1161`): `has_alpha` ORs `pipeline.introduces_alpha()`.
- **The shared keyword helper:** `pub fn parse_fill_color(text: &str) -> Result<[u8; 4],
  crate::text::TextError>` in `src/operation/mod.rs`: `"transparent"` (lower case only) →
  `[0, 0, 0, 0]`, else `crate::text::parse_color(text)`. `pad` and `trim` both call it; `parse_color`
  is untouched (SPEC-160 owns its sign bug). Their hex-error message stays as is
  (`'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "…"`, `pad.rs:157-161`,
  `trim.rs:72-76`), so `Transparent` and `TRANSPARENT` stay refused with it.
- **What `trim` prints for the keyword: the keyword, as written.** `whole_match_reason`
  (`trim.rs:94-106`) prints `'color' = {text:?}` from the params, so it reads `'color' =
  "transparent"`, never `#00000000`. Pinned (FT-18).
- **`pad`**: `fill: [u8; 4]`; the load refusal and the `translucent` closure
  (`pad.rs:147-164`) go; the widening match (`pad.rs:303-309`) gains the alpha arms; the fill arms
  (`pad.rs:312-330`) write the fill's alpha (`a`, `wide(a)`, `unit(a)`) instead of the type's
  maximum. An opaque fill writes `a = 0xff`, `wide(0xff) = 65535`, `unit(0xff) = 1.0`: the same
  values as today, which the byte pin and the existing colour-type tables hold.

**Fill semantics (frame, accepted):** `transparent` is `[0,0,0,0]`; `#RRGGBBAA` is straight alpha
stored as written (`#ff000000` writes `(255,0,0,0)`); nothing is blended; 16 bits `c × 257`,
float `c / 255`, alpha included.

**Widening (DEC-095 applied, unamended).** With `chromatic = r != g || g != b` and
`opaque = a == 0xff`:
- `L8`: opaque grey → `L8`; opaque colour → `RGB8`; translucent grey → `LA8`; translucent colour
  → `RGBA8`. `L16` the same at 16 bits.
- `LA8`/`LA16`: grey → unchanged; colour → `RGBA8`/`RGBA16` (as today, any alpha).
- `RGB8`/`RGB16`/`Rgb32F`: opaque → unchanged; translucent → `RGBA8`/`RGBA16`/`Rgba32F`.
- `RGBA8`/`RGBA16`/`Rgba32F`: unchanged.
- An image already at the aspect is returned unchanged with its own type, whatever the fill.
- Bit depth never changes. `transparent` (`r = g = b = 0`) is grey.

### Every message, verbatim

`{aa}` is the alpha in two lower-case hex digits, never decimal (`80`, not `128`: trap 10's
formatted-value rule; the tests use `#ff000080`, where the two renderings differ). `{written}` is
the `color` value as written, quoted with `{:?}`. The prefix `recipe step {i} ('{op}'): ` reuses
`RecipeError::InvalidOperation`'s wording (`src/recipe/mod.rs:124`) and wasm's asset refusal
(`src/wasm.rs:236`).

- **JPEG (exit 4):** `error: recipe step 0 ('pad'): 'color' = "transparent" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque`
- **JPEG, partial alpha (exit 4):** `error: recipe step 0 ('pad'): 'color' = "#ff000080" has alpha 80, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque`
- **GIF (exit 4):** `error: recipe step 0 ('pad'): 'color' = "#ff000080" has alpha 80, and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead`
- **wasm `transform` (thrown `JsError`):** the same line with no `error: ` prefix, e.g. `recipe step 0 ('pad'): 'color' = "transparent" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque`
- **A batch that keeps each input's format (exit 6):** per input, `error: <label>: ` + the line
  (`input_label`, `optimize.rs:1093`), then the existing summary
  `error: 1 of 2 inputs failed` (`CliError::PartialBatch`, `src/cli/mod.rs:719`, unchanged).
  `build` prints the same per-input line, its existing summary
  `built 1 target, 1 output (0 cached, 1 rebuilt), 1 failed` (`build.rs:860-863`), then
  `error: 1 of 2 inputs failed`.
- **`build`, pinned target (exit 4):** `error: target #0 (recipe t.toml, out b1)` (`build.rs:730`,
  unchanged) then the JPEG line.
- No message names a spec, stage or DEC id; no "watermark skipped" line is printed (this refusal is
  a failure, not a refusal-hook skip).

### Byte pin (recorded on `main` now)

Fixture: a 5×3 PNG per type, samples from a 64-bit xorshift (`x ^= x << 13; x ^= x >> 7;
x ^= x << 17`, one `next()` per sample in pixel-then-channel order, masked to the sample width),
seeded `0x1590000` (L8), `0x1590001` (RGB8), `0x1590002` (RGBA8), `0x1590003` (RGB16), written by
`image`'s PNG encoder in the test (no crustyimg code). Recipe: `op = "pad"`, `border = 0.2`
(1 px each side → 7×5), `color` below. Run `apply --recipe p.toml -o N.out.png -y N.png`; decode with
`image::open`; assert the colour type equals the input's and FNV-1a 64 of `as_bytes()` equals:

- L8, `#808080`: `0xc9a61194fef51c54` (35 bytes)
- RGB8, `#123456`: `0xd66d98995f850ac0` (105 bytes)
- RGBA8, `#123456`: `0xb8cb45d4dd540cc3` (140 bytes)
- RGB16, `#123456`: `0x3107368671842126` (210 bytes, little-endian samples)

How they were derived: a Python model built the expected output independently (source pixels at
(1,1), fill elsewhere, `c × 257` at 16 bits, alpha `ff`/65535 for RGBA) and hashed it; `main`'s
binary output, decoded by ImageMagick (`-depth N -endian LSB gray:/rgb:/rgba:`), matched the model
byte for byte on all four, and a model with one fill byte flipped did not (negative control). The
prototype's `byte_pin_opaque_pad` test asserted the same four literals green; under trap 14
(main's `pad.rs`) it stayed green, as it should. The build runs FT-6 before its first code change
and after its last.

### wasm

- **Size (driven):** `just wasm-build` (`--no-default-features --features avif,svg`, fat LTO), `main` vs the prototype, same toolchain: brotli 1,305,512 → 1,307,952 B (+2,440 B, +0.19 %); raw 6,123,528 → 6,146,024 B. Both sit inside the guard's ±5 % band around `WASM_BROTLI_BASELINE = 1_266_535` (`scripts/lib/wasm-artifact.mjs:69-72`, max 1,329,862). The baseline does **not** move. (`main` already sits +3.1 % over the recorded baseline: toolchain drift, not this spec.)
- **`tests/ddphotos_parity.mjs`:** a separate `alphaRows()` block after `TABLE`, not new `TABLE`
  rows, so `ROW_COUNT = 16` and the self-test's `N1_EXPECTED_RED`/`N2_EXPECTED_RED` don't move.
  Recipe text written literally (the editor's `recipe.js` has no `pad`):
  `version = "2"\n[[step]]\nop = "pad"\nborder = 0.05\ncolor = "transparent"\n` on `photo.webp`.
  Row A: `transform(bytes, text, "png")` and native `apply … -o A.png` both succeed and decode to
  identical RGBA8 bytes (padded pixels are copied, so exact, not within 1/255). Row B:
  `transform(bytes, text, "jpeg")` throws, and its message equals the JPEG line without `error: `;
  native `apply … -o B.jpg` exits 4 with stderr `error: ` + that line. A failure in either row is a
  harness failure (exit non-zero), like a `TABLE` row.

### Cache and lockfile

No `CACHE_SCHEMA_VERSION` bump and no lockfile format change. The cache key hashes the recipe text
(`build::cache::recipe_hash`) and a translucent `pad` never loaded on any released binary, so no
cache entry or lock record exists for one; an opaque `pad` encodes the same bytes (FT-6, the SPEC-141
corpus guard). The `has_alpha` change (call site 7) only changes the decision when a step
introduces alpha, which no loadable recipe could do before.

### Size after design: [M] holds

Derivation: 22 ACs (AC-1 to AC-22) and 21 FTs (FT-1 to FT-21). The unit tables are 40 capacity
cells (FT-1: 8 formats × 5 alphas `00, 01, 80, fe, ff`) and 50 fill cells (FT-4: 10 colour types ×
5 fills `transparent, #ff000000, #ff000080, #ff0000, #80808080`) = 90 cells; the integration rows
are FT-8 (5) + FT-9 (5 alphas + 2 read-backs) + FT-10 (3) + FT-11 (1) + FT-12 (2) + FT-13 (3) + FT-14
(2) + FT-15 (4) + FT-16 (2) + FT-17 (1) + FT-19 (2) = 32. Code: one new file of ~100 lines, ~60
lines in `pad.rs`, 7 call sites of 3 lines each, 1 line in `trim.rs`, one `CliError` arm. That is
SPEC-153's shape (surfaces and tests, small code): [M], not [L]; nothing splits cleanly (each
surface is half a rule without the others).

## Inputs

- **Files to read:** `src/operation/pad.rs`, `src/operation/trim.rs`, `src/operation/mod.rs`
  (`Operation`), `src/pipeline/mod.rs`, `src/cli/optimize.rs` (`run_apply`, `optimize_decide_one`),
  `src/cli/common.rs` (`encode_one`), `src/cli/ops.rs` (`output_format_for`, `run_pixel_op`),
  `src/cli/build.rs` (`prepare_target`), `src/wasm.rs` (`transform`), `tests/pad.rs`,
  `tests/trim.rs`, `tests/ddphotos_parity.mjs`; DEC-095, DEC-110, DEC-115.
- **External APIs:** none.
- **Related code paths:** `src/operation/`, `src/pipeline/`, `src/cli/`, `src/wasm.rs`.

## Outputs

- **Files created:** `src/operation/alpha.rs` (capacity, `stores`, `IntroducedAlpha`,
  `UnstorableAlpha`); `decisions/DEC-115-…md` (written at design).
- **Files modified:** see "Build file list" in Implementation Context.
- **New exports:** `operation::alpha::{AlphaCapacity, alpha_capacity, stores, IntroducedAlpha,
  UnstorableAlpha}`, `operation::parse_fill_color`, `Operation::introduced_alpha`,
  `Pipeline::{introduces_alpha, unstorable_alpha}`, `CliError::UnstorableAlpha`.
- **Database changes:** none.

## Acceptance Criteria

Each AC names its tests. "FT-n" is the Failing Tests list below.

- [ ] **AC-1 DEC-115.** `decisions/DEC-115-a-step-whose-alpha-the-output-format-cannot-store-is-refused-before-the-pipeline-runs.md`
  states the eight parts, the alternatives and its relation to DEC-015, DEC-095 (unamended),
  DEC-102 and DEC-110. Its `affected_scope` lists every `src/` file the diff touches for this rule.
  `just decisions-audit` passes. Verify reads it.
- [ ] **AC-2 `pad` loads a translucent fill.** `color = "transparent"`, `"#ff000080"`,
  `"ff000000"` and `"#ffffffFE"` load (they were P29-P32's refusals); `"Transparent"` (P28) and
  `"TRANSPARENT"` are still refused at load, exit 1, with the hex message. FT-7.
- [ ] **AC-3 Fill values.** `transparent` writes `[0,0,0,0]`; `#RRGGBBAA` writes its four values
  unchanged (`#ff000000` → `(255,0,0,0)`); at 16 bits every channel, alpha included, is `c × 257`
  (`#ff000080` → `[65535,0,0,32896]`); in float `c / 255` (`[1.0,0.0,0.0,128/255]`). Every input
  pixel is copied unchanged. FT-4.
- [ ] **AC-4 Widening.** The design's widening list, cell by cell over 10 colour types × 5 fills,
  including L8 + `#80808080` → LA8 (not RGBA8) and L8 + `#ff000080` → RGBA8; bit depth kept. An
  image already at the aspect keeps its type and bytes with a translucent fill. FT-4, FT-5.
- [ ] **AC-5 Opaque output unchanged.** The four byte-pin literals hold before the first code
  change and after the last; the SPEC-141 corpus guard (`tests/recipe_corpus_output.rs`) and the
  existing `tests/pad.rs` colour-type tables stay green. FT-6.
- [ ] **AC-6 Capacity.** `alpha_capacity` × `stores` over 8 formats (png, jpeg, gif, bmp, tiff, ico,
  webp, avif) × 5 alphas (`00, 01, 80, fe, ff`): JPEG stores only `ff`; GIF `00` and `ff`; the rest
  all five. FT-1.
- [ ] **AC-7 Messages.** The three lines in "Every message, verbatim" (JPEG `transparent`, JPEG
  `#ff000080`, GIF `#ff000080`) are produced exactly by `UnstorableAlpha`'s `Display` (unit) and on
  stderr (CLI). No message names a spec, stage or DEC id. FT-2, FT-8.
- [ ] **AC-8 Single input, every format rung.** `apply` with one input refuses, exit 4, nothing
  written, with the exact line, for `-o o.jpg`; `--format jpeg -o o.bin`; `recipe.format = "jpeg"`
  with `-o o2` (no extension); a JPEG source with `-o o3` (kept format); and `-o o.gif` with
  `#ff000080`. FT-8.
- [ ] **AC-9 GIF.** Into GIF, fill alpha `00` and `ff` and `transparent` exit 0; `01`, `80`, `fe`
  exit 4 with the GIF line. A `#ff000000` border read back from the GIF by `image::open` is
  `(255,0,0,0)`, and `trim color = "transparent"` on that GIF gives back the 4×4 original's RGB
  bytes. FT-9.
- [ ] **AC-10 Batch, one format.** `apply` over 2 inputs with `--format jpeg`, with
  `--name-template '{stem}.jpg'`, and with `recipe.format = "jpeg"`: exit 4, stderr exactly the
  JPEG line (once, no input label), the out-dir not created. FT-10.
- [ ] **AC-11 Batch, kept formats.** `apply` over `src.jpg` + `rgb.png` with no format: exit 6;
  stderr exactly `error: src.jpg: <line>` then `error: 1 of 2 inputs failed`; `rgb.png` written,
  `src.jpg` not. FT-11.
- [ ] **AC-12 Terminal `optimize`, pinned.** `[pad transparent, optimize]` with `-o p.jpg` (one
  input) and with `--format jpeg --out-dir tb` (two inputs): exit 4, the JPEG line, nothing written,
  `tb` not created. FT-12.
- [ ] **AC-13 Terminal `optimize`, decided.** A 64×48 JPEG source with
  `[pad border = 0.25 transparent, optimize]` and no pinned format, through `apply --out-dir` and
  through `build` (fresh project dir, no cache): exit 0, the one output is not `.jpg`, decodes to a
  colour type with alpha, and its pixel (0,0) has alpha 0. The same recipe with an
  `aspect` the image already has ships the source unchanged (`kept jpeg`). FT-13.
- [ ] **AC-14 `build`.** A target pinned by `name = "{stem}.jpg"`: exit 4, stderr exactly
  `error: target #0 (recipe t.toml, out b1)` + newline + the JPEG line, `b1` not created. A target
  keeping formats over `src.jpg` + `rgb.png`: exit 6, stderr starts with `error: in/src.jpg: <line>`,
  `b2/rgb.png` written. FT-14.
- [ ] **AC-15 wasm.** `transform` throws the line (no `error: ` prefix) for `out_format = "jpeg"`
  and agrees with native on a transparent PNG pad exactly. FT-21.
- [ ] **AC-16 Static.** An image already at the aspect, `transparent` fill: `-o a.jpg` exit 4;
  `-o a.png` exit 0, written with the input's colour type. `[#ffffff, transparent]` to JPEG names
  `recipe step 1`; `[#ff000000, #ff000080]` to GIF names `recipe step 1` and to JPEG
  `recipe step 0`. FT-3, FT-15.
- [ ] **AC-17 Format before the pipeline.** Single-input `apply` with `--format bogus` and a border
  that rounds to 0 px exits 4 with `error: unsupported output extension: bogus` (was exit 1). FT-16.
- [ ] **AC-18 Unchanged rows.** An RGBA PNG with half its pixels at alpha 0, recipe `flip`, `-o f.jpg`:
  exit 0, stderr empty (source alpha stays out). An opaque `pad` (`#ffffff`) to `-o o.jpg`: exit 0,
  stderr empty. FT-17.
- [ ] **AC-19 `trim` takes `transparent`.** Through `parse_fill_color`; it matches exactly as
  `#00000000` does (an alpha-0 border under any colour is cut); T19 moves to the accepted list;
  a whole match prints `'color' = "transparent"`. FT-18.
- [ ] **AC-20 Round trip.** RGB8 padded `transparent` then trimmed `transparent` gives RGBA8 whose RGB
  equals the original; RGB16 padded `#ff000080` then trimmed `#ff000080` gives RGBA16 whose RGB
  equals the original. FT-19.
- [ ] **AC-21 Exit code.** `CliError::UnstorableAlpha(_).exit_code() == 4`. FT-20.
- [ ] **AC-22 Docs.** Each row of "Docs" in Implementation Context says what it lists, and verify
  drives each behaviour sentence (not only reads it). No `CACHE_SCHEMA_VERSION` bump, no lockfile
  change (reason: "Cache and lockfile" above). `WASM_BROTLI_BASELINE` per "wasm" above.

## Failing Tests

Written at design. Fixtures are written by each test with `image`'s encoders: `rgb.png` and
`src.jpg` are a 4×4 RGB8 (`Rgb([x*50, y*50, 90])`); `rgba.png` a 4×4 RGBA8 with the left half
opaque and the right half alpha 0; the photo is 64×48 RGB8 xorshift noise (seed `0x159a070`)
saved as JPEG. "Exact line" means the whole stderr compared with `==`. The prototype names (in
brackets) map the mutation table's driven results.

**`src/operation/alpha.rs` (unit)**
- **FT-1 `capacity_and_stores_table`** [`capacity_table`]: 8 formats × 5 alphas, asserts the
  `[bool; 5]` per format.
- **FT-2 `messages_verbatim`** [reasoned; the CLI rows in FT-8 drove the same text]: `Display` of
  three `UnstorableAlpha` values equals the three lines without `error: `.

**`src/pipeline/mod.rs` (unit) or `tests/pad.rs`**
- **FT-3 `unstorable_alpha_names_the_first_step`** [`first_unstorable_step_is_named`]:
  `[#ff000000, #ff000080]` → GIF step 1, JPEG step 0; `[#ffffff, transparent]` → JPEG step 1, GIF
  `None`; `[#ffffff]` → JPEG `None`.

**`tests/pad.rs`**
- **FT-4 `translucent_fill_by_colour_type`** [`fill_widens_by_colour_type`, 7 of the 50 cells
  driven]: 10 types × 5 fills: output `ColorType`, the fill pixel's `as_bytes()` at (0,0), and the
  source copied at its offset. Use a source value that differs from the fill in every channel.
- **FT-5 `at_aspect_translucent_fill_keeps_type_and_bytes`** [reasoned; FT-15's PNG half drove it]:
  each of the 10 types, `aspect` equal to the image's, `transparent`: same type, same bytes.
- **FT-6 `opaque_pad_byte_pin`** [`byte_pin_opaque_pad`]: the four literals.
- **FT-7 load rows:** in `every_pad_refusal_names_its_reason`, delete P29-P32 and change the count
  37 → 33; add `"TRANSPARENT"` beside P28 if absent (count then 34); in
  `accepted_colour_and_value_spellings_load`, add the four (count 6 → 10).
  `apply_refuses_a_translucent_fill_and_writes_nothing` is deleted (FT-8 replaces it).
- **FT-8 `apply_one_input_refuses_on_every_format_rung`** [`single_apply_refuses_per_format_rung`]:
  AC-8's five rows, exit 4, exact line, file absent.
- **FT-9 `gif_takes_alpha_00_and_ff_only`** [`gif_threshold`]: AC-9.
- **FT-10 `batch_with_one_format_refuses_before_reading`** [`batch_refuses_once_or_per_input`, first
  half]: AC-10.
- **FT-11 `batch_keeping_formats_fails_that_input`** [same test, second half]: AC-11.
- **FT-12 `terminal_optimize_pinned_refuses`** [same name]: AC-12.
- **FT-13 `auto_decided_format_keeps_the_alpha`** [`auto_decided_format_keeps_alpha`; the
  at-aspect `kept jpeg` row was driven by hand, not in the prototype test]: AC-13.
- **FT-14 `build_refuses_pinned_once_and_kept_formats_per_input`** [same]: AC-14.
- **FT-15 `static_rule_at_aspect_and_two_pads`** [`static_rule_rows`]: AC-16's CLI rows.
- **FT-16 `format_is_resolved_before_the_pipeline`** [`unchanged_rows`, third row]: AC-17.
- **FT-17 `source_alpha_and_opaque_pad_into_jpeg_unchanged`** [`unchanged_rows`, first two rows]:
  AC-18.
- **FT-19 `pad_then_trim_transparent_round_trip`** [`pad_then_trim_round_trips_at_16_bits` and
  `gif_threshold`'s trim half]: AC-20.

**`tests/trim.rs`**
- **FT-18 `transparent_keyword_matches_like_00000000`** [`trim_reads_transparent_like_00000000`]:
  both spellings cut a 4×4 RGBA8 with an alpha-0 `(255,0,0,0)` ring to 2×2; T19 moved to the accepted
  list (refusal count 23 → 22, accepted 7 → 8); an all-transparent image fails with
  `… matches 'color' = "transparent" within 'tolerance' = 0, so a trim would leave nothing`.

**`src/cli/mod.rs` (unit)**
- **FT-20** a row in the existing exit-code test: `CliError::UnstorableAlpha(..) => 4`.

**`tests/ddphotos_parity.mjs`**
- **FT-21 `alphaRows()`**: rows A and B in "wasm" above.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-115` — this spec's rule (written at design). Read all eight parts.
- `DEC-095` — colour type preservation; the widening is its alpha and chroma clauses, unamended.
- `DEC-110` — wasm agrees with native; the refusal is the same line.
- `DEC-015` — per-input batch failures, exit 6; the kept-format batch follows it.
- `DEC-102` — load errors stay exit 1; this refusal is not a load error.
- `DEC-098` — `build` has no `--format`/`-o`; why the message names none.
- `DEC-112`/`DEC-113` — geometry steps act on the buffer they receive and write no metadata.

### Constraints that apply

- `untrusted-input-hardening`, `no-unwrap-on-recoverable-paths` — `introduced_alpha` reads params
  with `unwrap_or_default`, never `unwrap`.
- `single-image-library` — `image` only.
- `every-public-fn-tested` — `alpha_capacity`, `stores`, `parse_fill_color`, `introduces_alpha`,
  `unstorable_alpha` each have a test (FT-1, FT-18, FT-13, FT-3).
- `test-before-implementation`, `one-spec-per-pr`.

### Prior related work

- `SPEC-151` (shipped) — opaque `pad`; this spec removes its load refusal.
- `SPEC-152` (shipped) — `trim`; this spec adds the keyword.
- `SPEC-141` (shipped) — the recipe corpus guard; must stay green.
- `SPEC-153` (shipped) — the shape of this design; its only defects were two undriven doc sentences.

### Overlap with other open work (from the PRs' file lists, `gh pr list --state open --json number,title,files`, 2026-11-11)

No `feat/spec-15[4-8]*` branch is pushed (`git branch -r | grep -E "spec-15[4-8]"`: none). Open PRs
touching this spec's files:
- **#240** `chore/wasm-clippy` — `src/cli/ops.rs` (also `.github/workflows/ci.yml`, `justfile`,
  `src/metadata/mod.rs`).
- **#237** `heifer-migration` — `CHANGELOG.md` (also many non-overlapping files).
- **#226** `docs/agents-md-gates` — `src/cli/mod.rs` (the `CliError` arm here; also `AGENTS.md`).
- **#229** — `docs/roadmap.md` only; no overlap.

STAGE-063's SPEC-154 to SPEC-158 may touch `src/cli/build.rs`, `src/cli/common.rs`,
`src/cli/ops.rs` and `CHANGELOG.md` once they open PRs. **Rebase onto `main` before each push, and
re-run the `gh pr list … files` map after each rebase.**

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
- `run_pixel_op`'s per-input check for an unforced format: unreachable until `edit --pad`; SPEC-160 adds it with its tests (mutation M18).
- The "watermark skipped" batch-summary wording: its own STAGE-058 entry; this refusal never goes through the refusal hook.

### Build file list

- `src/operation/alpha.rs` (new) — capacity, `stores`, the two structs, `Display`, FT-1/FT-2.
- `src/operation/mod.rs` — `pub mod alpha` + re-exports, `Operation::introduced_alpha`,
  `parse_fill_color`.
- `src/operation/pad.rs` — 4-channel fill, widening arms, `introduced_alpha`, doc comments (module
  doc lines 11-17, `Pad` doc line 75, the `fill` field doc line 83, `from_params` doc lines 89-94,
  the "A new pixel's alpha is the type's maximum" comment line 311).
- `src/operation/trim.rs` — `parse_fill_color`; module doc: the keyword.
- `src/pipeline/mod.rs` — `introduces_alpha`, `unstorable_alpha`.
- `src/cli/mod.rs` — `CliError::UnstorableAlpha`, exit 4, FT-20.
- `src/cli/optimize.rs` — call sites 1, 2, 7.
- `src/cli/common.rs` — call site 3.
- `src/cli/ops.rs` — call site 4.
- `src/cli/build.rs` — call site 5.
- `src/wasm.rs` — call site 6.
- `tests/pad.rs`, `tests/trim.rs`, `tests/ddphotos_parity.mjs` — FTs.
- `scripts/lib/wasm-artifact.mjs` — only if "wasm" above says the baseline moves.
- Docs below.

### Docs (each an AC-22 row; verify drives each sentence's claim)

- `src/operation/pad.rs:11-17` — "`color` is required and must be opaque" → `color` is required: a
  hex colour (any alpha) or `transparent`; new pixels carry the fill's alpha; the widening; "a
  recipe that writes alpha the output format can't store is refused before it runs". `:75`, `:83`,
  `:311` to match.
- `docs/recipes.md:112-116` — replace "A transparent fill is not supported yet." with:
  `color` may be `"transparent"` (alpha 0 over black) or `"#RRGGBBAA"`; a fill with alpha below
  `ff` makes an image with no alpha gain one (grey stays grey + alpha); JPEG can't store it and GIF
  stores only `00` or `ff`, so such a recipe is refused for those (exit 4) — write PNG, WebP or AVIF;
  AVIF keeps alpha approximately, and some TIFF readers show it opaque; a source's own alpha into
  JPEG is still dropped without a warning. `:141-143` (the round trip): add "an RGB image padded with
  `transparent` comes back as RGBA". `:139` "a transparent `color` (such as `"#00000000"`)" → add
  `or "transparent"`.
- `docs/api-contract.md:476-481` — "a required opaque `color`… a translucent colour and
  `transparent` are refused when the recipe loads" → any alpha or `transparent`; a fill alpha the
  output can't store is refused once the format is known: exit 4 for one input or a batch with one
  format (nothing written), exit 6 per input when each keeps its own format; wasm throws the same
  line; the auto-decided format always keeps alpha. `:59` (exit 4 row) — add "a recipe step writes
  alpha the output format can't store". `:484` trim — `color` may be `transparent`.
- `docs/cli-reference.md:54` — the exit-4 row gains the same case.
- `docs/data-model.md:399` — `color` row: "required: `RRGGBB`, `#RRGGBB`, `RRGGBBAA` (any alpha) or
  `transparent`"; `:428` trim row: add `or transparent`; `:401-404` the paragraph gains the widening
  sentence.
- `recipes/README.md:296-297` — "hex only, and opaque. A transparent fill is not supported yet." →
  the new rule in one bullet (hex or `transparent`, any alpha, JPEG/GIF refusal); `:297` "A grey image
  with a colour fill becomes RGB" → "…, a translucent fill adds alpha (grey + alpha for a grey
  fill)". `:322` trim — "`color` is hex or `transparent`, and may be translucent".
- `skills/crustyimg/SKILL.md:237` — "a required opaque `color`" → "a required `color` (hex, any
  alpha, or `transparent`)"; `:334` exit-4 row — add "a recipe's fill alpha the output format can't
  store".
- `CHANGELOG.md:19-21` (Unreleased, "A recipe can pad.") — "a required opaque `color`" → "a required
  `color`, hex with any alpha or `transparent`; a fill alpha JPEG or GIF can't store is refused
  (exit 4)"; and the trim bullet (`:22-24`) — "`color` may be `transparent`".
- `docs/backlog.md:194` — "(…, opaque fill); a transparent fill is a later STAGE-058 entry" →
  "(…, a colour or `transparent` fill)".
- `README.md` — no `pad` fill text (`/usr/bin/grep -n "pad" README.md` at design: none about the
  fill); unchanged.

### Mutation table

Driven on the prototype: `cargo test --release --no-fail-fast --test pad_alpha --test pad --test
trim --test recipe_corpus_output`, one mutant at a time from a clean copy (`git checkout -- .`),
59 tests run (the baseline's `test result` lines: 17 `pad` + 14 prototype + 9 `recipe_corpus_output` (1 ignored) + 19 `trim` = 59), 58 once the artefact below is excluded. Baseline
green except one prototype artefact: `tests/trim.rs` `every_trim_refusal_names_its_reason` was red
in the baseline and in every row because the prototype deleted T19 without changing its count
(23 → 22): it is excluded from every red set below and is not a finding. "Green" = every other
driven test. FT ids follow the prototype names in Failing Tests. FT-2, FT-5, FT-7, FT-20, FT-21 were
not in the prototype; their column is *reasoned*.

- **M1 — GIF as full.** `src/operation/alpha.rs`, `alpha_capacity`: `ImageFormat::Gif => AlphaCapacity::Binary,` → `ImageFormat::Gif => AlphaCapacity::Full,`. Red (driven): FT-1 (`Gif left: [true, true, true, true, true]`), FT-2 (the GIF/JPEG `Display` line reads the capacity; verify drove it: the GIF line became "GIF output has no alpha channel…") *(corrected at ship)*, FT-3 (`refused` expect failed), FT-8 (the `-o o.gif` row: `left: (0, "")`), FT-9 (`01: left: true`), FT-15 (`left: 0 right: 4`). Green (driven): all others. *Driven.*
- **M2 — GIF as none.** Same line → `AlphaCapacity::None`. Red (driven): FT-1 (`left: [false, false, false, false, true]`), FT-2 (`Display` reads the capacity, so the GIF line changes) *(corrected at ship)*, FT-3 (`left: 0 right: 1`: the GIF step is now 0), FT-8 (GIF row gets the "no alpha channel" wording), FT-9 (`00:` refused), FT-15. *Driven.*
- **M3 — JPEG as binary.** `ImageFormat::Jpeg => AlphaCapacity::None,` → `ImageFormat::Jpeg => AlphaCapacity::Binary,`. Red (driven): FT-1, FT-2 (`Display` reads the capacity, so the JPEG line changes) *(corrected at ship)*, FT-3 (`left: 1 right: 0`), FT-8 (`-o o.jpg`: `left: (0, "")`), FT-10 (`--format jpeg`: `left: (0, "")`), FT-12, FT-14 (pinned: `left: (0, "target #0: 2 inputs …")`), FT-15. Reasoned red: FT-11 (alpha 00 now stored in JPEG; it shares a prototype test with FT-10, which failed first). Green (driven): FT-4, FT-6, FT-9, FT-13, FT-16, FT-17, FT-18, FT-19, the existing suites. *Driven.*
- **M4 — GIF threshold `alpha < 2`.** `stores`: `AlphaCapacity::Binary => alpha == 0 || alpha == u8::MAX,` → `AlphaCapacity::Binary => alpha < 2 || alpha == u8::MAX,`. Red (driven): FT-1 (`left: [true, true, false, false, true]`), FT-9 (`01: left: true`). Green (driven): everything else, including FT-8's GIF row (it uses `80`). This is why FT-9 must keep the `01` row. *Driven.*
- **M5 — GIF threshold `alpha <= 0xfe`.** Same line → `alpha <= 0xfe || alpha == u8::MAX`. Red (driven): FT-1 (all `true`), FT-3, FT-8 (GIF row), FT-9 (`01`), FT-15. *Driven.*
- **M6 — no check in single-input `apply`.** `src/cli/optimize.rs` single-input branch: delete `if let Some(e) = pipeline.unstorable_alpha(fmt) { return Err(e.into()); }` before `let out_img = pipeline.run(img)?;`. Red (driven): FT-8 (`-o o.jpg`: `left: (0, "")`), FT-9 (`01: left: true`), FT-15 (`left: (0, "")`). Green (driven): FT-10 to FT-14 (other paths), FT-16, FT-17. *Driven.*
- **M7 — no per-input check in `encode_one`.** `src/cli/common.rs`: delete the check before `// Run.`. Red (driven): FT-11 (`left: 0 right: 6`), FT-14 (kept-format target: exit 0, `built 1 target, 2 outputs`). Green (driven): FT-10 (the pre-check still fires), FT-8, FT-12. *Driven.*
- **M8 — no batch pre-check.** `src/cli/optimize.rs`: delete `if let Some(e) = format_override.and_then(|fmt| probe.unstorable_alpha(fmt)) { return Err(e.into()); }`. Red (driven): FT-10 (`left: (6, "error: rgb.png: recipe step 0 …")`: the per-input check is the fallback; FT-10 asserts exit 4, the unlabelled line and no out-dir, so the fallback can't pass it). Green (driven): FT-11, FT-14. *Driven.*
- **M9 — no check in `run_pixel_op`.** `src/cli/ops.rs`: delete the `forced_format.and_then(…)` check at the top. Red (driven): FT-12 (batch row `left: (6, "error: src.jpg: …")`). In the prototype the single `-o p.jpg` row stayed exit 4 only because the prototype also had the per-input checks M18 removes; with this design (no per-input check in `run_pixel_op`) that row reds too (*driven by build, no longer reasoned*) *(corrected at ship)*. *Driven.*
- **M10 — no check in `build`'s prepare.** `src/cli/build.rs` `prepare_target`: replace the `Pinned` check's body with `let _ = (probe.len(), fmt);`. Red (driven): FT-14 (pinned: `left: (6, "error: in/rgb.png: …")`). Green (driven): all others. *Driven.*
- **M11 — no check in wasm.** `src/wasm.rs` `transform`: delete the check. Red (driven by build: the parity harness exits 1 on `alpha-B`, no longer reasoned) *(corrected at ship)*: FT-21 row B (`transform` returns JPEG bytes, no throw). Green (reasoned): every Rust test (no native test calls `transform`). *Driven by build, via the parity harness* (the design's prototype ran no wasm tests; the wasm build was driven only for size).
- **M12 — the check made pixel-based.** `src/cli/optimize.rs` single-input: move the check after `pipeline.run` and wrap it in `if out_img.pixels().to_rgba8().pixels().any(|p| p.0[3] != 255) { … }`. Red (driven): FT-15 only (`left: (0, "")`: the already-at-aspect image has no transparent pixel). Green (driven): everything else, FT-17's RGBA-source row included (no pad, so no step to name). *Driven.*
- **M13 — 16-bit alpha `× 256`.** `src/operation/pad.rs`: in the `ImageRgba16` arm `wide(a)` → `u16::from(a) << 8`, and in the `ImageLumaA16` arm likewise. Red (driven): FT-4 (`#80808080 on L16 left: [128, 128, 0, 128]`), FT-19 (`left: [65535, 0, 0, 65535, …`), and four existing tests, because an opaque fill also goes through `wide(a)`: `tests/pad.rs` `colour_type_table_row_by_row` (`la16 + #808080: fill … [32896, 65280], expected [32896, 65535]`), `every_colour_type_pads_exactly` (160 rows), `sixteen_bit_fill_is_times_257`, and `pad_then_trim_is_the_identity` (in `tests/trim.rs`, not `tests/pad.rs`) *(corrected at ship)*; and `tests/recipe_corpus_output.rs` `every_recipe_output_matches_its_golden`. Green (driven): FT-6 (no RGBA16/LA16 row). *Driven.*
- **M14 — L8 grey translucent to RGBA8.** `pad.rs` widening: `(DynamicImage::ImageLuma8(_), false, false) => Cow::Owned(px.to_luma_alpha8().into()),` → `… px.to_rgba8() …`. Red (driven): FT-4 only (`transparent on L8 left: Rgba8 right: La8`). *Driven.*
- **M15 — `trim` not using the helper.** `src/operation/trim.rs`: `super::parse_fill_color(text)` → `crate::text::parse_color(text)`. Red (driven): FT-18 (`loads: InvalidOperation … got "transparent"`), FT-9 (its trim half: `left: 1`). Reasoned red: FT-18's moved accepted row and FT-19's RGB8 `transparent` half. *Driven.*
- **M16 — trap 14: `main`'s `pad.rs` restored verbatim** (`git show origin/main:src/operation/pad.rs > src/operation/pad.rs`; it compiles, since `introduced_alpha` has a default). Red (driven), every one with `a fill with transparency is not supported yet` (exit 1 at load): FT-3, FT-4, FT-8, FT-9, FT-10, FT-11, FT-12, FT-13, FT-14, FT-15, FT-19, and `introduces_alpha_is_any_translucent_step` *(corrected at ship)*; FT-5 and FT-7 are driven red, no longer reasoned *(corrected at ship)*. Green (driven): FT-1, FT-6 (the byte pin: opaque output is unchanged, as required), FT-16, FT-17, FT-18, and every existing `pad`/`trim`/corpus test. *Driven.*
- **M17 — the decided path reads only the source's alpha.** `src/cli/optimize.rs` `optimize_decide_one`: `let has_alpha = source_info.has_alpha || pipeline.introduces_alpha();` → `let has_alpha = source_info.has_alpha;`. Red (driven): FT-13 only (`left: "jpg" right: "jpg"`: the output is `photo.jpg`). *Driven.*
- **M18 — per-input checks in `run_pixel_op` (unforced format) removed.** Both arms in `src/cli/ops.rs`. Expected all green, and it was: 58 green (driven). The search that found no splitting input: `/usr/bin/grep -n "run_pixel_op(" src/cli/*.rs` lists nine callers; eight build flag pipelines with no `pad`, and the ninth (`optimize.rs:95`) always forces the format. So this design **omits** those checks; SPEC-160's `edit --pad` adds them with the tests that reach them. *Driven.*

## Notes for the Implementer

- **Checkpoint** (SPEC-113): commit and push after (1) `alpha.rs` + `introduced_alpha` +
  `parse_fill_color` + FT-1/FT-2/FT-3 green; (2) `pad.rs`/`trim.rs` + FT-4 to FT-7, FT-18, FT-19 green;
  (3) the seven call sites + FT-8 to FT-17, FT-20 green; (4) wasm + FT-21; (5) docs.
- **Run FT-6 first, on unchanged code**, and record its result: it is the "before" of the byte pin.
- **Fresh project dirs for `build` tests.** A stale `.crustyimg` cache replays old bytes (design hit
  it).
- **`cargo test` in an interactive terminal fails `display_sink_refuses_non_tty`**: redirect stdout to
  a file. A piped command reports the pipe's exit code.
- **CI legs:** the wasm leg (`just wasm-test`, the parity harness) is not in `just check` (HANDOFF
  trap 5): run `just wasm-build` and `node tests/ddphotos_parity.mjs` locally before the PR. The lean
  `--no-default-features` build must compile (`alpha.rs` uses only `image::ImageFormat`).
- **Never blend.** Copy the fill value; `watermark`'s `f32` compositing is the trap
  (`restore_opaque_alpha`, DEC-095).
- Comments and messages read behaviour-first; no spec or DEC ids in them.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-159-pad-transparent`
- **PR (if applicable):** #262 (draft; a separate verify session approves it)
- **All acceptance criteria met?** yes (22 of 22; two notes under AC-1 and AC-22)
- **New decisions emitted:** none in build. `DEC-115` was written at design and is untouched.
- **Files this diff touches** (`git diff --name-only origin/main...HEAD`: 22 files, 21 without this spec file; this spec's own edits land in the final commit and make it 22 / 23):
  - `src/operation/alpha.rs` (new) — `AlphaCapacity`, `alpha_capacity`, `stores`, `IntroducedAlpha`, `UnstorableAlpha` with its `Display`; FT-1, FT-2
  - `src/operation/mod.rs` — `pub mod alpha` and re-exports, `Operation::introduced_alpha`, `parse_fill_color`, a unit test for it
  - `src/operation/pad.rs` — 4-channel fill, widening arms, fill alpha written, load refusal removed, `introduced_alpha`, doc comments
  - `src/operation/trim.rs` — reads `color` through `parse_fill_color`; module doc
  - `src/pipeline/mod.rs` — `introduces_alpha`, `unstorable_alpha`; FT-3 and an `introduces_alpha` test
  - `src/cli/mod.rs` — `CliError::UnstorableAlpha`, exit 4; FT-20
  - `src/cli/optimize.rs` — sites 1 (format before the run), 2 (batch pre-check, `probe`), 7 (`has_alpha`)
  - `src/cli/common.rs` — site 3, `encode_one`
  - `src/cli/ops.rs` — site 4, `run_pixel_op`
  - `src/cli/build.rs` — site 5, `prepare_target` for `Pinned`
  - `src/wasm.rs` — site 6, `transform`
  - `tests/pad.rs` — FT-4 to FT-17, FT-19; FT-7 edits; P29-P32 and the old translucent `apply` test removed
  - `tests/trim.rs` — FT-18; T19 moved to accepted (refusal count 23 → 22, accepted 7 → 8)
  - `tests/ddphotos_parity.mjs` — FT-21 `alphaRows()`
  - `scripts/wasm-ci-check.sh` — the wasm test counts, 55 → 56 and 53 → 54
  - `tests/wasm_roundtrip.rs` — row P30 removed, a translucent-fill test added (see Deviations)
  - `docs/recipes.md`, `docs/api-contract.md`, `docs/data-model.md`, `docs/cli-reference.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`, `docs/backlog.md` — the AC-22 rows
  - No change to `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION`, the lockfile format or `scripts/lib/wasm-artifact.mjs` (`git diff --stat origin/main...HEAD` on the two Cargo files is empty).
- **Deviations from spec:**
  - AC-1: DEC-115's `affected_scope` lacks `src/cli/mod.rs` (the new `CliError` arm) and `src/operation/trim.rs`. Not edited, per the build prompt; verify/ship should add them.
  - `tests/wasm_roundtrip.rs` (not in the spec's file list): its row P30 asserted the old load refusal of `#ff000080`, which CI's wasm leg caught (`just wasm-test` is not in `just check`). The row is removed and `transform_refuses_a_fill_alpha_the_format_cannot_store` added; 56 passed locally.
  - `scripts/wasm-ci-check.sh` (not in the spec's file list): `FULL_COUNT` 55 → 56 and `SLIM_COUNT` 53 → 54 for the added wasm test; CI's count guard caught it.
  - `tests/pad.rs` FT-14: the `build` input label is built with the platform's path separator (the Windows leg printed `in\src.jpg`).
  - Tests that call the new API (FT-3) live in `src/pipeline/mod.rs`, not `tests/pad.rs`, so stage 1 could run on `main`.
  - Mutation edits differ in spelling only: my variable is `refused`, not `e`; M14's arm is `(ImageLuma8, false, true)` (my tuple's third element is "translucent").
- **Follow-up work identified:**
  - `run_pixel_op`'s per-input check for an unforced format belongs to SPEC-160 (M18, unchanged).
  - Stray `*.txt` files from this session remain in the system temp dir (a delete there was blocked by the safety check); harmless.

**FT-6 (byte pin).** On `main`'s unchanged code, before any source edit: `test opaque_pad_byte_pin ... ok`, `test result: ok. 1 passed`. After the last change (full suite, default features): the same test passes, and again in the lean and `webp-lossy` legs.

**Baseline on `main` (stage 1):** the translucent rows failed at load with `a fill with transparency is not supported yet; 'color' must be opaque, such as "#ffffff", got "transparent"`; FT-16 exited 1 with `operation 'pad' failed: the border rounds to 0 px …` where 4 is due; FT-17 and FT-6 were green.

**ACs.**
- AC-1: met except the `affected_scope` gap above; `just decisions-audit` not re-run by build.
- AC-2 FT-7, AC-3/AC-4 FT-4 and FT-5 (50 + 10 cells), AC-5 FT-6 + corpus guard, AC-6 FT-1 (40 cells), AC-7 FT-2 + FT-8, AC-8 FT-8, AC-9 FT-9 (00/01/80/fe/ff, keyword, GIF read-back `(255,0,0,0)`, trim back to the 4×4), AC-10 FT-10 (3 variants), AC-11 FT-11, AC-12 FT-12, AC-13 FT-13 (apply, build, at-aspect `kept jpeg`), AC-14 FT-14, AC-15 FT-21, AC-16 FT-3 + FT-15, AC-17 FT-16, AC-18 FT-17, AC-19 FT-18, AC-20 FT-19, AC-21 FT-20: met.
- AC-22: docs done at every row (the stored line numbers had shifted; each found by content). `WASM_BROTLI_BASELINE` unchanged.

**Docs, each driven on a release build** (all as written):
- JPEG refuses `transparent`: `exit 4`, the JPEG line, `a.jpg` absent.
- GIF: `transparent` exit 0; `#ff000080` exit 4 with the GIF line.
- Colour types after padding: RGB + `transparent` → `rgba8`; grey + `#80808080` → `la8`; grey + `#ff000080` → `rgba8`; RGB + opaque `#808080ff` → `rgb8`, `alpha: no`.
- Source alpha into JPEG, `flip` only: exit 0, stderr empty.
- AVIF "approximately": `#ff000080` read back as alpha 0.4549 at the corner on a 4×4 image (128 would be 0.502), 0.976 at the far corner.
- TIFF: crustyimg `info` says `alpha: yes`; ImageMagick warns about ExtraSamples and shows `srgb(255,0,0)` (opaque).
- Pad then trim with `transparent`: back to 4×4, `rgba8`.
- Batch with one format: exit 4, `b1` absent. Kept formats: exit 6, only `rgb.png` written.
- Terminal `optimize`, auto-decided, JPEG source: `jpeg → png`, output `src.png`.
- `build` pinned `{stem}.jpg`: exit 4, `error: target #0 (recipe bt.toml, out bo1)` then the line.
- `Transparent` refused at load, exit 1, hex message.

**Mutations** (one at a time, scratch worktree and target dir, clean rebuild, full suite `--no-fail-fast`; red = test names; the unit tests are `unittests::`). The edits were the quoted ones except as noted in Deviations.
- M1 GIF full: red FT-1, FT-2, FT-3, FT-8, FT-9, FT-15 (6). Spec: FT-1/3/8/9/15; FT-2 is extra, because `Display` reads the capacity.
- M2 GIF none: the same six. Matches, FT-2 extra.
- M3 JPEG binary: red FT-1, FT-2, FT-3, FT-8, FT-10, FT-11, FT-12, FT-14, FT-15 (9). Confirms the reasoned FT-11 red; FT-2 extra; FT-9, FT-13 etc. green.
- M4 `< 2`: red FT-1, FT-9 only. Matches.
- M5 `<= 0xfe`: red FT-1, FT-3, FT-8, FT-9, FT-15. Matches.
- M6 no single-input check: red FT-8, FT-9, FT-15. Matches.
- M7 no `encode_one` check: red FT-11, FT-14. Matches.
- M8 no batch pre-check: red FT-10 only. Matches.
- M9 no `run_pixel_op` check: red FT-12 (one test holds the single and batch rows). Confirms the reasoned single-input red.
- M10 no build pinned check: red FT-14 only. Matches.
- M11 no wasm check: all Rust tests green (0 red); the parity harness against a wasm build of the mutant: `FAIL alpha-B: transform to jpeg did not throw`, `16 rows, 0 row failures, 1 table failure (alpha-B)`, exit 1. Reasoned cell confirmed.
- M12 pixel-based check: red FT-15 only. Matches.
- M13 `<< 8`: red FT-4, FT-19, `colour_type_table_row_by_row`, `every_colour_type_pads_exactly`, `sixteen_bit_fill_is_times_257`, `recipe_corpus_output::every_recipe_output_matches_its_golden`, and `trim::pad_then_trim_is_the_identity` (7). Matches (the spec lists `pad_then_trim_is_the_identity` under `tests/pad.rs`; it is in `tests/trim.rs`).
- M14 L8 grey to RGBA8: red FT-4 only. Matches.
- M15 `trim` not on the helper: red FT-18 (both tests), FT-19, FT-9 (4). Matches; the FT-19 and moved-row reds are confirmed.
- M16 `main`'s `pad.rs`: red FT-3, FT-4, FT-5, FT-7, FT-8 to FT-15, FT-19 (14, plus the `introduces_alpha` unit test). FT-6, FT-16, FT-17, FT-18 and the existing suites green. Confirms FT-5 and FT-7.
- M17 decided path: red FT-13 only. Matches.
- M18: driven as a search. `/usr/bin/grep -n "run_pixel_op(" src/cli/*.rs` lists nine call sites (`ops.rs:299,1401,1503,1527,1791`, `optimize.rs:95,620,751,872`); only `optimize.rs:95` can carry a pad and it forces the format. There is no per-input check to remove, so all green holds.
- Every mutant's worktree was reverted; the restored head was green (the full-suite and matrix runs).

**wasm size.** Default `just wasm-build` (fresh target dir): brotli 1,308,562 B, +3.3% over `WASM_BROTLI_BASELINE` 1,266,535, inside the ±5% band (max 1,329,862). The design measured 1,307,952 on `main`'s successor; the difference from `main` is the spec's +0.19%. Slim parity build: 833,254 B. `node tests/ddphotos_parity.mjs` (via `just ddphotos-parity`): 16 rows, 0 row failures, 0 table failures; the alpha rows are inside; self-test N1/N2 green.

**Matrix** (each leg in a fresh `CARGO_TARGET_DIR`, rerun before the last push): default, `--no-default-features`, `--features webp-lossy`; `fmt --check`, `clippy --all-targets -D warnings` and `cargo test --no-fail-fast` all exit 0 on each. CI legs: see the PR.

**Overlap after the last rebase** (`gh pr list`, filtered): #240 `src/cli/ops.rs`; #237 `CHANGELOG.md`, `docs/licensing.md`; #229 `docs/roadmap.md`; #226 `src/cli/mod.rs`. No new PR touches this spec's files; the four known overlaps are unchanged and my edits to them are small, separate hunks.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. The two verbatim-message sections and the mutation table with exact edits made every row checkable. The prototype names in brackets map cleanly to the tests.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-115's `affected_scope` omits `src/cli/mod.rs` and `src/operation/trim.rs`, which AC-1 requires it to list.

3. **If you did this task again, what would you do differently?**
   — Script the mutation table from the start (done here, 17 rows in one background run). No design call turned out wrong. One test bug of mine: the existing `widen_pixel` helper assumes grey to RGB, so FT-4 needed its own widening helper.

### Punch list (verify)

Wording only; the PR head was `920b5ef` and no code or test changed.

The four sentences as they now read:

- `docs/recipes.md`: "...so a recipe that would write one is refused for those formats: exit 4,
  nothing written, for one input or a batch with one format; when each input keeps its own format,
  that input fails and the batch exits 6. Write PNG, WebP or AVIF."
- `docs/recipes.md`: "AVIF keeps the alpha approximately (on very small images it can be off by a
  few levels, for example a `transparent` border read back as alpha 5 of 255 on a 4×4 image), and
  some TIFF readers show it opaque."
- `recipes/README.md`: "...is refused for that output: exit `4` (nothing written) for one input or a
  batch with one format, and when each input keeps its own format that input fails and the batch
  exits `6`. Write PNG, WebP or AVIF."
- `CHANGELOG.md`: "A fill alpha JPEG or GIF can't store is refused for that output: exit 4, nothing
  written, for one input or a batch with one format; when each input keeps its own format, that
  input fails and the batch exits 6."

The drives (release build, fresh `CARGO_TARGET_DIR`, recipe `version = "1"` + `pad`, `border = 0.25`,
`color = "transparent"`; 4×4 `src.jpg` and `rgb.png`):

- `apply --recipe r.toml src.jpg rgb.png --out-dir kb`: exit 6. stderr:
  `error: src.jpg: recipe step 0 ('pad'): 'color' = "transparent" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque`
  then `error: 1 of 2 inputs failed`. `kb` holds `rgb.png` only.
- The same with `--format jpeg --out-dir kj`: exit 4, one line
  (`error: recipe step 0 ('pad'): 'color' = "transparent" has alpha 00, and JPEG output has no alpha channel; ...`), no `kj`.
- 4×4 RGB PNG to `-o t.avif`: exit 0; `magick t.avif -format '%[pixel:p{0,0}]' info:` gave
  `srgba(0,3,12,0.0196078)`, alpha 5 of 255. A 64×48 gradient gave `srgba(0,0,0,0)`.
- `cargo test --test docs_ops`: exit 0, 3 passed.
- Changed-line grep for `SPEC-`, `STAGE-`, `DEC-`, `PROJ-`: none.
- `git diff --stat 920b5ef`: `CHANGELOG.md`, `docs/recipes.md`, `recipes/README.md`, plus this spec.
- CI snapshot: see the PR (filled in below after the push).

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Four things, all wording or reporting; no code or test was wrong at merge. (a) Build drove the
   "exit 4" doc sentences and still missed the per-input exit 6 for a batch that keeps each input's
   format. `docs/api-contract.md` already had both, so the other three docs were the odd ones out.
   "Drive each sentence" is necessary but checks only the case the driver thinks of; verify found the
   gap by driving a batch where one input fails. (b) DEC-115 was written at design and described
   `run_pixel_op` as checking one input and resolving the format first, and then design correction 2
   moved `run_pixel_op` to a forced-format-only check. Nobody re-read the DEC against the corrections;
   verify found the sentences, and build had already seen the `affected_scope` gap. (c) Build reported
   "no spec cell was wrong" next to its own account that FT-2 was an extra red in M1 to M3. The table
   had three cells off, plus M9, M11 (reasoned, then driven by build), M13's file and M16's red set.
   Read the red set off a run of the final tests, as SPEC-153's reflection proposed. This spec did it
   for 17 of 18 rows at design and built them all, which is why only these cells moved. (d) Two files
   outside the spec's list, `tests/wasm_roundtrip.rs` and `scripts/wasm-ci-check.sh`, were caught only
   by CI's wasm leg, because `just check` does not run it. SPEC-153's two proposals, on this spec:
   build drove each doc sentence that states behaviour (yes, on a release build, but one exit path per
   sentence); a mutation row's red set was read off a run (yes at design for 17 rows; the M3, M9 and
   M11 cells that stayed reasoned were driven or confirmed by build, and the ship corrections are the
   residue). Verify's declined cleanups A and B are not a regret: A would move the wasm test counts
   again, and parity row A covers B's bytes.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied. (i) Build's checklist should say "drive each doc sentence on every exit
   path the spec names, not one". Nothing covers it: AGENTS §15 "During build" lists reading, filling
   Build Completion, cost, advance, DECs and the PR; SPEC-153's reflection proposed driving each
   sentence but not each path. (ii) Design should re-read its DEC against its own corrections before
   it hands off. Not covered: §15 has no design section for DECs, and SPEC-152's and SPEC-153's
   reflections are silent on it. (iii) "Files outside the list" should include whatever
   `just wasm-test` touches when `src/wasm.rs` changes. Partly covered: AGENTS line 191 says `just
   wasm-test` is not in `just check` and that CI's wasm-roundtrip job runs it; it does not tell a
   builder to widen the file list. The repo tracks no HANDOFF file, so I could not check trap 5 there.
   `guidance/` has no entry on any of the three. No DEC changes beyond the DEC-115 wording at ship.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new entry. Verify's declined cleanups A and B are declined with reasons above. The two ideas
   the frame filed stay where they are: `alpha-dropped-silently-on-encode` (source alpha into JPEG is
   still dropped; DEC-115's Negative consequence names it) and `tiff-alpha-extrasamples` (RGBA TIFF
   shows opaque in libtiff readers; the docs now say so). The one open item with a home is SPEC-160:
   `edit --pad` adds `run_pixel_op`'s per-input check for an unforced format, and DEC-115 part 4 now
   says so, so there is nothing to file.
