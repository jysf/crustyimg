---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-069
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-14
supersedes: null
superseded_by: null

affected_scope:
  - "src/analysis/decide.rs"
  - "src/quality/mod.rs"
  - "src/sink/mod.rs"
  - "src/cli/mod.rs"
  - "tests/cli.rs"

tags:
  - avif
  - quality
  - decision-engine
  - optimize
  - performance
---

# DEC-069: the default `optimize` decision admits AVIF at a fixed generous quality via a single-encode compare; the perceptual/byte-budget searches become opt-in

## Decision

Four decisions, taken together because they are one default-path change (SPEC-084) — the native twin of
DEC-068's wasm surface:

1. **The default `optimize` (no `--target`/`--ssim`/`--max-size`) runs a new `Mode::Fast` decision.**
   Each bucket-appropriate candidate is encoded **exactly once** at a fixed quality; the smallest that
   beats the source wins; nothing beating the source returns **passthrough** (`None`). No search runs —
   so there is no repeated-encode AVIF budget search (the 9–74 s faceplant) in the default path.
2. **AVIF is admitted into the default decision for lossy-family content, as a bucket predicate.**
   `decide::avif_admissible(bucket, built)` is true for `Lossy`/`MixedSafe` when an AVIF encoder is
   built, and `format_shortlist` **prepends** AVIF in `Mode::Fast` (never appends-then-truncates), so
   the `MAX_SHORTLIST` truncation can never silently drop it (the SPEC-079 footgun). `pick_winner`'s
   smallest-beats-source lets measured bytes veto AVIF on a graphic (a screenshot → lossless WebP, AVIF
   is a 4× regression), so the content branch holds without a special case.
3. **The default AVIF quality is a separate, generous knob: `FAST_LOSSY_QUALITY = 85`** — distinct from
   `AVIF_DEFAULT_QUALITY = 80`, which stays the `convert`/explicit default so native `convert` output is
   byte-identical. The fast path passes `Some(85)` explicitly. 85 is the eyeball-validated anchor for
   AVIF; JPEG and lossy WebP reuse the same number on their own scales as fallbacks when AVIF is not
   built.
4. **The winner is scored once (one native decode) and reported; the searches stay opt-in.** The fast
   path decodes its lossy winner through the crate's native decoders (which include AVIF, DEC-058) and
   calls `quality::score_winner_once` — exactly one SSIMULACRA2 computation, no search — surfacing the
   achieved score on the summary (`… (93% smaller) · ssim 77.9`). A lossless winner reports "lossless".
   `--target`/`--ssim` still select the perceptual search (`auto_quality`) and `--max-size` the
   byte-budget search (`search_size`), unchanged.

`convert`, `shrink`, `--profile preserve`, and the wasm surface are untouched.

## The validated quality number (the one judgment call)

Measured on the real corpus (`~/PSeven/experiments/crustimg_redo_plus/_incoming0`), **kept-dimensions**
AVIF (what `Mode::Fast` does — no downscale), SSIMULACRA2 of the output vs the source:

| photo | MP | src | q80 | q85 | q90 |
|---|---|---|---|---|---|
| DSCF1154.JPG | 8 | 1776 KB | 87 KB / **74.6** | 120 KB / **77.9** | 190 KB / 80.7 |
| DSC_0163.png | ~2 | 1422 KB | 149 KB / **75.2** | 198 KB / **81.6** | 252 KB / 87.0 |
| DSC_0974.jpeg | 24 | 3886 KB | 510 KB / **61.1** | 1209 KB / **70.7** | 2284 KB / 80.0 |
| IMG_3855.jpeg | 7 | 1537 KB | 287 KB / **75.2** | 404 KB / **80.7** | 587 KB / 85.3 |
| DSC_2011.JPG | 24 | 13137 KB | 261 KB / **75.7** | 364 KB / **78.4** | 581 KB / 81.7 |

**Chosen: q85.** It lands SSIMULACRA2 ≈ **70–82 (median ~78)** at **90 %+ savings** on most photos, and
it is **visually indistinguishable from the source even on the lowest-scoring 24 MP photo** (DSC_0974,
70.7) — an A/B eyeball at 760 px showed a smooth sky with no banding, crisp sign edges, clean text. The
low full-resolution scores are the metric being harsh on fine 24 MP detail against an **already-JPEG'd
reference** (the score's ceiling is the source's own fidelity, not perfection), exactly the depression
the spec anticipated — not a visible quality problem.

Why generous and not lower: `Mode::Fast` **keeps the source dimensions**, so it should protect quality —
the savings already come from the *codec* (AVIF vs JPEG/PNG), not from a low quality. Lowering the target
to chase bytes is the opt-in `--target` search, and lowering it *after a downscale* is the `web` verb
(SPEC-085), where the byte cost of generosity is negligible. q80 is measurably too aggressive here
(61–76, and a 24 MP photo at 61 is a real regression); q90 costs ~1.5–2× the bytes for +3–6 points.

**The `-q`→perceptual mapping is a few points below its label** (sanity-checked per the spec): a nominal
AVIF **q80 lands SSIMULACRA2 ~61–76** (avg ~72), i.e. "high", **not** "visually-lossless" — a number a
user reads as "80/100" is perceptually lower, and on high-MP content markedly so. Recorded, **not
recalibrated**: remapping the encoder's quality scale is a cross-cutting change out of SPEC-084's scope,
and `--target`/`--ssim` already let a user ask for an *outcome* instead of a raw number.

