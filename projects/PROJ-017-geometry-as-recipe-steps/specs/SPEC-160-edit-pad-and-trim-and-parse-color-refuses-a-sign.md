---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-160
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
  created_at: 2026-11-11

references:
  decisions: [DEC-015, DEC-095, DEC-102, DEC-106, DEC-112, DEC-114, DEC-115]
  constraints: []
  related_specs: [SPEC-151, SPEC-152, SPEC-153, SPEC-159]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-058's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-058's last promise, `edit` flags for `pad` and `trim` that `--save-recipe` writes and `apply` replays, and closes the stage."

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
      date: 2026-11-11
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 5656151
      tokens_breakdown:
        input: 86
        cache_creation: 152401
        cache_read: 5459360
        output: 44304
      estimated_usd: 4.79
      note: >
        Measured from the session JSONL, deduped by message.id (43 messages; max output_tokens per id),
        after the frame push and before this cost commit. Rates (Opus): $5/MTok input, $6.25 cache_creation
        (x1.25), $0.50 cache_read (x0.10), $25 output.
    - cycle: design
      date: 2026-11-11
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 19975874
      tokens_breakdown:
        input: 188
        cache_creation: 280411
        cache_read: 19578340
        output: 116935
      estimated_usd: 14.47
      note: >
        Measured from the session JSONL (found by content), deduped by message.id (94 messages; max
        output_tokens per id), no subagents, after the design push and before this cost commit. Rates
        (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.50 cache_read (x0.10), $25 output.
        Includes 22 prototype mutation builds and test runs.
  totals:
    tokens_total: 25632025
    estimated_usd: 19.26
    session_count: 2
---

# SPEC-160: edit --pad and --trim, and parse_color refuses a sign

## Context

STAGE-058 promises five lossless geometry ops as recipe steps **and** as `edit` flags
(`delivers`: "`edit` flags for all five, with a written ordering rule, and `--save-recipe` that
writes them"). SPEC-153 gave `edit` `--crop`, `--rotate` and `--flip` under DEC-114's typed-order
rule. `pad` (SPEC-151, with SPEC-159's transparent fill and DEC-115's alpha rule) and `trim`
(SPEC-152) exist as recipe steps only. This is the STAGE-058 entry split out of SPEC-153 at its
frame (2026-11-10), reserved SPEC-160 on 2026-11-10, and the last spec in PROJ-017's STAGE-058.

Two more items are ruled into it: the STAGE-058 entry "**kind: claim. `parse_color` accepts a
sign, so `+f+f+f` is read as `#0f0f0f`**" (ruled 2026-11-10: folds in), and the stale "single
image" wording on three sites, filed at SPEC-153's verify.

## Goal

Today a CLI user who wants a border or a trim must write a recipe by hand: `crustyimg edit in.png
--pad 10 --pad-color red` and `edit in.png --trim` both exit 2 ("unexpected argument").
When this ships, `edit` takes `--pad N|W:H` (with `--pad-color`, `--pad-gravity`) and `--trim`
(with `--trim-color`, `--trim-tolerance`) in typed order, `--save-recipe` writes steps that `apply`
replays to the same pixels, and a hex colour with a sign in it is refused on every route instead
of loading as a different colour.

## Frame

*Framed 2026-11-11 (Opus 5.5) on `main` at `bf7fd03`, release build of 0.8.2 in a scratch
`CARGO_TARGET_DIR` (deleted after). Every "driven" line below was run in that session; probes lived
in the session scratchpad and are not committed.*

### Verdict: GO at [M], one spec, no new DEC

One spec, sized **[M]** (the backlog said [S]; see "Sizing"). No DEC: DEC-114 parts 1–6 already
name `--pad`, `--trim` and their modifiers, and part 6 already says "a flag that takes pixels needs a
single input image". DEC-115 needs a ship-time **wording follow-up** on part 4 (see "The alpha
rule on `edit`"), not an amendment. No other STAGE-058 entry folded (question 10 goes to the
maintainer).

### Premise check (confirmed)

On `main` (0.8.2):
```
edit in.png --pad 10 --pad-color red -o o.png  -> exit 2: error: unexpected argument '--pad' found
edit in.png --trim -o o.png                    -> exit 2: error: unexpected argument '--trim' found
edit --trim in.png -o o.png                    -> exit 2: error: unexpected argument '--trim' found
```
`pad` and `trim` run from a recipe (`apply`), so the gap is the flags only.

### Syntax (recommended)

**One `--pad` taking either form**, as `--crop` takes `WxH+X+Y|W:H`: a `:` makes it an aspect,
otherwise it is `N`, whole pixels. Why one flag and not two (`--pad-border`/`--pad-aspect`):
it matches `--crop` (the user learns one shape), DEC-114 part 3's "each op flag at most once"
then means "one pad step" with no `conflicts_with` to keep in step, and `pad`'s own
`aspect`/`border` exclusivity is enforced by the form, not by a second clap rule.

```
--pad N|W:H            N: whole pixels >= 1, added on every side; saved as border = N/long
                       W:H: grow the canvas to that aspect; saved as aspect = "W:H"
--pad-color COLOR      required with --pad; "transparent" or hex RRGGBB/#RRGGBB/RRGGBBAA (any alpha)
--pad-gravity G        aspect form only; nine names; default center; written only when given
--trim                 plain switch; colour defaults to the top-left pixel
--trim-color COLOR     optional; same parser as --pad-color; written only when given
--trim-tolerance N     optional; u8 0..=255 (8-bit steps, x257 at 16 bits); written only when given
```
clap attributes, driven in a scratch crate on the repo's clap 4.6.1: `--pad` `requires =
"pad_color"`; `--pad-color` and `--pad-gravity` `requires = "pad"`; `--trim` a `bool` switch;
`--trim-color`, `--trim-tolerance` `requires = "trim"`; `--trim-tolerance: Option<u8>`. `--pad`
does **not** set `allow_negative_numbers` (so `--pad -5` is clap's "unexpected argument").
`edit_op_order`'s `known` table gains `("pad", Pad)` and `("trim", Trim)`; the typed order was
driven: `--rotate 90 --pad 10 --pad-color red --trim` gives `[rotate, pad, trim]`, `--trim --pad …`
gives `[trim, pad]`, and `--trim --rotate 90 in.png` (switch before the input path) gives
`[trim, rotate]` with the input intact.

**Every parse-time refusal, exit 2, before any file is read** (driven = on the clap probe; the
`edit`-parser rows are recommended wording, since the parser doesn't exist yet):
- `--pad 10` alone (driven): clap `the following required arguments were not provided: --pad-color <COLOR>`.
- `--pad-color red` or `--pad-gravity north` without `--pad` (driven for `--pad-color`): clap, names `--pad <N|W:H>`.
- `--trim-color red` / `--trim-tolerance 8` without `--trim` (driven for `--trim-color`): clap, names `--trim`.
- `--pad … --pad …` (driven): `the argument '--pad <N|W:H>' cannot be used multiple times`; `--trim --trim` (driven): `the argument '--trim' cannot be used multiple times`. Same for each modifier.
- `--trim-tolerance 256` (driven): `invalid value '256' for '--trim-tolerance <N>': 256 is not in 0..=255`. `--trim-tolerance -1` (driven): `unexpected argument '-1' found`. `--trim-tolerance x`: clap's invalid-digit line.
- `--pad -5` (driven): `unexpected argument '-5' found`.
- `--pad=-5`, `--pad 0`, `--pad 10px`, `--pad 99999999999` (driven: `--pad=-5` and `--pad 0` **pass clap**, so `edit`'s parser must refuse them; reuse `crop_number`: digits only, fits `u32`): `--pad must be N, a whole number of pixels of at least 1, or an aspect W:H, such as 4:5; got "-5"`.
- `--pad 10 --pad-gravity north`: `--pad-gravity only places an aspect --pad W:H; a border --pad N adds the same width on every side` (mirrors the `--crop-gravity` refusal at `src/cli/ops.rs` `build_edit_ops`).
- A bad aspect (`--pad 0:5`, `--pad 4:`), a bad gravity, a bad `--pad-color` or `--trim-color`: the op's own load message with `'aspect'`→`--pad`, `'gravity'`→`--pad-gravity`, `'color'`→`--pad-color`/`--trim-color` through `build_flag_op`'s key swap (DEC-106). E.g. `--pad-color must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "zzzzzz"`.
- The "at least one operation flag" message lists `--pad` and `--trim` too.

### Pixels to a fraction: the bound, driven

`--pad N` resolves when its step runs, like `PixelCrop`: `border = N as f64 / long as f64` of the
image the step **receives** (after `--resize-max`, `--rotate`, `--crop`, `--trim` typed before it),
written untouched; `pad` then computes `(border * long).round()`.

**Driven** (a scratch Rust program, the same `f64` operations): every `long` in 1..=50,000 × every
`N` in 1..=50,000, **2,500,000,000 pairs: 0 mismatches**. The product is not bit-exact in 235,986,718
pairs (9.4 %; e.g. `1/49*49`), worst absolute error **7.28e-12 px**, so `round` absorbs it with a
margin of ~0.5 px. A sampled shortest-repr print/parse of the fraction (every 997th `long`) read
back bit-identical. `N` past 50,000 can't pass the canvas cap anyway (`long + 2N <= 50,000`).

Consequences:
- After `--resize-max 1600`, `--pad 20` is 20 px of the 1600 px image, and the saved
  `border = 20/1600` replays as 20 px on that image.
- `--rotate 90` doesn't change the long edge, and a border pad is uniform, so **`--pad N --rotate 90`
  and `--rotate 90 --pad N` give byte-identical output** (driven via `apply` recipes on a 6×4 RGB:
  both 8×10, `cmp` equal). An order test built on border pad + quarter turn can't fail. The order
  rows must use the aspect form (driven: `[pad 4:5, rotate 90]` → 8×6, `[rotate 90, pad 4:5]` → 5×6)
  or `trim` (driven: `[pad red, trim]` gives back the 6×4 original; `[trim, pad red]` gives 10×8).
- The canvas cap is checked by the wrapper **in pixels** before conversion, naming `--pad` and the
  image size (DEC-114 part 5), e.g. `--pad 30000 on this 4000×3000 px image needs a 64000×63000 px
  canvas, which exceeds the maximum edge limit of 50000 px`. Exit 1, or that input fails (exit 6).

### `--save-recipe` on a batch

**Refuse `--pad N` with `--save-recipe` on a batch, exactly as a pixel `--crop`** (exit 2, before
any image is read). The edit run itself puts N px on every image (each resolves against its own
long edge), but the recipe can hold only one fraction, and it would replay as N px only on images
with that long edge. That is DEC-114 part 6 as written ("a flag that takes pixels needs a single
input image"), so **no DEC change**; only `run_edit`'s `has_pixel_crop` check and message widen:
`--save-recipe with a pixel --crop or --pad N needs one input image, and "dir/" names 3: …; use an
aspect --crop W:H or --pad W:H, or edit each image on its own`.

The **aspect form** saves `aspect`/`gravity`/`color` and is size-independent: allowed on a batch.
`--trim` saves only what was given (`color`, `tolerance`), size-independent: allowed.

### The alpha rule on `edit`, and where its check lives

**Driven on `main`, the nearest route** (`edit` can't add alpha today): `apply` with
`pad border = 0.25, color = "transparent"`:
```
apply --out-dir od in.jpg          -> exit 4, nothing written:
  error: recipe step 0 ('pad'): 'color' = "transparent" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque
apply --out-dir od in.jpg in.png   -> exit 6, in.png written:
  error: in.jpg: recipe step 0 ('pad'): … ; error: 1 of 2 inputs failed
```
**Reasoned from `run_pixel_op`** (`src/cli/ops.rs`): `edit` passes `forced_format: None`, so the
top-of-function check never fires; the format is resolved by `output_format_for` **after**
`pipeline.run`, with no alpha check. Today `edit in.jpg --pad 10 --pad-color transparent` would
therefore write a JPEG with a black border, exit 0 (SPEC-151's frame drove that encoder behaviour).

**Recommendation: two checks, both gated on `pipeline.introduces_alpha()`.**
1. **Once, before any input is read**, when the format is the same for every input: `--format`,
   the `-o` path's extension (one input), or the `--out-dir` template's literal extension. Exit 4,
   nothing written, out-dir not created. Matches `apply`. Without this, `edit dir/ --format jpeg
   --out-dir o --pad 10 --pad-color transparent` would fail **every** input one by one (exit 6),
   where `apply` exits 4 once: `--format` on `edit` is not a forced format, it's resolved per input.
2. **Per input, after load and before `pipeline.run`**, for a format that follows the input
   (`output_format_for(.., img.source_format())`). One input: exit 4, nothing written. Batch: that
   input's `error: <input>: …` line, the others written, exit 6.

Both can run before the pipeline: `output_format_for` reads the flags, the output name and the
source format, never the processed image. **Why gated:** hoisting `output_format_for` above
`pipeline.run` for every caller changes existing exit codes. Driven on `main`: `edit in.png --crop
10x10+0+0 --format bogus` → exit 1 (the crop fails first); `resize --exact 60000x10 --format bogus`
→ exit 1. Hoisted, both would become exit 4. Gated, only a pipeline that introduces alpha (today:
only `edit --pad` with a translucent fill) takes the new path, and no existing output changes.

**Wording on `edit`:** `UnstorableAlpha`'s Display says `recipe step 1 ('pad'): 'color' = …`
(index 1, because `edit`'s pipeline starts with the `auto-orient` bake), naming a key and a step
the user never typed. Recommend: `edit` names the flag (DEC-106): `--pad-color "transparent" has
alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour
opaque` (GIF: the existing GIF tail). Same exit codes.

⚠ **Mutation the design must carry:** `--pad N` runs through an `edit`-side wrapper (like
`PixelCrop`). If the wrapper doesn't forward `introduced_alpha()` (the trait default returns
`None`), every check above silently passes and JPEG gets a black border, exit 0. Mutant: delete the
wrapper's `introduced_alpha` override. The aspect form builds the real `Pad` and is not exposed.

**DEC-115 follow-up (at ship, wording only):** part 4's "that call site has no per-input check for
an unforced format yet. It arrives with `edit --pad` (SPEC-160)" becomes a statement of what
`run_pixel_op` does: a batch-wide format checked once up front, else per input after load, both
only when a step introduces alpha. The Neutral consequence ("`run_pixel_op` still runs the pipeline
first and resolves the format after") stays true for every other pipeline; add "unless a step
introduces alpha". Design re-reads DEC-115 against this (SPEC-159's ship lesson).

### `trim`'s whole match on `edit`

`trim` fails at **apply**, not through the refusal hook (SPEC-152; `refusal_hook_is_not_used` in
`src/operation/trim.rs`). Driven on `main`:
```
apply (trim, no keys) -o o.png in.png       -> exit 1: error: operation 'trim' failed: every pixel of this 4×4 px image matches its top-left pixel within 'tolerance' = 0, so a trim would leave nothing
apply (trim) --out-dir od in.png wide.png   -> exit 6: error: in.png: operation 'trim' failed: …; wide.png written
```
**Recommended `edit` wording** (the `edit` wrapper rewrites the apply reason, as `PixelCrop` speaks
pixels): `--trim would leave nothing: every pixel of this 4×4 px image matches its top-left pixel
within --trim-tolerance 0`, and with a colour, `… matches --trim-color "#ffffff" within
--trim-tolerance 8`. Exit 1 for one input; in a batch that input fails, the others are written,
exit 6. Not skipped.

**So the "watermark skipped" entry is not made wrong by this spec:** neither `pad` nor `trim` uses
the hook (both `refusal_hook_is_not_used` tests), so `report_skipped_images` still only ever sees
`watermark`. See the stage-closing list.

### `parse_color` refuses a sign

**Driven on `main`**, 4×4 PNG of (200, 100, 50); `trim` on a 4×4 with a `#0f0f0f` border and red centre:
```
pad recipe  color = "+f+f+f"    -> exit 0, top-left (15, 15, 15)            [= "#0f0f0f"]
pad recipe  color = "+f+f+f+f"  -> exit 0, RGBA, top-left (15, 15, 15, 15)  [a translucent fill]
pad recipe  color = "-f-f-f"    -> exit 1 (from_str_radix refuses '-' for u8)
trim recipe color = "+f+f+f"    -> exit 0, 2×2 red                           [= "#0f0f0f"]
watermark recipe color = "+f+f+f"      -> exit 0, mark pixel (15, 15, 15)
watermark --color "+f+f+f" (flag)      -> exit 0, mark pixel (15, 15, 15)
```
New in this frame: **`+f+f+f+f` introduces alpha 0f**, so a sign can also turn on DEC-115's
refusal or widen RGB to RGBA. wasm `transform` reads the same recipe loader, so it accepts the
same strings (reasoned; no wasm drive).

**The fix:** in `parse_color` (`src/text/mod.rs`), after stripping one optional leading `#`,
refuse unless the remainder is 6 or 8 bytes **and every byte is an ASCII hex digit**
(`u8::is_ascii_hexdigit`), then parse the pairs. Upper and lower case stay accepted. The explicit
`is_ascii()` check goes: an ASCII-hex test also refuses every non-ASCII byte before any slicing.
Keep SPEC-152's non-ASCII test rows (`aéaaa`, `#éééé`, `aéaaaaa`) as the regression for that.

**What a user sees after** (messages unchanged, only which strings reach them):
- `pad`/`trim`/`watermark` recipe, `apply`/`build`: exit **1**, `recipe step N ('<op>'): 'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "+f+f+f"`.
- `watermark --color +f+f+f`: exit **2**, `invalid color '+f+f+f': expected RRGGBB, #RRGGBB, or RRGGBBAA`.
- `edit --pad-color +f+f+f` / `--trim-color +f+f+f`: exit **2**, the recipe message with the flag name.
- wasm `transform`: throws the recipe message.

**CHANGELOG: yes, a "Changed" line**: a recipe or `--color` that loaded before is now refused,
e.g. "A hex colour with a `+` in it (`+f+f+f`) is now refused; it used to load as a different
colour (`#0f0f0f`)". **No shipped recipe, doc example, golden or test uses a sign:**
`/usr/bin/grep -rnE 'color *= *"[^"]*[+]|--(pad-|trim-)?color[ =]"?[+]'` over `*.toml *.md *.rs
*.golden *.js *.ts *.html` (outside `target/`, `projects/`, `decisions/`): 0 hits. SPEC-141's
recipe-output guard therefore stays green unchanged.

### Typed order and replay, the test to extend

SPEC-153's "`--save-recipe` replays exactly" test, extended: for each row, `edit … --save-recipe
r.toml -o a.png` then `apply --recipe r.toml -o b.png` on the same input, `a == b` pixel for pixel.
Rows (design picks the final set; the derivation is the sizing):
- Order rows that **can** fail: `--pad 4:5 --pad-color red --rotate 90` vs `--rotate 90 --pad 4:5 …`
  (driven via recipes: 8×6 vs 5×6); `--pad 2 --pad-color red --trim` (trim removes the pad) vs
  `--trim --pad 2 --pad-color red`; `--resize-max N --pad M` (fraction of the resized edge).
- Sources: L8, LA8, RGB8, RGBA8, L16, RGBA16 (DEC-095), each with `--pad N` opaque, `--pad N`
  transparent (widens RGB→RGBA, L→LA), `--pad W:H --pad-gravity`, and `--trim` with and without
  `--trim-color`/`--trim-tolerance` (16-bit: tolerance ×257).
- The saved recipe's text: `border` written at shortest-repr `f64` (DEC-114 part 5); `gravity`,
  `color`, `tolerance` absent when not typed.

### Where each failure happens, and its exit code

- **Parse (clap or `build_edit_ops`), exit 2, nothing read:** every refusal listed under "Syntax"; a
  bad `--pad-color`/`--trim-color`, including `+f+f+f` after the fix.
- **Before any input is read, exit 2:** `--save-recipe` + pixel `--crop` or `--pad N` + a batch.
- **Before any input is read, exit 4:** a translucent `--pad-color` into a batch-wide JPEG/GIF
  format (`--format`, `-o` extension, template literal).
- **After load, before the pipeline:** the same into a per-input format. One input exit 4, batch
  exit 6.
- **Apply:** `--pad` canvas over the caps; `--trim` whole match. One input exit 1, batch exit 6.

### Stage closing: the five other open entries (maintainer question; nothing moved)

Recount (driven, below): STAGE-058 has 7 `- [ ] (not yet written)` entries: this one, the
`parse_color` entry (folds here) and five others.
1. **Recipe-output guard reports both failures together** (test-only, "optional"): **move** to
   `guidance/ideas.yaml` with the trigger "a spec trips the file-set and line-count checks at once".
   The stage doesn't need it.
2. **`crop`'s aspect follows each image's orientation** (a new key on `crop` and `pad`, a feature):
   **move** to STAGE-060 (the page's aspect presets are its first user) or a new PROJ-017 stage. It
   changes no shipped output, so STAGE-058 can ship without it. Maintainer's call: it was filed at
   the maintainer's request.
3. **"watermark skipped" names the wrong op**: **move** to `guidance/ideas.yaml` with the trigger
   "a second op uses `Operation::refusal`". Its premise ("`pad` or `trim` may be the first") is now
   false: both fail at apply (driven above for `trim`).
4. **`apply`/`build` warn on a tagged input with no `auto-orient` step**: **move** to a later stage
   or `guidance/ideas.yaml`. It's about every recipe op, not geometry flags (`edit` always bakes);
   design must still choose warn/refuse/silent. STAGE-058 can ship without it.
5. **Pin `watermark`'s nine gravities with a margin** (test-only): **fold**, on the maintainer's
   word. Nine rows and two mutants, no product code, so [M] holds. It has nothing to do with `edit`,
   but folding it lets SPEC-160 close the stage as its entry says. If the maintainer would rather
   not, leave it for a small test-only spec.

If 1–4 move and 5 folds, SPEC-160 closes STAGE-058 as its entry says. Otherwise the stage stays
open on what's left.

### Sizing: [M], and why

Code is modest (two `EditOp`s, two wrappers, five clap fields, a two-place alpha check, a
one-line `parse_color` rule). Verification is not:
- replay rows: 6 sources × 4 shapes (pad N opaque, pad N transparent, pad aspect + gravity,
  trim ± keys) = 24, plus ~4 order rows that can fail;
- alpha cells: 4 formats (JPEG, GIF, PNG, WebP) × 3 fill alphas (00, 80, ff) × 3 shapes (one input,
  batch-wide format, kept-format batch) = 36 before design trims it;
- `parse_color`: 7 routes (pad, trim, watermark recipes; `watermark --color`; wasm `transform`;
  `--pad-color`; `--trim-color`) × 5 strings (`+f+f+f`, `#+f+f+f`, `+f+f+f+f`, and the controls
  `-f-f-f`, `#0f0f0f`);
