# PROJ-002 design brief — optimization planner (the general goal-solver)

> The validated apex ("declare intent, tool decides" — the f_auto pattern). Generalizes the
> shipped quality search from (fixed-format, quality-only) to (choose format × quality ×
> dimensions), then explains it. Design-only, 2026-07-05. **Overlaps the format-engine brief:
> the planner's Phase B IS the format engine. Reconcile into one "decision engine" spec set —
> `optimize` = auto-decide default (no explicit goal); `plan`/goal-driven = explicit constraints.**

## Reuse verbatim (src/quality/mod.rs) — no new search math
`search_quality`/`auto_quality` (binary search ≤`MAX_SEARCH_ITERS=8` for lowest quality scoring
≥ SSIMULACRA2 target) · `search_under_size`/`auto_under_size` (highest quality ≤ byte budget) ·
`fit_under_size`→`SizeFit` (quality search at full size; **only if min quality still overflows**
does it pin quality to `MIN_SEARCH_QUALITY` and binary-search the scale-percent axis — the exact
quality→dimension precedence the planner reuses) · `LossyFormat` (predicates
`supports_lossy_quality` vs `supports_perceptual_quality`; AVIF excluded from perceptual, no
decoder) · `QualityChoice { quality, score, iterations, met_target }`. CLI: `AutoQuality`,
`QualityTarget {VL=90,High=70,Med=50}`, `resolve_effective_quality`, `output_format_for`,
`parse_size`. **Gap the planner fills: no cross-format loop, no explain/manifest surface.**

## Goal schema (the declarative intent)
`max_size: Option<u64>` (hard ceiling) · `min_quality: Option<f64>`/preset (floor) · `profile`
(named bundle → expands into fields; never overrides an explicit flag) · `allowed_formats: Vec`
(ordered = preference) · `allow_upscale=false` · `preserve_metadata` (Strip/Preserve/OrientationOnly,
feeds the metadata lane) · `max_dimensions` (pre-solve resize) · `allow_downscale_to_fit`
(true when max_size set).

**Objective + precedence (load-bearing):**
1. `max_size` set → **maximize quality s.t. bytes ≤ max_size** (`search_under_size` semantics).
2. else `min_quality` set → **minimize bytes s.t. score ≥ min_quality** (`search_quality`).
3. **both set → `max_size` is HARD, `min_quality` SOFT.** Find smallest candidate meeting quality;
   if it fits size → both met; else ship highest-quality that fits size, `met_quality=false` +
   explain "could not meet min-quality within budget; raise --max-size or lower --min-quality."
   Rationale: too-big **breaks** the use case (upload rejected); slightly-under-quality merely
   looks marginally worse. (Matches imgproxy: size = hard, dssim = soft.)
4. neither → default = today's `optimize` (minimize bytes s.t. score ≥ 90). `plan` degrades
   gracefully to `optimize` with no numeric goal.

## Solve algorithm (deterministic, bounded, 5 phases)
**A Classify** (cheap probe on decoded pixels via the `Analysis` layer: alpha, photo-vs-flat,
format, dims). **B Shortlist ≤3 formats** from `allowed_formats`/classifier ∩ built+capability-valid,
ordered by preference; filter by the objective's required predicate (perceptual objectives need
`supports_perceptual_quality`; pure byte-budget needs only `supports_lossy_quality`; lossless
formats enter only for byte-budget or alpha/line-art, driven on the scale axis only); drop
unbuilt codecs **before** searching. **C Per-format solve** — call the existing search unchanged
per fmt → `Candidate { fmt, quality, image_override, bytes, score, met_size, met_quality,
iterations }`. **D Pick winner** — smallest bytes (obj 2/4/default) or highest score within budget
(obj 1/3); tie-breaks: meets-both > preference-order > no-dimension-change > lower-iterations >
format-name. If none satisfy the hard constraint, apply §precedence best-effort, `met_*=false`.
**E Emit** `Plan { winner, trace }`; write `winner.image_override.unwrap_or(pipeline_output)` at
`winner.quality` in `winner.fmt` through the existing sink (DEC-016 contract extends to every
shortlisted format).

## Search-space control
Format axis capped 2–3 (only linear multiplier) · quality axis = existing binary search (≤8) ·
dimension axis conditional+lazy (only on constraint overflow, quality pinned to floor — never a
nested loop). **Worst case ≤ `shortlist·2·MAX_SEARCH_ITERS` ≈ 48 in-memory encodes, bounded
independent of image size/goal values** (add a top-level assertion). v1 non-goals: no joint
quality×dimension grid, no ML quality prediction (imgproxy `ml` — violates pure-Rust/zero-deps),
no chroma-subsampling/progressive sub-axis (future axes behind the same `Candidate` seam).

## Composition (what changes / doesn't)
**Unchanged:** all of `search_*`, `auto_*`, `fit_under_size`, `score`, `SearchConfig`,
`QualityChoice`, `SizeFit`, `encode_candidate_bytes`, both `LossyFormat` predicates — no signature
churn. **Additive:** a `src/quality/plan.rs` (or `src/planner/`) `solve(reference, goal) -> Plan`
depending only on `::image`+`quality` (preserves the "no clap/sink/files" rule); a
`format_shortlist(class, goal)` helper that receives the built+valid format set **from the CLI**
(keeps `quality` free of `sink`); a `plan` subcommand (or promote `optimize` to accept goal fields)
routing the winner through the existing `run_pixel_op` fan-out. New logic is only Phase A/B/D — all
pure, unit-testable with synthetic candidates like the existing quality tests.

## Explain trace (serde-serializable; doubles as explain output + manifest field)
`ExplainTrace { goal (resolved), objective, classification, shortlist, candidates[{format,
quality, scale_percent, out_dims, bytes, ssimulacra2, iterations, met_size, met_quality,
excluded_reason?}], winner_format, win_reason, met_goal, warnings[], savings{in,out,percent} }`.
Human render (`--explain`) + `--json`. Deterministic/reproducible (no timestamps/abs-paths in the
metric portion) → golden-testable.

## Failure modes → exit (reuse DEC-007)
Searches never panic on unsatisfiability → best-effort + `met_goal=false` + warning, exit 0 by
default; **`--strict` turns an unmet goal into exit 1** (for CI hard budgets). Conflicting size+
quality → size wins (§3). Tiny image (SSIMULACRA2 can't score) → fall back to byte-budget for that
fmt or skip perceptual candidates. Unbuilt codec → dropped from shortlist with `excluded_reason`
(exit 0); pinned-unbuilt with no alternative → exit 4. All formats fail to encode → exit 1. Batch
partial → exit 6 under `--strict`.

## Recommended framing
New spec "Optimization Planner — declarative goal solver" (depends SPEC-016/017/021/022) + a DEC
(objective precedence: size-hard/quality-soft; bounded format×quality×dimension search; ExplainTrace
schema). Failing tests to write: synthetic Phase-D pick + every tie-break; conflict (size+quality
unsat → size wins + message); codec-unavailable-drops-from-shortlist; golden ExplainTrace JSON;
probe-budget assertion. License: no AGPL codecs as default candidates (crustyimg-no-agpl-deps).

Prior art: Cloudinary q_auto/f_auto (content-aware cross-format pick), imgproxy autoquality
(size/dssim/ml; format-specific bounds "AVIF 63 ≈ JPEG 80" — a future per-format SearchConfig
refinement), Thumbor (fixed format priority order).
