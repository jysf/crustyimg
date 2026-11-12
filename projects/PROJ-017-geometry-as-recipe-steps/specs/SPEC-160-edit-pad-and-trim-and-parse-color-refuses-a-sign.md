---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-160
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
  created_at: 2026-11-11

references:
  decisions: [DEC-095, DEC-106, DEC-112, DEC-114, DEC-115]
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
  totals:
    tokens_total: 5656151
    estimated_usd: 4.79
    session_count: 1
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

- `edit --trim[=COLOR]` (an optional-value flag): clap swallows the input path (SPEC-153's frame).
- A CLI-only default `--pad-color`: none; the flag is required.
- `--pad WxH` (pad to an exact pixel size): not a `pad` form; a recipe has only `border` and `aspect`.
- `--pad N` with `--save-recipe` on a batch: refused, as a pixel `--crop` is (DEC-114 part 6).
- Hoisting `output_format_for` above the pipeline for every `run_pixel_op` caller: it would change existing exit codes (driven). The alpha check is gated on `introduces_alpha()`.
- `crop`/`pad` aspect following each image's orientation (STAGE-058 entry; recommended to STAGE-060).
- The "watermark skipped" batch message (neither `pad` nor `trim` uses the refusal hook).
- `apply`/`build` warning on a tagged input with no `auto-orient` step.
- The recipe-output guard reporting two failures together.
- `watermark`'s nine gravities with a margin, unless the maintainer folds it (stage-closing item 5).
- Any DEC: none is written. DEC-115 part 4 gets a wording follow-up at ship.

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