- ~14 parse-time refusals, each with its message;
- a shared-code change in `run_pixel_op` (a clean full-matrix build, lean leg included).
SPEC-153 was [M] for three flags; this is two flags plus a behaviour change and a shared check.
**To keep [S]** you would cut the `parse_color` fold back out (but it's ruled in) or cut the replay
matrix to RGB8 only (which drops DEC-095's 16-bit/grey precedent). Neither is worth it:
**recommend [M]**.

### Scope

**In:** `--pad`, `--pad-color`, `--pad-gravity`, `--trim`, `--trim-color`, `--trim-tolerance` on
`edit`, in typed order; their `--save-recipe` output and replay; the batch refusal widened to
`--pad N`; the alpha check in `run_pixel_op` (two places, gated) and its `edit` wording; `trim`'s
whole-match wording on `edit`; the `parse_color` sign fix (folded); the three "single image" sites;
docs and CHANGELOG; DEC-115 part 4 wording at ship.

**Out:** see "Out of scope" under Implementation Context.

### Docs: what each says after

- `edit --help` (`src/cli/mod.rs` doc comments): the summary line (below), and one line each for the six flags.
- `docs/cli-reference.md` `edit` heading and body: both flags in the synopsis and the op-flag list; `--pad N` is pixels of the long edge at that point; `--save-recipe` + pixel `--crop` or `--pad N` needs one input.
- `docs/api-contract.md` `edit` heading and body: the same, plus exits 2/4/6/1 for the paths above.
- `docs/recipes.md` (:215–219) and `recipes/README.md` (:53–54): the op list gains `--pad` and `--trim`; "a pixel `--crop` or `--pad N` is saved as fractions".
- `skills/crustyimg/SKILL.md`: lists no `edit` op flags today (grep); add an `edit --pad` example only if design wants it.
- `CHANGELOG.md` `[Unreleased]`: "Added" lines for `edit --pad` and `edit --trim`; a "Changed" line for `parse_color`.

**Behaviour sentences the build must drive on every exit path they name** (SPEC-159's ship lesson):
- "A translucent `--pad-color` into JPEG is refused": one input exit 4 (nothing written);
  `--format jpeg` batch exit 4 (no out-dir); kept-format batch exit 6 (others written). Plus GIF 80.
- "`--save-recipe` with `--pad N` needs one input": dir batch exit 2 (no image read); one input and
  stdin `-` allowed.
- "`--trim` refuses a uniform image": one input exit 1; batch exit 6 (the others written).
- "A hex colour with a sign is refused": each of the 7 routes, with its own exit (1, 2 or a throw).

### The three "single image" sites (quoted from `main`)

- `src/cli/mod.rs:453`: before "One-shot multi-op on a single image; optionally saves the recipe."
  After: "One-shot multi-op on one image or a batch; optionally saves the recipe."
- `docs/api-contract.md:542` (was :536): before "One-shot multi-op on a single image — the
  "experiment like an editor" mode." After: "One-shot multi-op on one image or a batch
  (`--out-dir`) — the "experiment like an editor" mode. `--save-recipe` with a pixel `--crop` or
  `--pad N` needs one input image."
- `docs/cli-reference.md:301`: before "Chain an ordered op list on a **single** image in one decode →
  ops → encode pass." After: "Chain an ordered op list on one image or a batch (`--out-dir`), each in
  one decode → ops → encode pass."

### wasm

`edit` isn't in wasm. The `parse_color` fix reaches `transform` through recipe load: a `+f+f+f`
recipe that loads today will throw the recipe message. No export, row or count changes: `scripts/wasm-ci-check.sh` has no line naming a colour (`grep -c color`: 0), and a sign grep over `*.mjs *.js *.sh *.py` (outside `target/`, `node_modules/`) finds 0 signed colours, so `tests/ddphotos_parity.mjs` and the npm smoke are unaffected. Optional: one wasm refusal row for `+f+f+f`.

### Rough build file list

- `src/cli/mod.rs`: `Edit` fields and help, `EditOp::{Pad, Trim}`, `edit_op_order`'s table, dispatch.
- `src/cli/ops.rs`: `EditFlags`, `build_edit_ops`, a `PixelPad` wrapper (forwards `introduced_alpha`), a trim wrapper (rewords the whole match), `run_edit`'s batch check and message, `run_pixel_op`'s two gated alpha checks, `edit`'s alpha wording.
- `src/text/mod.rs`: `parse_color`'s ASCII-hex rule; tests.
- `src/operation/alpha.rs` (only if `edit`'s wording is built from `UnstorableAlpha`).
- `tests/` (`tests/edit_geometry.rs` or SPEC-153's file, `tests/pad.rs`, `tests/trim.rs`, a watermark colour row); `tests/wasm_roundtrip.rs` if a wasm row is added.
- Docs: `docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md`, `CHANGELOG.md`; at ship, `decisions/DEC-115-…md` part 4 and `affected_scope`.

### Questions only the maintainer can answer

1. **Stage close (question 10):** move entries 1–4 as above and fold entry 5, so SPEC-160 closes
   STAGE-058? *Recommend: yes, with entry 2 to STAGE-060.*
2. **Size:** accept [M] rather than cutting the replay matrix or the `parse_color` fold? *Recommend: [M].*
3. **One `--pad` (N or W:H) rather than two flags?** *Recommend: one, to match `--crop`.*

### Maintainer rulings (2026-11-11)

The maintainer accepted all three recommendations:
- **Q1, stage close:** move entries 1–4 and fold entry 5, so this spec closes STAGE-058. Done on
  `main` by the orchestrator: entry 1 is `guidance/ideas.yaml` `recipe-output-guard-reports-both-failures`
  (deferred), entry 2 moved to STAGE-060's backlog, entry 3 is `refusal-summary-names-the-op`
  (deferred), entry 4 is `warn-tagged-input-without-auto-orient` (idea). Entry 5 (`watermark`'s nine
  gravities with a margin, test-only) is `- [x] (folded into SPEC-160)`: design adds its rows and
  mutants to this spec.
- **Q2, size:** [M], uncut. The `parse_color` fold and the 16-bit and grey replay rows stay.
- **Q3, `--pad`:** one flag, `--pad N|W:H`, matching `--crop`; `--pad-gravity` with the aspect form
  only.

## Design

*Designed 2026-11-11 (Opus 5.5) on `main` at `8ebc81a`. Every "driven" line ran in that session on a
release build of `main` (`ci-main`) or of a prototype of this design (`ci-proto`), each in a fresh
`CARGO_TARGET_DIR` in the session scratchpad (deleted after). The prototype was a detached scratch
worktree, never pushed; its test list is the Failing Tests below, run as written, and every mutation
row was applied to it and the whole list re-run.*

### Design corrections to the frame

1. **`--pad N` wider than the long edge can't be saved, so it fails at apply.** `pad`'s key table
   caps `border` at 1 (`src/operation/pad.rs` `PAD_PARAMS`, `max: Some(1.0)`). Driven on `main`:
   `border = 1.0` and `border = 1` pad a 4×4 to 12×12, exit 0; `border = 1.5` exits 1 with
   `recipe step 0 ('pad'): 'border' must be a number > 0 and <= 1, got 1.5`. So `--pad 5` on a 4×4
   image has no recipe form. The wrapper refuses it in pixels before any conversion:
   `--pad 5 is wider than the 4 px long edge of the 4×4 px image it receives; a border can be at most
   as wide as the long edge` (exit 1; a batch input fails, exit 6). `N == long` is allowed (border 1).
   The frame's bound (2.5e9 pairs) still covers every `N` that passes, since `N <= long`.
2. **No named colours.** The frame's examples use `--pad-color red`. `parse_color` reads hex only:
   on the prototype `--pad-color red` exits 2 with `--pad-color must be a hex colour RRGGBB, #RRGGBB
   or RRGGBBAA, got "red"`. Every example and test here uses hex. That message is FT-11's row.
3. **The batch-wide format is not "`-o`'s extension (one input)" on a batch.** In `run_pixel_op`'s
   batch branch `-o` is never read (`src/cli/ops.rs:445-470` on `main`: only `--out-dir` and the
   template). Driven on the prototype: `edit dir/ -o x.jpg --out-dir od --pad 1 --pad-color
   transparent` writes `b.png` and fails `a.jpg` alone (exit 6), as the kept-format batch does. So the
   batch-wide format is `--format`, else the template's literal extension (DEC-115 part 4's list).
   For one input there is no separate pre-check: the per-input check runs after load and before the
   pipeline, and exits 4 with nothing written (driven: `edit in.png -o x.jpg --out-dir od …` → exit 4,
   no `od`). `edit` takes `--name-template` (a global flag, `src/cli/mod.rs:84`).
4. **`edit`'s alpha line is built from the step, not the call site.** The frame offered two homes
   (a second `Display` mode on `UnstorableAlpha`, or a rewording at the call site). `run_pixel_op`
   is shared and doesn't know its caller is `edit`, so the flag travels with the step:
   `IntroducedAlpha` gains `flag: Option<&'static str>`, set by `edit`'s two pad wrappers, and
   `UnstorableAlpha`'s `Display` names the flag when it is set. A recipe step leaves it `None` and
   prints today's line byte for byte (pinned by `messages_verbatim`). The step index the frame
   predicted is right and is the reason: with the flag stripped (mutation M1c), `edit in.png
   --rotate 90 --pad 2 --pad-color transparent -o o.jpg` prints `recipe step 2 ('pad'): 'color' =
   "transparent" has alpha 00, …` (index 2: the bake, then `rotate`), driven.
5. **The aspect form is exposed too.** The frame said the aspect form "builds the real `Pad` and is
   not exposed". It still needs a wrapper, to name `--pad-color` in the alpha line and `--pad` in an
   apply-time cap failure (`'aspect' = "65535:1" on this 4×4 px image needs …` otherwise), so it has
   the same forwarding mutant (M1b), driven red.
6. **`--trim`'s whole-match wording keeps the op's sentence.** Rather than the frame's new
   `--trim would leave nothing: …` lead, the one wrapper that serves both `pad` forms and `trim`
   rewrites each `'key' = value` in an apply failure to `flag value`, so the line is the recipe
   line with the flags in it: `every pixel of this 4×4 px image matches its top-left pixel within
   --trim-tolerance 0, so a trim would leave nothing`. One rule, one wrapper, and no new sentence to
   keep in step with `trim.rs`.
7. **The batch `--save-recipe` check widens with no condition change.** `run_edit`'s check
   (`src/cli/ops.rs:1375-1377`) already matches any `StepRecord::Resolved`, and `--pad N` records
   that way, so the check covers it. Only the message is new, and it is chosen by the first
   pixel-length step in typed order: the existing crop line is kept byte for byte (SPEC-153's
   `batch_save_recipe_with_pixel_crop_is_refused` stays green unchanged), and a pad line is added.
8. **`--pad-gravity` alone names both missing flags.** clap chains the requirements: on the
   prototype, `--pad-gravity north --rotate 90` prints `the following required arguments were not
   provided:` then `--pad-color <COLOR>` and `--pad <N|W:H>`. `--pad-color` alone names only
   `--pad <N|W:H>`.

### Drives the prompt asked for

**The two gated checks.** `output_format_for` (`src/cli/ops.rs:214-244`) reads `global.format`,
the `-o` path, the template, the recipe format (always `None` on `run_pixel_op`) and the source
format, which `Image::source_format()` gives right after load. It never reads the processed image,
so both checks can run before `pipeline.run`. Rows on `ci-proto` (dir `bd/` = `a.jpg` 4×4 +
`b.png` 6×4):
- `edit bd --pad 1 --pad-color transparent --format jpeg --out-dir od` → exit 4, the JPEG line,
  no `od`.
- `… --name-template '{stem}.gif'` with `transparent` → exit 0, `a.gif b.gif` (GIF stores `00`).
- `… --name-template '{stem}.gif'` with `#ff000080` → exit 4, the GIF line, no `od` (FT-6).
- `… --name-template '{stem}_x.png'` → exit 0.
- `edit bd --pad 1 --pad-color transparent --out-dir od` (kept format) → exit 6,
  `error: <…>/bd/a.jpg: --pad-color "transparent" has alpha 00, …`, `b.png` written.
- `edit bd -o x.jpg --out-dir od …` → the same as kept format (correction 3).
- One input `edit flat.jpg --pad 1 --pad-color transparent -o o.jpg` → exit 4; `--out-dir od` with
  no `-o` and a JPEG source → exit 4, no `od` (FT-5).

**Gating rows, driven on `ci-main` and `ci-proto` (same codes on both):**
- `edit in.png --crop 10x10+0+0 --format bogus -o o.png` → exit 1 (the crop fails first).
- `resize in.png --exact 60000x10 --format bogus -o o.png` → exit 1.
- `watermark in.png --text hi --size 500 --format bogus -o o.png` → exit 2 (the mark is refused
  first). A third verb whose code an ungated check would move: the single-input refusal exits 2
  before the format today.
- `edit in.png --pad 9 --pad-color ff0000 --format bogus` → exit 1 on `ci-proto` (an opaque pad adds
  no alpha, so it takes the old path and fails at apply first).
Every `run_pixel_op` caller (`/usr/bin/grep -n "run_pixel_op(" src/cli/*.rs`: `ops.rs` resize,
edit, thumbnail, auto-orient, watermark; `optimize.rs` four calls for `convert`/`optimize`): only
`edit` can build a step that adds alpha, so only `edit --pad` with a fill below `ff` takes the new
path. `convert` and `optimize` pass a forced format, which the existing top-of-function check
covers; the new checks are skipped when `forced_format` is `Some`.

**The wrapper's trait methods.** `Operation` (`src/operation/mod.rs:307-348`) has five methods:
`name`, `params`, `apply` (required) and `refusal`, `introduced_alpha` (defaults `None`).
- `PixelPad` (`--pad N`): `name` "pad", `params` the resolved step (as `PixelCrop`), `apply`, and
  **`introduced_alpha` must be overridden** (M1: deleting it lets JPEG get a black border, exit 0,
  driven red on FT-5/6/7). `refusal` is not overridden: `Pad` has none (`refusal_hook_is_not_used`).
- `FlagWorded` (`--pad W:H`, `--trim`): forwards all five. `introduced_alpha` (M1b driven red) and
  `refusal` (M14: neither `pad` nor `trim` refuses, so dropping it is green; the search that found no
  splitting input is `/usr/bin/grep -n "fn refusal(" src/operation/pad.rs src/operation/trim.rs`, 0
  hits). It forwards `refusal` anyway, so a later op wrapped this way keeps its skip in a batch.

**`watermark --color`'s parser.** `run_watermark` calls `crate::text::parse_color` directly
(`src/cli/ops.rs:1738`) and maps `TextError::Color` to `CliError::Usage` (`text_error`,
`ops.rs:1562`): `invalid color '{0}': expected RRGGBB, #RRGGBB, or RRGGBBAA` (`src/text/mod.rs`
`TextError::Color`). One fix covers it. Driven on `ci-main`: `--color +f+f+f`, `#+f+f+f` and
`+f+f+f+f` all exit 0 on a 256×256 image; `--color=-f-f-f` exits 2 with that line. (Without `=`,
`--color -f-f-f` is clap's `unexpected argument '-f' found`, exit 2. FT-13 uses `=`.)

**Every `parse_color` caller** (`/usr/bin/grep -rn "parse_color" src/`, non-test):
- `src/operation/mod.rs:359` `parse_fill_color` → `pad.rs:165` (`Pad::from_params`) and `trim.rs:73`
  (`Trim::from_params`): recipe load on `apply`/`build`/wasm `transform`, and `edit`'s
  `build_flag_op` for `--pad-color`/`--trim-color`.
- `src/operation/mod.rs:1709`: `Watermark::from_params` (text mode `color`), recipe load.
- `src/cli/ops.rs:1738`: `watermark --color`.
After the fix each prints its existing message for a signed string (FT-13 drives every native one):
recipe `pad`/`trim`/`watermark` exit 1 `recipe step 0 ('<op>'): 'color' must be a hex colour RRGGBB,
#RRGGBB or RRGGBBAA, got "+f+f+f"`; `watermark --color` exit 2 `invalid color '+f+f+f': expected
RRGGBB, #RRGGBB, or RRGGBBAA`; `edit --pad-color`/`--trim-color` exit 2 `--pad-color must be a hex
colour RRGGBB, #RRGGBB or RRGGBBAA, got "+f+f+f"`; wasm `transform` throws the recipe line (same
`from_params`; not driven, see "wasm"). `src/wasm.rs` has no direct caller (grep: 0).

**The order rows, both orders driven on `ci-proto`** (FT-3), on a 60×40 RGB8 noise image in a
#101010 frame:
- `--pad 4:5 --pad-color ff0000 --rotate 90` → 75×60; `--rotate 90 --pad 4:5 …` → 48×60.
- `--pad 2 --pad-color 101010 --trim` → 52×32 (the trim takes the pad and the frame);
  `--trim --pad 2 --pad-color 101010` → 56×36.
- `--pad 3 --pad-color ff0000 --resize-max 30` → 30×21; `--resize-max 30 --pad 3 …` → 36×26,
  saved `border = 0.1` (3 of the 30 px the step receives, not 3 of 60: the derived value differs
  from the naive one, trap 10).
Each pair's two outputs differ in bytes (asserted). The commuting pair (border pad × quarter turn)
is left out, as the frame found.

**The canvas cap in pixels.** Driven on `ci-proto`:
- 2×49990 L8, `--pad 6` → `needs a 14×50002 px canvas, which exceeds the maximum edge limit of
  50000 px` (exit 1). The short side fits; only the long one doesn't. M13 (check width only) lets it
  through to `pad`'s own check, which prints `'border' = 0.00012002400480096019 on this …`, so FT-9
  goes red.
- 2×11000 L8, `--pad 5000` → `needs a 10002×21000 px canvas = 210042000 px, which exceeds the
  maximum area of 134217728 px` (both sides under 50000).
- 4×4, `--pad 65535:1` (aspect form, through `pad`'s own check, reworded) → `--pad "65535:1" on this
  4×4 px image needs a 262140×4 px canvas, which exceeds the maximum edge limit of 50000 px`.

### The code shape

Line numbers are `main` at `8ebc81a`.
- **`src/cli/mod.rs:453`** `Edit`'s summary: "One-shot multi-op on one image or a batch; optionally
  saves the recipe."
- **`src/cli/mod.rs:486`** (after `flip`) six fields, in this order: `pad: Option<String>`
  (`value_name = "N|W:H"`, `requires = "pad_color"`), `pad_color: Option<String>` (`value_name =
  "COLOR"`, `requires = "pad"`), `pad_gravity: Option<String>` (`value_name = "GRAVITY"`, `requires =
  "pad"`), `trim: bool`, `trim_color: Option<String>` (`"COLOR"`, `requires = "trim"`),
  `trim_tolerance: Option<u8>` (`"N"`, `requires = "trim"`). No `allow_negative_numbers`. Help
  lines: see "Docs".
- **`src/cli/mod.rs:898-911`** `EditOp` gains `Pad` and `Trim`; **`:924`** `edit_op_order`'s
  `known` gains `("pad", EditOp::Pad)` and `("trim", EditOp::Trim)`.
- **`src/cli/mod.rs:1167-1191`** dispatch passes `pad`, `pad_color`, `pad_gravity`, `trim_color`,
  `trim_tolerance` into `EditFlags`. `trim` itself is not copied: `order` carries it (a copied bool
  is dead code, and CI's clippy denies warnings).
- **`src/cli/mod.rs:1619`** the `UnstorableAlpha` exit-code test adds `flag: None`.
- **`src/cli/ops.rs:214-244`** split `output_format_for`: rungs 1-2 move into
  `pub(super) fn format_fixed_by_flags(global, output_path, template) ->
  Result<Option<ImageFormat>, CliError>`, which `output_format_for` calls first. No behaviour change
  (its seven `output_format_for_*` tests pin it).
- **`src/cli/ops.rs:347`** after `resolve_all`: `let alpha_check = forced_format.is_none() &&
  pipeline.introduces_alpha();` with a comment saying why it's gated.
- **`src/cli/ops.rs:376-381`** (one input, after the warnings, before `source_info`): when
  `alpha_check`, resolve `output_format_for` with the same `output_path`/`template` rule as `:388-395`
  and return `pipeline.unstorable_alpha(fmt)` as an error.
- **`src/cli/ops.rs:462`** (batch, before `let total`): when `alpha_check` and
  `format_fixed_by_flags(global, None, Some(&template))?` is `Some(fmt)`, refuse once.
- **`src/cli/ops.rs:488-490`** (batch, per input, after the warnings): when `alpha_check`,
  `output_format_for(global, None, Some(&template), None, img.source_format())` and refuse; the
  closure's `Err` makes it that input's `error: <input>: …` line.
- **`src/cli/ops.rs:1046-1054`** `EditFlags` gains `pad`, `pad_color`, `pad_gravity: Option<String>`,
  `trim_color: Option<String>`, `trim_tolerance: Option<u8>`. `StepRecord::Resolved`'s doc names
  `--pad N` too.
- **`src/cli/ops.rs:1196`** (after `impl Operation for PixelCrop`): `enum PadFlag { Border(u32),
  Aspect }`; `fn parse_pad_flag` (a `:` → `Aspect`; else `crop_number` and `>= 1` → `Border`; else
  the syntax message); `struct PixelPad { n, color, alpha: Option<IntroducedAlpha>, resolved }` with
  `impl Operation` (long-edge check, edge cap, area cap, all in pixels; then `border = n as f64 / long
  as f64` and `color`, built with `Pad::from_params`, applied, params stored; `introduced_alpha`
  returns `alpha`); `struct FlagWorded { inner: Box<dyn Operation>, keys: &'static [(&str, &str)] }`
  with `impl Operation` forwarding all five methods (`apply` rewrites each `'key' = ` to `flag ` in
  an `OperationError::Apply` reason; `introduced_alpha` sets `flag` from `keys`); `const PAD_FLAGS =
  [("aspect", "--pad"), ("gravity", "--pad-gravity"), ("color", "--pad-color")]` and `const
  TRIM_FLAGS = [("color", "--trim-color"), ("tolerance", "--trim-tolerance")]`.
- **`src/cli/ops.rs:1250`** the no-op-flag message adds `, --pad, --trim`.
- **`src/cli/ops.rs:1342`** (before `Ok(steps)`) two arms. `EditOp::Pad`: read `pad` and `pad_color`
  (`missing_edit_value` if absent); `Aspect` → params `aspect` (+ `gravity` when given) + `color`,
  `build_flag_op("pad", …, PAD_FLAGS)`, pushed as `built(Box::new(FlagWorded { … }))`; `Border(n)` →
  refuse `--pad-gravity`; build a stand-in `{border = 1.0, color}` through `build_flag_op` (so a bad
  colour exits 2 before any file is read, with the op's own message), take its `introduced_alpha()`
  with `flag: Some("--pad-color")`, push `PixelPad` with `StepRecord::Resolved`. `EditOp::Trim`:
  params `color` and `tolerance` only when given (`Integer`), `build_flag_op("trim", …, TRIM_FLAGS)`,
  pushed as `FlagWorded`.
- **`src/cli/ops.rs:1374-1388`** the batch check: `find` the first `Resolved` step and read its
  `op.name()`; `"pad"` gets the pad line, anything else keeps the crop line verbatim.
- **`src/cli/ops.rs:1421-1426`** the "never ran" internal error names the step's op, not `"crop"`.
- **`src/operation/alpha.rs:52-60`** `IntroducedAlpha` gains `pub flag: Option<&'static str>` ("the
  CLI flag that set the key, when the step was built from a flag; `None` for a recipe step").
  **`:74-89`** `Display`: `Some(flag)` → `{flag} {written:?} has alpha {alpha:02x}, and `; `None` →
  today's `recipe step …` lead. The tails are unchanged. The doc comment on `fmt` says both.
- **`src/operation/alpha.rs:144`** the test helper adds `flag: None`.
- **`src/operation/pad.rs:358-362`** `introduced_alpha` adds `flag: None`.
- **`src/text/mod.rs:54-58`** `parse_color`: replace `if !hex.is_ascii()` with `if
  !hex.bytes().all(|b| b.is_ascii_hexdigit())`, and the comment with why (a sign would parse; a
  multi-byte character would split). The length match stays. Upper and lower case stay accepted.
- **No change** to `src/wasm.rs`, `src/pipeline/mod.rs`, `src/operation/trim.rs` or the recipe
  loader.

### Every message, verbatim

Exit code first. "Reused" cites the source of the wording.
- **2** `--pad must be N, a whole number of pixels of at least 1, or an aspect W:H, such as 4:5; got "-5"` (also `"0"`, `"10px"`, `"4294967296"`, `"+3"`; new, shaped as `--crop`'s at `ops.rs:1101`).
- **2** `--pad-gravity only places an aspect --pad W:H; a border --pad N adds the same width on every side` (new, mirrors `--crop-gravity`'s at `ops.rs:1320`).
- **2** `--pad must be two whole numbers from 1 to 65535 written W:H, such as "16:9"; got "4:"` (reused, `crop.rs:229`, key swapped).
- **2** `--pad-gravity must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest; got "up"` (reused, the key table, key swapped).
- **2** `--pad-color must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "red"`; `--trim-color must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "zz"` (reused, `pad.rs:167`, `trim.rs:75`, key swapped).
- **2** clap: `the following required arguments were not provided:` + `--pad-color <COLOR>` (for `--pad 2`); + `--pad <N|W:H>` (for `--pad-color` alone); + both (for `--pad-gravity` alone); + `--trim` (for `--trim-color` or `--trim-tolerance` alone); `the argument '--pad <N|W:H>' cannot be used multiple times`; `the argument '--trim' cannot be used multiple times`; `invalid value '256' for '--trim-tolerance <N>': 256 is not in 0..=255`; `unexpected argument '-1' found`; `unexpected argument '-5' found`.
- **2** `edit requires at least one operation flag (--auto-orient, --resize-max, --invert, --crop, --rotate, --flip, --pad, --trim)` (extended, `ops.rs:1250`).
- **2** `--save-recipe with a border --pad N needs one input image, and "in" names 2: the border is saved as a fraction of the image it ran on; use an aspect --pad W:H, or edit each image on its own` (new; the crop line at `ops.rs:1384` unchanged).
- **4** `--pad-color "transparent" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque` (tail reused, `alpha.rs:97`).
- **4** `--pad-color "#ff000080" has alpha 80, and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead` (tail reused, `alpha.rs:90`).
- **6** batch, per input: `error: <input>: --pad-color "transparent" has alpha 00, and JPEG output …` then `error: 1 of 2 inputs failed`.
- **1** `operation 'pad' failed: --pad 5 is wider than the 4 px long edge of the 4×4 px image it receives; a border can be at most as wide as the long edge` (new).
- **1** `operation 'pad' failed: --pad 6 on this 2×49990 px image needs a 14×50002 px canvas, which exceeds the maximum edge limit of 50000 px` (shape reused, `pad.rs:210`).
- **1** `operation 'pad' failed: --pad 5000 on this 2×11000 px image needs a 10002×21000 px canvas = 210042000 px, which exceeds the maximum area of 134217728 px` (shape reused, `pad.rs:216`).
- **1** `operation 'pad' failed: --pad "65535:1" on this 4×4 px image needs a 262140×4 px canvas, which exceeds the maximum edge limit of 50000 px` (`pad.rs`'s line, reworded by the wrapper).
- **1** `operation 'trim' failed: every pixel of this 4×4 px image matches its top-left pixel within --trim-tolerance 0, so a trim would leave nothing` (reused, `trim.rs:96-105` `whole_match_reason`, reworded).
- **1** `operation 'trim' failed: every pixel of this 4×4 px image matches --trim-color "#C86432" within --trim-tolerance 8, so a trim would leave nothing`.
- **1** recipe `pad`/`trim`/`watermark`: `recipe step 0 ('pad'): 'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got "+f+f+f"` (unchanged text; newly reached).
- **2** `watermark --color=+f+f+f`: `invalid color '+f+f+f': expected RRGGBB, #RRGGBB, or RRGGBBAA` (unchanged text).
- **2** `edit --pad-color=+f+f+f` / `--trim-color=+f+f+f`: the `--pad-color must be a hex colour …` line.
None names a spec, stage or DEC id. Every one is an exact-string assertion in FT-5 to FT-13.

### The replay matrix (FT-1 to FT-4), and why each row

Fixture `framed(kind)`: 60×40, 52×32 seeded noise at (3, 3) in an opaque #101010 frame (3 px top and
left, 5 px bottom and right), so `--trim` has something to take and the expected size (52×32) is a
literal. Six kinds: `l8`, `la8`, `rgb8`, `rgba8`, `l16`, `rgba16` (DEC-095: grey, alpha and 16-bit
each take a different branch in `pad`'s widening table, `pad.rs:313-327`, and in `trim`'s level
reader). Five shapes per kind, 30 rows:
- `--pad 3 --pad-color 336699` → 66×46: the opaque chromatic fill. Grey widens to RGB (L8→Rgb8,
  L16→Rgb16), LA8→Rgba8, the rest keep their type. Saved `border = 0.05`.
- `--pad 3 --pad-color transparent` → 66×46: L→LA, RGB→RGBA, the rest keep.
- `--pad 1:1 --pad-gravity southwest --pad-color ff000080` → 60×60: the aspect form, gravity saved,
  every type ends with alpha (L8→Rgba8, L16→Rgba16).
- `--trim` → 52×32: the top-left colour, type kept, no `color`/`tolerance` key saved.
- `--trim --trim-color 111111 --trim-tolerance 1` → 52×32 (one 8-bit step from #101010; ×257 at 16
  bits), `tolerance = 1` saved; and the control `--trim --trim-color 111111` → 60×40 (nothing
  trimmed). The pair shows the tolerance is used and is in 8-bit steps at 16 bits.
The 24 + 4 the frame counted become 30 + 3 (FT-3's pairs) + 1 (FT-4). The frame's fourth order row
(`--resize-max N --pad M`) is FT-3's third pair. Expected values are literals worked out by hand; no
expected value is read from the code under test.

FT-2 pins the saved text: `--pad 7` on 60×40 → `border = 0.11666666666666667` (7/60 printed in full;
a fixed-decimal print would differ, trap 10); `color = "AbCdEf"` as typed; no `gravity` key unless
typed (and the aspect form saves `aspect = "4:5"`, no `gravity`).

### What doesn't change, and what pins it

- Every `edit` output with today's flags: SPEC-153's byte pins (`tests/edit.rs`,
  `fnv1a64` rows) and `tests/edit_geometry.rs`'s replay rows. One `edit_geometry.rs` assertion
  changes text: `no_op_flag_message_lists_every_flag` gains `, --pad, --trim` (red on the
  prototype until updated; it is the only pre-existing test this spec edits).
- Every recipe output: SPEC-141's `every_recipe_output_matches_its_golden`
  (`tests/recipe_corpus_output.rs`), and `parse_color` changes no accepted string's value (a
  string that was all hex parses as before; the grep in the frame found no shipped signed colour).
- `watermark` outputs with an unsigned colour: `tests/watermark_cli_params.rs` and
  `tests/watermark_damage.rs` goldens. `pad` outputs: `tests/pad.rs`'s `fnv1a64` pins.
- `apply`'s alpha line: `messages_verbatim` (`alpha.rs`), unchanged, with `flag: None`.
- `run_pixel_op`'s exit codes for pipelines that add no alpha: FT-8 (new; no pin existed).
No new pin is added except FT-8.

### wasm: no row

The `parse_color` fix reaches `transform` through the same `Pad`/`Trim`/`Watermark::from_params`
the native rows drive, and `parse_color` has no target-dependent code (no `cfg`, no float, no
time). A wasm row would prove only that wasm calls the same function, at the cost of
`scripts/wasm-ci-check.sh`'s `FULL_COUNT`/`SLIM_COUNT` and its self-test fixtures. So no wasm row,
`scripts/wasm-ci-check.sh` and `tests/wasm_roundtrip.rs` are **not** in the file list, and no doc
or CHANGELOG sentence names the browser build for this change (so none needs a wasm drive).

### Cache

No `CACHE_SCHEMA_VERSION` or lockfile change. A `build` lockfile and the cache key hash a recipe's
text and inputs; a recipe with a signed colour now fails at load, before any key is computed or
looked up, so no stale entry can be served for it. Every recipe that still loads produces the same
pixels as before (the corpus guard), so every existing key still names the right output.

### Size: [M] holds

The prototype diff (`git diff --stat` against `origin/main`): 7 files, 1041 insertions, 38
deletions. Product code 481 lines (`ops.rs` 364, `mod.rs` 50, `alpha.rs` 49, `text/mod.rs` 17,
`pad.rs` 1); tests 598 (`tests/edit_pad_trim.rs` 558 new, `tests/watermark.rs` 40). The build adds
five doc files (`docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md`,
`CHANGELOG.md`) and one string in `tests/edit_geometry.rs`: 13 files. SPEC-153 was [M] at a similar size. The verification is the weight: 30 replay rows + 4 order
rows; 18 Failing Tests (`/usr/bin/grep -c '^- \*\*FT-'` on that section: 15 integration, FT-1 to
FT-13, FT-17, FT-18, and 3 unit, FT-14 to FT-16); 22 mutation rows (21 mutants + trap 14,
`/usr/bin/grep -c '^- \*\*'` on the table); 16 ACs. Not [L]: one code path
(`build_edit_ops` → `run_pixel_op`), no new DEC, no new dependency.

## Inputs

- **Files to read:** `src/cli/mod.rs` (`Edit`, `EditOp`, `edit_op_order`, dispatch), `src/cli/ops.rs`
  (`output_format_for`, `run_pixel_op`, `EditFlags`, `PixelCrop`, `build_flag_op`,
  `build_edit_ops`, `run_edit`), `src/operation/alpha.rs`, `src/operation/pad.rs`,
  `src/operation/trim.rs`, `src/text/mod.rs` (`parse_color`), `src/pipeline/mod.rs`
  (`introduces_alpha`, `unstorable_alpha`), `tests/edit_geometry.rs` (the helpers to copy).
- **Related code paths:** `src/cli/`, `src/operation/`, `src/text/`.

## Outputs

- **Files created:** `tests/edit_pad_trim.rs`: FT-1 to FT-13.
- **Files modified:** `src/cli/mod.rs`, `src/cli/ops.rs`, `src/operation/alpha.rs`,
  `src/operation/pad.rs`, `src/text/mod.rs`, `tests/watermark.rs`, `tests/edit_geometry.rs` (one
  expected string), `docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`,
  `recipes/README.md`, `CHANGELOG.md`. At ship: `decisions/DEC-115-…md`.
- **New exports:** `pub flag: Option<&'static str>` on `operation::IntroducedAlpha` (a public field
  on a public struct: library code that builds one adds `flag: None`; the CHANGELOG's "Breaking"
  library line says so, as `ParamKind::UInt`'s did).
- **Database changes:** none.

## Acceptance Criteria

Each AC names its tests. "FT-n" is the Failing Tests list below; "M-n" the mutation table.

- [ ] **AC-1 Replay, six colour types × five shapes.** On `framed(kind)` for `l8`, `la8`, `rgb8`,
  `rgba8`, `l16`, `rgba16`: `--pad 3` opaque chromatic, `--pad 3` transparent, `--pad 1:1
  --pad-gravity southwest` with alpha 80, `--trim`, and `--trim --trim-color 111111
  --trim-tolerance 1` each run, save `[auto-orient, pad|trim]`, and replay through `apply` to the
  same colour type, size and samples; each gives the colour type listed in "The replay matrix";
  `--trim --trim-color 111111` without the tolerance trims nothing. FT-1.
- [ ] **AC-2 Saved text.** `--pad N` saves `border` = N / the long edge it received at shortest-repr
  `f64` (`0.11666666666666667` for 7 on 60×40) and `color` as typed; `gravity`, `color` (trim) and
  `tolerance` are written only when typed. FT-1, FT-2.
- [ ] **AC-3 Typed order.** `--pad W:H` × `--rotate`, `--pad N` × `--trim`, `--pad N` ×
  `--resize-max`: each pair both ways runs and saves in typed order, replays, and the two orders
  differ; after `--resize-max 30`, `--pad 3` saves `border = 0.1`. A switch before the input path
  keeps its position (`--trim --rotate=90 IN --pad=1` → `[auto-orient, trim, rotate, pad]`).
  `edit_op_order` returns `Trim, Rotate, Pad` for that shape. FT-3, FT-4, FT-14.
- [ ] **AC-4 Alpha, one input.** A fill whose alpha the format can't store exits 4 with nothing
  written, for both pad forms, JPEG (`transparent`) and GIF (`#ff000080`), with an `-o` extension
  and with a kept JPEG source into `--out-dir` (no out-dir made); the line names `--pad-color` and
  no step. Opaque into JPEG, `transparent` into GIF and alpha 80 into PNG exit 0 (the PNG keeps
  `(255, 0, 0, 128)`). FT-5, FT-15.
- [ ] **AC-5 Alpha, batch-wide format.** `--format jpeg`, `--name-template '{stem}.jpg'` and
  `'{stem}.gif'` (alpha 80) on a two-image batch exit 4 once, with the flag-named line, and make no
  out-dir. FT-6.
- [ ] **AC-6 Alpha, kept-format batch.** The JPEG input fails with one `error: <input>: …` line, the
  PNG is written, the last line is `error: 1 of 2 inputs failed`, exit 6. FT-7.
- [ ] **AC-7 The gate.** A pipeline that adds no alpha keeps today's exit code when it is wrong two
  ways: `edit --crop 10x10+0+0 --format bogus` 1, `resize --exact 60000x10 --format bogus` 1,
  `edit --pad 9 --pad-color ff0000 --format bogus` 1, `watermark --size 500 --format bogus` 2. FT-8.
- [ ] **AC-8 `--pad N` apply failures speak pixels.** Wider than the long edge; past the edge cap on
  the long side only (2×49990); past the area cap with both sides under it (2×11000); an aspect past
  the edge cap (reworded to `--pad "65535:1"`): each exits 1 with its exact line. `--pad 4` on 4×4
  (border 1) exits 0, 12×12. FT-9.
- [ ] **AC-9 `--trim` whole match.** Exit 1 with the flag-named line, without and with
  `--trim-color`/`--trim-tolerance`; in a batch the uniform input fails, the other is written,
  exit 6. FT-10.
- [ ] **AC-10 Parse refusals.** Every exit-2 line in "Every message, verbatim" is produced as written,
  nothing is written, no file is read. FT-11.
- [ ] **AC-11 Batch `--save-recipe`.** `--pad N` (alone, and typed before a pixel `--crop`) on a
  two-image directory exits 2 with the pad line, no out-dir and no recipe; the aspect form and
  `--trim` save from the same directory; one file saves. SPEC-153's crop line is unchanged. FT-12,
  `batch_save_recipe_with_pixel_crop_is_refused` (existing, unchanged).
- [ ] **AC-12 A signed colour is refused on every native route.** `+f+f+f`, `#+f+f+f`, `+f+f+f+f`
  and `-f-f-f` in a recipe's `pad`, `trim` and `watermark` `color` (exit 1), `watermark --color`
  (exit 2), `edit --pad-color` and `--trim-color` (exit 2), each with its exact line; `#0f0f0f` and
  `0F0F0F` load on the same routes. `parse_color` refuses each signed string and keeps upper and
  lower case. FT-13, FT-16.
- [ ] **AC-13 `edit`'s alpha line.** `UnstorableAlpha` names the flag when `introduced.flag` is set
  (JPEG and GIF tails) and keeps today's recipe line when it is `None`. FT-15, `messages_verbatim`
  (existing).
- [ ] **AC-14 `watermark`'s nine gravities with a margin.** A 4×3 mark on a 41×31 image with margin
  3 sits at the hand-worked offsets for each gravity. FT-17.
- [ ] **AC-15 Help and docs.** `edit --help` shows the six flags and the new summary; the docs
  surfaces under "Docs" say what that list says, and each behaviour sentence is driven on every exit
  path it names (build and verify). FT-18; verify reads the docs.
- [ ] **AC-16 Nothing else changes.** `tests/edit.rs`, `tests/edit_geometry.rs` (with the one
  updated string), `tests/recipe_corpus_output.rs`, `tests/pad.rs`, `tests/trim.rs`,
  `tests/watermark*.rs` and `messages_verbatim` pass unchanged; `just check` and the lean build pass.

## Failing Tests

Written at design and run on the prototype as written (green there; each mutation row below was
applied and the whole list re-run). Fixtures are written by each test from a seeded xorshift
(`noise(kind, w, h, seed)`, one value per pixel, channels from its bytes; alpha samples ORed with
128 or 0x8000 so no pixel is fully transparent) and `framed(kind)` (see "The replay matrix").
`flat()` is a 4×4 RGB (200, 100, 50). Each replay row runs `edit IN ARGS -o e.png -y --save-recipe
r.toml`, then `apply --recipe r.toml IN -o a.png -y`, and asserts both exit 0, the same `ColorType`,
dimensions equal to the literal, the same `as_bytes()`, and the saved `op` list **equal** to the
expected list. `tests/edit_pad_trim.rs` copies `step`, `write_img`, the `run` helper and `replay`
from `tests/edit_geometry.rs` (a test file has no shared helper module for these today; the copy is
~120 lines).

**`tests/edit_pad_trim.rs`** (new)
- **FT-1 `replay_pad_and_trim_on_six_colour_types`**: AC-1's 36 replays (30 + the six untrimmed
  controls), the colour type after each pad shape, `border = 0.05`, `gravity = "southwest"`, no
  `color`/`tolerance` after plain `--trim`, `tolerance = 1`.
- **FT-2 `saved_recipe_text_is_exact`**: `--pad 7 --pad-color AbCdEf` on 60×40 noise: keys exactly
  `border, color, op`; text contains `\nborder = 0.11666666666666667\n` and `\ncolor = "AbCdEf"\n`,
  no `gravity`; `--pad 4:5` saves `aspect = "4:5"` and no `gravity`.
- **FT-3 `pad_and_trim_follow_typed_order`**: the three pairs, both orders, sizes 75×60/48×60,
  52×32/56×36, 30×21/36×26, outputs differ, `border = 0.1` in the resize-first order.
- **FT-4 `typed_order_with_trim_before_the_input`**: `edit --trim --rotate=90 IN --pad=1
  --pad-color 000000` saves `[auto-orient, trim, rotate, pad]`.
- **FT-5 `translucent_pad_into_jpeg_or_gif_one_input`**: four refusals (`--rotate 90` then `--pad 2`
  and `--pad 4:5`, `.jpg` and `.gif`), exact stderr, exit 4, no output; a JPEG source into
  `--out-dir` → exit 4, no out-dir; three controls exit 0; the PNG's (0, 0) is `[255, 0, 0, 128]`.
- **FT-6 `translucent_pad_batch_wide_format_refused_once`**: `--format jpeg`,
  `--name-template {stem}.jpg` (transparent) and `{stem}.gif` (`#ff000080`, aspect form) on
  `in/{a.jpg, b.png}`: exit 4, exact stderr, no out-dir.
- **FT-7 `translucent_pad_kept_format_batch_fails_that_input`**: exit 6; stderr starts `error: `,
  ends with `/in/a.jpg: --pad-color "transparent" has alpha 00, … colour opaque\nerror: 1 of 2
  inputs failed\n`, two lines; `od/b.png` written, `od/a.jpg` not. (The tail match is because the
  resolver prints a canonical path, `/private/var/…` on macOS.)
- **FT-8 `exit_codes_unchanged_without_added_alpha`**: AC-7's four rows.
- **FT-9 `pad_n_failures_speak_pixels`**: AC-8's four exit-1 lines exactly, and the 12×12 control.
  (The prototype carried a fifth placeholder row skipped by an empty string; the build drops it.)
- **FT-10 `trim_whole_match_names_the_flags`**: the two exit-1 lines exactly; the batch row.
- **FT-11 `parse_refusals`**: 21 rows: the 10 whole-line `edit` refusals (pad syntax ×5, gravity
  with a border, a bad aspect, a bad gravity, `red`, `zz`), the no-op-flag line, and 10 clap rows
  matched by prefix (the first line plus the named flag lines). Each: exit 2, no output.
- **FT-12 `batch_save_recipe_with_pad_n_is_refused`**: two refusal rows, exact stderr, no out-dir
  or recipe; the aspect form and `--trim --trim-tolerance 3` save from the batch; one file saves.
- **FT-13 `a_signed_hex_colour_is_refused_on_every_route`**: 4 strings × 6 routes = 24 exact-line
  rows (pad/trim/watermark recipes exit 1 on `apply`; `--color=` exit 2; `--pad-color=` and
  `--trim-color=` exit 2), and the controls `#0f0f0f`, `0F0F0F` on the pad recipe, `--color` and
  `--pad-color` (exit 0).

**`src/cli/mod.rs`** (`mod tests`)
- **FT-14** `edit_op_order_follows_indices` gains the row `edit --trim --rotate=90 in --pad 4:5
  --pad-color 000000 --trim-tolerance 2` → `[Trim, Rotate, Pad]`.

**`src/operation/alpha.rs`** (`mod tests`)
- **FT-15 `messages_name_the_flag_when_a_flag_set_it`**: `flag: Some("--pad-color")`, step 1:
  the JPEG and GIF lines exactly as in "Every message, verbatim" (no `recipe step`).

**`src/text/mod.rs`** (`mod tests`)
- **FT-16 `parse_color_refuses_a_sign`**: `+f+f+f`, `#+f+f+f`, `+f+f+f+f`, `-f-f-f`, `+0+0+0`,
  `0x0f0f`, ` fffff` → `TextError::Color`; `#0f0f0f` → `[15, 15, 15, 255]`, `0F0F0F0F` → `[15, 15,
  15, 15]`, `AbCdEf` → `[0xab, 0xcd, 0xef, 255]`. `parse_color_refuses_non_ascii` stays as it is.

**`tests/watermark.rs`**
- **FT-17 `watermark_nine_gravities_with_a_margin`**: base 41×31 opaque black, mark 4×3 opaque white
  (`--image`), `--margin 3`; for each gravity the white pixels are exactly 12, the first at the
  literal top-left and the last at top-left + (3, 2): center (18, 14), north (18, 3), south (18,
  25), east (34, 14), west (3, 14), northeast (34, 3), northwest (3, 3), southeast (34, 25),
  southwest (3, 25). (Free space 37×28; centre floors; margin 3 from each edge; centre ignores the
  margin.)

**`tests/edit_geometry.rs`**
- **FT-18** `edit_help_states_typed_order` gains `--pad <N|W:H>`, `--pad-color <COLOR>`,
  `--pad-gravity <GRAVITY>`, `--trim`, `--trim-color <COLOR>`, `--trim-tolerance <N>` and `one image
  or a batch` in its `want` list. `no_op_flag_message_lists_every_flag`'s expected line gains
  `, --pad, --trim` (it goes red on the prototype's baseline until then; the only existing
  assertion this spec changes).

Count: 13 + 1 (FT-14 row) + 1 + 1 + 1 + 1 (FT-18) = 18 (`/usr/bin/grep -c '^- \*\*FT-'` on this
section).

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-114`: typed order (part 1), modifiers with no position (part 4), pixels of the image the step
  receives, as an `f64` fraction written at full precision (part 5), `--save-recipe` with a pixel
  flag needs one input (part 6). `--pad N` is part 5's second user after `--crop`. No change.
- `DEC-115`: the alpha rule. Part 4's `run_pixel_op` sentence becomes true with this spec; ship
  applies the replacement text below. Part 8 (the message) holds for recipes; `edit`'s line names
  the flag instead of the step (part 8 lists what a recipe line names; the new text adds the flag
  case).
- `DEC-106`: a CLI verb's refusal names the flag (`build_flag_op`'s key swap; `FlagWorded` applies
  the same rule to apply-time lines).
- `DEC-112`: a recipe carries fractions; `border` is at most 1, which is why `--pad N > long` fails.
- `DEC-095`: colour type and bit depth; FT-1's six kinds.
- `DEC-102`: a recipe's load-time error is exit 1 (the signed-colour rows).
- `DEC-015`: batch failures are per input, exit 6.

### Constraints that apply

- `untrusted-input-hardening`: `--pad N` is parsed by `crop_number` (digits only, fits `u32`); the
  canvas is computed in `u64` before any allocation; `parse_color` refuses any non-hex byte before
  slicing.
- `no-unwrap-on-recoverable-paths`: no `unwrap`/`expect` in `build_edit_ops`, `PixelPad`,
  `FlagWorded`, `run_pixel_op`'s new checks.
- `clippy-fmt-clean`: CI denies warnings (a copied but unread `trim: bool` fails it).
- `every-public-fn-tested`, `test-before-implementation`: write FT-1 to FT-18 first and record each
  RED on `main` with its failure message (most fail on clap's `unexpected argument '--pad'`; FT-17
  and three of FT-8's four rows are green on `main`, as pins should be, and FT-8's `--pad` row exits
  2 there; FT-16's sign rows fail on the value).
- `single-image-library`, `no-new-top-level-deps-without-decision`: no new dependency.
- `one-spec-per-pr`.

### Prior related work

- `SPEC-153` (shipped): `edit --crop/--rotate/--flip`, `PixelCrop`, typed order, the replay test.
- `SPEC-151`, `SPEC-159` (shipped): `pad`, its transparent fill, `UnstorableAlpha`.
- `SPEC-152` (shipped): `trim`, its whole-match failure, the non-ASCII `parse_color` fix.
- `SPEC-141` (shipped): the recipe-output corpus guard.

### Overlap with other open work (from the PRs' file lists, `gh pr list --state open --json number,title,files`, 2026-11-11)

No pushed `feat/spec-15[4-8]*` branch (`git ls-remote --heads origin | grep -E "spec-15[4-8]"`: 0).
Open PRs touching files this spec touches:
- **#240** (ci: clippy on the wasm32 library build): `src/cli/ops.rs` (adds a
  `MANIFEST_DROPPED_WARNING` const at `:28` and rewrites three `eprintln!`s in `run_metadata_lane`,
  `:799-990`), plus `.github/workflows/ci.yml`, `justfile`, `src/metadata/mod.rs`. Different hunks
  from this spec's (`:214-490`, `:1040-1430`); the import block at `:15-27` is adjacent to its
  insertion, so a rebase may need a hand merge there.
- **#226** (docs(agents)): `src/cli/mod.rs` (the module doc `:1-12` and `dispatch`'s doc
  `:888-895`), plus `AGENTS.md`. This spec touches `:453-494`, `:898-925`, `:1167-1191`.
- **#237** (heifer): `CHANGELOG.md` (its own `[Unreleased]` lines). Both add bullets there; expect a
  textual conflict, resolved by keeping both.
- #229 touches `docs/roadmap.md` only: no overlap.
**Rebase onto `main` before each push and re-check this map after each rebase**
(`gh pr list --state open --json number,files`).

### Docs (each an AC-15 row; verify reads them)

- **`edit --help`** (`src/cli/mod.rs` doc comments): summary `One-shot multi-op on one image or a
  batch; optionally saves the recipe.` Flag lines:
  - `--pad <N|W:H>`: "Grow the canvas: N adds N pixels (of the image at this point in the command)
    on every side; W:H grows it to that aspect."
  - `--pad-color <COLOR>`: "The fill for --pad: \"transparent\", or a hex colour RRGGBB, #RRGGBB or
    RRGGBBAA. Required with --pad."
  - `--pad-gravity <GRAVITY>`: "Where the image sits in an aspect --pad W:H: center (default),
    north, south, east, west, northeast, northwest, southeast, southwest."
  - `--trim`: "Remove the outer rows and columns that match the top-left pixel (or --trim-color)."
  - `--trim-color <COLOR>`: "The colour --trim removes, instead of the top-left pixel:
    \"transparent\", or a hex colour RRGGBB, #RRGGBB or RRGGBBAA."
  - `--trim-tolerance <N>`: "How far a channel may differ from the colour and still be trimmed, in
    8-bit steps, 0 to 255 (default 0)."
- **`docs/cli-reference.md:300`** heading: add `[--pad N|W:H --pad-color C [--pad-gravity G]]
  [--trim [--trim-color C] [--trim-tolerance N]]` after `--flip AXIS`. **`:301`**: "Chain an
  ordered op list on one image or a batch (`--out-dir`), each in one decode → ops → encode pass."
  **`:306`** op flag list adds `--pad`, `--trim`. After **`:319`** two bullets:
  - "`--pad N` adds N pixels on every side, in pixels of the image at that point in the command; N
    may be at most that image's long edge, and a wider N or a canvas past 50000 px a side (or
    134217728 px in all) exits 1, naming the flag and the image's size. `--pad W:H` grows the
    canvas to that aspect, placed by `--pad-gravity` (aspect form only). `--pad-color` is required:
    `transparent` or hex with any alpha. A fill alpha the output format can't store (any below `ff`
    into JPEG; `01`-`fe` into GIF) is refused before anything is written: exit 4 for one input or
    for a batch whose format `--format` or the name template fixes; in a batch that keeps each
    input's format, that input fails and the run exits 6."
  - "`--trim` removes the outer rows and columns that match the top-left pixel, or `--trim-color`,
    within `--trim-tolerance` (8-bit steps, default 0). An image that matches all over exits 1; in a
    batch that input fails, the others are written, exit 6."
  **`:321-325`** save-recipe paragraph: "A pixel `--crop` is saved as fractions of the image the step
  received, and a border `--pad N` as `border`, N over that image's long edge; either needs one
  input image when `--save-recipe` is given (a directory or glob naming several exits 2; use an
  aspect `--crop W:H` or `--pad W:H`, or edit each image on its own)."
- **`docs/api-contract.md:541`** heading: the same flags as the cli-reference heading. **`:542`**:
  "One-shot multi-op on one image or a batch (`--out-dir`) — the "experiment like an editor" mode."
  **`:549-550`** op list adds `--pad`, `--trim`. **`:557-561`**: after the `--crop-gravity` sentence,
  "`--pad-color`/`--pad-gravity` without `--pad`, `--pad-gravity` with a border `--pad N`, and
  `--trim-color`/`--trim-tolerance` without `--trim` exit 2 before any file is read." After the
  pixel-crop sentence: "A border `--pad N` is in pixels of the image the step receives: N past that
  image's long edge, or a canvas past the size caps, exits 1. A `--pad-color` whose alpha the output
  format can't store exits 4 before anything is written (one input, or a batch whose format the
  flags fix), or fails that input in a batch that keeps each input's format (exit 6). `--trim` on
  an image that matches all over exits 1 (exit 6 in a batch). With `--save-recipe`, a border `--pad
  N` is saved as a fraction of the long edge and needs one input image, as a pixel `--crop` does."
- **`docs/recipes.md:216-219`**: the op list becomes `--resize-max`, `--invert`, `--crop`,
  `--rotate`, `--flip`, `--pad`, `--trim`; add "and a border `--pad N` as a fraction of that image's
  long edge" after the pixel-crop sentence.
- **`recipes/README.md:53-55`**: the same op list; "a pixel `--crop` or a border `--pad N` is saved as
  fractions of the image at that step."
- **`CHANGELOG.md` `[Unreleased]`**: below.
- **Not changed, checked:** `README.md` (`:34`, `:262` name `edit --save-recipe` with no op list),
  `skills/crustyimg/SKILL.md` (no `edit` op flags; `/usr/bin/grep -n "edit " skills/crustyimg/SKILL.md`
  hits `:145` and `:156`, both about metadata). A repo-wide `/usr/bin/grep -rln -e "--flip" -e
  "--crop-gravity"` outside `projects/`, `decisions/`, `src/`, `tests/` finds only the five files
  above and two `docs/probes/` scripts unrelated to `edit`.

**Behaviour sentences and every exit path each names** (build drives each on each path and records
the command and exit; verify drives them again):
1. "A `--pad-color` whose alpha the output format can't store exits 4 before anything is written":
   one input `-o o.jpg` (4, no file); one input JPEG source into `--out-dir` (4, no dir); batch
   `--format jpeg` (4, no dir); batch `--name-template {stem}.gif` with `#ff000080` (4, no dir);
   kept-format batch (6, `b.png` written, `a.jpg` not). Both pad forms on at least one path each.
2. "`transparent` into GIF is stored": batch `{stem}.gif` with `transparent` → 0.
3. "N past the long edge, or a canvas past the caps, exits 1": `--pad 5` on 4×4 (1); 2×49990
   `--pad 6` (1); 2×11000 `--pad 5000` (1); the same `--pad 5` in a batch with a 6×4 image (6, the
   6×4 written).
4. "`--trim` on an image that matches all over exits 1 (6 in a batch)": both paths.
5. "`--save-recipe` with a border `--pad N` needs one input image": directory (2, no dir, no
   recipe); one file (0); stdin `-` (0, recipe written).
6. "`--pad-gravity` with a border `--pad N` exits 2": one row.
7. CHANGELOG "a recipe's colour with a sign exits 1 on `apply` and `build`": `apply` (1) and `build`
   with a manifest naming that recipe (1). "`watermark --color` exits 2": one row.

### CHANGELOG, verbatim

Under `### Added`, after the `edit --flip` bullet:
```
- **`edit --pad`.** `--pad N` adds N pixels (of the image at that point in the command) on every
  side, and `--pad W:H` grows the canvas to that aspect, placed by `--pad-gravity`; `--pad-color`
  (hex with any alpha, or `transparent`) is required. `--save-recipe` saves `--pad N` as a fraction
  of the image's long edge, so it needs one input image. N wider than that long edge, or a canvas
  past the size caps, exits 1. A fill alpha JPEG or GIF can't store is refused: exit 4, nothing
  written, for one input or a batch whose format `--format` or the name template fixes; when each
  input keeps its own format, that input fails and the batch exits 6.
- **`edit --trim`.** `--trim` removes the border that matches the top-left pixel, or `--trim-color`,
  within `--trim-tolerance` (8-bit steps); `--save-recipe` saves a `trim` step with only the keys
  you gave. An image that matches all over exits 1; in a batch that input fails and the run exits 6.
```
Under `### Changed`:
```
- **A hex colour with a sign in it is refused.** `+f+f+f` used to load as `#0f0f0f`, and
  `+f+f+f+f` as a translucent `#0f0f0f0f`. A recipe's `color` (`pad`, `trim`, `watermark`) with a
  sign now fails to load (`apply` and `build` exit 1), and `watermark --color` exits 2, with the
  colour message.
```
Under `### Breaking`:
```
- **Library only: `IntroducedAlpha` has a `flag` field.** Code that builds one adds `flag: None`.
  The CLI is unchanged by it.
```

### DEC-115's ship-time replacement text (ship applies it after the merge; not now)

Part 4, first bullet, replace from "The pixel verbs through `run_pixel_op` check" to the bullet's end
with:
```
     The pixel verbs through `run_pixel_op` check a **forced** format once, before any input is
     read. With no forced format, and only when a step introduces alpha, they check a format the
     flags fix for every input of a batch (`--format`, or the name template's literal extension)
     once before any image is read, and otherwise each input's own format after it loads and
     before its pipeline runs. `edit --pad` is the caller that reaches these checks, and its line
     names the flag (`--pad-color "transparent" has alpha 00, …`) instead of a step.
```
Part 8, append: "A step built from a CLI flag names that flag and its value instead of the step
index, op and key, since the user typed no step."

Neutral consequence, replace the parenthesis "(`run_pixel_op` still runs the pipeline first and
resolves the format after)" with "(`run_pixel_op` still runs the pipeline first and resolves the
format after, unless a step introduces alpha)".

References, replace "SPEC-160 (`edit --pad`, which adds `run_pixel_op`'s per-input check for an
unforced format)" with "SPEC-160 (`edit --pad`: `run_pixel_op`'s checks for an unforced format, and
the flag-named line)".

`affected_scope`: already lists `src/cli/ops.rs`, `src/cli/mod.rs` and `src/operation/alpha.rs`; no
path to add (`src/text/mod.rs` is the sign fix, not this rule).

### Mutation table

Applied to the prototype one at a time; the whole Failing Tests list plus `cargo test --release
--lib` and `--test edit_geometry --test watermark` re-run (`--no-fail-fast`). The red set is read off
that run. `no_op_flag_message_lists_every_flag` is red in every row on the prototype because its
string was not yet updated there (FT-18); it is left out of each red set below. "Green" means every
other test in those four targets.

- **M1 `PixelPad` doesn't forward its alpha** (`ops.rs`, `impl Operation for PixelPad`): delete
  the `fn introduced_alpha(&self) -> Option<IntroducedAlpha> { self.alpha.clone() }` override.
  Red: FT-5, FT-6, FT-7. *Driven.* (The black-border JPEG, exit 0, the frame predicted.)
- **M1b `FlagWorded` doesn't forward alpha** (aspect form): delete its `introduced_alpha` override.
  Red: FT-5, FT-6. *Driven.* (FT-7 uses `--pad 1`, the border form, so it stays green.)
- **M1c `FlagWorded` forwards alpha without the flag**: `flag: self.flag_for(a.key),` →
  `flag: None,`. Red: FT-5, FT-6 (stderr reads `recipe step 2 ('pad'): 'color' = …`). *Driven.*
- **M1d `PixelPad` alpha without the flag**: `flag: Some("--pad-color"),` → `flag: None,` in the
  `Border` arm. Red: FT-5, FT-6, FT-7. *Driven.*
- **M2 batch-wide pre-check removed** (`run_pixel_op`, batch branch): delete the `if let Some(fmt)
  = format_fixed_by_flags(global, None, Some(&template))? { … }` block. Red: FT-6 (exit 6 per input
  instead of 4 once). *Driven.*
- **M3a per-input check removed, one input**: replace the single-branch `let fmt =
  output_format_for(…)?; if let Some(refused) = … { return Err(…) }` with `let _ = (output_path,
  template);`. Red: FT-5. *Driven.*
- **M3b per-input check removed, batch**: replace the batch closure's `if let Some(refused) =
  pipeline.unstorable_alpha(fmt) { return Err(refused.into()); }` with `let _ = fmt;`. Red: FT-7.
  *Driven.*
- **M4 gate removed**: `let alpha_check = forced_format.is_none() && pipeline.introduces_alpha();`
  → `let alpha_check = forced_format.is_none();`. Red: FT-8 (the `--format bogus` rows exit 4).
  *Driven.*
- **M5 batch `--save-recipe` check not widened**: `.find(|s| matches!(s.record,
  StepRecord::Resolved(_)))` → `… && s.op.name() == "crop")`. Red: FT-12. *Driven.*
- **M6a `from_str_radix` back on the raw pair** (`text/mod.rs`): delete the `if
  !hex.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(bad()); }` block. Red: FT-13, FT-16,
  `parse_color_refuses_non_ascii` (it panics slicing `aéaaa`). *Driven.*
- **M6b the ASCII-hex test replaced by `is_ascii()`**: `if !hex.bytes().all(|b|
  b.is_ascii_hexdigit()) {` → `if !hex.is_ascii() {`. Red: FT-13, FT-16. Green:
  `parse_color_refuses_non_ascii`. *Driven.*
- **M7 `--trim`'s whole match left in key form**: replace `FlagWorded::apply`'s `for (key, flag) in
  self.keys { reason = reason.replace(…); }` with `let _ = self.keys;`. Red: FT-10, FT-9 (the aspect
  cap row reads `'aspect' = "65535:1"`). *Driven.*
- **M8a `pad` missing from `edit_op_order`'s table** (`mod.rs`): delete `("pad", EditOp::Pad),`.
  `--pad` is parsed and silently dropped: alone it gives `edit requires at least one operation
  flag …`, beside another op flag the pad step is skipped with exit 0. Red: FT-1, FT-2, FT-3, FT-4,
  FT-5, FT-6, FT-7, FT-8, FT-9, FT-11, FT-12, FT-13, FT-14. *Driven.*
- **M8b `trim` missing from the table**: delete `("trim", EditOp::Trim),`. Red: FT-1, FT-3, FT-4,
  FT-10, FT-11, FT-12, FT-13, FT-14. *Driven.*
- **M9 the fraction divides by the short edge**: `f64::from(self.n) / f64::from(long)` →
  `f64::from(self.n) / f64::from(iw.min(ih))`. Red: FT-1, FT-2, FT-3 (sizes and `border`
  literals). *Driven.*
- **M10a `north` placed as `south`** (`Gravity::placement`): `Gravity::North => (x_center, y_top),`
  → `Gravity::North => (x_center, y_bottom),`. Red: FT-17, `crop::tests::aspect_gravity_places_all_nine`,
  `pad::tests::gravity_places_by_absolute_position`. *Driven.* (Before FT-17, no `watermark` test
  went red: the stage entry's finding.)
- **M10b bottom margin dropped**: `let y_bottom = bh.saturating_sub(oh).saturating_sub(margin);` →
  `let y_bottom = bh.saturating_sub(oh);`. Red: FT-17,
  `operation::tests::watermark_margin_offsets_anchor`. *Driven.*
- **M10c left margin dropped**: `let x_left = margin;` → `let x_left = 0;`. Red: FT-17 only (no
  existing test sees it). *Driven.*
- **M12 no long-edge check**: `if self.n > long {` → `if false && self.n > long {`. Red: FT-9 (the
  op's own `'border' must be a number > 0 and <= 1, got 1.25` leaks out). *Driven.*
- **M13 the edge cap checks width only**: `if cw > u64::from(MAX_EDGE) || ch > u64::from(MAX_EDGE)
  {` → `if cw > u64::from(MAX_EDGE) {`. Red: FT-9 (the 2×49990 row reaches `pad`'s own
  `'border' = …` line). *Driven.*
- **M14 `FlagWorded` drops `refusal`**: its `refusal` returns `None`. Green, all. *Driven.* No
  splitting input exists: neither wrapped op refuses (`/usr/bin/grep -n "fn refusal(" src/operation/pad.rs
  src/operation/trim.rs`: 0 hits). Kept as a forward for a later wrapped op; not a test target.
- **Trap 14, `main`'s `src/cli/ops.rs` and `src/cli/mod.rs` restored verbatim** (the rest of the
  prototype kept): every one of FT-1 to FT-13 goes red (clap refuses `--pad`/`--trim`, exit 2; FT-8 on its
  `--pad 9` row), FT-17 and all 23 `edit_geometry.rs` tests stay green, and the lib's unit tests do
  **not compile** (`error[E0063]: missing field 'flag' in initializer of 'alpha::IntroducedAlpha'`,
  from `main`'s `mod.rs` test), so FT-14 to FT-16 can't run. *Driven.* The build records this as the
  RED baseline for the restored files.

### Build checkpoints and CI

- Commit and push after each of: (1) the failing tests, RED recorded; (2) `parse_color` + FT-16; (3)
  `EditOp`/clap/`build_edit_ops`/wrappers + FT-1 to FT-4, FT-9 to FT-14; (4) `run_pixel_op`'s checks
  + `alpha.rs` + FT-5 to FT-8, FT-15; (5) docs, CHANGELOG, FT-17, FT-18; (6) the mutation table
  re-driven. Draft PR from the first push.
- `run_pixel_op` is shared code: before the last push run a **clean** full matrix, the lean leg
  included (`cargo test --no-default-features`), each in its own `CARGO_TARGET_DIR`. `just check`
  doesn't run `just wasm-test`; this spec changes `alpha.rs` (compiled for wasm) and `parse_color`,
  so run `just wasm-check` and `just wasm-test` once too, and read CI's wasm leg. No count in
  `scripts/wasm-ci-check.sh` changes (no wasm test added).
- Don't wait on CI in a loop (AGENTS §16 habit 6).

### Out of scope (for this spec specifically)

- `edit --trim[=COLOR]` (an optional-value flag): clap swallows the input path (SPEC-153's frame).
- A CLI-only default `--pad-color`, and named colours (`red`): none; hex or `transparent` only.
- `--pad WxH` (pad to an exact pixel size): not a `pad` form.
- `--pad N` wider than the long edge as a recipe: `border` stays at most 1 (DEC-112's table).
- Hoisting `output_format_for` above the pipeline for every `run_pixel_op` caller: it would change
  existing exit codes (FT-8).
- A wasm test row for the sign (see "wasm").
- `crop`/`pad` aspect following each image's orientation (moved to STAGE-060).
- The "watermark skipped" batch message (`refusal-summary-names-the-op`, ideas.yaml).
- `apply`/`build` warning on a tagged input with no `auto-orient` step (ideas.yaml).
- The recipe-output guard reporting two failures together (ideas.yaml).
- Any DEC: none is written. DEC-115 gets the wording above at ship.

## Notes for the Implementer

- Copy `PixelCrop`'s shape for `PixelPad` (the `Rc<RefCell<Option<OperationParams>>>`, set only after
  a successful apply). `run_edit`'s save loop reads it unchanged.
- `IntroducedAlpha` is built with struct-update syntax in both wrappers (`IntroducedAlpha { flag:
  …, ..a }`), so a later field is carried through.
- The stand-in `{border = 1.0, color}` op in the `Border` arm exists only to run the colour through
  `pad`'s own key table and parser before any file is read, and to read its `introduced_alpha`. It
  is dropped after.
- Keep `FlagWorded`'s rewrite to `'key' = ` → `flag ` exactly; `build_flag_op`'s load-time swap
  (`'key'` → `flag`, first occurrence) is unchanged.
- In FT-13 pass colours as `--flag=VALUE`: `-f-f-f` after a space is clap's `unexpected argument`.
- `framed()` must write its frame through a helper that knows each colour type (16-bit as c × 257);
  `image`'s `DynamicImage::put_pixel` would round-trip through RGBA8.
- Comments and messages read behaviour-first: no spec, stage or DEC id in code or output.

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
