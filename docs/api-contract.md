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
`-q/--quality` is not yet honored by `resize` (encoder default); quality-aware
encode is the `shrink`/`convert` story.

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

#### `shrink <INPUT...> [--max N] [-q Q]`  *(S3)*
Optimize-for-web: resize (default max long edge) + real quality encode +
strip metadata (respecting `--keep-gps`). The headline web-prep command.

#### `convert <INPUT...> --format FMT [-q Q]`  *(S3)*
Re-encode to another core format (JPEG/PNG/GIF/BMP/TIFF/ICO). WebP is
fast-follow; AVIF is feature-gated (exit 4 if not built — DEC-004).

#### `auto-orient <INPUT...>`  *(S3)*
Apply EXIF orientation to pixels, then clear the orientation tag. Fixes the
most common silent rotation bug.

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
