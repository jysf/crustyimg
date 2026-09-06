# CLI Contract

> `crustyimg` has no network API. Its public contract is the **command-line
> interface**: the subcommand surface, global arguments, stdin/stdout
> behavior, and exit codes. Authored during PROJECT DESIGN (Prompt 2a) for
> the whole PROJ-001 MVP. Each subcommand notes the stage that delivers it.
> Until a subcommand's stage lands, the binary dispatches it as a stub that
> reports "not yet implemented" and exits non-zero.

## Overview

```
crustyimg [GLOBAL OPTIONS] <SUBCOMMAND> [INPUT...] [OPTIONS]
```

- Binary name: `crustyimg`.
- Built with `clap` derive, subcommand style (no boolean flag-soup — the
  prototype's mistake).
- `crustyimg --help` lists subcommands; `crustyimg <cmd> --help` shows that
  command's options; `crustyimg --version` prints the version.
- Inputs are positional. Most commands accept one path, a glob, a
  directory, or `-` (stdin). Batch-aware commands accept many.

## Global Options

Apply to all subcommands (parsed before/around the subcommand).

| Option | Short | Description |
|---|---|---|
| `--output <PATH>` | `-o` | Output file for single-input commands. `-` means stdout. |
| `--out-dir <DIR>` | | Output directory for multi-input/batch commands. |
| `--name-template <T>` | | Output name template, e.g. `{stem}_web.{ext}` (see data-model). |
| `--jobs <N>` | `-j` | Parallel workers for batch (rayon). Default = CPU count. Placeholder in STAGE-001; honored in STAGE-005. |
| `--format <FMT>` | | Force output format (else inferred from `-o` extension or kept). |
| `--quality <0-100>` | `-q` | Encoder quality where the format supports it (e.g. JPEG). |
| `--verbose` | `-v` | Increase verbosity (repeatable: `-vv`). Logs to stderr. |
| `--quiet` | `-Q` | Suppress non-error output. |
| `--yes` | `-y` | Assume "yes" to overwrite prompts (non-interactive). |
| `--keep-gps` | | Opt out of the default-drop-GPS policy on pixel-lane encodes. |
| `--version` / `--help` | `-V` / `-h` | Standard clap. |

### stdin / stdout (`-`)

- A positional input of `-` reads an encoded image from **stdin**.
- `-o -` writes the encoded result to **stdout**.
- When writing to stdout, all human-readable/log output goes to **stderr**
  so pipes stay clean: `crustyimg resize - --max 800 -o - < in.jpg > out.jpg`.
- `view` to a non-tty refuses (terminal display requires a tty); other
  commands work headless.

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Generic runtime error (decode/encode/op failed; includes an input that exceeds decode resource limits — see below). |
| `2` | Usage error (bad args) — clap's standard code. |
| `3` | Input not found / unreadable. |
| `4` | Unsupported or undetectable format. Three cases: the input bytes are not a recognisable image at all (zero-byte file, arbitrary text/bytes) — the message just says so, there is no feature to name; a DECODER is recognized but not built (a `.heic` input without `--features heic`, SPEC-062/DEC-052) — the message names the feature to rebuild with; or an ENCODER is not built (AVIF output on a `--no-default-features` lean build) — likewise names the feature. |
| `5` | Output write failed / refused (exists without `--yes`; name/path traversal; a symlinked destination, refused even with `--yes` — SPEC-034 / DEC-035). |
| `6` | Partial batch failure (some inputs failed; summary on stderr). |
| `7` | A check/gate was not satisfied (e.g. `diff --fail-under` scored below the threshold). Distinct from a runtime error so CI can tell "regression detected" from "couldn't run" (S9/SPEC-023, DEC-025). |

The library returns typed `thiserror` errors; `main` maps them to these
codes and prints a friendly `anyhow`-formatted message to stderr (DEC-007).

**Decode resource limits (SPEC-033 / DEC-034, SPEC-070 / DEC-063):** every command
that loads an image bounds the decode before pixels are produced:

- **per-dimension ≤ 65 535** (DEC-034)
- **decoded allocation ≤ 512 MiB** (DEC-034)
- **total pixels ≤ 64 Mpix** (67 108 864 px, ≈ 8192×8192 — [DEC-063](../decisions/DEC-063-peak-decode-memory-pixel-budget.md)),
  checked against the image's *declared* dimensions **before** the decode
  allocates. This is the peak-decode-memory bound: a 1 GiB budget over a 4×
  amplification factor on the RGBA output.

An input that exceeds any of these (a decompression bomb, forged dimensions) is
**rejected with a typed error and exit `1`** — never a panic or OOM.

The pixel cap has a stated tradeoff: it also rejects a legitimate **> 64 MP**
image (a 100 MP medium-format frame, a very large stitched panorama), which is
indistinguishable from a bomb by its header. Essentially all consumer and prosumer
photography (24 MP, 50 MP) fits. An opt-in `--max-pixels` override is **filed, not
built** — deliberately, since adding an escape hatch to a security bound deserves
its own spec; the revisit trigger is a real user with a > 64 MP workflow. See
DEC-063 for the derivation and the alternatives weighed.

**Recipe resource limits (SPEC-035 / DEC-036):** `apply --recipe` bounds an
untrusted recipe — a recipe text over **64 KiB** or with more than **1024 steps**
is rejected with a typed error (exit `1`), and an over-size recipe *file* is
refused before it is read into memory.

**Resize output limit (SPEC-037 / DEC-038):** a `resize` whose output buffer
would exceed **512 MiB** (≈ the same cap as decode) — an upscale bomb via
`exact`/`percent`/`cover`/`fill`, from a recipe or the CLI — is rejected with a
typed error (exit `1`) before allocation. (`max`/`fit` never upscale.)

**Truncated JPEG warning (SPEC-107 / DEC-085):** print a one-line
`warning: <input>: truncated JPEG: …` to stderr when a JPEG is missing its
trailing end-of-image marker (`FF D9`) — the `image` crate's JPEG decoder
tolerates this by design and decodes a (possibly incomplete) frame rather than
erroring, unlike PNG/AVIF. The command still **exits `0`** and still writes its
output: this is a warning, not a failure, and — unlike this CLI's other advisory
warnings — is **not** suppressed by `--quiet` (the whole point is that a
truncated JPEG must never again pass through unremarked).
Wired wherever a verb decodes pixels through the shared `run_pixel_op`/
`optimize_decide_one` seams — **not** an exhaustive-sounding short list:
`info`, `web`, `convert`, `resize`, `optimize`, `thumbnail`, `auto-orient`,
`edit`, `watermark` (its *primary* input only — the `--image` overlay loads
separately and does not warn), `apply --recipe <name>` when the recipe ends
in the terminal `optimize` step (e.g. the bundled `web` recipe), and `build`
on a target whose output format is auto-decided (`OutputFormatPlan::Decide`
— SPEC-116 finished threading the warning through this seam, the last one).
Verbs that decode pixels through a *different* seam do not warn yet: `diff`,
`responsive`, `apply`/`build` with a plain pixel recipe, and `view` — filed as
a follow-up candidate (SPEC-107 punch list), not fixed here.

**Animated-input warning (SPEC-119 / DEC-093):** print a one-line
`warning: <input>: animated input flattened to a single frame — …` to stderr
when the input is an animated GIF, APNG, or animated WebP — the pixel path
decodes exactly one frame, so every frame after the first is otherwise
discarded without comment. The command still **exits `0`** and still writes
the first frame: this is a warning, not a failure, and — like the truncated-
JPEG warning above — is **not** suppressed by `--quiet`. Wired at the
`run_pixel_op`/`optimize_decide_one` seams, but **not the identical set** the
truncated-JPEG warning above uses: `convert`, `optimize`, `web`, `resize`,
`thumbnail`, `auto-orient`, `edit`, `watermark` (its *primary* input only),
`apply --recipe <name>` when the recipe ends in the terminal `optimize` step,
and `build` on a target whose output format is auto-decided
(`OutputFormatPlan::Decide`). Verbs that decode pixels through a *different*
seam, or decode pixels directly without checking this flag, do not warn yet:
`info` (decodes via `Image::decode_path`/`from_bytes` directly, checks
`is_truncated_jpeg()` but not `is_animated_input()`), `diff`, `responsive`,
`apply`/`build` with a plain pixel recipe, and `view`. The strict path for a
caller that cannot tolerate a silent flatten is `lint --max-warnings 0`, which
fails on any of the three formats via `format/animated-gif` (GIF) or
`format/animated-input` (APNG, animated WebP) — see DEC-093. **Qualifier
(added 2026-08-16, SPEC-119 punch list):** this holds when `lint` is given the
file directly, or the bytes over stdin. It does **not** currently hold for
animated WebP in **directory-discovery mode** — `lint --max-warnings 0` on a
directory containing an animated `.webp` exits `0` (the file is silently
absent from the scan), while the identical bytes named directly
(`lint --max-warnings 0 <dir>/anim.webp`) exit `7`, because `webp` is absent
from `IMAGE_EXTENSIONS` (`src/source/mod.rs:105-113`) and directory/glob
discovery silently skips it. GIF and APNG are unaffected (both extensions are
present in the list). Filed as a `[S]` PRIORITY item on STAGE-042's backlog —
"the `IMAGE_EXTENSIONS` gap silently defeats the strict gate that a
maintainer decision rests on" — not fixed here.

## Subcommand Surface (full MVP)

`(Sx)` marks the stage that delivers each command.

### Inspect / view

#### `view <INPUT> [--width N] [--height N]`  *(S2; smoke stub in S1)*
Display an image in the terminal via `viuer`. The `display` feature is **on by
default** (DEC-027), so `view` works out of the box; a headless
`--no-default-features` build omits it and `view` then reports the rebuild hint
(exit 5). Requires a tty — a non-tty stdout refuses with exit code **5**
(`SinkError::NotATty`). Optional sizing fits to terminal by default. Resolves the
first input when given a directory/glob (single-image command).

#### `info <INPUT> [--exif] [--json]`  *(S2)*
Print dimensions, format, **file size on disk** (bytes), color type, bit
depth, alpha, and ICC/EXIF presence. `--exif` dumps EXIF tags read-only (via
`kamadak-exif`, DEC-013); an image with no EXIF reports "no EXIF" and exits 0
(not an error). `--json` emits machine-readable JSON to **stdout** (all
diagnostics on stderr, so `info --json | jq` stays clean). Single-image
command: resolves the first input on a directory/glob. "byte size" is the
**encoded file size on disk**, not the decoded in-memory pixel buffer length
(the latter, if surfaced, is a distinct `decoded_bytes` field).

#### `diff <A> <B> [--fail-under N] [--json]`  *(S9/SPEC-023; DEC-025)*
Print the **SSIMULACRA2** perceptual score of `<B>` relative to `<A>` (higher =
more similar, ~100 = visually identical; reuses the auto-quality metric, DEC-019).
`--fail-under <0-100>` turns it into a **CI visual-regression gate**: score below
`N` exits **7** (a distinct "check not satisfied" code), the score line still
printed to stdout. The two inputs must have **equal dimensions** (else exit **2**;
no implicit resize). `--json` emits `{"a","b","score","fail_under","passed"}` to
stdout. (v1 is score + gate only; a highlighted visual-diff heatmap image is a
deferred follow-up.)

#### `responsive <INPUT> --widths W1,W2,… --out-dir DIR [--formats F1,F2,…] [--no-snippet]`  *(S9/SPEC-024; DEC-026)*
Generate a responsive image set: one width-scaled variant per (width × format),
written as `{stem}-{width}w.{ext}` into `DIR` (created if missing), plus a
paste-ready **`<picture>`/srcset** snippet on **stdout** (suppress with
`--no-snippet`). Decodes once, then EXIF **orientation is baked into the pixels
first** (SPEC-110) — the source width every requested width is compared
against is measured AFTER baking, so a sideways source is bounded by its
visually-correct width, not its stored one (a 1200×800 source with
`Orientation=6` compares against 800, and `--widths 600` on it comes back
600×**900**, not a plain dimension swap). Resizes **by target width**,
preserving aspect, **never upscaling** (widths above the source width are
skipped with a warning; variants dedupe by actual width). `--formats`
defaults to the input's format; a feature-gated unbuilt codec
exits **4** up front (DEC-004). `-q` sets the lossy quality (default 80; ignored for
lossless). Single input (no glob/batch in v1). Reuses the resize op + per-format
sink; no new dependency. (blurhash placeholder, perceptual-per-variant, and a
`sizes` attribute are deferred.)

### Geometry / transform

#### `resize <INPUT...> --max N | --exact WxH | --percent P | --fit WxH | --fill WxH | --cover WxH`  *(S3)*
Resize using the SIMD backend (DEC-008). Mutually exclusive modes (exactly
one required; zero or two → exit **2**). Multi-input + `--out-dir` for batch
(SEQUENTIAL, no parallelism until STAGE-005). `--cover` scales to cover the
box (aspect kept, may upscale, no crop); `--fill` is `--cover` **then** a
center-crop to exactly the box (i.e. fill = cover + center-crop). `--max`/
`--fit` never upscale.

**Output format:** defaults to **preserving the input's source format** —
`resize a.jpg --max 800 --out-dir web/` writes `web/a.jpg`, not `web/a.png`.
`--format FMT` forces a format; an `-o <path>` extension also decides
(precedence: `--format` > `-o` extension > preserve source). (DEC-015.)
EXIF **orientation is baked into the pixels first** (SPEC-110), so the output
is never sideways even though the tag itself does not survive; `--max`/etc.
bound the visually-correct (post-bake) dimensions. **Metadata** (EXIF/ICC)
is otherwise **dropped** on the resize re-encode — the pixel lane does not
carry container metadata; that is the STAGE-004 container lane (DEC-003).
**Batch failures:** a multi-input batch with any
per-input failure writes the successes, prints a per-file summary to stderr,
and exits **6**; a single-input failure keeps its natural code (3/1/4/5).
`-q/--quality` is threaded to the encoder where the format supports it (JPEG;
ignored for lossless formats — DEC-016); `resize` forces no default quality
(the encoder default unless `-q` is given). `optimize`'s fast decision supplies its
own fixed quality (SPEC-084).

#### `thumbnail <INPUT...> [--size N] [--square]`  *(S3)*
Convenience resize to a small bounded size — a thin wrapper over `resize`.
`--size N` bounds the **longest edge to N** (aspect preserved, **never
upscaled**) — i.e. `resize --max N`. `--size` defaults to **256** when omitted.
`--square` makes the output **exactly N×N** by covering then **center-cropping**
— i.e. `resize --fill NxN`. EXIF **orientation is baked into the pixels
first** (SPEC-110), so `--size`/`--square` bound the visually-correct
dimensions. Multi-input + `--out-dir` for batch (SEQUENTIAL, no
parallelism until STAGE-005). **Output format** defaults to **preserving the
input's source format** (`--format` / `-o` extension override; DEC-015);
other **metadata is dropped** on the re-encode (pixel lane; DEC-003). **Batch
failures:** any per-input failure writes the successes, prints a per-file summary
to stderr, and exits **6**; a single-input failure keeps its natural code
(3/1/4/5). `-q/--quality` is not honored (encoder default); `--size 0` → exit 2.

#### `web <INPUT...> [--max N]`  *(the flagship; SPEC-085)*
Make an image web-ready in one step: bake EXIF orientation + strip metadata →
**downscale** the long edge to a web-friendly default (**2048**, aspect preserved,
never upscaled; `--max N` overrides) → the **fast AVIF-aware decision** (below) that
picks the smallest modern format beating the **downscaled** image → **report the
winner's SSIMULACRA2 score** — qualified when the winning format cannot hold the
source's real bit depth, since the score is computed at 8-bit either way and would
otherwise read a false-perfect match for a depth reduction it cannot see (`ssim 100.0
(8-bit comparison; source was 16-bit)`, plus an additive `ssim_source_depth` in `--json`
— SPEC-125, DEC-097). The downscale to a dimension bound is the contract, so an
already-small source **above** that bound can re-encode **larger than the original** —
reported honestly (`N% larger`, plus a `larger_than_source` flag in `--json`), never
hidden (SPEC-090, DEC-075). For an *unconditional* never-bigger guarantee that keeps
dimensions, use **`optimize`**. Size-insensitive (a 24 MP photo finishes as fast as a
small one because it downscales first). Equivalent to `apply --recipe web`. `-o`/`--format`
pin the output format (bypassing the auto-decision + score).
Multi-input `--out-dir` fan-out (sequential; partial failure → exit 6; missing input
→ 3; multi-input without `--out-dir` → 2).

#### `optimize <INPUT...> [--max N] [--verify] [-q Q] [--target visually-lossless|high|medium | --ssim 0-100 | --max-size SIZE]`  *(S3+; SPEC-084/086)*
The **keep-dimensions byte-primitive**. By DEFAULT (no flags) it runs the **fast
fixed-quality decision** (SPEC-084, DEC-069): auto-orient + strip metadata + a single
fixed-quality encode that picks the smallest modern format beating the source and
**never ships a larger file** — no perceptual search. Dimensions are **preserved**
(`--max N` optionally bounds the long edge). The default is lean and **score-free**
(scoring a full-resolution winner is too costly to run unconditionally); **`--verify`**
opts in to a single **SSIMULACRA2** readout for this run (reported on the summary and
in the JSON explain). For downscale-and-modernize, use **`web`**.

**Perceptual auto-quality** (SPEC-016, DEC-019 — opt-in): `--target
<visually-lossless|high|medium>` / `--ssim <0-100>` auto-tune the **JPEG** encode
quality to a perceptual **SSIMULACRA2** target — the command binary-searches the
**lowest** quality whose decoded round-trip scores at/above the target (capped at
8 in-memory candidate evaluations; the original is still decoded once, DEC-002).
The presets map to SSIMULACRA2 scores (visually-lossless ≈ 90, high ≈ 70, medium ≈
50; tunable). `--target`, `--ssim`, and `-q` are **mutually exclusive** (you either
pin a quality or search for one → exit **2** if combined; `--ssim` outside 0–100 →
exit **2**). For a **non-JPEG** output format the target is
**ignored** (encoder default), mirroring `-q` on lossless formats (DEC-016). If the
target is unreachable even at quality 100, `optimize` emits the highest-quality encode
(best-effort). A scoring failure (e.g. a pathologically tiny image) is a typed error
(single-input exit **1**; one input in a batch → exit **6**).

**Byte budget** (SPEC-017 + SPEC-021 — opt-in): `--max-size <SIZE>` (e.g. `200KB`,
`1.5MB`, `200000`, `64KiB`) fits the output under the budget. For a **lossy** target
(JPEG; AVIF/WebP with their features) it first auto-tunes the quality to the
**highest** quality whose encoded output is ≤ the budget (the perceptual search
inverted; capped, in-memory). Units are decimal (`KB`=1000, `MB`=1e6); `KiB`/`MiB`
are binary. Mutually exclusive with `--target`/`--ssim`/`-q` (combined → exit **2**;
a malformed size → exit **2**). **Dimension-reduction fallback (SPEC-021, DEC-023):**
when lowering quality alone cannot meet the budget — or for a **lossless** output
(PNG, lossless WebP, …) which has no quality knob — the output is **progressively
downscaled** until it fits; a downscale prints a `scaled to WxH` warning (unless
`--quiet`). So `--max-size` works for **every** output format and for very small
budgets; the result is the largest image that fits. If even the smallest size
doesn't fit, the best-effort smallest is written with a warning. A budget already met
at full size never resizes.

Output follows DEC-015 precedence (`--format` > `-o` ext > the auto-decision, unless
`--profile preserve` keeps the source format). **Metadata is dropped** on the pixel-lane
re-encode (privacy incl. GPS); selective preservation is the STAGE-004 container lane
(DEC-003), not active here. Multi-input `--out-dir` fan-out (sequential; partial failure
→ exit 6; missing input → 3; multi-input without `--out-dir` → 2).

#### `convert <INPUT...> --format FMT [-q Q]`  *(S3)*
Re-encode to another core format (JPEG/PNG/GIF/BMP/TIFF/ICO/WebP). EXIF
**orientation is baked into the pixels first** (SPEC-110) — no OTHER pixel
transform runs; for the overwhelming majority of inputs (orientation 1, or no
EXIF at all) that bake is a genuine no-op, so `convert` stays byte-identical
to a plain decode→re-encode for those. `--format` is **required**
(omitted → exit **2**, clap) and **forces** the output format for every input,
overriding both the DEC-015 source-preserve default and any `-o <path>`
extension (precedence: `--format` > `-o` ext > preserve source; here `--format`
is always present, so it wins). `-q/--quality` is threaded to the encoder where
the format supports it (JPEG; **ignored** for lossless formats — DEC-016); unlike
`optimize`, `convert` forces **no** default quality (encoder default unless `-q`).
Other **metadata is dropped** on the re-encode (pixel lane; DEC-003). Multi-input
`--out-dir` fan-out (sequential; output names take the target `{ext}`); a
per-input **load/write** failure writes the successes, prints a per-file summary
to stderr, and exits **6** (DEC-015); a single-input failure keeps its natural
code (3/1/5); multi-input without `--out-dir` → exit **2**; missing input → exit
**3**. An **unsupported or unbuilt target codec** → exit **4** (DEC-004) —
resolved **once up front**, so even a multi-input convert to an unbuilt codec is a
single exit 4, **not** a partial-batch exit 6. **AVIF** output is a **default**
`avif` feature (SPEC-018/DEC-020, moved into `default` by SPEC-102/DEC-081): a
plain build encodes `--format avif` (and `-o x.avif`) — pure-Rust via `ravif`, no
system deps — while a `--no-default-features` (lean) build keeps AVIF output at
exit 4 with a "rebuild with --features avif" hint. **AVIF
input (decode) is not supported** (output-only v1; reading an `.avif` fails). **WebP**
is a **pure-Rust DEFAULT format** (SPEC-019, DEC-021): `.webp` reads as INPUT (lossy +
lossless) everywhere, and `--format webp` / `-o x.webp` write **lossless** WebP
(smaller than PNG). In the DEFAULT build lossless WebP has no quality knob, so
`-q`/`--max-size`/`--target` are **ignored** for WebP output (like PNG, DEC-016). With
the off-by-default **`webp-lossy`** feature (libwebp, SPEC-020/DEC-022) WebP gains a
quality knob: a WebP output is encoded **lossy** when a quality is set — an explicit
`-q`, or one chosen by `--max-size`/`--target`/`--ssim` — and stays **lossless** for a
bare `convert --format webp`. (Because the WebP decoder ships by default, BOTH the
byte-budget AND the perceptual searches drive WebP — the AVIF contrast.) `--max-size
<SIZE>` (SPEC-017 + SPEC-021) fits the output under a byte budget for **every**
format: a lossy target (**JPEG**, **AVIF** (default), **WebP**
`--features webp-lossy`) lowers quality first, and any target — lossy that still
overflows, or a **lossless** one (PNG, lossless WebP) — then **downscales dimensions**
until it fits (DEC-023), warning `scaled to WxH` (unless `--quiet`). Mutually
exclusive with `-q` → exit 2; see `optimize` for the size-unit and best-effort
semantics. (The perceptual `--target`/`--ssim` auto-quality is `optimize`-only and, for
AVIF, falls back to the encoder default with a warning because it needs an AVIF
decoder — use `--max-size` for an AVIF byte budget.)

#### `auto-orient <INPUT...>`  *(S3)*
Apply the EXIF orientation to pixels, then clear the tag — fixes the most common
silent rotation bug (a portrait photo stored sideways with an Orientation tag).
A new recipe-usable `Operation` (`auto-orient`) that **reads** the EXIF
orientation captured at load (DEC-003/DEC-017) and bakes the corresponding
rotation/flip into the pixels via the `image` crate's native `Orientation`; the
pixel-lane re-encode then drops the (now-satisfied) tag inherently. An image with
**no EXIF, no orientation tag, or orientation 1** is a **no-op** (exit 0, not an
error). Output **preserves the input's source format** (`--format` / `-o`
extension override; DEC-015); other metadata is dropped on the re-encode (pixel
lane; DEC-003). Multi-input `--out-dir` fan-out (sequential; partial failure →
exit 6; missing input → 3; multi-input without `--out-dir` → 2). Capture
currently covers JPEG/PNG; for formats without EXIF capture `auto-orient` is a
safe no-op.

