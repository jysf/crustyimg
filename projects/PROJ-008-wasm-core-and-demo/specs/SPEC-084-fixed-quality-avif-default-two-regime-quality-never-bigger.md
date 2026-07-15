---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-084
  type: story
  cycle: design            # framed build-ready (draft adopted from the strategy session, validated)
  blocked: false
  priority: high
  complexity: M
project:
  id: PROJ-008             # folded into PROJ-008 as a pre-launch stage (maintainer decision 2026-07-14)
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-14
references:
  decisions: [DEC-016, DEC-019, DEC-020, DEC-048, DEC-058, DEC-059, DEC-068]
  constraints: [pure-rust-codecs-default, no-unwrap-on-recoverable-paths, untrusted-input-hardening,
                every-public-fn-tested, test-before-implementation, ergonomic-defaults,
                no-new-top-level-deps-without-decision]
  related_specs: [SPEC-079]
value_link: >
  The engine foundation for STAGE-030: make the DEFAULT format decision fast and AVIF-aware so the
  verb everyone runs stops optimizing the least-valuable variable. Measured: today's default gets 24%
  median in 16.5 s (2/8 passthrough, 49 s on 47 MP); a downscale+AVIF path gets 98% in 2.7 s. SPEC-085
  (web) and SPEC-086 (optimize) consume this; SPEC-079 already shipped the wasm twin.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-084: fixed-quality AVIF in the default path + two-regime quality + never-bigger

## Context

A measured benchmark (2026-07-14, real 8-photo corpus 0.7–47 MP + an AVIF quality sweep) found the
native default format decision optimizes the wrong variable:

- `optimize` default (perceptual, visually-lossless, format-preserving-ish) → **24% median savings in
  16.5 s**, 2/8 full passthrough, up to **49 s** on a 47 MP photo.
- A downscale→AVIF path → **98% median in 2.7 s** (size-insensitive — it downscales first).
- The catastrophic times are the **byte-budget AVIF search** re-encoding rav1e (9–74 s), NOT
  perceptual scoring itself.

Root cause in the engine: `decide::format_shortlist` admits AVIF **only in `Mode::SizeBudget`**
(`src/analysis/decide.rs` ~line 152) — a gate set when there was no AVIF decoder. Native AVIF
**decode shipped (DEC-058)**, so the gate is now vestigial on native, and it's excluding the only 2×
win from the default path. SPEC-079 already solved the wasm side (`optimize_detailed`: fixed-quality
Auto-AVIF, no search, DEC-068); **this spec is its native twin** — engine only, no CLI surface, no
`web` verb (SPEC-085), no `optimize`/`shrink` surface work (SPEC-086).

## Goal

Give the engine a **fast default decision**: for a lossy-family (photographic) input, admit **AVIF at
a fixed, validated, generous quality via a single-encode compare** (never the repeated-encode search);
for a graphic input keep the existing lossless branch; pick smallest-beats-source with the clear-win
guard; **score the winner once** (native decode) for the report; and return a first-class
**passthrough** when nothing beats the source. The perceptual **search** stays available but becomes
**opt-in** (`--target`/`--ssim` → the existing `auto_quality`; `--max-size` → `search_size`). Native
`convert` byte-output and the wasm surface are unchanged.

## Inputs — files to read

- `src/analysis/decide.rs` — `format_shortlist` (the `Mode::SizeBudget` AVIF gate, ~152), `pick_winner`
  (smallest-beats-source + clear-win + never-bigger `None`), `Mode`, `OptBucket`, `BuiltCodecs`, `MAX_SHORTLIST`.
- `src/quality/mod.rs` — `auto_quality` → `QualityChoice { quality, score }` (~148), `search_size`
  (~455), public `score()` (~99).
- `src/sink/mod.rs` — `encode_to_bytes_with(img, fmt, quality, speed)` (added by SPEC-079),
  `AVIF_SPEED = 6` (~48), `AVIF_DEFAULT_QUALITY = 80` (~54).
- `src/cli/mod.rs` — `optimize_decide_one` (~4145), `solve_candidate` (~4081), the `AutoQuality`
  enum + `Mode` mapping (~4174) — where the default vs `--target`/`--max-size` paths are chosen.
- SPEC-079 (`src/wasm.rs` `optimize_detailed`) — the wasm twin of this decision; mirror its shape.

