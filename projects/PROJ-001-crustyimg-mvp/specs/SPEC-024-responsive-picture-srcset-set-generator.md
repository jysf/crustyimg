---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-024
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it) — M, upper end

project:
  id: PROJ-001
  stage: STAGE-009
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build cycle (or orchestrator-direct main loop)
  created_at: 2026-06-18

references:
  decisions: [DEC-026, DEC-015, DEC-008, DEC-016, DEC-004, DEC-002]
  constraints:
    - ergonomic-defaults
    - clippy-fmt-clean
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - decode-once-no-per-op-disk
    - untrusted-input-hardening
  related_specs: [SPEC-011, SPEC-014, SPEC-022]

value_link: "Delivers STAGE-009's web-delivery surface — the responsive <picture>/srcset set a web developer actually ships, generated in one command from the modern-format engine."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-024: responsive — <picture>/srcset set generator

## Context

STAGE-009 turned the modern-format engine into user-facing commands: `optimize`
(one button) and `diff` (verification). This spec delivers the **web-delivery**
surface — the artifact a web developer actually ships: a set of width-scaled
variants (optionally in multiple formats) **plus a paste-ready `<picture>`/srcset
HTML snippet**. No incumbent CLI produces this cleanly in one command; it is a
direct competitive differentiator (the "format auto-negotiation in HTML" gap).

It is mostly composition — the resize op (DEC-008), the per-format sink
(`encode_to_bytes`, DEC-016/004), and the decode-once pipeline (DEC-002) — plus
one genuinely new, dependency-free piece: **HTML emission**. Settling "is emitting
HTML in scope?" (roadmap open question #3 — recommended yes, opt-out) is part of
this spec's decision (DEC-026).

## Goal

Add `crustyimg responsive <input> --widths <list> [--formats <list>] --out-dir
<dir>` that writes one width-scaled variant per (width × format) into `<dir>` as
`{stem}-{width}w.{ext}` and prints a paste-ready `<picture>`/srcset snippet to
stdout — never upscaling, reusing the resize op and per-format sink, with no new
dependency.

## Inputs

- **Files to read:** `src/cli/mod.rs` — `Commands`, `dispatch`, `run_convert`
  (the up-front `ensure_codec_built` + forced-format pattern), `run_pixel_op`,
  `shrink_params`, `resolve_format`/`output_format_for`, `escape_json` (string
  helpers), the `GlobalArgs`. `src/sink/mod.rs` — `extension_for_format`,
  `ensure_codec_built`, `safe_join`, `Sink::File`, `encode_to_bytes`. `src/operation/mod.rs`
  — the `resize` op's **`fit` mode** (`fit Wx{BIG}` scales to width W, preserving
  aspect, **no upscale** via the `.min(1.0)` clamp). `src/image/mod.rs`
  (`Image::load`, `pixels`, `width`/`height`, `source_format`).
- **Decisions:** DEC-026 (this spec's command + HTML-emission DEC — author during
  build), DEC-008 (resize backend), DEC-015 (format precedence), DEC-016 (quality),
  DEC-004 (feature-gated codecs), DEC-002 (decode-once).
- **Tests to mirror:** `tests/cli.rs` (`Command::new(BIN)`, `write_bytes`, temp
  dirs, `image::open` dim checks, `convert_to_avif_*`/`convert_unbuilt_codec_*` for
  the feature-gate pattern). Fixtures: `common::detailed_jpeg`, `common::detailed_png`.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — add `Commands::Responsive { input, widths, formats, out_dir,
    quality?, no_snippet }`, a `dispatch` arm, `run_responsive`, helpers
    (`parse_widths`, `parse_formats`, `mime_for_format`, `build_picture_html`),
    and unit tests.
  - `decisions/DEC-026-responsive-command-and-html-emission.md` — the new decision.
  - `tests/cli.rs` — `responsive_*` integration tests + add `responsive` to the two
    subcommand lists.
  - `docs/api-contract.md` — a `responsive` command entry.
- **New exports:** none outside `src/cli`.
- **No new dependency. No database changes.**

## Command surface (PINNED)

```
crustyimg responsive <input>
    --widths <W1,W2,…>      # required; comma-separated target WIDTHS in px
    --out-dir <DIR>         # required; created if missing
    [--formats <F1,F2,…>]   # optional; default = the input's source format
    [--no-snippet]          # suppress the <picture> snippet on stdout
    # global: -q <0-100> | --quiet | --yes
```