### Compositing

#### `watermark <INPUT...> --image LOGO [--gravity G] [--opacity O] [--scale S] [--margin M] [--tile]`  *(SPEC-029; orientation baking SPEC-110)*
Overlay an image watermark (`--image`, required) onto each base at a compass
**gravity** anchor (default `southeast`; `center`/`north`/…/`southwest`). A
pixel-lane `Operation` (DEC-002) — the first that composes a second image, loaded
once at the CLI boundary (DEC-031). The base image's EXIF **orientation is baked
into the pixels first** (SPEC-110), matching every other pixel-lane verb, so the
overlay composites onto the display-correct orientation, not the stored one.
`--opacity O` (0–1, default 1) scales the
overlay alpha; `--scale S` resizes the overlay to `S ×` base width; `--margin M`
insets the anchor; `--tile` repeats the overlay across the whole base (ignores
gravity/margin). Missing/unreadable `--image` → exit **3**; bad opacity/scale or
unknown gravity → exit **2**. Standard fan-out (single → stdout/`-o`/`--out-dir`,
multi → `--out-dir`, per-input failure → exit 6). **Not recipe-round-trippable until
STAGE-005** (DEC-031).

**Text mode (SPEC-030, DEC-032):** `watermark <INPUT...> --text STRING [--font PATH]
[--size N] [--color HEX] [--gravity G] [--opacity O] [--margin M]` rasterizes the
text (via `ab_glyph`) into an overlay composited through the same path. `--image` and
`--text` are mutually exclusive — exactly one required (neither/both → exit **2**).
Default font is the **bundled BSD-3 Go font**; `--font PATH` (a TTF/OTF) overrides it
(missing/unreadable → exit **3**). `--size` (px, default 32; `≤0` → exit 2);
`--color` (`RRGGBB`/`#RRGGBB`/`RRGGBBAA`, default white; malformed → exit 2). No
`imageproc` (it pulls sdl2/nalgebra) — DEC-032.

