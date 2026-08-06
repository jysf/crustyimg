# Photo presets, photographic operations, and the RAW angle — an exploration

Read-only exploration session, 2026-07-25/26, at `main` ≈ `e8cdbc5`. Everything here was
researched without modifying the product: the only file this session wrote into the repo is
this document (and its companion stage draft), copied in afterwards at the maintainer's
request. All fetched preset files, probe outputs and crate tarballs stayed in the session
scratchpad. Two commits and a `PROJ-010` directory appeared in the tree mid-session from a
**concurrent** maintainer session; they are unrelated to this work.

## What this covers

It started as one question and grew into five. Each part has its own recommendation.

| Part | Question | Verdict |
|---|---|---|
| 1 (§1–6) | Can crustyimg import Lightroom presets? | **Don't** — measured overlap is zero |
| 2 (§7–11) | What would it cost to BUILD the develop side? | Affordable, off-thesis, and still can't match Lightroom |
| 3 (§12–18) | Does reading RAW unlock anything competitors can't do? | **Yes, narrowly** — preview + sidecar crop |
| 4 (§19–27) | What innovations does the evidence actually support? | Linear-light resize #1; smart crop **don't** |
| 5 (§28–42) | Which Rust libraries help, and which claims are real? | 3 to act on; 2 handed crates don't exist |

**A note on how to read this.** Claims are marked verified vs inferred throughout, and several
findings *contradict* current roadmap positioning or correct an earlier statement in this same
document (§29 corrects §21; §33 confirms §20; §34 resolves the contradiction flagged in §24).
Where research was lost to a crashed sub-agent, the gap is stated rather than papered over —
see §41 in particular.

**Recommendation up front: DON'T — for Lightroom. And the reason is not the one the question
assumes.** See §6.

---

## 1. crustyimg's real operation set

The recipe-addressable operation set is **four operations**, all registered in one place —
`OperationRegistry::with_builtins`, `src/operation/registry.rs:78-85`. A repo-wide grep for
`.register(` finds no other registration site; there is no plugin/inventory mechanism.

| op | file:line | params | notes |
|---|---|---|---|
| `identity` | `src/operation/mod.rs:159` | none | no-op passthrough |
| `invert` | `src/operation/mod.rs:183` | none | per-channel `255-x`, alpha preserved |
| `resize` | `src/operation/mod.rs:225` | `mode` (req.), `width`, `height`, `percent` | Lanczos3 (DEC-008); caps `MAX_EDGE=50_000`, `MAX_AREA=134_217_728` |
| `auto-orient` | `src/operation/mod.rs:586` | none | bakes EXIF orientation into pixels, **then drops the metadata bundle** (`:614`, DEC-017) |

`resize` modes (`Resize::from_params`, `src/operation/mod.rs:238-345`) — no defaults, every
key required per-mode:

| mode | required | geometry |
|---|---|---|
| `max` | `width` (long-edge cap) | `s = min(N/max(w,h), 1.0)` — never upscales |
| `exact` | `width`, `height` | forces W×H, aspect ignored |
| `fit` | `width`, `height` | `s = min(W/w, H/h, 1.0)` — never upscales |
| `fill` | `width`, `height` | cover-scale then **center-crop** to exactly W×H |
| `cover` | `width`, `height` | `s = max(W/w, H/h)` — may upscale, no crop |
| `percent` | `percent` (f32 > 0) | scale both dims by P/100 |

Plus one non-registry pseudo-step: **`op = "optimize"`** (`src/cli/optimize.rs:32`, DEC-070).
Legal only as the last step; stripped before `build_pipeline`; dispatches to the auto-decision
with **hard-coded `AutoQuality::Fast` + `Profile::Web` and no parameters**.

**`watermark` is implemented but deliberately unregistered** (`src/operation/mod.rs:784`;
`src/cli/ops.rs:945` states this explicitly) — a recipe naming it fails `UnknownOperation`.

### The recipe schema in full

`src/recipe/mod.rs:169-190`, `#[serde(deny_unknown_fields)]`:

| key | type | notes |
|---|---|---|
| `version` | String | only `"1"` (`SUPPORTED_VERSION`, `:50`) |
| `name`, `description` | Option\<String\> | cosmetic |
| `[[step]]` | Vec\<RecipeStep\> | `op: String` + `#[serde(flatten)] params` |

Limits: `RECIPE_MAX_BYTES = 65_536` (`:60`), `RECIPE_MAX_STEPS = 1024` (`:67`).

**There is no `output`, `format`, `quality`, `strip`, `profile`, or `budget` key. Anywhere.**
All three bundled recipes ([web](recipes/web.toml) 2048 / [product](recipes/product.toml) 1600 /
[gallery](recipes/gallery.toml) 2560) are structurally identical:

```toml
version = "1"
name = "web"
[[step]]
op = "auto-orient"
[[step]]
op = "resize"
mode = "max"
width = 2048
[[step]]
op = "optimize"
```

⚠️ Note: the worked example in `docs/data-model.md` shows `unsharp`, `watermark` and
`clean-gps` steps. It labels them illustrative, and **none of them exist**. Read the registry,
not the doc.

### Stated scope boundary

`docs/research/proj-002-findings.md:15, :113, :212` — pre-existing, not my framing:

> "crustyimg = an opinionated image *optimization engine*, not an editor" … "Not a Photoshop-lite."
> "**Draw the line at *automatic*** — normalize/auto-contrast/gray-world-WB = optimization;
> manual brightness/saturation sliders = the editor rabbit hole, skip."

---

## 2. Real Lightroom presets — what's actually in them

Corpus: **4,780 real preset files** fetched from three public repos (clebert/lightroom-preset,
xsynaptic/synaptic-lightroom-presets, deep5050/awesome-lightroom-presets) — 12 `.xmp` +
4,768 `.lrtemplate`. (No Adobe install exists on this machine; `~/Library/Application
Support/Adobe/` does not exist, and a `find` for `*.xmp`/`*.lrtemplate` over `$HOME` returned 0.)

A modern preset (`example.xmp`, LR 13.3) carries **~71 settings**: `AutoTone`, tone curve
(`ToneCurvePV2012` + 3 channel curves + 7 `Parametric*`), `WhiteBalance`, 24 HSL keys
(8 bands × hue/sat/lum), 14 split-tone/color-grade keys, `Texture`/`Clarity2012`/`Dehaze`,
vignette + grain, 4 sharpening keys, 4 noise-reduction keys, lens correction, defringe.

### Mapping table

| Preset setting group | keys | crustyimg equivalent |
|---|---|---|
| Tone (`Exposure2012`, `Contrast2012`, `Highlights/Shadows/Whites/Blacks2012`) | ~8 | **(c) none** |
| Tone curves (`ToneCurvePV2012*`, `Parametric*`) | ~11 | **(c) none** |
| White balance (`Temperature`, `Tint`, `ShadowTint`) | ~4 | **(c) none** |
| HSL (`Hue/Saturation/LuminanceAdjustment*`) | 24 | **(c) none** |
| Split-tone / color grading | 14 | **(c) none** |
| Presence (`Texture`, `Clarity2012`, `Dehaze`, `Vibrance`, `Saturation`) | ~5 | **(c) none** |
| Detail (`Sharpness`, `SharpenRadius/Detail/EdgeMasking`, `LuminanceSmoothing`, `ColorNoiseReduction*`) | ~8 | **(c) none** — no `unsharp` op exists |
| Effects (`PostCropVignette*`, `Grain*`) | ~9 | **(c) none** |
| B&W (`ConvertToGrayscale`, `GrayMixer*`) | ~9 | **(c) none** |
| Lens/perspective (`LensProfile*`, `Perspective*`, `Upright*` incl. 3×3 homographies) | ~20 | **(c) none** |
| Local masks (`GradientBasedCorrections`, brush `Dabs`) | unbounded | **(c) none** |
| Camera calibration (`RedHue`…`BlueSaturation`, `CameraProfile`) | ~7 | **(c) none** |
| `orientation` (`"AB"`/`"BC"`/`"CD"`/`"DA"`) | 1 | **(c) none — and a trap**: Adobe crop-orientation letter codes, *not* EXIF orientation 1–8. `auto-orient` reads EXIF. |

### Two findings that settle it, both verified with controls

1. **Develop presets contain no crop.** `grep -ril "croptop|cropleft|hascrop|cropangle"` across
   all 4,780 files → **0 files**. Positive controls on the identical command form returned 4,068
   files for `Exposure2012` and 3,251 for `orientation` — the tooling can and does return
   non-zero. Crop rectangles live in *per-image XMP sidecars*, not presets.
2. **Develop presets contain no output settings.** `grep -ril "LR_format|jpeg_quality|colorSpace"`
   → **0 files**. No format, no quality, no resize, no color space, no filename.

**Overlap: 0 of ~71 settings. Not "small" — zero.** The intersection between "what a Lightroom
develop preset says" and "what crustyimg can do" is the empty set, in both directions:
crustyimg's `resize`/`invert` have no preset counterpart either.

---

## 3. The one Lightroom artifact that *does* map — and why it doesn't save the idea

Lightroom has a **separate, disjoint** file type: `type = "Export"` `.lrtemplate` (a Lua table
literal) under `Export Presets/User Presets/`, using an `LR_*` key namespace. Develop presets
migrated to `.xmp` in LR Classic CC 7.3 (2018); **export presets did not** — the extension that
survived is precisely the one carrying output settings.

Mapping the real sample export preset (14 keys):

| `LR_*` key | maps to | verdict |
|---|---|---|
| `LR_format = "JPEG"` | `--format` | **(a) direct** |
| `LR_jpeg_quality = 1` | `-q` — ⚠️ **0..1 scale, not 0..100** | **(a) direct** |
| `LR_size_doConstrain` / `_maxWidth` / `_maxHeight` / `_resizeType` (`longEdge`/`megapixels`/…) | `resize mode=max|fit` | **(a) direct** |
| `LR_size_doNotEnlarge` | free — `max`/`fit` never upscale | **(a) direct** |
| `LR_jpeg_useLimitSize` + `LR_jpeg_limitSize` (KB) | `--max-size` byte budget | **(a) direct** — genuinely the same concept |
| `LR_removeLocationMetadata` | `meta clean --gps` / `--keep-gps` | **(a) direct** |
| `LR_minimizeEmbeddedMetadata` / `LR_embeddedMetadataOption` | `meta strip` | **(b) approximable** |
| `LR_tokens` (`{{date_YY}}…`) | `--name-template` (`{stem}_web.{ext}`) | **(b) approximable** |
| `LR_export_colorSpace` (AdobeRGB/ProPhoto) | — | **(c) none** |
| `LR_size_resolution` (DPI) | — | **(c) none**, irrelevant to pixels |
| `LR_outputSharpening*` | — | **(c) none** |
| `LR_useWatermark` / `LR_watermarking_id` | `watermark` (CLI-only, references an LR-internal id) | **(c) none in practice** |
| `LR_collisionHandling`, `LR_export_destinationType`, `LR_reimportExportedPhoto` | `--out-dir` / host behavior | **(c) none** |

