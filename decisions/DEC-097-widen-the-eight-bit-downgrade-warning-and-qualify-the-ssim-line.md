---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-097
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-21
supersedes: null
superseded_by: null

affected_scope:
  - src/sink/mod.rs
  - src/analysis/decide.rs
  - src/cli/optimize.rs
  # Added at verify (2026-08-21). Call 2's ENTIRE justification is that
  # `to_ss_rgb` converts both sides to 8-bit sRGB before scoring (DEC-019). If
  # that ever changes, the shipped string "(8-bit comparison; source was
  # 16-bit)" becomes a lie and `ssim_source_depth` becomes meaningless — and
  # without this entry, DEC-097 would not surface in `decisions-audit --changed`
  # for whoever changed it. DEC-096, the immediately preceding record touching
  # this same file, lists it for exactly this kind of coupling.
  - src/quality/mod.rs

tags:
  - bit-depth
  - colour-type
  - sink
  - ssim
  - ssimulacra2
  - honest-reporting
  - webp
  - avif
  - bmp
  - gif
  - ico
---

# DEC-097: widen the 8-bit downgrade warning to every MEASURED 8-bit-only target, and qualify a SSIM score that cannot see a depth reduction

## Decision

`sink::encode_to_bytes_with` now calls `eight_bit_downgrade_warning` (SPEC-121,
Call 3) at **every** site that silently narrows a >8-bit source to 8 bits per
channel, not only JPEG and lossy WebP: **BMP, lossless WebP, and AVIF** join
the warned set. **PNG and TIFF** were re-measured and confirmed to hold the
full depth, so they stay silent — the spec's prior held. **GIF and ICO are
deliberately excluded**, for two different reasons, neither of which is "this
format holds the depth" (see Call 1 below).

Separately, `optimize`/`web`'s SSIMULACRA2 report (`ExplainTrace`, SPEC-049)
now carries `scored_source_depth: Option<u8>` — `Some(depth)` exactly when
the winner's format narrowed the reference below its own depth. The default
summary line, `--explain human`, and `--explain json`/`--json` all qualify
the score in that case (`"ssim 100.0 (8-bit comparison; source was
16-bit)"` / `"ssim_source_depth":16`) rather than printing a bare, falsely
perfect number.

**This is a reporting fix. It changes no output bytes** (AC-5/AC-6, both
driven — see Validation).

## Call 1 — the set was measured, not copied from the spec

The spec named `Gif`, `Bmp`, `Ico`, lossless `WebP` as candidates and called
that a **prior to check, not a conclusion**. It was checked behaviourally:
encode a >8-bit source to each target with `sink::encode_to_bytes`, decode
the result back, and read what survived (`--features avif`, the default
build).

| target | encode | decode-back | result | verdict |
|---|---|---|---|---|
| PNG | ok | ok | **16-bit preserved** | silent (prior held) |
| TIFF | ok | ok | **16-bit preserved** | silent (prior held) |
| GIF | **`SinkError::Encode`** — `image`'s own error: *"the encoder or decoder for Gif does not support the color type `Rgb16`"* | — | **rejected outright**, not narrowed | **excluded** — already loud (exit 5), a warning here would misdescribe a hard failure as a soft downgrade |
| BMP | ok | ok | **8-bit** | **now warns** |
| ICO | ok (writes bytes, exit 0) | **fails independent of depth** — `Format error decoding Ico: The PNG is not in RGBA format!`, reproduced for 8-bit RGB (no alpha), 16-bit RGB and 16-bit RGBA alike — but **NOT** for `Rgba8`, which round-trips correctly. ⚠ **Corrected at verify (2026-08-21)**: this row originally listed 8-bit RGBA among the failures. It is the one case that works, which is what the mechanism sentence below and the proposed `Rgba8` pre-conversion fix both depend on | **undetermined — the round-trip is broken for reasons that have nothing to do with bit depth** | **excluded** — see below |
| lossless WebP | ok | ok | **8-bit** | **now warns** |
| AVIF (`avif` feature) | ok | ok | **8-bit** | **now warns** — not named in the spec's candidate list, found by measuring rather than copying it |