### Metadata lane *(container-level; no pixel decode — DEC-003)*

#### `meta strip <INPUT...>`  *(SPEC-026; grouped under `meta` in SPEC-087)*
Remove **all** container metadata (EXIF/IPTC/XMP/ICC) via `img-parts`
segment/chunk removal — no pixel re-decode (decoded pixels byte-identical).
**v1 covers JPEG + PNG**; any other format → exit **4**. Fan-out mirrors the
pixel ops (DEC-015): single input → stdout / `-o` / `--out-dir`; multiple inputs
require `--out-dir`; a per-input failure in a batch → exit **6**; overwrite refused
without `-y`. Format is preserved (`-q`/`--format` ignored). A no-metadata input is
a clean no-op (exit 0).

#### `meta clean <INPUT...> --gps`  *(SPEC-026; grouped under `meta` in SPEC-087)*
Remove **only** GPS/location metadata via in-house TIFF-IFD tag removal (DEC-046),
preserving everything else (orientation, copyright, ICC) — privacy-focused, no pixel
re-decode. Every untargeted tag round-trips **byte-identically**, for every TIFF type
and for **both** input byte orders (`II` and `MM`) — the block is re-emitted in the
byte order it arrived in (SPEC-093/DEC-076; before that fix, a big-endian input's
Orientation `6` silently became `1536`). **`--gps` is required in v1**
(omitted → exit **2**). Same JPEG+PNG coverage, fan-out, and exit codes as
`meta strip`. A JPEG with no EXIF is a no-op (exit 0).

