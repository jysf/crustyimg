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
| `1` | Generic runtime error (decode/encode/op failed). |
| `2` | Usage error (bad args) — clap's standard code. |
| `3` | Input not found / unreadable. |
| `4` | Unsupported format / codec not built (e.g. AVIF without the feature). |
| `5` | Output write failed / refused (exists without `--yes`, traversal). |
| `6` | Partial batch failure (some inputs failed; summary on stderr). |

The library returns typed `thiserror` errors; `main` maps them to these
codes and prints a friendly `anyhow`-formatted message to stderr (DEC-007).

## Subcommand Surface (full MVP)

`(Sx)` marks the stage that delivers each command.

### Inspect / view

#### `view <INPUT> [--width N] [--height N]`  *(S2; smoke stub in S1)*
Display an image in the terminal via `viuer`. Requires a tty — a non-tty
stdout refuses with exit code **5** (`SinkError::NotATty`). Optional sizing
fits to terminal by default. Resolves the first input when given a
directory/glob (single-image command).

#### `info <INPUT> [--exif] [--json]`  *(S2)*
Print dimensions, format, **file size on disk** (bytes), color type, bit
depth, alpha, and ICC/EXIF presence. `--exif` dumps EXIF tags read-only (via
`kamadak-exif`, DEC-013); an image with no EXIF reports "no EXIF" and exits 0
(not an error). `--json` emits machine-readable JSON to **stdout** (all
diagnostics on stderr, so `info --json | jq` stays clean). Single-image
command: resolves the first input on a directory/glob. "byte size" is the
**encoded file size on disk**, not the decoded in-memory pixel buffer length
(the latter, if surfaced, is a distinct `decoded_bytes` field).

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
(the encoder default unless `-q` is given). `shrink` is the command with a
quality default.

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

#### `shrink <INPUT...> [--max N] [-q Q] [--target visually-lossless|high|medium | --ssim 0-100]`  *(S3; `--target`/`--ssim` S8/SPEC-016)*
**Perceptual auto-quality** (SPEC-016, DEC-019): `--target
<visually-lossless|high|medium>` / `--ssim <0-100>` auto-tune the **JPEG** encode
quality to a perceptual **SSIMULACRA2** target — the command binary-searches the
**lowest** quality whose decoded round-trip scores at/above the target (capped at
8 in-memory candidate evaluations; the original is still decoded once, DEC-002).
The presets map to SSIMULACRA2 scores (visually-lossless ≈ 90, high ≈ 70, medium ≈
50; tunable). `--target`, `--ssim`, and `-q` are **mutually exclusive** (you either
pin a quality or search for one → exit **2** if combined; `--ssim` outside 0–100 →
exit **2**). It is **opt-in**: without `--target`/`--ssim`, `shrink` uses the fixed
default quality (80, below). For a **non-JPEG** output format the target is
**ignored** (encoder default), mirroring `-q` on lossless formats (DEC-016). If the
target is unreachable even at quality 100, `shrink` emits the highest-quality encode
(best-effort). A scoring failure (e.g. a pathologically tiny image) is a typed error
(single-input exit **1**; one input in a batch → exit **6**).

Optimize-for-web: resize to a default long-edge bound + a real quality-aware
encode + drop metadata. The headline web-prep command. `--max` defaults to
**1600** (long edge, aspect preserved, never upscaled); `-q/--quality` defaults
to **80** and maps to **JPEG** quality — it is **ignored for lossless formats**
(PNG/GIF/BMP/TIFF/ICO), which re-encode unchanged (DEC-016). Output **preserves
the input's source format** (`--format` / `-o` extension override; DEC-015) — a
JPEG stays JPEG, a PNG stays PNG. **Metadata is dropped** on the re-encode (the
pixel lane carries no container metadata); selective preservation and
`--keep-gps` are the STAGE-004 container lane and are **not yet active for
`shrink`** (DEC-003). Multi-input `--out-dir` fan-out (sequential; partial
failure → exit 6; missing input → 3; multi-input without `--out-dir` → 2).

#### `convert <INPUT...> --format FMT [-q Q]`  *(S3)*
Re-encode to another core format (JPEG/PNG/GIF/BMP/TIFF/ICO) — a **pure
re-encode** (decode once, no pixel transform). `--format` is **required**
(omitted → exit **2**, clap) and **forces** the output format for every input,
overriding both the DEC-015 source-preserve default and any `-o <path>`
extension (precedence: `--format` > `-o` ext > preserve source; here `--format`
is always present, so it wins). `-q/--quality` is threaded to the encoder where
the format supports it (JPEG; **ignored** for lossless formats — DEC-016); unlike
`shrink`, `convert` forces **no** default quality (encoder default unless `-q`).
**Metadata is dropped** on the re-encode (pixel lane; DEC-003). Multi-input
`--out-dir` fan-out (sequential; output names take the target `{ext}`); a
per-input **load/write** failure writes the successes, prints a per-file summary
to stderr, and exits **6** (DEC-015); a single-input failure keeps its natural
code (3/1/5); multi-input without `--out-dir` → exit **2**; missing input → exit
**3**. An **unsupported or unbuilt target codec** (e.g. AVIF without the feature,
or WebP which is fast-follow) → exit **4** (DEC-004) — resolved **once up front**,
so even a multi-input convert to an unbuilt codec is a single exit 4, **not** a
partial-batch exit 6. WebP output is fast-follow; AVIF is feature-gated.

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

#### `watermark <INPUT...> --image LOGO [--gravity G] [--opacity O] [--scale S] [--margin M] [--tile]`  *(S4)*
Overlay an image watermark at a gravity anchor. (Text watermark is a
trailing addition within S4.)

### Metadata lane *(container-level; no pixel decode — DEC-003)*

#### `strip <INPUT...>`  *(S4)*
Remove all metadata (EXIF/IPTC/XMP/ICC) at the container level.

#### `clean <INPUT...> --gps`  *(S4)*
Remove only GPS/location metadata; keep the rest. Privacy-focused.

#### `set <INPUT...> [--artist S] [--copyright S] [--description S]`  *(S4)*
Write specific EXIF tags (via `little_exif`), pixels untouched.

#### `copy-metadata --from SRC --to DST`  *(S4)*
Copy metadata from one image's container to another's. Pixels untouched.

### Recipes / batch

#### `edit <INPUT> [op flags...] [--save-recipe FILE] -o OUT`  *(S5)*
One-shot multi-op on a single image — the "experiment like an editor" mode.
Op flags (`--resize-max`, `--unsharp`, `--watermark`, …) build an ordered
operation list; `--save-recipe` writes that list as a recipe (DEC-005).

#### `apply --recipe FILE <INPUT...> [--out-dir DIR] [--name-template T] [-j N]`  *(S5)*
Run a saved recipe over one image or a batch. `rayon`-parallel across
inputs with an `indicatif` progress bar. The proof of the thesis: the same
recipe tuned on one image runs unchanged across many.

## Stage Map (summary)

| Stage | Commands |
|---|---|
| STAGE-001 | (no real commands) skeleton + dispatch + global args + smoke stub |
| STAGE-002 | `view`, `info` (+ `--exif`) |
| STAGE-003 | `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient` |
| STAGE-004 | `watermark`; `strip`, `clean --gps`, `set`, `copy-metadata` |
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