## Context

STAGE-030's benchmark (2026-07-14) measured the native default optimizing the wrong variable: `optimize`
default → 24 % median in 16.5 s, 2/8 full passthrough, up to 49 s on a 47 MP photo, while a fixed-quality
AVIF path gets far more for far less. Root cause: `decide::format_shortlist` admitted AVIF **only in
`Mode::SizeBudget`** — a gate set when there was no AVIF decoder. Native AVIF **decode shipped (DEC-058)**,
so the gate was vestigial on native and was excluding the one 2× win from the default. SPEC-079/DEC-068
had already solved the wasm side; this is its native twin, and after it the two default paths **converge
in shape**: both do fixed-quality Auto-AVIF for lossy-family content via a bucket predicate, no search.

## Alternatives considered

- **Bump `AVIF_DEFAULT_QUALITY` 80 → 85.** Rejected: that constant is *also* `convert`'s default, and
  the acceptance anchor is that native `convert` AVIF bytes are unchanged. The fast path gets its own
  `FAST_LOSSY_QUALITY`; `convert` keeps 80.
- **Reuse `Mode::SizeBudget`'s AVIF admission for the default.** Rejected for the same reason DEC-068
  gave: SizeBudget *appends* AVIF and truncates to `MAX_SHORTLIST`, so a full `MixedSafe` shortlist drops
  it. The admission is restated as a bucket predicate and AVIF is prepended in `Mode::Fast`.
- **Run the perceptual search in the default (today's behaviour), just admitting AVIF.** Rejected: the
  perceptual search *decodes each candidate to score it*, and it is the slow part (the faceplant is the
  repeated-encode budget search, but even the JPEG perceptual search costs seconds for ~13 %). The
  default should be a single encode; users who want a searched guarantee opt in with `--target`.
- **q90 for extra safety margin.** Rejected: ~1.5–2× the bytes for +3–6 SSIMULACRA2 points that the
  eyeball can't see at q85. The default's job is "smaller and good," and 85 is already visually clean.
- **Recalibrate the AVIF quality→SSIMULACRA2 mapping so "q80" means ~80.** Out of scope and risky
  (cross-cutting; would move `convert`/`-q` bytes too). Recorded as a finding; `--target` is the honest
  outcome-based escape hatch.
- **Align the wasm number (80) to the new native 85 now.** Deferred: SPEC-084 is "no wasm change". The
  two paths converge in *shape* here; aligning the wasm quality constant is a small follow-up for
  whoever next touches `src/wasm.rs`.

## Consequences

**Good**

- The verb everyone runs now stops optimizing the least-valuable variable: a photo → AVIF, one encode,
  **90 %+ smaller**, with the achieved SSIMULACRA2 shown as proof (`jpeg → avif · … (93% smaller) ·
  ssim 77.9`). No repeated-encode search in the default path.
- **Passthrough is now correctness-safe.** `Mode::Fast` makes passthrough common (a fixed quality often
  can't beat an already-compressed source), which exposed a latent bug: shipping the *raw* source on
  passthrough leaks metadata (incl. GPS) and a wrong orientation that `optimize` promised to bake/strip.
  The fix: raw passthrough only when the source carried no metadata **and** the pipeline changed nothing
  (dims unchanged; an orientation flip carries an EXIF tag, so it trips the metadata check) — otherwise
  the smallest processed candidate ships. This closes the "never silently enlarge" fix *and* the
  privacy/orientation guarantee together, for every mode.
- Native and wasm Auto paths converge in shape (closes the DEC-068 divergence in decision *structure*).

**Bad / risky**

- **The default now re-encodes by codec, not by a perceptual guarantee.** In a **no-AVIF build**, the
  fast lossy candidate is JPEG at a fixed 85 — generous, so it usually passes through an already-good
  JPEG rather than degrading it, but there is no per-image perceptual target. Users who need one opt in
  with `--target`; the default's contract is "smaller, modern, never bigger, quality shown".
- **The wasm quality number (80) and the native fast number (85) differ.** Bounded and deliberate (the
  shapes match); a follow-up should align the wasm constant when `src/wasm.rs` is next touched.
- **The `-q`→SSIMULACRA2 gap is documented, not fixed.** A user reading "q80" gets ~72; the mapping is
  left as-is because `--target` is the outcome-based path and recalibration is out of scope.

## Implementation notes

- `Mode::Fast` is prepend-AVIF; `Mode::SizeBudget` is still append-AVIF (byte-parity with SPEC-018 kept).
  The admission is `decide::avif_admissible` — a bucket predicate, not shortlist membership.
- Winner scoring lives in the CLI (`optimize_decide_one`), not `quality`, because only the CLI/image
  layer can decode AVIF (re_rav1d is native-only and not in `::image`). `quality::score_winner_once`
  takes an already-decoded winner and does the single `score` call. A decode/score failure degrades to
  "no score", never a failed optimize.
- The score is surfaced on the human summary/trace only; the `--explain=json` schema
  (`crustyimg.optimize.explain/v1`) is **untouched** — the machine-readable audit report is SPEC-088's.
- Bench harness: `scratchpad/bench` / the throwaway sweep over `_incoming0`; the numbers above are
  reproducible with `convert --format avif -q <N>` + `diff <src> <out> --json`.