#### `meta set <INPUT...> [--artist S] [--copyright S] [--description S]`  *(SPEC-027; grouped under `meta` in SPEC-089)*
Write the named EXIF tags (Artist/Copyright/ImageDescription) via the in-house
TIFF-IFD writer (DEC-046), **preserving all other metadata and the pixels** (no
re-decode) — including numeric tags (Orientation, GPS) in either byte order,
byte-identically (SPEC-093/DEC-076). At least one tag
flag is required (none → exit **2**). **v1 covers JPEG + PNG**; other formats →
exit **4**. Writing overwrites an existing same-tag value and creates a fresh EXIF
block when the input has none. Same fan-out + exit codes as `meta strip`/`meta clean`
(reuses the container lane; single → stdout/`-o`/`--out-dir`, multi → `--out-dir`,
per-input failure → exit **6**, overwrite refused without `-y`).

#### `meta copy --from SRC --to DST [-o OUT] [-y]`  *(SPEC-028; grouped under `meta` in SPEC-087)*
Copy SRC's container **EXIF + ICC** onto DST, preserving DST's pixels exactly (no
re-decode); DST's prior EXIF/ICC are replaced by SRC's. **JPEG only in v1**
(DEC-030 — `little_exif`/`img-parts` use incompatible PNG EXIF chunks); a non-JPEG
`--from`/`--to` → exit **4**. Output: `-o PATH`/`-o -` writes the result there
(DST untouched); with no `-o` it writes **back to DST in place**, which (as an
overwrite) is refused without `-y` (exit **5**). Single fixed output — not a
fan-out; XMP/IPTC not transferred.