**GIF is a finding, not a gap.** `image`'s GIF encoder hard-rejects a >8-bit
`DynamicImage` variant before any pixel is written — the user already gets a
loud, typed failure (`convert --format gif` on a 16-bit source: exit 5, "The
encoder or decoder for Gif does not support the color type `Rgb16`"). Adding
`eight_bit_downgrade_warning` in front of that would tell the user their
depth was *silently narrowed*, when what actually happened is the encode
*never ran at all*. `tests/sink.rs::gif_target_errors_loudly_instead_of_downgrading`
pins this as a regression guard: if `image` ever changes GIF to narrow
silently instead of erroring, that test goes red and GIF needs revisiting.

**ICO is a bigger, orthogonal finding, filed rather than fixed here.**
`image` 0.25.10's ICO encoder embeds a PNG sub-image at the source's own
colour type (consistent with DEC-095's preserve policy — the encoder is not
doing anything wrong per se), but `image`'s **own** ICO decoder hard-requires
that embedded PNG to be exactly `Rgba8`. The result: `convert --format ico`
succeeds (exit 0) and writes a file that **the very next `crustyimg info` on
that file cannot open** — for a plain opaque 8-bit RGB source with no alpha
and no depth question anywhere in play, not just for the >8-bit case this
spec is about. Framing this as a depth-downgrade warning would misattribute
it: the defect is not "your bits were narrowed", it is "this file is
unreadable by the very tool that wrote it." `tests/sink.rs
::ico_round_trip_defect_is_orthogonal_to_depth` pins the measurement (an
8-bit, alpha-less source, so the control is the strongest form: depth is not
even a variable). Filed to STAGE-042's backlog for a maintainer ruling — a
real fix (pre-converting to RGBA8 before ICO encode) would change output
bytes for every non-alpha ICO source, out of scope for a reporting-only
spec.

**Feature-set invariance, measured, not assumed:** `Cargo.toml`'s `image`
dependency enables `png`/`jpeg`/`gif`/`bmp`/`tiff`/`ico`/`webp` unconditionally
— none of those seven are gated by crustyimg's own Cargo features. Only
`avif` is. Driven directly: BMP, lossless WebP, PNG, TIFF, GIF, and ICO
produced **identical** behaviour under `default`, `--no-default-features`,
and `--features webp-lossy` (16-bit BMP/WebP → 8-bit in all three; PNG/TIFF →
16-bit preserved in all three; GIF → the same encode error in all three; ICO
→ the same decode failure in all three). AVIF is unreachable under
`--no-default-features` (`ensure_codec_built` → `SinkError::CodecNotBuilt`,
exit 4, matching existing behaviour) — the new AVIF warning is naturally
`#[cfg(feature = "avif")]`-gated and simply never fires there, since the
encode never runs.

**A pre-existing, accepted pattern extends unchanged.** `solve_candidate`
(the `web`/`optimize` candidate search) calls `sink::encode_to_bytes` for
**every** shortlisted candidate, not only the winner — so a search that
tries both AVIF and JPEG on a 16-bit source now prints *two* warning lines
even though only one format ships. This is not new: SPEC-121's shipped code
already does this for JPEG when it loses to another candidate (driven and
confirmed on this branch — a photo-bucket search over {AVIF, lossy WebP,
JPEG} printed both the "lossy WebP" and "JPEG" downgrade warnings, though
only WebP won). Widening the warned set widens how often this fires but does
not change the shape of the behaviour.

## Call 2 — the SSIM line, and the DEC-019 boundary

The scorer (`quality::to_ss_rgb`, DEC-019) converts **both** the reference and
the candidate to 8-bit sRGB before comparing — that is the metric's settled
input contract, not a bug. So when the winning candidate's format cannot hold
the reference's real depth (Call 1's set), the reference itself gets
downsampled to 8 bits *before* the comparison runs, and if the candidate is a
lossless-at-8-bit re-encode of that same truncation, the two sides are
pixel-identical and the score reads a perfect 100.0 — a real depth loss
reported as no loss at all.

**Root cause, precisely:** `optimize_decide_one` (SPEC-085's `web`-always /
`optimize --verify` scoring path) already decodes and scores **every**
winner, lossy or lossless — `score_winner_once`'s own doc comment describes a
`None`-for-lossless branch that its one real caller has never actually
exercised. The bug is not that lossless winners get scored (that is
arguably correct — a lossless-at-8-bit encode of a 16-bit source is NOT
lossless overall); it is that the score, once computed, was reported with no
indication that the comparison itself is blind to the exact loss in
question.

**Trigger condition, chosen to be structural rather than magic-numbered:**
`scored_source_depth` is set whenever `color_type_bit_depth(reference) >
color_type_bit_depth(decoded winner)` — compared directly from the two
already-in-hand `DynamicImage`s, not re-derived from a format name. This
fires for **any** depth-reducing winner, not only the ones that happen to
score near 100 — a JPEG/AVIF lossy re-encode of a 16-bit source (ssim 77.6,
measured) gets the same qualifier as a lossless-at-8-bit WebP winner (ssim
100.0, measured), because the same structural blind spot applies to both: the
metric cannot see whatever was lost purely to the 8-bit truncation of the
*reference*, independent of what else the encoder also threw away. A
threshold on the numeric score itself ("only qualify above N") was
considered and rejected — it would be an invented policy the spec never
asked for, and the structural condition already only ever fires when the
source was genuinely >8-bit, which is rare in practice (most sources are
8-bit already), so the common case is untouched.

