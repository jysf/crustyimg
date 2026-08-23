---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-098
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-08-23
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/optimize.rs
  - src/cli/common.rs
  - src/cli/ops.rs

tags:
  - apply
  - build
  - output-format
  - dec-015
  - dec-058
  - byte-changing
---

# DEC-098: `apply` at one input moves to PRESERVE THE SOURCE FORMAT — the rest of the surface does not move

## Decision

`apply`'s single-input path now resolves its output format through the exact
same rule every other pixel-lane verb (`resize`/`thumbnail`/`watermark`) and
`build` already use (DEC-015): **`--format` > `-o` path extension > preserve
the source format.** Concretely, `apply`'s single-input branch
(`optimize::run_apply`) now captures the source format before the pipeline
consumes the image, resolves it via `ops::output_format_for` (widened from
private to `pub(super)` for this reuse), and builds the sink with that
resolved format always `Some` — never leaving `format: None` for
`Sink::Dir`/`Sink::Stdout` to fall through to their own generic fallbacks.

Separately, `apply`'s multi-input (`--out-dir`) batch path now resolves
`--format` **once**, uniformly, before the rayon fan-out, and threads it
through `common::apply_one` → `common::encode_one` as `format_override` — a
parameter `apply_one` did not have before this spec, because nothing upstream
of it ever passed `--format` through at all.

**`apply` moves. Nothing else does — including `build`.** This is Call 1's
own framing, restated as the decision it is: the alternative (moving `build`,
or moving `apply` to something other than source-preserving) was rejected,
not merely not chosen.

## Context

Driven on `main` at `232c9cf`/`9b4fb80`, JPEG source, plain pixel recipe
(auto-orient + resize, no terminal `optimize`), no `--format` unless stated:

| invocation | output |
|---|---|
| `apply` **1** input, no `--format` | `src.png` — the source format is CHANGED |
| `apply` **2** inputs, no `--format` | `src.jpg` — preserved |
| `apply` **1** input, `--format png` | `src.png` ✅ honoured |
| `apply` **2** inputs, `--format png` | `src.jpg` — the flag is SILENTLY IGNORED |

**Root cause, per arity, and they are two unrelated bugs wearing one
symptom:**

- **Single input:** `optimize::run_apply` built its sink via
  `common::build_sink(global)`, which resolved `--format` but had no
  fallback for "neither `--format` nor `-o` is set" — `Sink::Dir::write`
  (`src/sink/mod.rs`) then defaulted an unset `format` to PNG (a
  `--out-dir`-only fallback that exists so a directory sink always has SOME
  format to encode with), and `Sink::Stdout::write` would have returned
  `UnknownFormat` (exit 4) for the same case. Neither is a deliberate
  "apply's default is PNG" decision; both are a generic sink fallback firing
  because nothing upstream ever asked "what should this preserve?"
- **N inputs:** `common::apply_one` called `common::encode_one` with
  `format_override` **hardcoded to `None`**, unconditionally — `encode_one`'s
  own fallback (`format_override.unwrap_or_else(|| img.source_format())`)
  was already correct, but nothing ever passed `global.format` into it. The
  multi-input path did not almost-work and default wrong; it never
  attempted format resolution in the first place.

### Call 1's measured table — the evidence for "apply moves, nothing else does"

Driven on `main`, same JPEG source, no `--format`:

| verb | output |
|---|---|
| `resize` | `src.jpg` — preserved |
| `thumbnail` | `src.jpg` — preserved |
| `watermark` | `src.jpg` — preserved |
| `build` | preserved |
| `apply`, **2 inputs** | preserved |
| **`apply`, 1 input** | **`src.png` — the sole outlier on the entire surface** |

Five of six paths already preserve the source format by default. The
opposite case — defaulting to PNG — is not unreasonable in isolation: a
plain pixel recipe with no terminal `optimize` genuinely has no format
opinion, and PNG avoids JPEG→JPEG generation loss on a re-encode. But that
argument, applied consistently, would mean moving **five** paths to match
`apply`'s one, and one of those five is `build`, whose output is pinned by a
`*.build.lock` (DEC-058) — moving it invalidates every lockfile in
existence for a weaker reason than the one that justifies moving `apply`'s
single outlier instead. Consistency across six paths beats a local optimum
on one.