### Recipes / batch

#### `edit <INPUT> [--auto-orient] [--resize-max N] [--invert] [-o OUT | --out-dir DIR] [--format FMT] [-q Q] [-y] [--save-recipe FILE]`  *(SPEC-032; orientation baking SPEC-110)*
One-shot multi-op on a single image — the "experiment like an editor" mode.
Every invocation **bakes EXIF orientation first** (SPEC-110), before any of the
flag-driven ops run — so `edit --invert` and `edit --resize-max N` are never
sideways, matching every other pixel-lane verb. `--auto-orient` is now an
**accepted no-op**: since baking is unconditional, the flag changes nothing
about the output whether it is passed or not (kept because the CLI surface was
frozen in STAGE-030; no opt-out flag exists). The remaining op flags build an
ordered operation list (v1: `--resize-max N`, `--invert` — only ops that
round-trip through the registry, DEC-005). **At least one op flag is required**
(else exit 2) — `--auto-orient` alone still satisfies this and exits 0.
Regardless of the order the flags are typed, ops apply in a fixed **canonical
order: `auto-orient` → `resize` → `invert`** (orientation → geometry → color),
so the result — and any saved recipe — is deterministic. Output, format,
`-q`/`-y` behave as for the other pixel commands (`-o`/`-o -`/`--out-dir`;
`--format` › `-o` ext › preserve). `--save-recipe FILE` serializes the exact op
chain to a TOML recipe (DEC-005, `version = "1"`) that `apply --recipe FILE`
replays; a recipe write failure exits 5. The saved recipe **names `auto-orient`
explicitly as its first step** (SPEC-111), whether or not `--auto-orient` was
passed — matching what the CLI prefix always bakes, and how the bundled
`recipes/web.toml` already writes it, so a saved recipe stays a complete,
reproducible description of what `edit` did. (Before SPEC-111 the CLI-level
bake was NOT recorded as a step, so `edit --invert`'s output and its own
replayed recipe disagreed — a divergence SPEC-110 introduced, DEC-086; closed
here.) Watermark/compose ops are not in `edit` yet (need registry wiring
first, DEC-031).