- **`--widths`** — comma list of positive integers (the srcset target widths).
  Whitespace-tolerant; empty/zero/non-integer → usage error (exit 2).
- **`--out-dir <DIR>`** — required (this command always fans out). Created with
  `create_dir_all` if missing. Single positional `<input>` only (no glob/batch in v1).
- **`--formats`** — comma list of format names (`jpeg`/`jpg`, `png`, `webp`,
  `avif`, …) resolved via `resolve_format`. Default: the **input's source format**.
  Each requested format is checked up front with `ensure_codec_built` — an unbuilt
  feature-gated codec (e.g. `avif` without `--features avif`) is a single **exit 4**
  before any file is written (mirrors `convert`, DEC-004).
- **`-q`** sets the lossy encode quality (JPEG, lossy WebP-with-feature); default
  **80** for lossy formats when omitted; ignored for lossless formats (DEC-016).
- The output format **always** comes from `--formats` (or the source) — `--format`
  global and `-o` are NOT used by this command.

### Variant generation (PINNED)

- For each requested width `W` with `W ≤ source_width`: resize the **once-decoded**
  image to width `W` via the resize `fit` mode (`mode=fit, width=W, height=<BIG>`
  where BIG is large enough that width binds, e.g. `1_000_000`), preserving aspect,
  **never upscaling**. Read the variant's **actual** output width for the srcset
  descriptor and filename.
- **No upscaling:** a width `W > source_width` is **skipped** with a stderr warning
  (unless `--quiet`). Variants are **deduped by actual width** (two requested widths
  that clamp to the same actual width yield one file). If *every* requested width is
  `> source_width`, that is a usage error (exit 2) naming the source width.
- For each surviving width × each requested format: encode via `encode_to_bytes`
  and write `{stem}-{actual_width}w.{ext}` into `<out-dir>` (path built with
  `safe_join`; overwrite guarded by `--yes` like other sinks).

### HTML snippet (PINNED)

Printed to **stdout** unless `--no-snippet` (diagnostics stay on stderr):

- **One format** → a bare `<img>`:
  ```html
  <img srcset="stem-320w.jpg 320w, stem-640w.jpg 640w" src="stem-640w.jpg" width="640" height="427" alt="">
  ```
- **Multiple formats** → `<picture>` with one `<source>` per format (in the given
  order — modern formats first by convention) + an `<img>` fallback in the **last**
  format at the **largest** width:
  ```html
  <picture>
    <source type="image/webp" srcset="stem-320w.webp 320w, stem-640w.webp 640w">
    <source type="image/jpeg" srcset="stem-320w.jpg 320w, stem-640w.jpg 640w">
    <img src="stem-640w.jpg" width="640" height="427" alt="">
  </picture>
  ```
- `width`/`height` on the `<img>` are the fallback variant's actual pixel dimensions
  (reduces layout shift). `alt=""` is a placeholder for the user to fill. No `sizes`
  attribute (browsers default width-descriptor srcset to `100vw`; leaving it lets the
  user set their own).

## Acceptance Criteria

- [ ] `responsive in.jpg --widths 320,640 --out-dir d/` writes `d/in-320w.jpg` and
  `d/in-640w.jpg`, each a valid JPEG with the stated width (aspect preserved), and
  prints an `<img srcset=…>` snippet to stdout listing both with `w` descriptors.
- [ ] `--formats webp,jpeg` writes both `*-{w}w.webp` and `*-{w}w.jpg` per width and
  prints a `<picture>` with a `<source type="image/webp">`, a `<source
  type="image/jpeg">`, and an `<img>` fallback in jpeg at the largest width.
- [ ] A requested width **greater than** the source width is skipped (not upscaled),
  with a stderr warning; the file is absent. Two widths clamping to the same actual
  width produce one file (dedup).
- [ ] If every requested width exceeds the source width → exit **2** (usage).
- [ ] `--formats avif` without `--features avif` → exit **4** before writing anything.
- [ ] `--no-snippet` suppresses the stdout HTML (files still written, exit 0).
- [ ] `--out-dir` is created if missing.
- [ ] Malformed `--widths` (empty, `0`, non-integer) → exit **2**.
- [ ] `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test`
  pass; `cargo deny check licenses` green (no new dep).

## Failing Tests

Written during **design**, BEFORE build.

