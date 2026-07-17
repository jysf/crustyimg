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
| `4` | Unsupported format / codec not built — an ENCODER (AVIF output without `--features avif`) or a DECODER (a `.heic` input without `--features heic`, SPEC-062/DEC-052). The message names the feature to rebuild with. |
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
`--no-snippet`). Resizes **by target width**, preserving aspect, **never upscaling**
(widths above the source width are skipped with a warning; variants dedupe by actual
width). `--formats` defaults to the input's format; a feature-gated unbuilt codec
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
**Metadata** (EXIF/ICC/orientation) is **dropped** on the resize re-encode —
the pixel lane does not carry container metadata; that is the STAGE-004
container lane (DEC-003). **Batch failures:** a multi-input batch with any
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
— i.e. `resize --fill NxN`. Multi-input + `--out-dir` for batch (SEQUENTIAL, no
parallelism until STAGE-005). **Output format** defaults to **preserving the
input's source format** (`--format` / `-o` extension override; DEC-015);
**metadata is dropped** on the re-encode (pixel lane; DEC-003). **Batch
failures:** any per-input failure writes the successes, prints a per-file summary
to stderr, and exits **6**; a single-input failure keeps its natural code
(3/1/4/5). `-q/--quality` is not honored (encoder default); `--size 0` → exit 2.

#### `web <INPUT...> [--max N]`  *(the flagship; SPEC-085)*
Make an image web-ready in one step: bake EXIF orientation + strip metadata →
**downscale** the long edge to a web-friendly default (**2048**, aspect preserved,
never upscaled; `--max N` overrides) → the **fast AVIF-aware decision** (below) that
picks the smallest modern format beating the source and **never ships a larger file**
→ **report the winner's SSIMULACRA2 score**. Size-insensitive (a 24 MP photo finishes
as fast as a small one because it downscales first). Equivalent to `apply --recipe
web`. `-o`/`--format` pin the output format (bypassing the auto-decision + score).
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
Re-encode to another core format (JPEG/PNG/GIF/BMP/TIFF/ICO/WebP) — a **pure
re-encode** (decode once, no pixel transform). `--format` is **required**
(omitted → exit **2**, clap) and **forces** the output format for every input,
overriding both the DEC-015 source-preserve default and any `-o <path>`
extension (precedence: `--format` > `-o` ext > preserve source; here `--format`
is always present, so it wins). `-q/--quality` is threaded to the encoder where
the format supports it (JPEG; **ignored** for lossless formats — DEC-016); unlike
`optimize`, `convert` forces **no** default quality (encoder default unless `-q`).
**Metadata is dropped** on the re-encode (pixel lane; DEC-003). Multi-input
`--out-dir` fan-out (sequential; output names take the target `{ext}`); a
per-input **load/write** failure writes the successes, prints a per-file summary
to stderr, and exits **6** (DEC-015); a single-input failure keeps its natural
code (3/1/5); multi-input without `--out-dir` → exit **2**; missing input → exit
**3**. An **unsupported or unbuilt target codec** → exit **4** (DEC-004) —
resolved **once up front**, so even a multi-input convert to an unbuilt codec is a
single exit 4, **not** a partial-batch exit 6. **AVIF** output is the off-by-default
`avif` feature (SPEC-018, DEC-020): a `--features avif` build encodes `--format
avif` (and `-o x.avif`) — pure-Rust via `ravif`, no system deps — while the default
build keeps AVIF output at exit 4 with a "rebuild with --features avif" hint. **AVIF
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
format: a lossy target (**JPEG**, **AVIF** `--features avif`, **WebP**
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

#### `watermark <INPUT...> --image LOGO [--gravity G] [--opacity O] [--scale S] [--margin M] [--tile]`  *(SPEC-029)*
Overlay an image watermark (`--image`, required) onto each base at a compass
**gravity** anchor (default `southeast`; `center`/`north`/…/`southwest`). A
pixel-lane `Operation` (DEC-002) — the first that composes a second image, loaded
once at the CLI boundary (DEC-031). `--opacity O` (0–1, default 1) scales the
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

#### `edit <INPUT> [--auto-orient] [--resize-max N] [--invert] [-o OUT | --out-dir DIR] [--format FMT] [-q Q] [-y] [--save-recipe FILE]`  *(SPEC-032)*
One-shot multi-op on a single image — the "experiment like an editor" mode.
The op flags build an ordered operation list (v1: `--auto-orient`,
`--resize-max N`, `--invert` — only ops that round-trip through the registry,
DEC-005). **At least one op flag is required** (else exit 2). Regardless of the
order the flags are typed, ops apply in a fixed **canonical order: `auto-orient`
→ `resize` → `invert`** (orientation → geometry → color), so the result — and
any saved recipe — is deterministic. Output, format, `-q`/`-y` behave as for the
other pixel commands (`-o`/`-o -`/`--out-dir`; `--format` › `-o` ext › preserve).
`--save-recipe FILE` serializes the exact op chain to a TOML recipe (DEC-005,
`version = "1"`) that `apply --recipe FILE` replays identically; a recipe write
failure exits 5. Watermark/compose ops are not in `edit` yet (need registry
wiring first, DEC-031).

#### `apply --recipe FILE <INPUT...> [--out-dir DIR] [--name-template T] [-j N]`  *(SPEC-031)*
Run a saved recipe over one image or a batch. **`rayon`-parallel** across inputs
(`-j N` bounds workers, DEC-006) with an **`indicatif`** progress bar on stderr
(DEC-033; suppressed by `--quiet`). Recipe load reuses SPEC-006 validation (bad
`version` / unknown op → exit 1; recipe file unreadable → exit 3). Single input →
`-o`/`--out-dir`/stdout as before; **multiple inputs require `--out-dir`** (else exit
2) and write name-templated outputs (`{stem}.{ext}`, `--name-template` honored). A
per-input failure is summarized on stderr and exits **6** (others still written). The
proof of the thesis: the same recipe tuned on one image runs unchanged across many.
(`Operation` is not `Send`, so each task rebuilds its pipeline from the recipe +
registry — no async, DEC-006.)

#### `build [FILE]`  *(SPEC-063)*
Run every `[[target]]` in a declared build manifest (default `./crustyimg.build.toml`;
`version = 1`, DEC-057). A target binds `source` (a glob/dir/path or a list) × `recipe`
(a recipe file) → `out` (a directory, auto-created) + optional `name` template
(default `{stem}.{ext}`). Manifest paths resolve against the working directory.

Two phases: **every** target is validated first — recipe parsed + pipeline probed,
sources resolved — so a bad target aborts the build before any output is written; then
each target's inputs fan out over the same rayon path as `apply` (`-j N` bounds workers;
`--quiet` suppresses progress + summary). A per-output failure is reported on stderr and
exits **6** (others still written, DEC-015); a summary of targets run + outputs written
goes to stderr on success.

Unlike `apply`, `build` **overwrites its own declared outputs without `--yes`** — a build
owns its `out` tree and must be re-runnable (DEC-057); the sink still refuses
name-template escapes and symlinked destinations, so writes stay inside `out`.

Exit codes: malformed manifest (bad TOML, unknown field, unsupported `version`, oversize,
invalid target) → **2**; manifest or recipe file unreadable → **3**; invalid recipe
(unknown op/params) → **1**; missing source / empty glob → **3** (invalid glob pattern →
2); per-output failure → **6**. Manifest resource limits mirror recipes (DEC-036):
64 KiB size cap checked before read *and* before parse, 1024-target cap.

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