## Alternatives Considered

- **Move `build` (and the other five) to default-to-PNG instead of moving
  `apply`.** Rejected: `build` binds sources to a recipe and a
  `*.build.lock` pins the resulting bytes (DEC-058) — every existing
  lockfile would go stale simultaneously, which is a larger, more expensive
  migration than the one this spec already carries (batched into PROJ-011's
  single lockfile migration with STAGE-050). It would also move
  `resize`/`thumbnail`/`watermark`, none of which have a format-preserving
  bug to fix — the only thing wrong on the whole surface is `apply`'s two
  independent gaps.
- **Fix the multi-input silent-ignore (Call 2) but leave the single-input
  PNG default (Call 1) as `apply`'s deliberate behaviour.** Rejected: it
  would leave the two arities of the SAME verb disagreeing about format on
  purpose, and `apply`/`build` would still disagree for the exact case
  (single source, no `--format`) the spec's value_link names as the reason
  a `*.build.lock` cannot be reproduced by the `apply` spelling of the same
  recipe.
- **Warn on `apply` multi-input that `--format` was ignored, rather than
  honouring it (Call 2's alternative).** Rejected by the spec itself before
  build started: there is no behaviour worth preserving here, only a
  missing resolution step. A warning would tell the user their explicit
  flag does not work instead of making it work.
- **Give `apply`'s single-input path its own format-resolution logic,
  parallel to `ops::output_format_for`.** Rejected: `resize`/`thumbnail`/
  `watermark` already carry the correct, tested rule; duplicating it would
  create a second place for the two to drift apart. `output_format_for` was
  widened to `pub(super)` and reused instead (one call site added:
  `optimize::run_apply`'s single-input branch).

## Consequences

- **Positive:** `apply` and `build` now produce byte-identical output for
  the same recipe, input, and settings (AC-3) — the property the spec's
  value_link names directly. `--format` is honoured identically at both
  arities of `apply` (AC-1). `-o` and `--out-dir` agree for the same `apply`
  invocation (AC-4).
- **Negative, and deliberate — this is byte-changing on a shipped verb.**
  `apply` at one input, no `--format`, on a non-PNG source now writes bytes
  in the SOURCE format instead of PNG. Per Call 3, **this does not ship
  alone**: it batches into PROJ-011's single lockfile migration with
  STAGE-050 (⛔ do not cut a release for this spec; the PR is not merged by
  this build cycle).
- **Neutral — `common::build_sink`'s signature changed** from
  `fn(global: &GlobalArgs) -> Result<Sink, CliError>` (it resolved
  `--format` internally and could fail on an unrecognised format string) to
  `fn(global: &GlobalArgs, fmt: ImageFormat) -> Sink` (infallible; the
  caller resolves `fmt` — including the fallible `--format` parse — before
  calling). Its only caller was `optimize::run_apply`'s single-input branch,
  so this is not a public API break.
- **Neutral — `common::apply_one` gained a `format_override: Option<ImageFormat>`
  parameter**, threaded straight through to `encode_one` (whose own
  signature is unchanged — `build.rs`'s two direct `encode_one` call sites,
  `OutputFormatPlan::Preserve`/`Pinned`, are untouched by this spec; see
  AC-6). `apply_one`'s only callers are `optimize::run_apply`'s multi-input
  rayon closures and one `common.rs` unit test, both updated.

## Validation

- **AC-1** (`apply --format` honoured at 1 and N inputs, ≥2 target formats):
  `tests/apply_batch.rs::apply_honours_format_at_every_arity` — JPEG source →
  `--format png`, PNG source → `--format jpeg`, each at 1 and 2 inputs;
  asserted on `image::guess_format` of the written bytes, not the filename
  extension.
- **AC-2** (no `--format` preserves the source at every arity, ≥2 source
  formats): `tests/apply_batch.rs::apply_preserves_source_format_at_every_arity`
  — JPEG and PNG sources, each at 1 and 2 inputs.
- **AC-3** (`apply`/`build` byte-identical): `tests/apply_batch.rs::apply_and_build_agree_byte_for_byte`
  — same recipe, same JPEG input, no `--format` anywhere; asserts the raw
  bytes are equal (Call 4: a property, not a format string).
- **AC-4** (`-o`/`--out-dir` agree): `tests/apply_batch.rs::apply_output_flags_agree`
  — same recipe/input, asserts byte equality between the two output flags.
- **AC-5, negative controls, one revert per independent condition, driven
  during build (not committed):**
  - **Baseline** — all four new tests run against pristine `main` source
    (with the new test file layered on): all four FAIL (`apply_honours_...`,
    `apply_preserves_...`, `apply_and_build_agree_...`,
    `apply_output_flags_agree`), each on a "file not found" panic because
    the SPEC-driven filename (format-correct extension) was never written.
  - **Call 1 alone reverted** (`optimize::run_apply`'s single-input branch
    + `common::build_sink` restored to `main`'s form; Call 2's
    multi-input resolution left in place): `apply_preserves_source_format_at_every_arity`,
    `apply_and_build_agree_byte_for_byte`, and `apply_output_flags_agree`
    go **RED**; `apply_honours_format_at_every_arity` stays **GREEN** —
    confirming Call 2's multi-input `--format` honouring does not depend on
    Call 1's single-input fix.
  - **Call 2 alone reverted** (`format_override` hardcoded back to `None`
    in `run_apply`'s multi-input branch; Call 1's single-input fix left in
    place): only `apply_honours_format_at_every_arity` goes **RED** (its
    multi-input sub-case; the single-input sub-case inside the same test
    stays green); `apply_preserves_source_format_at_every_arity`,
    `apply_and_build_agree_byte_for_byte`, and `apply_output_flags_agree`
    stay **GREEN** — confirming the reverse independence.
  - Together: the two Calls are independent conditions, not co-dependent
    (AGENTS §15) — reverting either one flips only the tests that exercise
    it, never the other's.
- **AC-6, blast-radius control, driven manually (not a committed test,
  matching DEC-097/SPEC-121's precedent for this class of claim):** built
  `main` and this branch side by side; ran `resize --max 20`, `thumbnail`
  (default settings), `watermark --image <overlay>`, and `build` (a 2-target
  manifest) over the same 4-file corpus (2 PNG, 2 JPEG — `tests/fixtures/c2pa/signed.png`,
  `tests/fixtures/classify/checker_graphic.jpg`,
  `tests/fixtures/classify/color_photo_fuji.png`,
  `tests/fixtures/optimize/already_compressed.jpg`) on both binaries. All
  16 output files were **byte-identical** (`shasum -a 256` diff empty). A
  positive control on the same corpus confirms the comparison methodology
  can detect a real difference: `apply` (single input, no `--format`) on
  `already_compressed.jpg` wrote `already_compressed.png` on `main` and
  `already_compressed.jpg` on this branch — the exact defect this spec
  fixes, and the one place the corpus was expected, and found, to diverge.
- **AC-7:** full matrix (`default`, `--no-default-features`, `--features
  webp-lossy`), each in a fresh `CARGO_TARGET_DIR`, sequential; `cargo
  build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` clean on
  every leg, `cargo fmt --check` clean once (feature-independent). See the
  spec's `## Build Completion` for the per-leg outcome.

## References

- Related specs: **SPEC-126** (this spec), **SPEC-111** (`format_override`
  on `encode_one`, the seam this spec threads `--format` through for
  multi-input `apply`), **SPEC-115** (Call 4's "assert bytes, not the
  extension" style, established in `tests/input_svg.rs`).
- Related decisions: **DEC-015** (the `--format` > `-o` ext > preserve-source
  precedence this spec makes `apply` follow at every arity, matching
  `resize`/`thumbnail`/`watermark`), **DEC-058** (the build cache's key and
  the `*.build.lock` this spec's byte change must batch its migration with,
  via STAGE-050), **DEC-005** (recipes round-trip through the registry —
  why `apply`/`build` running "the same recipe" is a meaningful claim to
  assert byte-identity over).
- Code: `src/cli/optimize.rs` (`run_apply`), `src/cli/common.rs`
  (`build_sink`, `apply_one`, `encode_one`), `src/cli/ops.rs`
  (`output_format_for`, widened to `pub(super)`).
- Stage: `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-049-apply-and-build-agree.md`.