### Unit tests — `src/cli/mod.rs` (`#[cfg(test)] mod tests`)

- **`responsive_parses_args`** — `Cli::try_parse_from(["crustyimg","responsive",
  "in.jpg","--widths","320,640","--out-dir","d","--formats","webp,jpeg"])` →
  `Commands::Responsive` with `input == "in.jpg"`, `widths == "320,640"`,
  `out_dir == "d"`, `formats == Some("webp,jpeg")`.
- **`parse_widths_ok_and_dedup_sorted`** — `parse_widths("640, 320,640")` →
  `Ok(vec![320, 640])` (trimmed, sorted, deduped). (Pin the exact ordering/dedup the
  impl chooses; sorted-ascending + deduped is recommended.)
- **`parse_widths_rejects_junk`** — `""`, `"0"`, `"abc"`, `"320,0"`, `"-5"` each
  return `Err` with `code() == 2`.
- **`parse_formats_defaults_and_resolves`** — `parse_formats(None, ImageFormat::Jpeg)`
  → `[Jpeg]`; `parse_formats(Some("webp,jpeg"), _)` → `[WebP, Jpeg]`; an unknown
  format string → `Err` (code 4 or 2 — pin which; `SinkError::UnsupportedExtension`
  via `resolve_format` → 4 is consistent with `convert`).
- **`mime_for_format_maps_core`** — `mime_for_format(Jpeg) == "image/jpeg"`,
  `Png == "image/png"`, `WebP == "image/webp"`, `Avif == "image/avif"`.
- **`build_picture_html_single_vs_multi`** — given variant rows (format, width,
  filename, fallback dims), one-format input produces a bare `<img srcset=…>` (no
  `<picture>`/`<source>`); multi-format produces `<picture>` with one `<source
  type="image/…">` per format and an `<img>` fallback. Assert the substrings
  (`srcset=`, `320w`, `type="image/webp"`, the fallback `src=`/`width=`).

### Integration tests — `tests/cli.rs`

- **`responsive_writes_width_variants`** — `detailed_jpeg(800,600)`, `--widths
  320,640 --out-dir d/`: assert exit 0, `d/in-320w.jpg` and `d/in-640w.jpg` exist
  and decode with widths 320 and 640; stdout contains `srcset=` with `320w` and `640w`.
- **`responsive_multi_format_emits_picture`** — `--formats webp,jpeg` on a 800px
  input: both `.webp` and `.jpg` variants per width exist; stdout contains
  `<picture>`, `type="image/webp"`, `type="image/jpeg"`, and an `<img` fallback.
- **`responsive_no_upscale_skips_wide`** — `detailed_jpeg(400,300)`, `--widths
  320,800`: only `in-320w.jpg` exists, `in-800w.jpg` is absent, stderr warns about
  skipping 800.
- **`responsive_all_widths_exceed_source_exits_2`** — `detailed_jpeg(200,150)`,
  `--widths 320,640` → exit 2.
- **`responsive_avif_without_feature_exits_4`** — `--formats avif` (default build)
  → exit 4, and no files written. *(Gate with `#[cfg(not(feature = "avif"))]` like
  the existing `convert_avif_without_feature_exits_4`.)*
- **`responsive_no_snippet_suppresses_html`** — `--no-snippet`: exit 0, files
  written, stdout does NOT contain `srcset`.
- **`responsive_creates_out_dir`** — point `--out-dir` at a not-yet-existing nested
  path; assert exit 0 and the variants exist there.
- **`responsive_malformed_widths_exits_2`** — `--widths 0` (and `--widths abc`) → exit 2.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- **DEC-026 (NEW — author it)** — the `responsive` command shape: width×format
  variant generation, the **fit-by-width / no-upscale / dedup-by-actual-width**
  rule, **HTML `<picture>`/srcset emission to stdout** (opt-out `--no-snippet`),
  default lossy quality 80, auto-create `--out-dir`, and the deferrals (blurhash/
  thumbhash placeholder, perceptual/`--max-size` per-variant, glob/batch input, a
  `sizes` attribute). `affected_scope`: `src/cli/mod.rs`; confidence ~0.8.
- **DEC-008** — resize backend (`fast_image_resize`, Lanczos3) used via the `fit` op.
- **DEC-015** — output-format resolution precedence; here the format comes only from
  `--formats`/source (no `-o`/global `--format`).