## Outputs

- **`src/analysis/decide.rs`**
  - Admit AVIF into the **default (non-search) decision** for lossy-family buckets when `built.avif`
    — as a **fixed-quality single-encode candidate**, not a searched one, and independent of the
    `MAX_SHORTLIST` truncation footgun SPEC-079 flagged (predicate on the bucket, not shortlist
    membership).
  - A **default decision mode** (e.g. `Mode::Fast`) distinct from `Perceptual` (search) and
    `SizeBudget`: encode each bucket-appropriate candidate **once** at its default quality
    (lossy-family → AVIF + lossy-WebP(where built) + JPEG; graphic → lossless WebP/PNG), pick
    smallest that **beats the source**, return `None` (passthrough) if none do. Preserve the
    content branch (graphic → lossless; validated: screenshot → lossless WebP, AVIF is a 4× regression).
- **`src/quality/mod.rs`** — a helper to **score the chosen winner once** (native decode via the
  public `score()`), so the default path reports the achieved SSIMULACRA2 without a search. No change
  to `auto_quality`/`search_size` (the opt-in paths).
- **`src/sink/mod.rs`** — set the **validated default AVIF quality** (see Notes; recommend ~85, in
  [80,90], pending the eyeball pass). If the `-q`→perceptual mapping is confirmed aggressive, record
  the recalibration here. Reuse `encode_to_bytes_with` (no new encode entry).
- **`src/cli/mod.rs`** — route the default `optimize` (no `--target`/`--ssim`/`--max-size`) through the
  new `Mode::Fast` decision; the flags still select `Perceptual`/`SizeBudget`. No verb rename here
  (SPEC-086); `--profile preserve` still the engine-off anchor.
- **New decision `DEC-069`** — the default decision admits AVIF at fixed quality via single-encode
  compare; the perceptual/byte-budget searches are opt-in; the default AVIF quality value + rationale;
  the two-regime quality note; the native/wasm Auto convergence (closes the SPEC-079 divergence).

## Acceptance Criteria

- [ ] The default decision on a **photographic** input picks **AVIF** (fixed quality, single encode)
      and beats the source substantially, with **no repeated-encode search** — one AVIF encode, not the
      9–74 s budget search. (Drive a real corpus photo at fixed dimensions.)
- [ ] The default decision on a **graphic/screenshot** input stays **lossless** (WebP/PNG), never
      AVIF (the content branch holds).
- [ ] When nothing beats the source, the decision returns **passthrough** (`None` = keep original) —
      never a larger file (subsumes the "never silently enlarge" fix).
- [ ] The chosen winner carries a **reported SSIMULACRA2 score** (one native decode), for lossy
      outputs; lossless outputs report "lossless."
- [ ] `--target`/`--ssim` still run the **perceptual search** (`auto_quality`), unchanged; `--max-size`
      still runs the **byte-budget search** (`search_size`), unchanged.
- [ ] **Native `convert` byte-output is unchanged** (fixed-quality encodes go through
      `encode_to_bytes_with` at the documented defaults); `--profile preserve` still keeps the source
      format (DEC-059 anchor); **the wasm surface is unchanged** (SPEC-079 owns it).
- [ ] The default AVIF quality is **validated on a small diverse corpus** (eyeball + the q-sweep data),
      not hardcoded blind; the value + the score it lands are recorded in DEC-069.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
      `cargo build --no-default-features` (lean) all pass.

## Failing Tests (written at design)

- **`src/analysis/decide.rs`**
  - `default_decision_admits_avif_for_photo` — a lossy-family bucket in the default mode includes AVIF
    and picks it when it clear-wins.
  - `default_decision_keeps_graphic_lossless` — a graphic bucket picks lossless, never AVIF.
  - `default_decision_passthrough_when_nothing_beats_source` — an already-optimal input returns `None`.
  - `avif_admission_survives_shortlist_truncation` — the AVIF predicate is on the bucket, not shortlist
    position (guards the SPEC-079 `MAX_SHORTLIST` footgun).
- **`src/sink/mod.rs`**
  - `convert_avif_bytes_unchanged_at_default` — `encode_to_bytes(img, Avif, None)` bytes == pre-spec
    (native `convert` regression anchor).