#### `apply --recipe NAME_OR_FILE <INPUT...> [--out-dir DIR] [--name-template T] [-j N]`  *(SPEC-031; bundled recipes SPEC-085)*
Run a saved recipe over one image or a batch. `--recipe` resolves a real file on disk
first, falling back to a **bundled name** (`web`/`gallery`/`product`, SPEC-085) only
when no such file exists. **`rayon`-parallel** across inputs (`-j N` bounds workers,
DEC-006) with an **`indicatif`** progress bar on stderr (DEC-033; suppressed by
`--quiet`). Recipe load reuses SPEC-006 validation (bad `version` / unknown op → exit
1; recipe file unreadable → exit 3). Single input → `-o`/`--out-dir`/stdout as before;
**multiple inputs require `--out-dir`** (else exit 2) and write name-templated outputs
(`{stem}.{ext}`, `--name-template` honored). A per-input failure is summarized on
stderr and exits **6** (others still written). The proof of the thesis: the same
recipe tuned on one image runs unchanged across many. (`Operation` is not `Send`, so
each task rebuilds its pipeline from the recipe + registry — no async, DEC-006.)

**Output format (SPEC-126, DEC-015/DEC-098; the `recipe.format` rung is SPEC-127, Call 2).** For a
plain pixel recipe, `apply` resolves its output format the way `resize`, `thumbnail` and
`watermark` do, identically **at every arity**: `--format` > a recognized `-o` extension (single
input only — the fan-out path has no `-o`) > **the recipe's own `format` field, if set** >
**preserve the source format**. A literal extension in `--name-template` does **not** pin the
format; it names the file only. Before SPEC-126 the two arities disagreed in both directions —
one input with no `--format` wrote PNG whatever the source was, and multiple inputs ignored
`--format` entirely. Quality follows the same shape: `-q` > **the recipe's own `quality` field, if
set** > the format's own default.