So ~6 direct + 2 approximable of 14 ≈ **45%** — respectable in isolation. Two things kill it:

- **Availability.** GitHub code search for `LR_format path:*.lrtemplate` and
  `LR_export_destinationPathPrefix filename:*.lrtemplate` both return **0**. The corpus of 4,768
  `.lrtemplate` files contained **zero** `type = "Export"` entries (1,252 were `Develop`, 38
  `Print`). People trade *develop looks*; export presets stay on one machine.
- **Naming.** "Lightroom preset" colloquially means the develop preset. A feature advertising
  "imports Lightroom presets" would be read as a promise crustyimg deliberately does not make —
  and the 45% number applies to the artifact nobody shares.

---

## 4. Better-fit sources

| source | in-domain | shareable artifact exists | stable/documented |
|---|---|---|---|
| **Squoosh CLI JSON** (`--resize`/`--mozjpeg`/`--webp`/`--avif`/`--oxipng`) | 5/5 — every option is something crustyimg models | 3/5 — usually a command line in `package.json`, not a file | 5/5 — schemas in `meta.ts`, frozen because the project is dead |
| **imgproxy processing options + presets** | 5/5 — incl. `max_bytes`, `strip_metadata`, `auto_rotate` | 3/5 — real preset file via `IMGPROXY_PRESETS_PATH` | 5/5 |
| **thumbor URL grammar** | 4/5 | 2/5 — ad-hoc URLs | 4/5 |
| **LR export presets** | 4/5 | 5/5 *in principle*, ~0 in practice (see §3) | 3/5 — format undocumented, keys fully documented in the SDK |
| **sharp** | 4/5 | **1/5 — no serialized format exists** (verified: sharp-cli has no config file) | 5/5 |
| **ImageMagick pipelines** | 2/5 — in-domain operators are a minority of real commands | 4/5, but they're *code*: order-dependent, `-repage`, `mpr:` registers, `-clone` | 3/5 |
| **ImageMagick MSL** | 3/5 | 1/5 — essentially nobody has MSL files | 2/5 — IM6-era docs |
| **imgix / Cloudinary** | 2/5 — dominated by overlays/faces/AI/video | 2/5 | 4/5, vendor-versioned |
| **imagemin** | 3/5 — recompressor, **no resize at all** | 1/5 — JS code | 3/5 |
| **cwebp/avifenc/oxipng flags** | 5/5 | 1/5 — no config file anywhere | 4/5 |