- **DEC-016** — quality policy: `-q` for lossy formats, ignored for lossless.
- **DEC-004** — feature-gated codecs: `ensure_codec_built` per format up front →
  single exit 4 (no partial output).
- **DEC-002** — decode-once: load the input ONCE, then resize/encode each variant in
  memory (no per-variant re-decode).

### Constraints that apply

- `decode-once-no-per-op-disk` — load once; the variant loop must not re-load.
- `untrusted-input-hardening` — write variant paths via `safe_join` (the `{stem}`
  is separator-free per SPEC-004, but use `safe_join` for the traversal guard
  anyway); cap is the resize op's existing oversize guard.
- `ergonomic-defaults` — sensible defaults (source format, q80 lossy, snippet on,
  out-dir auto-created); the common case is `responsive img.jpg --widths … --out-dir dist/`.
- `clippy-fmt-clean` (run `cargo fmt` then `git add -u`), `no-unwrap-on-recoverable-paths`,
  `every-public-fn-tested`.

### Prior related work

- `SPEC-011` (shipped) — `resize` CLI + the multi-input fan-out (`run_pixel_op`);
  reuse the resize-op construction + format/sink plumbing patterns.
- `SPEC-014` (shipped) — `convert`'s up-front `ensure_codec_built` + forced-format
  pattern to copy for the per-format feature check.
- `SPEC-022` (shipped) — the prior STAGE-009 command structure to mirror.

### Out of scope (for this spec specifically)

- **blurhash/thumbhash placeholder** — a rider that needs a new dependency + its own
  DEC; deferred follow-up.
- **Perceptual / `--max-size` per-variant** quality — v1 uses fixed `-q`/format
  defaults; reusing `resolve_effective_quality` per variant is a follow-up.
- **Glob / directory / multi-input** — single input in v1 (fan-out is over
  widths×formats, not inputs).
- **A `sizes` attribute / art-direction `<source media=…>`** — emit width-descriptor
  srcset only; the user adds `sizes` for their layout.
- **AVIF/WebP behavior beyond what the format already supports** — same gating as
  `convert` (DEC-004/020/022).

## Notes for the Implementer

- **Decode once, then loop.** Load `Image::load(input)` once. For each surviving
  width, build a `resize` op (`mode=fit`, `width=W`, `height=1_000_000`) via the
  registry, run it on a clone of the loaded image, and read `out.width()`/`height()`
  for the descriptor + filename. For each format, `encode_to_bytes(&out, fmt,
  quality)` and write via a `Sink::File { path: safe_join(out_dir, &name)?, format:
  Some(fmt) }` (or write the bytes directly after `safe_join`). `create_dir_all`
  the out-dir first (so `safe_join`'s canonicalize succeeds).
- **fit-by-width:** `fit W×1_000_000` → scale `s = min(W/w, 1_000_000/h).min(1.0)` ⇒
  for `w ≥ W`, width = W exactly; for `w < W`, `s = 1.0` (no upscale). So compute the
  surviving set as the requested widths `≤ source_width`, dedup by the **actual**
  resulting width.
- **`parse_widths`** → `Result<Vec<u32>, CliError>`: split on `,`, trim, parse u32,
  reject empty/0, sort ascending, dedup. **`parse_formats`** →
  `Result<Vec<ImageFormat>, CliError>`: `None` ⇒ `vec![source_format]`; else split,
  `resolve_format` each (an unknown ext → its `SinkError` → exit 4), preserve order.
  Run `ensure_codec_built` on each up front.
- **`mime_for_format`** — small match: Jpeg→`image/jpeg`, Png→`image/png`,
  WebP→`image/webp`, Avif→`image/avif`, Gif→`image/gif`, fallback
  `image/{extension_for_format}`.
- **`build_picture_html`** — pure fn over the variant table; return a `String`.
  Single format ⇒ `<img srcset>`; multi ⇒ `<picture>`. Keep it pure + unit-tested.
- **Quality default:** for a lossy format with no `-q`, use `DEFAULT_SHRINK_QUALITY`
  (80); pass `None` for lossless formats so `encode_to_bytes` takes the lossless path.
- **Warnings** go to stderr and are suppressed by `--quiet` (skipped-width, etc.).
- **Confirm every named failing test exists** before claiming green.
- **Cost:** append a build session to `cost.sessions` (real `tokens_total`, or a
  labeled estimate if main-loop), per AGENTS.md §4.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-026` — responsive command + HTML emission (scope + deferrals)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
