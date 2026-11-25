---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-106
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-27
# Refines DEC-102 (what the table checks) and DEC-031 (the CLI builds the op directly);
# supersedes neither.
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/ops.rs
  - src/operation/registry.rs
  - src/operation/mod.rs
  - src/cli/mod.rs
  - docs/api-contract.md
  - docs/data-model.md

tags:
  - watermark
  - cli
  - registry
  - validation
  - untrusted-input-hardening
  - dec-102
  - dec-031
---

# DEC-106: A CLI verb that builds its op directly checks its flags through the op's key table, before any file is read

## Decision

A CLI verb that builds its op by hand (DEC-031: `watermark` loads its overlay at the IO boundary
and constructs `Watermark` itself) still validates its numeric flags against the same
`&'static [ParamSpec]` table a recipe step is checked against (DEC-102), and does so before it
reads any file.

- **Three public items in `src/operation/registry.rs`** let the CLI check one value at a time:
  `ParamKind::accepts(&toml::Value) -> bool`, `ParamKind::expectation() -> String` (the phrase
  after "must be", now public) and `param_spec(&[ParamSpec], key) -> Option<&ParamSpec>`.
  `check_params` keeps its wording byte for byte.
- **The message names the flag the user typed** (`--size must be a number > 0 and <= 1024, got
  nan`) and prints the value as the CLI parsed it (`got 1.5`, `got inf`), not a value rebuilt
  from the f32. A table row that is missing is a usage error ("internal error"), never a panic;
  a unit test pins that `size`, `scale` and `opacity` all have rows.
- **Every `watermark` flag check precedes every file read**: `--size`, `--scale`, `--opacity`,
  then `--gravity` and `--color` (their own parsers, moved ahead), then the font or overlay
  load, then the inputs. A bad flag is exit 2 whatever the inputs, `--font` or `--image` are.
- **`size <= 1024` is a row in `WATERMARK_PARAMS`**, so the CLI, a recipe and wasm
  (`build_pipeline` reads the same table) share one bound.
- **The f32 residual is accepted and documented.** The CLI parses these flags as `f32`, so a
  value within one f32 step of a bound (`--size 1024.00001`, `--opacity 1.00000001`) reads as
  the bound and is accepted, while the same value in a recipe (an f64) is refused.
- **A scaled mark whose size cannot be represented** (a side past `u32`) is
  `Refusal::ScaledMarkTooLarge`, displayed as `the scaled mark does not fit inside the W×H px
  image`. `Watermark::mark_size` returns `None` wherever a side would not fit a `u32`. That covers the
  old saturating-cast cases (which produced `4294967295`) plus NaN and −inf, which the old cast
  turned into a 1×1 mark; only a direct library caller can reach those two, because the CLI and
  the schema refuse a scale of 0 or less first. For every size that is representable it computes
  the same values, so no accepted mark changes.

## Context

The `watermark` verb never saw the table SPEC-131 added for recipes, so the same value was
treated two ways: `--size nan`, `--size inf` and `--scale nan` exited 0 and drew nothing (or one
pixel); `--size 100000` panicked in the rasteriser (exit 101); `--scale inf` printed a refusal
naming a `4294967295`-pixel mark; and a bad flag lost to a missing `--font` or `--image`
(exit 3 instead of 2). The same values in a recipe were refused at load. The text raster is paid
before any input is read and grows with size squared (about 106 MiB at 1024, 1 GiB at 4096 for
an 11-character string), so the bound belongs where both paths read it.

## Alternatives Considered

- **Build the flags into `OperationParams` and run `check_params`** (the `resize` precedent).
  Rejected: its message names the recipe key (`'size'`), not the flag; the CLI holds an f32 and a
  `toml::Value` built from it prints `1.100000023841858`; and the op is built directly, so a
  params map would carry dummy `image`/`text` entries only to satisfy the table's mode rules.
- **Parse the flags as `f64` on the CLI.** Rejected: rounding f64 to f32 can differ by one ulp
  from parsing straight to f32, which would change the bytes of a mark accepted today.
- **Compute the scaled mark in `u64`.** Rejected: `--scale 1e30` on a 2048 px image is past
  `u64::MAX` too, so it would still saturate.
- **A DEC-036 resource limit for the size cap.** Rejected: DEC-036 bounds the recipe document
  (bytes, steps), not a key; only the table is read by both paths.

## Consequences

- `size > 1024` is refused on the CLI (exit 2) and in a recipe (exit 1): the one intended
  break, recorded under **Breaking** in the CHANGELOG. DEC-102's line that left the bound unset is
  amended to point here.
- Two CLI messages change wording because they now come from the table: `--opacity must be a
  number in 0.0..=1.0, got …` and `--size must be a number > 0 and <= 1024, got …`.
- `text::render_text` still panics for a direct library caller that passes a huge size; both
  shipped surfaces are capped before it. Filed separately.
- No `Cargo.lock`, version or `CACHE_SCHEMA_VERSION` change: checking never rewrites
  `OperationParams`.
- `--opacity 0` and a colour with alpha `00` join the flag checks that come before any file read;
  the checks that need the rendered or loaded mark sit between the font/overlay read and the
  inputs: see DEC-109.

## Validation

Right if `tests/watermark_cli_params.rs` passes: every bad value exits 2 with the table's
phrase on one input, a batch, a glob, stdin and a missing input, and wins over a missing font or
overlay; accepted marks hash identically to the goldens captured before the change; and
`--size 1024.00001` stays accepted on the CLI and refused in a recipe.

## Revisit if

- Another verb grows hand-written flag checks (route them through its table too).
- The CLI moves to `f64` parsing (the residual disappears; the golden must be re-proven).

## References

- Related specs: SPEC-135 (this rule), SPEC-131 (the table), SPEC-133 (the fit refusal),
  SPEC-128 (watermark in a recipe).
- Related decisions: DEC-102 (amended), DEC-031, DEC-104, DEC-015, DEC-036, DEC-063.
- Constraints: `untrusted-input-hardening`, `no-unwrap-on-recoverable-paths`.