- **`src/quality/mod.rs`**
  - `default_winner_is_scored_once` — the default path reports a score in `(0,100]` for a lossy winner
    without invoking `auto_quality`'s search (assert via a call-count/seam, not timing).
- **Integration (native CLI, `--features avif`)**
  - `optimize_default_photo_picks_avif_single_encode` — a real corpus photo → AVIF, smaller, and the
    run does a single AVIF encode (no budget search).
  - `optimize_target_flag_still_searches` / `optimize_max_size_still_budget` — the opt-in paths intact.

## Implementation Context

### Decisions that apply
- `DEC-048` — the decision engine (`format_shortlist`/`pick_winner`, clear-win + never-bigger). This
  is a **narrow widening** (admit AVIF in the default mode at fixed quality), not a re-architecture.
- `DEC-058` — native AVIF decode; what makes scoring an AVIF winner (and thus admitting it) sound.
- `DEC-020` — AVIF speed stays 6 on native (no `--speed` CLI flag); this spec touches the quality
  default, not speed. `DEC-016`/`DEC-019` — byte-parity: fixed-quality candidates must equal sink bytes.
- `DEC-059` — `--profile preserve` remains the keep-source-format / engine-off anchor.
- `DEC-068` (SPEC-079) — the wasm twin; the Auto-AVIF-for-photos rule + the score-truth-table. After
  this spec the native and wasm default paths **converge** — name this in DEC-069 (closes the
  SPEC-079 follow-up divergence).

