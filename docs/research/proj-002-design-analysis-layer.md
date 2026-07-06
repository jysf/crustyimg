# PROJ-002 design brief — the shared `Analysis` layer (foundation)

> Codebase-grounded architecture plan (file:line) for the foundational refactor every
> engine feature depends on. Design-only; feeds the planning session. Produced 2026-07-05.

## Grounding (today's code)
- **Load-once pipeline:** `Pipeline::run` (src/pipeline/mod.rs:51) folds `Box<dyn Operation>`
  over one owned `Image`, no intermediate clone, no disk IO. `Operation::apply` consumes
  `Image` by value (src/operation/mod.rs:150).
- **Operations are pure pixel transforms, no shared context;** trait = `name`/`params`/`apply`
  (src/operation/mod.rs:137-151); module forbids depending on clap/recipe/source/sink/disk.
- **Recipes serialize the op list and must round-trip** (src/recipe/mod.rs:187, 159); steps
  are `op`+flattened `params`.
- **Single decode at the IO boundary in `cli`** (`Image::load`/`from_bytes` at
  src/cli/mod.rs:698, 803, 1741, 1824, 2961); `Image` caps decode 512 MiB/65535 px.
- **Quality search re-encodes candidates in-memory** via the `LossyFormat` seam
  (src/quality/mod.rs:304-344); guard `resolve_effective_quality` (src/cli/mod.rs:1595).
- **JSON is hand-rolled, no `serde_json` runtime dep** (src/cli/mod.rs:1260-1283 `write_diff_json`).
- No analysis layer exists; `ImageInfo` (src/image/mod.rs:170-190) is only cheap metadata.

## The design
**New stable-core sibling `src/analysis/mod.rs`**, peer of `operation/` (depends only on
`::image`, `crate::image`, `std`). NOT part of the `Operation` hierarchy.

`Analysis` = **immutable, computed-once** struct: cheap facts (dims, color_type, alpha
coverage) + heavier derived facts (per-channel histogram, luma entropy, capped unique-color
count, dominant color, edge density, classification). All fields private + accessors, no
`&mut self`. **A few KB regardless of a 512 MiB decoded buffer** — the key memory invariant.
The one field that could blow memory (`unique_colors`) MUST be a bounded/`Saturated(cap)`
count, never an unbounded map (STAGE-006 guard).

**`Analyzer` shape — recommend a plain type with inherent `Analysis::compute(&Image) ->
Result<Analysis, AnalysisError>`, NOT a trait-object registry.** A registry (mirroring
`OperationRegistry`) buys nothing the wave needs and would tempt making analysis
recipe-addressable — which must never happen. If sub-analyses become independently expensive,
evolve to lazy-but-once per field via `std::cell::OnceCell` (no deps, no unsafe). Start eager
single-pass; split only if profiling shows a command pays for analysis it doesn't read.

```
Source ─► Image (decode-once) ─┬─► Analysis::compute(&img)  ─► planner / lint / explain
                               └─► Pipeline.run(img): Op→Op→…  ─► Sink.write
```
`Analysis` sits **beside** the Operation pipeline, both fed by the same decoded `Image`.
Operations stay unaware (still take/return bare `Image`). `Sink` unchanged.

## Integration points (file:line)
- **Runs** immediately after the single decode, on the borrowed buffer:
  `let img = Image::load(p)?; let analysis = Analysis::compute(&img)?;` — gated per command.
- **Consumers** at src/cli/mod.rs:1741/1824 (`run_optimize`→`run_pixel_op`, :2763), new
  `run_lint`/`run_explain` beside `run_info` (:1202) / `run_diff` (:1294).
- **`apply`/`edit` do NOT compute analysis** (:803, :2241) — they execute a user-authored op
  list verbatim; no decision to make. This is why the refactor touches none of the recipe
  round-trip or `apply` batch code.
- **Obtained by argument** (`fn plan(analysis: &Analysis, …) -> Pipeline`), never a global,
  never a field on `Image`, never on `Operation`.
- **Stays out of recipe serialization structurally:** `Analysis` is never an `Operation`/
  `RecipeStep`, so it cannot enter TOML. The invariant: **the planner reads `Analysis` and
  emits ordinary registry `Operation`s** — those ops (not the analysis) are what
  `--save-recipe` records and round-trips byte-stably. Spec this as an explicit non-goal:
  "Analysis is derived, never serialized; the planner's *output* is the serializable artifact."

## Load-once invariant
Compute on the single decoded buffer (`img.pixels()`), never re-decode; one traversal
(histogram/entropy/alpha/dominant/edges/capped-unique-colors in a single pass, derive scalars
after). Cost ≈ one extra linear scan — cheap vs the up-to-8 candidate re-encodes of the
quality search. Batch/`apply` under rayon: per-file `Analysis` computed inside each task,
dropped at task end — no `Send`/`Sync` needed on `Analysis`.

## Migration (tests stay green at each step)
1. **Land `src/analysis/` standalone** — type + `compute` + typed `AnalysisError` (thiserror,
   no-panic, mirroring `OperationError`) + unit tests on synthetic images (solid→entropy 0,
   gradient→known edge density, RGBA holes→partial alpha). Register in `src/lib.rs`. No CLI
   wiring → all existing tests untouched. **This is the whole foundational PR.**
2. **Wire one read-only consumer** (`lint` or `optimize --explain`) that computes and *prints*
   analysis, changing no bytes written. Golden/JSON tests.
3. **Then** let format-auto-decision / planner / classification consume `Analysis` to change op
   selection — each its own spec, behind the proven seam.

## What NOT to do
- Don't add analysis to the `Operation` trait / no "context" param on `apply` (would
  force-change every op + break the round-trip).
- Don't make `Analysis` a `RecipeStep` or an `Image` field.
- Don't build an `Analyzer` registry unless a real third-party-analyzer need appears.
- Don't add `serde_json` — use the existing hand-rolled JSON (`write_diff_json`/`escape_json`);
  revisit only if a manifest schema gets too big (a separate decision).
- Don't re-decode or touch disk in `analysis/`.

## STAGE-006 compliance
`compute` is a new untrusted-input surface: typed error, no unwrap/expect/panic on recoverable
paths; cap `unique_colors`; degenerate dims (0×0, 1-px) → typed error/default, not panic. The
512 MiB decode cap already bounds the input.