Best fits, in order: **Squoosh CLI JSON** (highest in-domain purity; a dead upstream is an asset
for an importer, and it carries a story — "Squoosh is unmaintained, here's a native tool that
reads your flags"), then **imgproxy/thumbor** ("run the CDN transform you're paying for locally
at build time"). Both are small enough to parse *completely*, which matters: a partial importer
that silently drops an operator is worse than none.

Explicitly bad: **sharp** (nothing to ingest — it's the right thing to *emit*), **ImageMagick
general pipelines** (high apparent value, low achievable coverage; the moment you hit `-unsharp`
or `-fx` you must fail, and every partial success trains users to expect the rest),
**imgix/Cloudinary** (~15% of the vocabulary, drifting reject list).

**EXIF orientation checked separately, no action needed.** Browsers honor EXIF orientation and
orientation 6 is the ordinary phone case, so a strip-without-baking path would silently rotate
output. crustyimg already bakes then strips — `src/cli/optimize.rs:585` ("PINNED order:
`auto-orient` … then drop"), DEC-017. Not a defect.

---

## 5. The structural blocker nobody's importer can route around

Even for the artifacts that *do* map, look at where they land:

| what an import would set | crustyimg plane |
|---|---|
| resize dimensions | **recipe** (`resize` op) ✅ |
| orientation | **recipe** (`auto-orient`) ✅ |
| output format | CLI flag `--format` ❌ |
| quality | CLI flag `-q` ❌ |
| byte budget | CLI flag `--max-size` ❌ |
| metadata strip / GPS | CLI `meta` verb, `--keep-gps` ❌ |
| output naming / dir | CLI `--name-template`, `--out-dir` ❌ |

**The recipe schema cannot express format, quality, budget, or metadata.** The only bridge is the
terminal `op = "optimize"` marker, which takes **no parameters** and hard-codes `Fast` + `Web`.

So "import a preset into a recipe" does not typecheck. An importer today could only emit a
*shell command*, not a recipe file — and the thing users want to commit to CI is the file.

This reframes the whole question: **the bottleneck is not the source format, it's that
crustyimg's recipe language is currently three ops wide.** That is exactly what
[roadmap.md:38](docs/roadmap.md:38) item 6 ("formalize the recipe format… a documented op
reference/schema") is for. An importer built before that has nowhere to write to.

---

## 6. Recommendation

**DON'T — for Lightroom.** Not a judgement call; the develop-preset overlap is measured at
**0 of ~71 settings**, with negative-and-positive-controlled greps over 4,780 real files showing
no crop keys and no output keys. The only Lightroom artifact that maps ~45% is the export preset,
which the same corpus shows people effectively never share. And "Lightroom import" would read as
a promise of tonal/color editing — the precise thing `proj-002-findings.md` calls "the editor
rabbit hole." Building it would be scope creep off the engine-led thesis with a measured payoff
of zero.

**DEFER — for the good sources.** Squoosh-CLI and imgproxy import are genuinely in-domain and
would be worth ~1 spec each *later*. They are blocked on §5: sequence them **after** roadmap
item 6 gives the recipe an output/encode block. Doing them first forces an importer that emits
shell strings, which is the wrong artifact and would bake a bad shape into the format.

**The narrow version worth doing now costs no code: a docs mapping table.** A short
`docs/migrating.md` — "your Squoosh flags / imgproxy params / Lightroom *export* settings →
the equivalent crustyimg command" — captures most of the practical value, is honest about the
0% develop-preset overlap (which is itself a useful, differentiating thing to say out loud),
and directly serves the stated goal-1 gap of real-world usage. It also doubles as the
translation table any future importer would need, so it is not throwaway.

One concrete correction for that doc regardless: `docs/data-model.md`'s recipe example advertises
`unsharp`, `watermark` and `clean-gps` steps that do not exist. Anyone evaluating "can it run my
pipeline?" reads that example first.

---

# Part 2 — What would it cost to BUILD the develop side?

Follow-up question: not "does it overlap" but "what's the space to build the features, and is
that a roadmap executable with the surface we have now?"

## 7. Two different questions

"Execute Lightroom operations" splits into two, with wildly different answers:

- **A. Have tonal/color operations at all** (exposure, contrast, curves, HSL, sharpening).
  Cheap. Genuinely executable with the current surface. See §9.
- **B. Execute *Lightroom's* operations — i.e. run a preset and get Lightroom's result.**
  Not achievable. Not "expensive": *not achievable*. See §10.

Almost all the intuitive appeal belongs to B. Almost all the feasibility belongs to A.

## 8. Architectural preconditions (what's actually blocking)

| # | Precondition | Current state | Evidence |
|---|---|---|---|
| 1 | High-precision working buffer | **None.** `Image` = a single `pixels: DynamicImage` field; every op opens with `to_rgba8()`. N tonal ops = **N 8-bit round-trips** → banding. | `src/image/mod.rs:103`; `src/operation/mod.rs:197, 396, 816` |
| 2 | Any colorspace awareness | **Zero.** ICC is opaque bytes in the container lane, never applied, dropped on re-encode. Resize runs on gamma-encoded, non-premultiplied sRGB. | `src/image/mod.rs:264` ("Not parsed"); `src/operation/mod.rs:511-533`; `docs/api-contract.md:158` |
| 3 | Op capability contract | **None.** Trait is 3 methods, `Image → Image`, no `wants_linear()`. Pipeline is a bare fold. | `src/operation/mod.rs:130-152`; `src/pipeline/mod.rs:51` |
| 4 | RAW develop headroom | **None.** Preview-JPEG extraction only; **no demosaic anywhere in the tree**; RAW loads with `metadata: None` — no EXIF, no ICC. | DEC-055:45-48, :88-90; `src/image/raw.rs:3-7`; `src/image/mod.rs:462-468` |

Two more measured facts: **10-bit AVIF is quantized to 8-bit at decode** and cannot be recovered
(`src/image/avif.rs:535`), and there is **no tonal math in the tree at all** — a grep for
`brightness|contrast|gamma|curve|saturat|hsl|hsv|unsharp|blur|convolv|sharpen` returns 50 hits,
**all false positives** (`saturating_sub`, the `UniqueColors::Saturated` sentinel, English prose).
`gamma`, `curve`, `hsl`, `unsharp`, `sharpen` → 0 hits each.

Dependency posture: a color-math crate (`palette`, MIT/Apache) would **not** trip
`single-image-library` — it's the same carve-out already used for `ssimulacra2` and `resvg`
("a metric crate only", "NOT a second pixel library") — but needs a DEC per the
`no-new-top-level-deps-without-decision` warning. A demosaic crate is a different story: see §10.

## 9. The roadmap that IS executable

Sized in this repo's own units. Calibration: **105 specs shipped for ~$855 / ~50h recorded**,
≈ $8/spec, and stages run 3–9 specs.

| Tier | Work | Size | In thesis? |
|---|---|---|---|
| **0. Preconditions** | f32 (or u16) linear working buffer; a colorspace contract; extend the op trait with a pixel-type/linearity hint so the pipeline converts **once** instead of per-op | **~1 stage, 6–10 specs.** Bounded but foundational — the largest architectural change since wave 1; touches `Image`, all 4 ops, DEC-002, and needs a new color-management DEC | Neutral — genuinely improves resize quality too |
| **1. Auto-only tone** | `normalize` / auto-contrast / gray-world WB | **2–4 specs**, cheap on top of Tier 0 | **Yes** — already sanctioned: *"Draw the line at automatic"* (`proj-002-findings.md:212`) |
| **2. Manual sliders** | exposure, contrast, highlights/shadows, curves, HSL, vibrance, unsharp, grain, vignette | ~10–15 ops. Each is ~40 lines + a registry line + tests → **8–12 specs** | **No** — this is verbatim *"the editor rabbit hole"* the same line rules out |
| **3. Match Lightroom** | see §10 | — | No |

So the answer to *"is that a roadmap executable with the surface we have now?"* is: **Tier 0–2,
yes — call it 2–3 stages / ~$150–250 of the repo's own measured spend.** The code is not the
hard part. Tier 0 is the only genuinely interesting engineering, and it has independent merit.

The catch: finishing Tier 2 buys you an image editor with sliders. It does **not** buy you the
ability to run a single Lightroom preset.

## 10. Why Tier 3 is not achievable

**The parameters' meaning is unpublished.** Exiv2's `crs` reference documents `Exposure2012`,
`Contrast2012`, `Highlights2012`, `Shadows2012`, `ProcessVersion` with the literal note
*"Not in XMP Specification. Found in sample files."* The **names** leaked by observation; the
transfer functions are published nowhere. What IS spec'd is the container and camera profile
(DNG/DCP) — the well-documented part is precisely the part you'd skip on a JPEG.

**The processing is adaptive.** Adobe's own docs warn PV2012 "may result in pixels of the same
input value having slightly different output values" — which rules out modelling it as a curve
or a 3D LUT, i.e. rules out every naive reimplementation *and* the LUT-export approximations.

**Adobe applies hidden per-camera, per-ISO baseline corrections.** RawDigger's teardown found
zeroing the visible defaults still needed `Exposure −1 stop, Contrast −33, Black +25` plus a
custom curve to reach linear — and the compensation may vary with ISO, not just camera model.

**The decisive evidence.** darktable — GPL, ~15 years, 17.5 MB of C, strong color scientists —
is the *only* open-source project that imports Lightroom develop settings. Its
`src/develop/lightroom.c` (1,631 lines) reads exactly five `*2012` keys, maps `Exposure2012`
**1:1 with no transform** (`data->pe.exposure = v;`), and implements `Blacks2012` as a
**hand-fitted 5-point lookup table**. It does not implement `Contrast2012`, `Highlights2012`, or
`Shadows2012` **at all**. darktable's own manual: *"This import process will never give identical
results."* That is not neglect — that is what "you cannot get there from here" looks like in
source. RawTherapee (~125 person-years per Open Hub COCOMO) doesn't even attempt the import.
Adobe's own sanctioned answer to "run these settings programmatically" is a **cloud API**.

**And the license wall is independent of all of the above.** The entire pure-Rust raw ecosystem
is copyleft: `rawler` LGPL-2.1, `rawloader` LGPL-2.1, `imagepipe` LGPL-3.0, `quickraw` LGPL-2.1;
LibRaw is LGPL/CDDL + C. Constraint `no-agpl-default-deps` is **severity: blocking**, and
`pure-rust-codecs-default` is too. The repo already ruled on exactly this
(`guidance/license-watchlist.yaml:130`: Tier-2 development "L" effort, LGPL, "overkill").
Even ignoring quality, crustyimg cannot take the dependency on its default path.

Effort, honestly: "tonal ops in the *spirit* of LR" ≈ 2–4 weeks. "Empirically fitted to match LR
on JPEGs" ≈ 3–9 months of measurement, needs an ACR license and a headless harness, works for
Exposure/Contrast, fails on the adaptive ops, and **rots** every time Adobe ships a process
version. "Match LR on raw" — no evidence anyone has done it.

## 11. Verdict on Part 2

The work is *affordable*; that's the trap. A competent 2–3 stage push gets you a slider-based
editor, at which point you discover the goal that motivated it — running a preset — is still
out of reach, for reasons that are not about effort and cannot be bought down. You'd have spent
the repo's identity on it: `proj-002-findings.md` names manual sliders as the rabbit hole, and
the RAW path (preview JPEG, `metadata: None`) means photographers' actual files arrive with
**zero develop headroom** — the camera's baked contrasty 8-bit JPEG.

The one piece worth wanting on its own merits is **Tier 0**, and it should be justified by
optimization, not editing: a linear-light, higher-precision working buffer would measurably
improve resize quality (downscaling in gamma-encoded sRGB is a real, well-known artifact), fix
the 10-bit AVIF truncation, and remove the N-round-trips problem. If that lands, Tier 1's
auto-only tone ops are cheap and already inside the stated thesis. Neither requires saying the
word "Lightroom", and that's the tell that they're the parts actually worth building.

---

# Part 3 — The RAW angle: what reading RAW actually unlocks

Follow-up: is there a subset of use-cases where, *because* crustyimg reads RAW, it can do
something current CLI tools can't?

**Yes — one, and it is narrow, in-domain, and genuinely unserved.** It requires no tonal or
colour work at all.

## 12. What crustyimg can build a batch out of today

crustyimg already *has* the preset mechanism: bundled recipes. The gap is not operations, it's
that the recipe schema can only say `resize` and `auto-orient` (Part 1 §5). Nearly every
ingredient of a photographer batch already exists as a CLI verb and **cannot be bundled**:

| batch ingredient | implemented? | expressible in a recipe? |
|---|---|---|
| downscale, bake orientation | ✅ | ✅ |
| format / quality / byte budget | ✅ `--format`, `-q`, `--max-size` | ❌ |
| strip GPS / strip all / set artist+copyright | ✅ `meta` verbs | ❌ |
| crop / rotate / flip | ❌ roadmap wave 5 | ❌ |

So the first unlock is **schema width** (roadmap wave 6), which converts preset-building from
code into content.

## 13. Rust crate reality for the develop ops (verified against crates.io, 2026-07-25)

**CAN, no new dependency** — exposure/contrast/blacks/whites, highlights/shadows, tone curves
(hand-rolled monotone cubic Hermite — better than the `splines` crate, which isn't monotone),
split toning, B&W mixer, vignette, arbitrary-angle rotate + perspective (one inverse-map warp +
`imageops::interpolate_bilinear`), local masks. And **already shipped in the pinned `image`
0.25.10**: `unsharpen`, `blur`, `blur_advanced`, `fast_blur`, `filter3x3`, `grayscale`, `invert`
— verified by reading the vendored crate source; crustyimg uses **none** of them.

**CAN, one permissive dep** — HSL/vibrance via `palette` 0.7.6 (MIT/Apache, only `fast-srgb8` +
a derive macro non-optional); clarity/texture/NR via `libblur` 0.24 (**Apache-2.0 OR BSD-3**,
SIMD, `image` dep optional) whose `fast_bilateral_filter` is the *only* permissive edge-aware
primitive in Rust; grain via `fastrand` (MIT/Apache, zero deps); **ICC via `moxcms`
(BSD-3/Apache) — already in the lockfile transitively under `image`**.

**CANNOT — lens corrections.** Double-blocked, and the worse block is the data: the Lensfun
library is LGPL-3.0, its Rust port has 252 downloads and a shifting API, and the **profile
database is CC BY-SA 3.0** — a share-alike *data* licence, a nastier problem inside an MIT/Apache
binary than any code licence. No permissive lens-profile corpus, no Rust LCP parser. (The math
is easy; you just can't ship profiles. DNG's own `WarpRectilinear`/`OpcodeList` sidesteps it.)

**CANNOT — RAW file parsing.** `rawler`/`rawloader`/`quickraw` LGPL-2.1, `imagepipe` LGPL-3.0-only,
`dng` and `zenraw` **AGPL-3.0**. Also absent from Rust entirely: guided filter, dark-channel prior.

**Correction to Part 2:** the raw ecosystem is not uniformly blocked. **Demosaic itself is now
permissive** — the `demosaic` crate is MIT/Apache with *zero* runtime deps (plain slices, so it
cannot even trip `single-image-library`), covering Bayer + X-Trans; 3 months old, 582 downloads,
so vendorable reference rather than load-bearing. The honest split: **you can demosaic
permissively; you cannot open the file permissively** — except **DNG**, which is TIFF-based and
publicly specified, and you already have `kamadak-exif` plus a TIFF path.

`imageproc` remains correctly rejected: `nalgebra` is non-optional in 0.27.

## 14. The differentiated use-case: preview + sidecar crop

**Verified with real files.** GitHub code search for `crs:HasCrop` returns 1,148 hits; the probe
parsed 315 candidates and found **134 genuine per-image sidecars** carrying `crs:HasCrop="True"`,
referencing CR2 ×26, ARW ×17, NEF ×14, JPG ×7, RAF ×4, CR3 ×2. Many stamped
`xmp:CreatorTool="Adobe Photoshop Lightroom Classic"`. `crs:CropAngle` present in **134/134**.

Coordinate system: `CropLeft/Right` are fractions of width, `CropTop/Bottom` of height, origin
upper-left; `CropAngle` in degrees −45..+45 about the rect centre, positive = clockwise.
**Mechanically verified with a control that could have failed:** on five real 6048×4024 Nikon Z6
sidecars with a 1:1 aspect lock, `wf×6048 × hf×4024` comes out exactly square (3014², 3040²,
3044², 3046²) while the swapped-axis alternative gives 2005×4530 — nowhere near square.

Two real gotchas found: real files use **`crs:CropUnit`** (singular), not Adobe's documented
`CropUnits`; orientation always arrives as **`tiff:Orientation`**, and `crs:Orientation` appears
in **0/134**. And `CropWidth`/`CropHeight`/`CropUnit` are **stale UI state**, not the crop's
aspect — observed `CropWidth=16 CropHeight=9` against a rectangle that is actually 3:2. The
rectangle is the truth.

**Why this is the opportunity:** for proprietary RAW, Adobe never writes into the file — that is
*why* sidecars exist. So the embedded preview stays the camera's original uncropped render
forever. **Preview + sidecar = the crop the photographer actually made, applied to the camera's
own render, with no demosaic.** Neither half is useful alone.

**And the camera render is free.** The embedded preview has the picture style baked in — Fuji
film simulations, Nikon Picture Controls, Canon Picture Styles, plus WB, tone and sharpening.
darktable's manual says so explicitly, and it's why images visibly shift when opened in any
converter. An entire tool category (Photo Mechanic, FastRawViewer's RawPreviewExtractor) is
built on exactly this.

## 15. What competitors actually do with RAW

| tool | opens RAW? | system dep | gets the embedded preview? |
|---|---|---|---|
| ImageMagick | conditional | **stock `brew install` is `--with-raw=no`**; delegate is `darktable-cli` | no usable path |
| libvips | yes, since 8.18.0 (Dec 2025) | libraw | no (metadata blob only) |
| sharp | **no** | — | no |
| @squoosh/cli | **no** (dead) | — | no |
| exiftool | metadata only | Perl | **yes — the only one** |
| dcraw / LibRaw | yes | dcraw / libraw | yes |
| imgproxy | Pro only, off by default | — | server, not batch CLI |
| rimage / oxipng / cwebp | no | — | — |

Re-verified first-hand on this machine, not relayed: ImageMagick 7.1.2-27's built-in delegates
list has **no `raw`**, and its `dng:decode` delegate shells out to `darktable-cli`, which is not
installed. So `magick photo.cr2 out.jpg` fails on a default Homebrew macOS install — and fails
*misleadingly* (`no images for write`, exit 1).

**The wedge, stated precisely:** every tool that "supports RAW" supports it by **demosaicing**
(slow, flat, no picture style). Every tool that gets the camera's own render cheaply (exiftool,
`dcraw -e`) **cannot resize or optimize**. Nobody bridges the two in one binary with zero system
deps — and **nobody at all reads the sidecar crop**.

Today's best option is two tools glued: `exiftool -b -preview:all` then `magick mogrify`. Its
sharp edges: a RAW with no preview writes a **0-byte file and exits 0** (silent), `-preview:all`
emits several files per RAW to dedupe, and which tag holds the big preview is vendor-dependent.

## 16. What would kill it

1. **Preview size is vendor-and-generation roulette.** Against a 2048 px target: Nikon NEF full
   sensor ✅, Canon CR2 full-res ✅, Fuji X-T3+ 4416 ✅ — but **Sony ARW pre-A1 is 1616×1080** ❌,
   Olympus ~1610 ❌, Panasonic 1920×1440 ❌, older Fuji 1920 ❌, and **DNG defaults to Medium
   ≈1024** ❌. A Sony shooter's honest verdict: it fails after *"the slightest crop."*
   The feature must **measure the preview and degrade honestly**, never silently ship a 1616 px
   "web image."
2. **The tag trap.** Reading `PreviewImage` instead of the largest preview silently turns a Canon
   CR3 from **8192 px into 1620 px**. Enumerate and pick by size (crustyimg's existing SOF peek
   already does the right thing here).
3. **The crop is Lightroom-only.** darktable stores crop as an **opaque versioned binary struct**
   in its history stack (decoded from a real file: `cx=0.023309, cy=0.012280, cw=0.990685,
   ch=0.978210, ratio 2:3`) — reverse-engineering a private layout. Capture One doesn't put crop
   in XMP at all (it's in a proprietary `.cos`). Scope the claim to Lightroom/ACR or it becomes a
   reach claim — see the repo lesson that a guard's advertised reach is itself a claim.
4. **Double-crop risk.** DNG's "Update DNG Preview & Metadata" re-renders the embedded preview
   *with* the crop; in-camera aspect modes (Fuji GFX100RF, X-T2) apply crop non-destructively.
   Reconcile preview dimensions against `tiff:ImageWidth/ImageLength` before applying anything.
5. **The pre-rotation frame claim is documented three ways but never falsifiably observed.**
   Adobe's SDK, a Lightroom plugin, and darktable's `lightroom.c` all agree the rectangle is in
   the as-stored sensor array *before* EXIF orientation. No real-file test in the corpus could
   distinguish the hypotheses. **One controlled Lightroom crop on a portrait RAW settles it in
   five minutes and must come before any code** — this is exactly
   the repo lesson that a plausible test result is not a checked one (a negative control is the cure).

## 17. Defects found in passing (recorded, not dispatched)

**(a) `convert` strips EXIF Orientation without baking it — measured, with a positive control.**
`ctrl.jpg` (1200×800, `Orientation=6`, 14 EXIF tags) → `convert` gives 1200×800, tag stripped,
pixels **not** rotated → displays sideways in every viewer. `web`, `optimize` and `auto-orient`
all correctly give 800×1200, which proves the harness can show the other result.
Root cause, code-confirmed: `run_convert` (`src/cli/optimize.rs:507-542`) builds
`Pipeline::new()` — an empty pipeline, "pure re-encode … pixels unchanged" — and the pixel-lane
re-encode drops metadata (`docs/api-contract.md:158`), so the tag is discarded but the rotation
it described is never applied. `optimize`/`web` instead pin `auto-orient` first
(`src/cli/optimize.rs:585, :779`, DEC-017). Orientation 6 is the ordinary phone case.
Worth sweeping every other re-encoding verb (`thumbnail`, `resize`, `responsive`, `edit` without
`--auto-orient`) **mechanically**, per the repo lesson that mechanical sweeps need a mechanical check — cite the grep, and treat its SCOPE as a claim too.

*(Resolved 2026-08-06: this is the defect SPEC-110/DEC-086 fixed — every pixel-lane verb now
bakes orientation via a shared `auto_orient_prefix()`, including `watermark`, caught in
SPEC-110's punch-list pass. Left as-written above since this section records what the research
session found at the time, not current behavior.)*

**(b) RAW loses 100% of EXIF.** The Leica Q2 DNG carries 92 tags; `convert`, `web` and `optimize`
outputs carry **zero** — no Make/Model, lens, DateTimeOriginal, Artist, Copyright, ISO, aperture,
shutter. Cause: `raw_preview` sets `metadata: None` (`src/image/mod.rs:462-468`). This is the
thing that makes "proof sheet from RAW" a toy rather than a workflow.

**(c) A correction to a planned fix.** DEC-055 contemplates threading the winning preview's own
APP1 forward. Measured: this DNG's embedded previews contain **no EXIF at all** — a JPEG marker
walk shows `FFDB → FFC0 → FFC4 → FFDD → SOS`, no APP0/APP1. That follow-up would not restore
orientation; the container's IFD0 tag is the only source.

**(d) RAW orientation is never read — mechanism confirmed, real-file case not provable here.**
`grep -i orient src/image/raw.rs` → 0 hits. A synthetic `Orientation=6` DNG comes out landscape
from `web`/`optimize`/`auto-orient` with no tag, while the JPEG control rotates correctly. But
the only RAW on this machine is landscape/Orientation=1, so the real-world portrait case is
**unproven** — the unverified link is whether a real portrait RAW stores its preview in sensor
orientation. Stated as cannot-determine, not confirmed.

## 18. Recommendation for Part 3

**Pursue — narrowly, in two separately-verified steps.**

**Step 1 — "RAW off the card to web, one binary."** Preview extraction (exists) + EXIF
carry-through (fix (b)) + orientation from IFD0 (fix (c)/(d)) + resize + optimize. No new
dependency, no coordinate-system risk, no tonal work. It already beats the
exiftool-plus-mogrify dance, and it fixes two measured defects. Must include an honest
preview-size report so a 1616 px Sony preview is never silently sold as a web image.

**Step 2 — sidecar crop, only after a controlled Lightroom experiment settles the rotation
frame.** `crustyimg view photo.NEF --show-crop` (a crop box drawn over the preview, using the
existing `Sink::Display` and the `Watermark` compositing loop as precedent) is the natural first
surface: it is the "auditable decision" idea — *show me what you'd do before you do it* — applied
to geometry, and it needs no crop op to exist yet. Applying the crop follows wave 5.

**Do not** build tonal/develop ops for this. The differentiation is entirely in geometry +
metadata + the camera's free render, and every hour spent on exposure curves is an hour spent
on the one axis where the answer is "you cannot match Lightroom anyway."

---

# Part 4 — Innovation opportunities: what the evidence supports and contradicts

Follow-up: beyond L1/L3, what genuine innovations are available, and what Rust libraries could
push further? Research below is evidence-led and deliberately skeptical; two findings
**contradict** current roadmap positioning.

## 19. Smart crop — DO NOT BUILD. The evidence is now stronger than the prior verdict.

The repo already demoted this (`docs/roadmap.md:53`: *"the code is an afternoon; the
tune-until-it-looks-right-on-a-diverse-corpus loop runs on human eyeballs"*). The new evidence
removes the wedge entirely:

- **libvips `smartcrop` / sharp `attention` is a pure heuristic** — one pass over edges, **skin
  tones**, and saturation, blurred to a low-res score image, max-scoring rectangle wins. No ML.
  It was adapted from sharp's port of smartcrop.js.
- **imgproxy's free "smart" gravity IS that libvips heuristic.** Its ML object detection
  (YOLO/ONNX) is Pro-only.
- **thumbor** ships `opencv-python-headless` as a normal dependency since 7.0.0 — a pip install,
  not a service.

So the hypothesised wedge ("we do smart crop without heavy deps") **describes what sharp already
does, for free, built in**. And the heuristic's quality problem is documented by the incumbent's
own maintainer (imgproxy discussion #1146): it *"sometimes fail[s] miserably"* because it has no
semantic understanding — *"they don't see any difference between a car, a tree, a fox, or a
wall"* — and his recommended fix is the ML detector, i.e. the paid tier. crustyimg's no-ML
identity blocks that escape hatch.

**Verdict: keep it demoted, and record the better reason.** You would ship the exact algorithm
whose failures are the top complaint against the incumbents, with no dependency advantage.

## 20. Region-adaptive encoding — park; blocked upstream

AV1 signals `delta_q_present_flag`/`delta_q_idx` per superblock and libaom exposes
`--deltaq-mode` (+ `AV1E_ENABLE_RATE_GUIDE_DELTAQ` in v3.7 for all-intra). But **rav1e exposes no
per-block quantizer map / ROI API** — Xiph's own "Unimplemented x264 features in rav1e" lists
ROI-with-delta-q as hard and unimplemented. Since AVIF encode goes through ravif/rav1e this is
exogenous, same category as the existing rav1d/jpegli gates. No published size-win numbers for
saliency-guided delta-q on *still* images, and no OSS still-image tool doing it.
**Revisit only if rav1e grows a quantizer-map API.**

## 21. Perceptual targeting — real, already built, but reposition and watch a new competitor

The unoccupied slot is **free + cross-format + SSIMULACRA2 + offline/in-browser**. Everything
else is either single-format-and-dying (`jpeg-recompress`, SSIM, unmaintained), not perceptual
(`cwebp -psnr`, `jpeg:extent`), absent (**sharp/libvips have an integer `quality` and no search
at all**), or paid (imgproxy autoquality Pro, Cloudinary `q_auto`, ModPageSpeed 2.0 — which
predicts q with LightGBM then verifies with SSIMULACRA2).

Two cautions:
- **Demand is inferred from vendor behaviour, not user requests.** No sharp issue asks for
  target-quality search; the mass market appears content with `quality: 80`.
- **A direct competitor appeared ~6 months ago:** Imazen's **`zensim`** — fast SSIMULACRA2
  approximation (~22 ms vs ~377 ms at 1080p), **MIT/Apache, pure Rust**, with zenjpeg/zenwebp
  built around it. Same language, same permissive-core split, same lane — and it is the
  ready-made answer to the "SSIMULACRA2 is too slow" objection every vendor cites.
  *(License/maturity pending independent verification.)*

**So "nobody does perceptual targeting" is already false and will read as stale within a year.**
The defensible claim narrows to *free, cross-format, offline/in-browser*. `zensim` is worth
evaluating as an optional fast search metric with the BSD-2 `ssimulacra2` crate as the
verification oracle. (Note: `ssimulacra2` is BSD-2-Clause — a third licence in an MIT/Apache tree.)

## 22. C2PA — real gap, imagined demand; ship a lint rule, not a signer

**Verified: optimization does destroy Content Credentials.** Cloudflare's own docs state that
with preserve off *"any existing Content Credentials will always be discarded"* — stripping is
the default; with it on they keep credentials and *append and cryptographically sign additional
actions*. Cloudinary's `fl_c2pa` does the same with a non-editorial allowlist.

Mechanism: in JPEG the manifest lives in JUMBF in **APP11**, not APP1 — so every tool that
"preserves metadata" by walking APP1 silently drops it. sharp/libvips/ImageMagick have zero C2PA
awareness. `c2patool` can sign but **cannot resize or re-encode at all**. Nobody has joined the
two halves in a free tool.

`c2pa-rs` is **MIT OR Apache-2.0**, actively developed, compiles to `wasm32-unknown-unknown` —
but it is 0.x with breaking releases roughly every two months.

**Two hard blockers on the Wave 7 signing plan:**
1. Signing needs an X.509 cert on the C2PA Trust List, which requires conformance evaluation of
   the *generator product*. A self-signed manifest renders as untrusted in CAI Verify —
   **shipping one would be worse than worthless.**
2. Naively copying a manifest through a re-encode produces a **hash mismatch that fails
   validation** — arguably worse than a clean strip.

And no developer was found complaining that their optimizer stripped credentials; all demand
traces to vendors, standards bodies and governments (there is even a counter-market of tools to
*remove* C2PA).

**The cheap honest play: a lint rule** — *"this input carries Content Credentials; this operation
will invalidate them."* No cert, no false trust claim, reader-falsifiable, reuses the shipped
`lint` surface, and makes crustyimg the first free tool to even notice. Treat signing as blocked.

## 23. Reproducible builds — largely imagined demand. Do not lead with it.

- **Every major SSG already ships a content-hashed derivative cache**: `@11ty/eleventy-img`
  (disk cache, hashed filenames, documented CI-cache-reuse pattern + a Netlify demo repo), Hugo
  `resources/_gen`, Astro `astro:assets`, Gatsby `.cache`, Next `.next/cache/images`.
  Turborepo/Nx cover the generic "don't redo the step" case.
- **The one standalone product in this exact category is dead**:
  `MarcusCemes/image-processing-pipeline` — declarative YAML, manifest output, libvips —
  **50 stars, archived 2024-05-22**. That is the honest market-size signal.
- **Complaint traffic is uniformly about build TIME and cache misses, never byte drift.** A hard
  search for "an encoder upgrade changed my bytes" found nothing.
- The reproducible-builds community touches images only at the metadata layer
  (`strip-nondeterminism` clamps PNG `tIME`/text); their stated position is that encoder-version
  drift is **expected**, cured by recording the build environment, not the artifact.

**Sell recipes + cache + lockfile as SPEED AND CONTROL** — "only re-encode what changed, across
machines, without a Node toolchain." That has complaint traffic behind it. "Byte-identical" does
not.

⚠️ **And verify before claiming**: `aomenc` and `vpxenc` both ship
`-D, --debug: Debug mode (makes output deterministic)` — proof the reference AV1 encoders are
**not** deterministic by default. rav1e/ravif determinism is unproven in public sources either
way. If crustyimg's AVIF is byte-deterministic across thread counts that is a small real
differentiator; if it isn't, existing "reproducible" language is a **false claim**.

## 24. Privacy / client-side — not a purchase driver. Cut the sector story.

- **~20:1 against**: iloveimg ~36.4M visits/mo + tinypng ~4.94M vs squoosh.app ~2.08M
  (third-party estimates, directional only).
- **The no-upload niche is saturated with builders and starved of users.** Show HN scores for
  privacy-first browser compressors, 2025-12 → 2026-07: **1, 2, 1, 2, 2 points.** Squoosh itself
  scored 241 — on codec quality and the comparison UI, **not** privacy.
- **Scale check**: `sharp` = 76.8M dl/wk. All browser/edge wasm image codecs summed ≈ 610k dl/wk
  (**~126:1**). `crustyimg-wasm` = 146 dl/wk. The mainstream client-side package is
  `browser-image-compression` at 1.42M dl/wk — a bandwidth-saver in upload forms, not a privacy
  tool.
- **Legal/medical/DLP/defence: zero concrete artifacts found.** No named policy, no blocked
  domain, no incident. The one strong privacy artifact — FBI Denver's 2025-03 warning about
  malicious free online converters — argues for *"install a trustworthy inspectable tool"*
  (favours the CLI), not for a browser demo.

**Cut the legal/medical/DLP story from any launch post; it reads as invented because it is
unverified.** The demo's real job is a zero-friction live trial of the engine.

**Correction to the launch narrative:** `GoogleChromeLabs/squoosh` is **not archived** — 25.5k
stars, `archived: false`, squoosh.app live at ~2M visits/mo, and Jake Archibald retains commit
access and has stated an intent to update JPEG XL/AVIF. The **npm CLI** is genuinely deprecated,
so *"squoosh-cli is abandoned"* holds; *"Squoosh is archived"* would be factually wrong and is
checkable in one click.

**Where a real capability hole exists — edge runtimes.** Photon (54,891 dl/wk on
`@cf-wasm/photon`) has **no AVIF and no output-quality control** — issue #52 (quality) open since
**2020**, #159 since 2023, #208 since 2025. Workers ceilings are real (3 MB gzip free / 10 MB
paid, 128 MB per isolate, 10 ms CPU free). Cloudflare sells the integrated alternative with a
free tier. **Genuine but modest, and it's a library play, not a product play.**

⚠️ **Unresolved contradiction between two research streams — do not rely on either until settled
from primary sources.** One reports sharp 0.34 (Q1 2026) shipped a hardened WASM build usable on
Workers/Vercel Edge/Deno Deploy at 4–6× native cost. The other reports no primary source, and
quotes sharp's own install docs — *"Use in web browsers is unsupported"*, *"Use in
single-threaded environments is unsupported"* — plus the maintainer on sharp#3860 saying Workers
can't work for lack of threading. This is load-bearing for the edge wedge.

## 25. Asset linting — keep it demand-gated, with one exception

Lighthouse's image audits are real but shallow: `uses-responsive-images` fires only above a
4 KiB waste threshold (12 KiB with srcset/`<picture>`) and **skips any image whose natural
dimensions or network info it can't obtain**. Long-standing complaints: it only cares about
savings not correct responsive usage (#10434), `unsized-images` false-flags intrinsically-sized
images (#11571), a live DPR>1 false-flag (#17080), and Lighthouse 13 renamed audits that CI
scripts reference.

But the structural asymmetry is fatal to replacement: **Lighthouse needs a deployed URL and a
viewport; a static linter can lint a PR diff but cannot know rendered display size** — which is
the single highest-value rule ("4000 px serving a 400 px slot"). The roadmap already gates lint
expansion on adoption signal; **that call is correct**. The one item worth pulling forward is the
C2PA-detection rule from §22 — cheap, unique, honest.

## 26. Ranking

1. **Gamma-correct / colour-managed resize, proven with SSIMULACRA2** (= L1/L2 from Part 3
   §"expansion path"). In-domain, no new deps, no eyeball-tuning loop, and **crustyimg is one of
   very few tools that can *prove* the difference because it already computes SSIMULACRA2.**
   Reader-falsifiable, mechanism-level, inside the stated domain. The opposite of smart crop on
   every axis that made smart crop a bad bet. ⚠️ libvips' own banding issues (#1144, #2238) show
   a naive implementation makes things **worse** — needs 16/32-bit intermediates, and the
   **magnitude must be measured before any claim ships**.
2. **Reposition perceptual targeting** (§21) — already built; the work is messaging, plus
   optionally `zensim` to kill the speed objection. The window on the current claim is closing.
3. **C2PA *detection* lint rule** (§22) — cheap, unique, honest, zero cert exposure.
4. **Edge/wasm as "the analysis engine where sharp can't go"** (§24) — genuine, modest, and
   contingent on resolving the sharp-WASM contradiction.
5. **Region-adaptive encoding** (§20) — park, blocked at rav1e.
6. **Lint breadth** (§25) — keep demand-gated, as planned.
7. **Reproducible builds** (§23) — already shipped; stop investing, don't market on it.
8. **Smart crop** (§19) — **do not build.** The evidence actively contradicts it.

## 27. The cross-cutting lesson

**The two claims that feel most compelling — "verifiable/reproducible" and "private/no-upload" —
are the two with the least evidence behind them.** The claims with actual complaint traffic are
mundane: build time, cache invalidation, CI/hosting cost, zero-dependency deployment, and
encoder capability holes at the edge (AVIF + quality control). A launch post leading with
reproducibility or privacy leads with the parts a skeptical reader cannot verify and a
knowledgeable one will recognise as already-solved-or-unwanted.

---

# Part 5 — Rust library landscape: verified, with the hype removed

All facts below pulled from the crates.io API + GitHub API on 2026-07-25/26, several by inspecting
the published `.crate` tarballs rather than READMEs. Current stable Rust: 1.97.1.

## 28. Act on these three (certainty order)

**1. `moxcms` — zero marginal cost.** `image` 0.25.10 depends on `moxcms ^0.8.0`
**non-optionally**, so it is ALREADY in crustyimg's tree. BSD-3-Clause OR Apache-2.0, v0.9.0
(2026-07-22), 69 commits/6mo, **1 open issue**; its only deps are `num-traits` + `pxfm`. Does
any-to-any Display-Class ICC profiles, CMYK↔RGB, LAB↔RGB. **Lowest-risk, highest-certainty item
in this whole document.** ⚠️ Gamut mapping specifically is NOT documented in its README — verify
before promising it.

**2. `pic-scale` — gamma-correct resize without writing it.** BSD-3 OR Apache-2.0, 0.7.10
(2026-07-06), 211K downloads / 135K recent. Ships **built-in linear / Lab / Luv / Oklab /
Jzazbz / sigmoidal resize** plus f16/f32/high-bit-depth, NEON/SSE/AVX2/AVX-512/AVX-VNNI, and
**mandatory `simd128` on wasm**. `ImageStore` wraps slices rather than owning a competing image
model → only a *minor* `single-image-library` question.
⚠️ Caveats: **bus factor 1** (one human, 0 forks, 39 stars); AVX-512 needs **nightly**; and the
"faster than fast_image_resize" claim is **stale and withdrawn** — it lived in the v0.1.1 README
(June 2024) and is absent from current master (search engines still serve the old text). A real
`speedtest/` harness exists but is run by pic-scale's own author; **no third-party head-to-head**.
Decide on your own numbers.

**3. `fast-ssim2` — the highest-leverage single swap.** BSD-2-Clause (same as the current dep),
0.8.2 (2026-06-10). An explicit fork of rust-av/ssimulacra2 computing **actual SSIMULACRA2**;
entry point `compute_ssimulacra2(src, dist) -> f64` matches upstream → near-drop-in.
**3× faster at 1080p (350 ms vs 1,056 ms)**, and adds wasm32 SIMD128 (via `archmage`), a
**bounded-memory strip path**, cooperative cancellation, and `#![forbid(unsafe_code)]`.
Meanwhile the incumbent `ssimulacra2` 0.5.1 **has not shipped since 2024-12-29** (19 months) and
pins `yuvxyb ^0.4.1` against a current 0.6.0.
⚠️ Gate on your own independently-seeded parity fixture, not on the fork's claim; the strip path's
~1e-5 agreement is *their* number until reproduced. Per
the repo lesson that fixtures produced by the code under test cannot fail — seed independently and assert values.

## 29. `zensim` — correcting an earlier overstatement in this document

Part 4 §21 framed `zensim` as a same-lane competitor. Refinement after verification: it is
MIT/Apache, pure Rust, 0.2.7 (2026-04-27), and the ~48× number is real and self-published with
methodology (1080p: 22 ms vs `ssimulacra2-rs` 1,056 ms, Ryzen 9 7950X, criterion median of 100).

**But it is NOT SSIMULACRA2 and NOT a drop-in.** Its README: built on the same psychovisual
foundations *"but with trained weights"* — a 372-input MLP with a 27 KB packed weight bake. Its own
accuracy disclosures: on the CID22 holdout it reaches SROCC ≈0.876 vs **fast-ssim2's 0.89**, and
its Z-RMSE (0.523) trails ssim2's 0.460 — *"None of the four hits all three holdouts."* Its repo
also flags that older trainers used ssim2-derived targets, which biases SROCC toward ssim2-shaped
surfaces — i.e. **self-referential**, and they say so. Published at 0.2.7 while the README
documents an unreleased v0.3. Trained weights add a **model-licensing surface** not verified here.

**Verdict: watch, don't adopt.** Adopting would mean re-baselining every published quality number.

## 30. `butteraugli` — a second, independent metric (the strategically interesting one)

imazen/`butteraugli` 0.9.3 (2026-05-28), **BSD-3-Clause**, pure Rust, wasm SIMD128, validated
*"< 0.001% relative difference vs libjxl `butteraugli_main`"*, ~2–3× faster than the C++ pipeline
single-threaded.

The argument for it is not speed — it's that **every quality claim crustyimg makes currently flows
through one metric implementation.** A second metric from a different lineage, validated against a
different reference, is precisely the independent check the repo's own lesson demands:
the repo lesson that a self-referential control cannot detect a broken pipeline. Discount for bus-factor-1 and the
AI-authorship notice (§32).

## 31. The Imazen license trap

Their **metrics and infrastructure are permissive; their ENCODERS are AGPL-3.0-only or
commercial** — and `zencodec`'s own README **misstates this**, claiming all zen* crates are
MIT/Apache. Verified by reading each `Cargo.toml` in-repo:

- **Permissive/usable:** `archmage`/`magetypes` (MIT/Apache), `fast-ssim2` (BSD-2),
  `butteraugli` (BSD-3), `zensim` (MIT/Apache), `mozjpeg-rs` (BSD-3), `ultrahdr-core` (Apache-2.0),
  `linear-srgb`, `enough`, `codec-eval`, `zenpixels-convert`, `zencodec`.
- **BLOCKED (AGPL):** `zenjpeg`, `zenwebp`, `zenquant`, `jxl-encoder`/`-simd`, `jpegli-rs`.

Contamination checked: the permissive ones have **no AGPL crate in their required dependency
sets**. Clean. → Watchlist the encoders with "revisit if relicensed", as was done for `dssim`
(which is itself **AGPL-3.0** and correctly blocked).

**There is no permissive Rust jpegli.** `jpegli-rs` is AGPL; the old BSD `jpegli` 0.1.0 is a dead
stub. The permissive path to better JPEG is `mozjpeg-rs` (BSD-3, byte-identical to C mozjpeg in
baseline/progressive; trellis 6% faster than C, baseline-only ~4.6× slower, no wasm SIMD).

## 32. An authorship caveat to record in any DEC

Several Imazen crates carry an explicit notice, e.g. `mozjpeg-rs`: *"developed with significant
assistance from Claude (Anthropic) … **not all code has been manually reviewed or human-audited**."*
`butteraugli` and `ultrahdr-core` carry equivalents. Not disqualifying — their FFI-parity testing
against the C reference is the right control and is more validation than most crates have — but it
belongs in the decision record, and it argues for running **your own** parity fixtures.

## 33. rav1e is near-dormant — this matters more than region-adaptive encoding

- Last release **0.8.1, 2025-06-16** (13 months). Last `master` commit **2025-12-03**.
  **Zero commits in 6 months.** 258 open issues. `ravif` 0.13.0's parent had 1 commit in 6 months.
- **No determinism guarantee found** — no issue, PR or doc asserts byte-identical output across
  thread counts. And issue **#2781 "Non-deterministic encodes on Speed 10 on fast-scene
  detection"** shows nondeterminism was a **real filed bug**, not hypothetical.
  → Any test or cache key assuming AVIF output is byte-reproducible is **unbacked by upstream**.
  Pin on decoded pixels or a perceptual score, or force single-threaded encode and *measure*.
- **No per-block quantizer / ROI / delta-q API.** Spatial AQ exists internally (closed PRs #2247,
  #2933, #2993) but is unexposed; #2512 "Varying quality within the frame" and the #2759
  "[Meta] Parity with aomenc" are open. **This definitively confirms Part 4 §20.** Content-aware
  bit allocation would have to precondition the input, not steer the encoder.
- ⚠️ A web source claiming rav1e ships *"weekly pre-releases every Tuesday"* contradicts the
  primary record and is **false**.

## 34. sharp on the edge — settled from primary sources

sharp's own install docs, verbatim: *"Runtime environments that provide multi-threaded Wasm via
Workers are supported by the optional `@img/sharp-wasm32` package."* · *"Use in web browsers is
unsupported."* · *"Use in single-threaded environments is unsupported."*

So the Part 4 §24 contradiction resolves **mostly in favour of the sceptical source**:
- An official WASM build **does** exist → "there is no wasm build" was too strong.
- But the docs **never mention** Cloudflare Workers, Vercel Edge or Deno Deploy; the support gate
  is *multi-threaded Wasm via Workers*, exactly what constrained edge runtimes lack; and the
  **"4–6× native speed" figure has no primary source and is incoherent** (a wasm build cannot
  outrun native). The version was also wrong — sharp is at **0.35.3 (2026-07-01)**, not 0.34.
- **The defensible claim: a pure-Rust wasm library with no threading requirement goes where sharp
  documents that it cannot.**

## 35. Banded/streaming execution — nobody has built it, but the substrate is moving

**No credible pure-Rust libvips equivalent exists.** Whole-image-in-RAM is still the norm. Signals:
- `image`'s open **1.0 milestone** contains **#2300** (a `rows()` iterator on
  `SubImage`/`GenericImageView` to unlock banded access) and **#2357** (better parallelism
  controls) — recognised 1.0-era goals, **not landed**.
- `zencodec` exposes a `StreamingDecode` trait, *"an intentionally `Send` alternative to one-shot
  decoding"*.
- Bounded-memory **strip** paths now ship in `fast-ssim2` and `butteraugli`.

So banded *metrics* are available today; banded decode→transform→encode is not. Recall this is not
merely a performance item — it is the **precondition for f32 at full resolution**: 64 Mpix ×
16 B/px = 1 GiB against a 512 MiB single-allocation cap (Part 3 §"the constraint").

**`viprs` (the claimed Rust libvips) is real code and unusable today.** MIT, ~51K LOC across 432
files / 12 workspace crates, 554 tests, a real `cargo xtask bench` against libvips — but
**one crates.io release ever (0.1.5, 2026-06-24), 24 total downloads, 0 stars, 0 forks,
37 commits**, most recent commit by `github-actions[bot]`, ~42 open issues, no published benchmark
numbers, no wasm story, one maintainer, its own image type, and an optional `libheif-rs` (C) dep.
libvips is 20+ years of accumulated correctness edge cases. **Watch; do not plan around it.**
⚠️ **`libviprs` is a completely different project** (DeepZoom/PDF tile pyramids for AEC) —
conflating the two would be an error.

## 36. SIMD — `std::simd` is not coming; use `pulp` or `archmage`

**VERIFIED: rust-lang/rust#86656 (Portable SIMD tracking issue) is still open and untouched since
2025-03-08** (~17 months). `rust-lang/portable-simd`'s own description still reads *"the testing
ground for the future of portable SIMD in Rust."* **Do not plan on `std::simd`.**

| option | license | state | wasm |
|---|---|---|---|
| **`pulp`** | **MIT** | **production** — 16.6M dl / 6.4M recent, 355★, 16 contributors, by the `faer` author | **yes** — real 57 KB `src/wasm.rs` simd128 backend |
| **`archmage`** | MIT OR Apache-2.0 | usable, rising — 0.9.28 (2026-07-21), 111k recent | **yes — first-class `Wasm128Token`/SIMD128 tier** |
| `wide` | **Zlib OR Apache-2.0 OR MIT** | **production — reached 1.0** (1.5.0, MSRV 1.89) | yes |
| `multiversion` | MIT/Apache | stale — no release since 2024-12-08 | n/a |

`pulp` is pure SIMD over slices → **no `single-image-library` conflict**; MIT-only means no
explicit Apache patent grant. `archmage` uniquely keeps `#![forbid(unsafe_code)]` while calling
intrinsics, with runtime dispatch across x86-64/AArch64/wasm128 — and adopting `fast-ssim2` pulls
it in transitively, so evaluating it is nearly free.

## 37. `quantette` — the backlog's instinct was right

MIT OR Apache-2.0, 0.6.0 (2026-05-15), **516K downloads / 335K recent**, active, and **`image` is
an optional feature** → sits cleanly inside `single-image-library`. This is the permissive
quantizer the PROJ-002 brief deferred indexed-PNG wins on. Bus factor effectively 1.

## 38. UltraHDR gain maps — the genuine new find

Every recent flagship phone emits gain-map JPEGs: a normal SDR JPEG with a second JPEG (the gain
map) plus metadata stapled on; HDR-capable readers reconstruct HDR, everything else sees the SDR
base. **An optimizer that silently discards the gain map destroys HDR information the user can see
on their own display** — the same class of defect as the `convert` orientation bug (§17a).

`ultrahdr-core` 0.6.0 (2026-07-24), **Apache-2.0**, pure Rust, `#![forbid(unsafe_code)]`,
`no_std + alloc`, **wasm-compatible**, MSRV 1.92 — encodes *and* decodes gain maps and can tone-map
HDR-only input to generate the SDR base. Independent second implementation: `gainforge` (BSD-3/
Apache, alpha). ⚠️ Self-described *"pre-1.0 and experimental"*, *"expect renames between point
releases"*, plus the §32 authorship notice.

**Cheapest correct move today: a test fixture that DETECTS gain-map input**, so you learn when
you're dropping one. Full integration later.

## 39. Other verified items worth knowing

- **`imagesize`** — MIT, 0.15.0, **9.6M recent dl**. Dimensions-only probing without decoding —
  exactly right for a hostile-input preflight that must not allocate.
- **`enough`** — MIT/Apache, 309k recent dl. The ecosystem's cooperative-cancellation token, now
  threaded through `fast-ssim2`, `butteraugli`, `zencodec`, `mozjpeg-rs` and `zune-jpeg`'s dev
  branch. For wasm this is the difference between a responsive tab and a frozen one.
- **`jxl-oxide`** (MIT/Apache, 0.12.6, production, 389k recent) — mature JXL **decode**, no
  encoder. The official libjxl-org `jxl` crate (BSD-3, the decoder Chromium shipped) is also decode
  only, self-described WIP. `zune-jpegxl` is the only permissive pure-Rust JXL *encoder* and is a
  self-described *"small POC"*, **lossless only**. JXL is ~12–17% global support (secondary
  sources) → **an archival/progressive-enhancement format in 2026, not a delivery default.**
  Decode before encode.
- **`image` 1.0 milestone**: #2636 "Replace in-tree JPEG encoder with `jpeg-encoder`" is
  **CLOSED** — note `jpeg-encoder`'s license is `(MIT OR Apache-2.0) AND IJG`; the IJG term rides
  along. And **#2748 "Rework WebpEncoder API to support lossy compression"** is open → pure-Rust
  lossy WebP encode is **not there yet**.
- `fast_image_resize` is now **6.1.0 (2026-07-21)** — check the pin.
- **`palette`** repo is alive but has had **no release since 2024-04-28** (27 months). Fine to use;
  wrong to assume it's moving.
- **ML runtimes**: `rten` (MIT/Apache, pure Rust, no C) and `tract-onnx` are the only two fitting
  the pure-Rust-default rule; `ort` is a C++ binding → feature-gate only. **The runtime license is
  the easy half — model WEIGHTS are separately licensed and frequently non-commercial or GPL.**
  Treat weights licensing as an unresolved blocker, not a detail.
- **No permissive Rust smart-crop exists**: `smartcrop` is **GPL-3.0 AND abandoned since 2018**
  (double disqualification); **`seam-carving` is NOT FOUND on crates.io**; `mss_saliency` is MIT but
  abandoned 2022 (a small classical algorithm — vendor it, don't depend on it). Combined with
  Part 4 §19, the demotion is firmer than ever.
- **No Rust XPSNR crate exists**; **no permissive pure-Rust VMAF** (FFI shims only).

## 40. Claims from the handed-me list that did not survive verification

| claim | reality |
|---|---|
| `ddot` — wgpu pipeline, 11 dither algos | **NOT FOUND on crates.io.** `ddot-core`/`ddot-cli` exist: 0.1.x, all published within 30 min on 2026-07-15, **61 downloads, 0 stars, 10 commits**. Toy. Own `src/image/` → constraint conflict |
| `ranga` — wgpu, blend modes, ICC, Delta-E | **GPL-3.0-only** (was **AGPL-3.0-only** ≤0.29.4). Hard license stop, twice. 0 stars, dead since 2026-04. Claims *do* check out (12 blend modes, real `GpuChain`, 38 KB `icc.rs`) — irrelevant |
| `gpush` — "GPU 418 ms vs CPU 3 ms" benchmark | **NOT FOUND on crates.io.** GitHub only: **2 stars, 2 commits**, untouched since 2025-10. README assertion, no harness, and **internally incoherent**: 418 ms total vs 12+1+3 = 16 ms accounted → ~400 ms unattributed = cold-start device init + shader compile. **Worthless as a number.** The underlying GPGPU principle is true but standard knowledge, not established here |
| `viprs` — Rust libvips | Real, MIT, ~51K LOC — but 24 downloads, 0 stars, 1 release, bot's last commit. Unusable. See §35 |
| `pic-scale` faster than `fast_image_resize` | **Stale/withdrawn claim** — v0.1.1 README (2024-06), absent from current master. Self-benchmarked only. AVX-512/VNNI support *is* real (AVX-512 needs nightly) |
| `yscv-imgproc` — 178 SIMD ops, "faster than OpenCV on all benchmarks" | **Inflated three ways.** Op count unsettled (README 160, top-level 171, actual count 186 — **178 appears nowhere**). Headline claim is vs **ONNX Runtime, not OpenCV**. The OpenCV table (1.20–3.27×) is 640×480 only, unspecified Apple Silicon, labelled *"pending re-measurement"*, doc says *"treat those as provisional"*, **no script shipped**, no independent verification. SIMD engineering *is* real (139 `target_feature`, 446 `unsafe`). Alpha: 305 downloads, own `Tensor` type → conflict |
| `yscv-imgproc` for feature matching | ⚠️ **Also a live patent exposure**: it ships **SURF** (US 8,165,401, ETH Zurich, ~2027 expiry) with **zero mentions of "patent"** in the tarball — no notice, no feature gate, no way to exclude. OpenCV gates SIFT/SURF in `xfeatures2d` NONFREE for exactly this reason. (SIFT itself is clear — expired 2020.) And SIFT/ORB/optical-flow features would make crustyimg a **CV library, not an optimizer** — off-thesis |
| `purecv` — pulp, zero unsafe | **LGPL, and inconsistent with itself**: crates.io `LGPL-2.1-or-later`, GitHub+README `LGPL-3.0`. Claims accurate, but `pulp` is *optional* there → **use `pulp` directly, skip the LGPL** |
| `fovea` — type-safe pixel formats | **57,401 LOC and 2,705 tests in 7 commits** by a 1-star account, **111 downloads**. That ratio is not human authorship. **The typestate idea is worth stealing for L1's pixel types; do not depend on the crate** |
| `oximedia-simd` — DCT, motion estimation, hand-written assembly | Claims mostly check out (24K LOC, 901 tests, Apache-2.0, **zero deps**, no constraint conflict) — but **"hand-written assembly" is false: zero `asm!` blocks**, it's intrinsics. 243 downloads, 11 commits. Video-codec kernels → irrelevant to a still-image optimizer |

## 41. GPU verdict, and its honest limits

**Not worth it now** — and one reason beyond bundle cost and transfer overhead: **wgpu's fallback is
WebGL2, which has no compute shaders**, so the CPU fallback cannot be a port of the same kernels —
you would owe a permanently divergent second path. wgpu also carries 1,195 open issues and a major
version every few months (v30.0.0, 2026-07-02), a fast-moving surface to pin against.

⚠️ **Stated limit:** the browser-WebGPU-availability half of this research was lost when a
sub-agent died. This verdict is **inference from first principles, not measurement.** Re-run the
browser-support half before recording it as a decision.

## 42. Revised ranking

1. **`moxcms` for ICC/color** — already in the tree, zero new license surface (§28).
2. **`fast-ssim2` to replace `ssimulacra2`** — real SSIMULACRA2, 3× faster, wasm SIMD128, strip
   path, cancellation; incumbent is 19 months stale (§28). Gate on your own parity fixture.
3. **Linear-light resize** — the Part 4 #1 item, now with `pic-scale` as a candidate backend
   instead of a from-scratch build (§28). Measure the magnitude first; it is the whole claim.
4. **`butteraugli` as an independent second metric** — breaks the single-metric self-reference
   (§30).
5. **`pulp` (or `archmage`) for SIMD inner loops** — `std::simd` is not coming (§36).
6. **A gain-map DETECTION fixture** — cheap; stops silent HDR destruction (§38).
7. **`quantette`** — the deferred indexed-PNG win (§37).
8. **Watch:** `viprs`/banded execution (§35), `zensim` at 0.3+ (§29), `ultrahdr-core` at 1.0 (§38).
9. **Decide against:** GPU/wgpu (§41), all AGPL Imazen encoders (§31), CV/feature-matching (§40),
   smart crop (Part 4 §19).

---

# Part 6 — Work catalogue from this session

Ranked by *(value × certainty) ÷ cost* from the exploring agent's point of view. **Priority is the
orchestrator's call** — this is a catalogue, not a plan. Sizes use the repo's S/M/L convention;
spec counts are rough. "Gates" means other work should not start until this resolves.

## Top 10

| # | Item | Why | Size | Notes |
|---|---|---|---|---|
| 1 | **Fix `convert` stripping EXIF Orientation without baking it**, then sweep every other re-encoding verb (`thumbnail`, `resize`, `responsive`, `edit` without `--auto-orient`) | Measured defect with a positive control (§17a). Orientation 6 is the ordinary phone case, so real photos come out sideways on a verb whose contract is a lossless-intent format change | S–M | Design call: bake vs preserve. Sweep must be mechanical, not by reading. Regression fixture must assert **dimensions**, not tag absence |
| 2 | **Measure the gamma-space resize penalty** with SSIMULACRA2 (before/after, plus a negative control) | The cheapest decision in this document. It **gates** items 5/10/16 and the whole linear-light claim — and if the win isn't measurable it *cancels* a stage. libvips #1144/#2238 show a naive implementation makes things **worse** | S | Do this before any linear-light work is scheduled. Magnitude is currently unmeasured (§26) |
| 3 | **Swap `ssimulacra2` → `fast-ssim2`** | Same BSD-2 licence, computes *real* SSIMULACRA2, matching entry point, **3× faster at 1080p**, adds wasm32 SIMD128 + bounded-memory strip path + cooperative cancellation. Incumbent hasn't shipped in **19 months** and pins a stale `yuvxyb` (§28) | M | Gate on an independently-seeded parity fixture asserting values against the current crate — not on the fork's claim |
| 4 | **RAW: carry EXIF forward + read orientation from container IFD0** | Fixes §17b (100% EXIF loss, 92 tags → 0), §17d (orientation never read), and corrects the planned fix in §17c — the preview has **no EXIF at all**, so DEC-055's "thread the preview's APP1" follow-up cannot work | M | The unlock for the only differentiated RAW workflow. Amends DEC-055 |
| 5 | **Use `moxcms` for ICC handling** | **Already in the tree** non-optionally via `image` 0.25.10. BSD-3/Apache, production, 1 open issue. Zero new dependency, zero new licence surface (§28). Fixes the Display-P3-renders-oversaturated class | M | Verify gamut mapping specifically — not in its README. Needs a DEC per `no-new-top-level-deps-without-decision` if promoted to a direct dep |
| 6 | **Verify AVIF byte-determinism across thread counts** | `aomenc`/`vpxenc` ship `-D, --debug` *to become* deterministic; rav1e has **no guarantee** and a closed nondeterminism bug (#2781) (§33). If crustyimg's AVIF isn't deterministic, existing "reproducible" language is a **false claim** | S | Gates item 18. Either pin on decoded pixels / perceptual score, or force single-thread and measure |
| 7 | **Make `build` able to run bundled recipes** | Live shipped defect, self-documented in DEC-070 point 4: `prepare_target` omits the terminal-`optimize` strip, so a manifest target bound to `web`/`product`/`gallery` fails `UnknownOperation { name: "optimize" }` | S–M | Subsumed if item 11 lands first — the `[output]` table removes the positional hack entirely |
| 8 | **Correct two factual claims in the docs** | (a) `docs/data-model.md`'s recipe example advertises `unsharp`, `watermark`, `clean-gps` — **none exist** (§1), and it's the first thing anyone evaluating "can it run my pipeline?" reads. (b) "Squoosh is archived" is **wrong** — `archived: false`, 25.5k stars, ~2M visits/mo, maintainer retains commit access; *squoosh-**cli*** is what's deprecated (§24) | S | Both are one-click-checkable credibility items |
| 9 | **Add a gain-map (UltraHDR) detection fixture** | Every recent flagship phone emits gain-map JPEGs. Silently discarding the gain map **destroys HDR the user can see on their own display** — same class as item 1 (§38) | S | Detection only. `ultrahdr-core` is Apache-2.0 and wasm-capable but self-described pre-1.0 experimental — don't integrate yet |
| 10 | **L1: give the pixel buffer a declared state** (precision, transfer function, alpha association) + an `Operation` capability hint so the pipeline converts **once** | The foundational unlock. Today every op opens with `to_rgba8()`, so N tonal ops = N 8-bit round-trips, and nothing records what the numbers *mean* (§8). Independently justified by resize quality and the AVIF bit-depth truncation | **L** (~1 stage, 6–10 specs) | **Start at u16, not f32**: 64 Mpix × 16 B/px = 1 GiB against a 512 MiB single-alloc cap (DEC-063). f32 at full res requires item 15. Gated by item 2. Steal `fovea`'s typestate idea; don't depend on the crate (§40) |

## Next 10

| # | Item | Why | Size | Notes |
|---|---|---|---|---|
| 11 | **Recipe schema v2: a top-level `[output]` table**, + per-op param-name validation, + the wasm `validate()` with structured errors | The schema can't express format/quality/budget/metadata, so no useful preset can be written (§12). Steps **cannot** carry `deny_unknown_fields`; the accepted-risk note justifying that ("never a wrong output") stops holding once steps carry quality | M–L | Land the validator in the same stage — error quality *is* the feature. Bump `version` to `"2"`, keep accepting terminal `optimize` under `"1"` for one release |
| 12 | **L3: metadata as a typed model** — ops repair the tag they invalidate; stripping becomes a **sink policy** | `auto-orient` discards all 92 tags because *one* went stale (§17b). The machinery already exists and is hardened: `clean_gps` does selective surgery preserving Orientation byte-exactly | M | `web` keeps strip-all by default. Moving strip to a sink policy trades a structural privacy guarantee for a configured one — must be default-deny |
| 13 | **Extend the metadata lane to AVIF + WebP** | `strip_all`/`clean_gps`/`set_tags` match only `Lane::Jpeg | Lane::Png`. So on the flagship RAW → `web` → **AVIF** path, metadata can't be written at all — item 4's value can't reach the default output | M | Prerequisite for any metadata-bearing preset on the AVIF path |
| 14 | **Adopt `butteraugli` as an independent second metric** | Every quality claim currently flows through **one** metric implementation. A second from a different lineage, validated against a different reference (<0.001% vs libjxl), is the independent check (§30) | M | BSD-3. Discount for bus-factor-1 and the AI-authorship notice (§32) |
| 15 | **De-risk the sidecar crop**: one controlled Lightroom crop on a **portrait** RAW; and obtain a portrait RAW to settle §17d | The pre-rotation reference frame is documented **three ways but never falsifiably observed** (§16.5). Five minutes of experiment gates a whole feature. The same sample settles whether the RAW orientation bug is real | S | Must precede any crop-from-sidecar code |
| 16 | **Evaluate `pic-scale` as the resize backend** | Ships built-in **linear / Lab / Luv / Oklab** resize — gamma-correct downscale without building it (§28) | M | Its "faster than `fast_image_resize`" claim is **stale and withdrawn**; measure yourself. Bus factor 1; AVX-512 needs nightly. Also bump the `fast_image_resize` pin — 6.1.0 is out |
| 17 | **`docs/migrating.md` translation table** — Squoosh flags / imgproxy params / Lightroom **export** settings → the equivalent crustyimg command | Captures most of the practical value of an importer at zero code cost, is honest about the 0% develop-preset overlap, and doubles as the table any future importer needs (§6) | S | The narrow answer to the original question |
| 18 | **Positioning corrections** | Stop leading on **reproducibility** (demand near-imagined; every SSG already caches; the one standalone product got 50 stars and was archived — §23) and on **privacy** (~20:1 traffic against; Show HN scores of 1–2 points; **zero** legal/medical/DLP artifacts found — §24). Reposition perceptual targeting as *free, cross-format, offline/in-browser* — "nobody does perceptual targeting" is already false (§21) | S | Sell recipes+cache+lockfile as **speed and control**. Gated by item 6 for any determinism language |
| 19 | **C2PA detection lint rule** — "this input carries Content Credentials; this operation will invalidate them" | Verified: optimization **does** destroy credentials, and the manifest lives in JPEG **APP11**, so every APP1-walking tool drops it silently. No free tool notices. Signing is blocked (Trust-List cert; a self-signed manifest is worse than useless) (§22) | S–M | Detection only. May not need the full `c2pa` crate, which pulls 90+ deps |
| 20 | **Dependency housekeeping cluster** | `pulp` **or** `archmage` for SIMD hot loops (`std::simd` is confirmed **not coming** — §36); `quantette` for the deferred indexed-PNG win (§37); `imagesize` for a non-allocating hostile-input preflight; add the AGPL Imazen encoders (`zenjpeg`, `zenwebp`, `zenquant`, `jxl-encoder`, `jpegli-rs`) to the licence watchlist with "revisit if relicensed" (§31) | M | `pulp` is the safest single pick (16.6M dl, real wasm simd128, no constraint conflict). `archmage` uniquely keeps `forbid(unsafe_code)` and arrives transitively with item 3 |

## Explicit DON'Ts (recorded so they aren't re-litigated)

- **Lightroom develop-preset import** — measured overlap **zero of ~71 settings** (§2).
- **Matching Lightroom's develop rendering** — parameters unpublished, processing adaptive, and
  darktable's own manual says it "will never give identical results" (§10).
- **Smart crop** — libvips/sharp `attention` is already a free built-in heuristic, so the
  dependency wedge doesn't exist; it's the exact algorithm whose failures are the incumbents' top
  complaint, and the no-ML identity blocks the escape hatch (§19).
- **GPU/wgpu** — WebGL2 fallback has no compute shaders → a permanently divergent second path
  (§41). ⚠️ This verdict is **inference; the browser-support research was lost.** Re-run before
  recording a DEC.
- **Region-adaptive encoding** — rav1e exposes no per-block quantizer/ROI API (§33).
- **CV / feature matching (SIFT/ORB/optical flow)** — makes crustyimg a computer-vision library,
  not an optimizer; and `yscv-imgproc` ships **SURF** with live patent exposure and no feature gate
  (§40).
- **Any AGPL/LGPL dependency on the default path** — the whole Rust RAW-parsing ecosystem, `dssim`,
  `ranga` (GPL), `purecv` (LGPL, inconsistent metadata) (§13, §31, §40).

## WATCH (re-check, don't build)

- **`viprs` / banded execution** — real code, 24 downloads, 0 stars, bot's last commit. Also the
  precondition for f32 at full resolution. `image`'s 1.0 milestone (#2300 `rows()`, #2357
  parallelism) is the signal to track (§35).
- **`zensim`** at 0.3+/1.0 — a *different* trained metric, not SSIMULACRA2; adopting it means
  re-baselining every published number (§29).
- **`ultrahdr-core`** at 1.0 — for full gain-map support beyond item 9's detection (§38).
- **`rav1e`** — zero commits in 6 months; your AVIF encode path runs through a stalled upstream
  (§33).
- **JPEG XL** — decode is mature (`jxl-oxide`); the only permissive pure-Rust *encoder* is a
  lossless POC. ~12–17% global support → archival/progressive-enhancement, not delivery (§39).