### The two-regime quality (the one judgment-bound number)
Measured q-sweep (resize→2048, SSIMULACRA2 vs ref): bytes are **trivial across the whole q range**
(15–116 KB from 12–47 MP), and q80 lands **~75–79 ("high", not visually-lossless)**, q90 ~82–87.
Therefore the default can be **generous** — set it high (recommend ~85), because after a downscale the
byte cost of generosity is negligible, and it keeps the "we can prove the quality" pitch honest. The
**lower-the-target-for-savings** logic belongs to the keep-dimensions `optimize` path (via `--target`),
NOT the default. Validate the exact value on eyeballs; also **sanity-check the `-q`→perceptual mapping**
(q80→~77 is more aggressive than the label implies; scores are partly depressed by already-JPEG'd refs).

### Constraints
- `pure-rust-codecs-default` — no new dep; AVIF encode/decode already built.
- `no-unwrap-on-recoverable-paths` / `untrusted-input-hardening` — decode caps (DEC-034/063) carry.
- `ergonomic-defaults` — the default must be the thing a photo-dropper wants (smaller, modern,
  never bigger); passthrough is a green result, not a failure.

### Out of scope (this spec)
- The `web` verb, resize defaults, bundled recipes (SPEC-085).
- `optimize` surface redefinition, `--verify`, `shrink` removal (SPEC-086).
- Any wasm/`src/wasm.rs` change (SPEC-079 owns it) or CLI `--speed` flag (DEC-020 stands).
- The unified audit report / `--json`/`--timing` (SPEC-088).

## Notes for the Implementer
- **Keep the seam thin.** The default decision is `Analysis::compute → format_shortlist (default mode)
  → encode each candidate once via encode_to_bytes_with → pick_winner → score winner once`. Don't
  re-implement encode or the search.
- **AVIF admission is a bucket predicate**, not shortlist membership (SPEC-079's lesson: `MAX_SHORTLIST`
  truncation would silently drop a last-appended AVIF).
- **Prove native `convert` byte-identity** before touching anything (the regression anchor).
- **Verify will drive the real corpus** (`~/PSeven/experiments/crustimg_redo_plus/_incoming0`, harness
  in `scratchpad/bench/`): default→AVIF for photos, lossless for the screenshot, passthrough for an
  already-optimal JPEG, and the chosen default quality's score sane — so acceptance is driven, not
  just unit-tested.

---

## Build Completion
- **Branch:** `spec-084-fast-avif-default`
- **PR (if applicable):** (opened; see PR link in the ship note)
- **All acceptance criteria met?** yes
  - Default → AVIF single-encode for a photo (verified on the real corpus: `jpeg → avif · (93% smaller)`,
    one encode, no budget search) ✓
  - Graphic/screenshot stays lossless, never AVIF (bucket predicate + `pick_winner`) ✓
  - Nothing-beats-source → passthrough (`None`), never larger — and the passthrough is now
    orientation/metadata-safe (a latent raw-passthrough leak Fast mode exposed) ✓
  - Winner carries a reported SSIMULACRA2 (one native decode); lossless reports "lossless" ✓
  - `--target`/`--ssim` still search perceptually; `--max-size` still runs the byte-budget search ✓
  - Native `convert` AVIF bytes unchanged (`AVIF_DEFAULT_QUALITY = 80` untouched; anchor test); wasm
    surface untouched; `--profile preserve` still keeps source format ✓
  - Default AVIF quality validated on the corpus (eyeball + q-sweep) and recorded in DEC-069 ✓
  - `cargo test` (default + `--features avif`), `cargo clippy` (both), `cargo fmt --check`,
    `cargo build --no-default-features` all pass ✓
- **New decisions emitted:**
  - `DEC-069` — the default `optimize` decision admits AVIF at a fixed generous quality (`FAST_LOSSY_QUALITY
    = 85`) via a single-encode compare; the perceptual/byte-budget searches become opt-in. Includes the
    validated q-sweep, the eyeball note, and the `-q`→SSIMULACRA2 aggressiveness finding.
- **Deviations from spec:**
  - **Passthrough is now correctness-safe, not raw-only.** The spec said passthrough = "keep original".
    `Mode::Fast` makes passthrough common, which exposed that shipping raw source bytes on passthrough
    leaks metadata (GPS) and a wrong orientation `optimize` promised to bake/strip — breaking two existing
    tests and the privacy guarantee. Passthrough now ships raw only when the source had no metadata **and**
    the pipeline changed nothing; otherwise it ships the smallest processed candidate. Applies to all modes.
  - **`AVIF_DEFAULT_QUALITY` was NOT bumped to 85.** The spec's sink note says "set the default AVIF
    quality (~85)", but that constant is also `convert`'s default and the acceptance criterion pins
    `convert` bytes unchanged. Resolved by a *separate* `FAST_LOSSY_QUALITY = 85`; the `None`/`convert`
    default stays 80.
  - **The winner-scoring compose lives in the CLI, not `quality`.** `quality` may not depend on
    `crate::image`, and only that layer can decode AVIF (re_rav1d, native-only, absent from `::image`).
    `quality::score_winner_once` does the single `score` call on an already-decoded winner; the CLI decodes.
  - **The score is surfaced on the human summary/trace only**, not the `--explain=json` schema (that is
    SPEC-088's audit report; `crustyimg.optimize.explain/v1` is left byte-stable).
- **Follow-up work identified:**
  - Align the wasm Auto AVIF quality (still 80, DEC-068) to the native fast 85 when `src/wasm.rs` is next
    touched — the two default paths converge in shape here but not in the number.
  - The `-q`→SSIMULACRA2 gap (q80 → ~72) is documented, not recalibrated; revisit if a quality-scale
    remap is ever in scope.
  - `SPEC-085` (`web` verb, downscale-then-AVIF) is where generosity is truly free — reframe SPEC-080's
    demo hero onto it (already tracked).

### Build-phase reflection (3 questions, short answers)
1. **What was unclear in the spec that slowed you down?** — The passthrough semantics vs the
   orientation/metadata guarantee. The spec framed passthrough as "keep original", but `Mode::Fast` makes
   passthrough common enough that shipping raw bytes silently violates the bake+strip promise. That
   interaction wasn't called out and was the bulk of the design work.
2. **Was there a constraint or decision that should have been listed but wasn't?** — The tension between
   "set the default AVIF quality ~85" and "native `convert` bytes unchanged": those are the *same*
   constant unless you split it. Listing `AVIF_DEFAULT_QUALITY`'s dual role (convert default AND the thing
   the note wanted to raise) up front would have pointed straight at the two-constant solution.
3. **If you did this task again, what would you do differently?** — Probe the fixture *classifier* buckets
   first. I burned a cycle assuming `detailed_jpeg`/`detailed_png` were photos; they classify as
   graphic-logo (flat_ratio ~0.8), so AVIF was never admitted. `jpeg_with_exif` (EXIF camera prior →
   Photograph) is the reliable "photo" fixture; I'd reach for it immediately next time.

---

## Reflection (Ship)
1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