⚠ **Three single-input invocations changed exit code in SPEC-126, from `4` to `0`:** `-o -` with
no `--format`, `-o` with a path carrying no extension, and `-o` with an unrecognised extension.
All three now preserve the source format and succeed — which is what `resize` and `thumbnail`
already did on the same invocations. The old `4` was outside this document's own enumeration of
that code (above): none of its three cases is "the output format could not be inferred from the
invocation".

A recipe ending in the reserved terminal `optimize` step (every bundled recipe) is not
a registry op: it is stripped before `build_pipeline`, and the preceding pixel steps
are run through the same fast AVIF-aware decision `web` uses instead of a plain
format-preserving write — so `apply --recipe web` == the `web` verb. A pinned format
(`--format`, a recognized `-o` extension, **or the recipe's own `format` field** — SPEC-127)
skips the decision and honors the pin instead (`apply --recipe web hero.jpg -o hero.png` writes a
real PNG, not AVIF-in-a-`.png`). A recipe that both ends in `optimize` and declares `format` is
asking for two contradictory things (the auto-decision AND a pin); the explicit field wins and the
decision is skipped, matching what `--format`/`-o` already do on this path.

**`Recipe.format` / `Recipe.quality` (SPEC-127).** A recipe may declare its own output `format`
(a string, resolved the same way `--format` is) and/or `quality` (0-100). Both are gated behind
`version = "2"`: a `version = "1"` recipe that sets either is rejected with a typed error naming
the field and the declared version — never a generic TOML parse failure — and `version = "1"`
stays valid, unchanged, and is still what a recipe using neither field serializes as. See
`data-model.md`'s Recipe Schema for the full field table.

#### `build [FILE]`  *(SPEC-063; bundled/terminal-`optimize` recipes SPEC-111)*
Run every `[[target]]` in a declared build manifest (default `./crustyimg.build.toml`;
`version = 1`, DEC-057). A target binds `source` (a glob/dir/path or a list) × `recipe`
(a file path or a **bundled name** — `web`/`gallery`/`product`) → `out` (a directory,
auto-created) + optional `name` template (default `{stem}.{ext}`). Manifest paths
resolve against the working directory.

Two phases: **every** target is validated first — recipe parsed, a terminal `optimize`
step stripped and its format PLAN resolved (below), pipeline probed, sources resolved
— so a bad target aborts the build before any output is written; then each target's
inputs fan out over the same rayon path as `apply` (`-j N` bounds workers; `--quiet`
suppresses progress + summary). A per-output failure is reported on stderr and exits
**6** (others still written, DEC-015); a summary of targets run + outputs written goes
to stderr on success.

**Format plan (SPEC-111; the `recipe.format` rung is SPEC-127, Call 2):** a target whose recipe
ends in the reserved terminal `optimize` step chooses its output format the same way `apply`
does — one rule, not two. The target's `name` template is the pin: a template naming a **literal
extension** (`name = "{stem}.png"`) pins that format and skips the decision, matching
`apply --recipe web -o hero.png`; a template using **`{ext}`** (including the default
`{stem}.{ext}`) lets the fast AVIF-aware decision choose per input, matching
`apply --recipe web` — **unless the recipe itself declares `format`**, in which case that pins
too (`build` has no `--format`/`-o` of its own — DEC-098 — so the template and the recipe's
`format` are its only two ways to pin; the template wins when both are present). A plain pixel
recipe (no terminal `optimize`) is likewise pinned by its own `format`, if set; otherwise `build`
preserves each input's own source format, as it always has. **Quality:** the global `-q` (if
given) applies uniformly to every target in the build, exactly as before; a target's
`recipe.quality`, if set, applies only when `-q` is absent — so it is the only way to give one
target a different quality than another in the same build.