**⛔ The DEC-019 boundary, respected:** the spec explicitly forbids touching
the scorer used by `optimize`'s byte-budget search (Call 2). This record does
NOT change `to_ss_rgb`, `score`, or `score_winner_once` — it reads the
*inputs* to that call (`out_img`'s and the decoded winner's own colour
depth) from the caller, entirely outside the scorer itself, and only changes
what the **caller** does with the number the scorer returns. `optimize`'s
candidate selection (`pick_winner`) never sees `scored_source_depth` — it is
report-only plumbing added after the winner is already chosen. AC-6 (below)
confirms this directly: candidate selection is byte-identical to `main` on
an 8-bit corpus, where the field is always `None` and the code path is a
no-op by construction.

**Option chosen: qualify (not suppress, not rescale).** Of the three options
the spec named:

- *Suppress the figure* — would throw away information the search genuinely
  computed (a real, if incomplete, signal), and does not match how this
  repo's other honest-reporting precedent (SPEC-090's larger-than-source
  line) works: it labels the caveat rather than hiding the number.
- *Compute at source depth* — this is exactly the DEC-019 boundary Call 2
  forbids crossing; `to_ss_rgb`/`score` are shared with the byte-budget
  search, and a >8-bit-capable scorer would need its own design (a `ssimulacra2::Rgb`
  built from `u16` samples, if the crate even supports it — not investigated,
  since crossing this boundary was explicitly out of scope).
- **Qualify (chosen)** — the number stays, with a machine- and
  human-readable caveat naming exactly what it cannot see. Matches SPEC-090's
  established idiom for "the number is real but needs a caveat."

## Acceptance Criteria — Build Completion

See the spec's own `## Build Completion` for the full checklist; this record
carries the evidence, not a restatement of the checklist.

## Alternatives Considered

- **Warn for GIF too, since it was named in the spec's candidate list.**
  Rejected — measured to be a hard encode error, not a silent downgrade; a
  warning there would be actively misleading about what happened.
- **Warn for ICO too, or suppress its output silently.** Rejected — the
  measured defect is orthogonal to depth (reproduces at 8-bit RGB with no
  alpha), so a depth-framed warning would misattribute it. Silently changing
  ICO's output (e.g. forcing RGBA8) would alter output bytes for every
  non-alpha ICO source, which this reporting-only spec's AC-5/AC-6 forbid.
  Filed to STAGE-042 for a maintainer ruling instead.