Unlike `apply`, `build` **overwrites its own declared outputs without `--yes`** — a build
owns its `out` tree and must be re-runnable (DEC-057); the sink still refuses
name-template escapes and symlinked destinations, so writes stay inside `out`. The
committed lockfile and the content-addressed cache both record the **real, decided**
output extension (never the unexpanded `{ext}` template token), so a cache hit
materializes to the exact path the miss that filled it wrote.

Exit codes: malformed manifest (bad TOML, unknown field, unsupported `version`, oversize,
invalid target) → **2**; manifest or recipe file unreadable → **3**; invalid recipe
(unknown op/params) → **1**; a `name` template that pins a literal extension `build`
cannot resolve to a real image format — either an extension that is not a recognized
format (`{stem}.txt`) or **no extension at all** (`{stem}`, which is not the `{ext}`
sentinel) → **4** (same family as an unsupported `--format`/`-o` extension elsewhere);
missing source / empty glob → **3** (invalid glob pattern → 2); per-output failure →
**6**. Manifest resource limits mirror recipes (DEC-036): 64 KiB size cap checked
before read *and* before parse, 1024-target cap.

## Stage Map (summary)

| Stage | Commands |
|---|---|
| STAGE-001 | (no real commands) skeleton + dispatch + global args + smoke stub |
| STAGE-002 | `view`, `info` (+ `--exif`) |
| STAGE-003 | `resize`, `thumbnail`, `convert`, `auto-orient` (also `shrink`, removed in SPEC-086 → `web`/`optimize`) |
| STAGE-004 | `watermark`; the metadata quartet (regrouped under `meta`: `meta strip`, `meta clean --gps`, `meta copy` in SPEC-087, `meta set` in SPEC-089) |
| STAGE-005 | `edit` (+ `--save-recipe`), `apply --recipe` (parallel + progress) |

## Error Output Shape

Human-readable to stderr, e.g.:

```
error: failed to decode `photos/broken.jpg`
  caused by: invalid JPEG marker at offset 0x4f1
```

`info --json` and any future `--json` flags emit structured output to
stdout; everything diagnostic stays on stderr so `-o -` pipes are clean.

## References

- Architecture: `./architecture.md`
- Data model / recipe schema: `./data-model.md`
- Decisions: `/decisions/` (DEC-004 codec policy, DEC-005 recipe, DEC-007 errors)
- Feature research: `./feature-exploration.md`