- **Qualify the SSIM line only when the score is suspiciously high (e.g. >
  95).** Rejected — an invented, unrequested threshold; the structural
  condition (did the winner's format narrow the reference?) is exact, cheap
  to compute from data already in hand, and does not require guessing at
  where "suspiciously high" begins.
- **Compute SSIM at the source's native depth.** Rejected — explicitly
  forbidden by Call 2's DEC-019 boundary; the shared scorer anchors the
  byte-budget search too, and AC-6 pins that path byte-identical.
- **Leave `score_winner_once`'s `None`-for-lossless branch to do the
  filtering (only score genuinely-lossy winners).** Rejected — this is not
  actually how the one real caller (`optimize_decide_one`) works today: it
  scores every winner regardless of disposition, and changing that would be
  a larger, riskier behavioural change than adding a qualifier field, for a
  spec whose whole premise is "changes no output bytes."

## Consequences

- **Positive — closes the exact defect the spec was written against.**
  `convert --format webp` on a >8-bit source now warns (AC-1), the warned set
  extends to every measured 8-bit-only target (AC-2), `web`/`optimize` reach
  it through their candidate search without a `--format` pin (AC-3), and no
  scored winner reports a bare, falsely-perfect SSIM figure across a depth
  change (AC-4) — all four driven via the compiled binary, not reasoned
  about.
- **Positive, unlooked-for — surfaces AVIF's identical gap**, which the spec
  itself did not name as a candidate and DEC-095 had explicitly listed as
  "not covered." Measuring rather than copying a list found it anyway.
- **Positive, unlooked-for — surfaces a second, more severe, orthogonal
  defect in ICO** (files `convert` writes without error that its own `info`
  cannot subsequently read), filed to STAGE-042 rather than silently folded
  into this spec's fix.
- **Negative — the candidate-search multi-warning pattern now fires more
  often.** A `web`/`optimize` search over multiple 8-bit-only candidates on a
  >8-bit source can print one warning line per attempted candidate, not only
  the winner's. Pre-existing (JPEG already did this), not introduced here,
  but the widened set makes it more visible. Not fixed — deferring warnings
  until after the winner is chosen would be a larger structural change to
  `solve_candidate`, unjustified for a reporting spec.
- **Neutral — the JSON explain schema gains one additive field**
  (`ssim_source_depth`), gated absent unless the qualifier applies (same
  discipline as `larger_than_source`, DEC-075). A run that never triggers the
  condition — the overwhelming majority, since most sources are already
  8-bit — has byte-identical JSON to before.
- **Neutral — no `Operation` change, no cache-key change, no scorer change.**
  Confirmed by AC-5/AC-6 (Validation).

## Validation

- **AC-1** — `tests/sink.rs::lossless_webp_reports_the_depth_downgrade`:
  `convert --format webp` on a 16-bit source warns on stderr; `-o -` stdout
  is verified to start with the WebP RIFF/WEBP magic bytes only (no
  diagnostic text on the pipe).
- **AC-2** — `tests/sink.rs::eight_bit_only_targets_all_warn_and_others_do_not`:
  table-driven over {BMP, lossless WebP, PNG, TIFF}, plus a `#[cfg(feature =
  "avif")]` AVIF case. Table-driven so a future `image` capability change
  goes red on its own.
- **AC-3** — `tests/sink.rs::web_and_optimize_reach_the_widened_downgrade_warning`:
  drives both verbs' candidate search (no `--format` pin) and asserts the
  warning appears.
- **AC-4** — `tests/sink.rs::ssim_line_is_qualified_across_a_depth_change`:
  drives `web` and asserts the rendered stderr line is NOT bare (does not
  end in the raw score) and IS qualified with the source's real depth.
- **AC-5, driven both ways (AGENTS §15):**
  - `tests/sink.rs::eight_bit_source_warns_nowhere` — an 8-bit source through
    `convert` (bmp/webp/png/tiff/avif) and `web` warns nowhere and the SSIM
    line stays unqualified.
  - **Byte-identity against `main`, driven manually** (not a committed test —
    matching SPEC-121's AC-8 / SPEC-124's AC-6 precedent for this class of
    claim): built `main` and this branch side by side; `convert --format
    webp` and `web` on the same 8-bit source produced **byte-identical
    output and byte-identical (empty) stderr** on both binaries.
  - **Negative controls, one revert per independent condition**, driven
    during build:
    - Call 1 alone reverted (`src/sink/mod.rs` → `main`): AC-1/AC-2/AC-3
      tests go **RED**; AC-4/AC-5 and the GIF/ICO pin tests stay **GREEN** —
      confirming Call 2's qualifier is derived from the decoded image, not
      from the diagnostic line, so it is independent of Call 1.
    - Call 2 alone reverted (`src/analysis/decide.rs` +
      `src/cli/optimize.rs` → `main`): only `ssim_line_is_qualified_across_a_depth_change`
      goes **RED**; every other new test stays **GREEN** — confirming the
      two Calls are independent conditions, not co-dependent.
- **AC-6, driven on 8-bit photo-like content:** `optimize --verify --explain
  json` on `main` and this branch produced **byte-identical output, JSON,
  and stderr** — candidate selection is untouched.
- **AC-7:** full matrix (`default`, `--no-default-features`, `--features
  webp-lossy`), each in a fresh `CARGO_TARGET_DIR`, sequential; `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` clean on
  every leg (see Build Completion for the per-leg log).
- `tests/sink.rs::gif_target_errors_loudly_instead_of_downgrading` and
  `::ico_round_trip_defect_is_orthogonal_to_depth` pin the two exclusion
  findings as regressions: if `image` ever changes either format's
  behaviour, these go red and the exclusion needs revisiting.

## References

- Related specs: **SPEC-125** (this spec), **SPEC-121** (Call 3, the
  JPEG/lossy-WebP warning this widens), **SPEC-090** (honest size
  reporting — the precedent Call 2's qualifier follows), **SPEC-085/086**
  (the `web`-always / `optimize --verify` scoring path this reads from).
- Related decisions: **DEC-095** (SPEC-121's colour-type/bit-depth
  preservation record — the Consequences entry that filed this gap),
  **DEC-019** (the SSIMULACRA2 scorer and its 8-bit sRGB input contract —
  the boundary Call 2 does not cross), **DEC-075** (the `larger_than_source`
  additive-JSON-field precedent this follows), **DEC-090** (diagnostic
  channel, still PROPOSED — this record's stderr lines use the existing
  plain `eprintln!` convention, matching Call 3's precedent, not DEC-090's
  unaccepted `log` facade).
- Code: `src/sink/mod.rs` (`eight_bit_downgrade_warning`,
  `encode_to_bytes_with`), `src/quality/mod.rs` (`to_ss_rgb`, `score`,
  `score_winner_once`), `src/analysis/decide.rs` (`ExplainTrace`,
  `write_json`), `src/cli/optimize.rs` (`optimize_decide_one`,
  `emit_optimize_report`, `solve_candidate`).
- Backlog: `projects/PROJ-010-post-launch-correctness-and-consolidation/stages/STAGE-042-release-safety-instruments.md`
  — the ICO round-trip finding filed there.
