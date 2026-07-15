---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-070
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

created_at: 2026-07-15
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/mod.rs"
  - "src/recipe/bundled.rs"
  - "recipes/*.toml"
  - "tests/cli.rs"

tags:
  - recipe
  - apply
  - bundled-recipes
  - web
  - optimize
  - build
---

# DEC-070: the terminal `optimize` recipe step, bundled-recipe precedence, and the build-manifest limitation

## Decision

The recipe-model change that makes `web <inputs>` == `apply --recipe web <inputs>` (SPEC-085) — the
follow-through DEC-069 deferred ("folded into DEC-069's follow-through"), now recorded as its own decision
because it extends the recipe format (DEC-005), not the decision engine.

1. **A recipe may end with a terminal `op = "optimize"` step — a magic marker handled in the CLI apply
   path, NOT an `OperationRegistry` operation.** A registry `Operation` maps `Image → Image`; the fast
   AVIF-aware decision (`Mode::Fast`: modernize format + never-bigger + score, DEC-069) instead produces
   **bytes plus a format choice**, so it cannot be an `Operation`. `run_apply` detects a recipe whose
   **last** step is `optimize` (`split_terminal_optimize`), strips it, runs the preceding pixel steps as
   the pipeline, and dispatches the result to the same `run_optimize_autodecide` fan-out the `web` verb
   uses (always scoring the downscaled winner). This is a **DEC-005 recipe-format extension**: the same
   TOML the file path and the wasm build parse now admits one reserved terminal op with apply-path
   meaning. An `optimize` step **anywhere but last** is left in the recipe so `build_pipeline` surfaces it
   as a typed `RecipeError::UnknownOperation` (never silently reordered).

2. **A pinned output format is honored on the terminal-`optimize` apply path exactly as on the verb.**
   `-o <name.ext>` (recognized extension) or `--format <fmt>` is an explicit override: the apply path
   computes the same `pinned` predicate `web`/`optimize` do and, when pinned, diverts to `run_pixel_op`
   (a plain format-honored re-encode of the downscaled image) instead of the auto-decision. So
   `apply --recipe web hero.jpg -o hero.png` writes a **real PNG**, byte-identical to
   `web hero.jpg -o hero.png` — not AVIF bytes in a `.png`. The equivalence holds for the pinned path too,
   not just the unpinned one. (This was Defect 1 in SPEC-085's verify; the unpinned equivalence was
   already correct.)

3. **Bundled-vs-file precedence: a real file on disk always wins.** `apply --recipe <arg>` treats `<arg>`
   as a path **first**, and only falls back to the in-binary bundled registry (`src/recipe/bundled.rs`:
   `web`/`gallery`/`product`, `include_str!`-embedded from `recipes/`) when no such file exists. A local
   `web.toml` (or a file literally named `web`) unambiguously shadows the bundle, so every existing
   file-path recipe is unchanged and the bundled names are a convenience layer, never an override.

4. **`build` (the manifest path) does NOT support terminal-`optimize` recipes — a documented known
   limitation, not wired.** `prepare_target` builds the full recipe pipeline via `build_pipeline` without
   the apply path's strip, so a manifest target bound to a terminal-`optimize` recipe (a bundled flow, or
   a file with an `optimize` step) fails with `RecipeError::UnknownOperation { name: "optimize" }` — a
   typed error (correct exit code, no panic), surfaced before any output is written. Wiring the same
   terminal-optimize split into `run_build` (so DEC-057 manifests can use bundled modern-format flows) is
   a deliberate follow-up, deferred to keep SPEC-085 scoped to `apply`/`web`.

## Context

STAGE-030's benchmark made `web` (downscale → modernize → never-bigger → score) the flagship flow, and
SPEC-085 put a verb on it. The load-bearing design goal was that the verb and `apply --recipe web` reach
the *identical* engine and produce byte-identical output — otherwise the bundled recipe is a second,
drifting implementation. The recipe model (DEC-005) could only encode to a **fixed** format via a sink
write; it had no way to say "encode via the `Mode::Fast` decision." Two shapes could deliver the
equivalence: a new registry operation, or a terminal marker in the apply path. The registry route is
impossible (the decision emits bytes + a format, not an `Image`), so the terminal marker is the mechanism,
and it is a recipe-format extension worth recording as a decision — it changes what a `.toml` recipe can
mean, and it is the seam a future `build` wiring or a mid-recipe-optimize error message would build on.

## Alternatives considered

- **A registry `optimize` operation (`Image → Image`).**
  - What it is: implement the fast decision as an `Operation` so it composes like `resize`/`auto-orient`
    and `build_pipeline` handles it uniformly.
  - Why rejected: the fast decision produces **encoded bytes and a format choice**, not a transformed
    `Image` — it *replaces* the sink's encode. An `Operation` cannot express "and now the output format is
    AVIF, and these are the bytes." Forcing it into the trait would mean a fake `Image` round-trip and
    losing the format/score. The marker lives where it belongs: the apply path that owns the sink.

- **Descope: `web` the verb ships the full flow; the bundled recipes are fixed-format (or omitted).**
  - What it is: SPEC-085's own descope escape hatch — deliver `web` and document that `apply --recipe web`
    can't reach the auto-decision.
  - Why rejected: the equivalence resolved cleanly (`optimize_decide_one` already took a `&Pipeline`, so
    the verb and the recipe share the fan-out by handing it different pipelines). Descoping would ship a
    documented gap where none was needed, and would make `gallery`/`product` inconsistent with `web`.

- **Bundled name wins over a same-named file.**
  - What it is: resolve `--recipe web` to the bundled `web` even when a `web.toml`/`web` exists on disk.
  - Why rejected: silently shadowing a user's on-disk recipe with an in-binary one is a surprising,
    hard-to-debug override. "A real file always wins" keeps every existing file recipe working untouched
    and makes the bundle purely additive.

- **Wire `build` to strip the terminal `optimize` step now.**
  - What it is: teach `prepare_target`/`run_build` the same `split_terminal_optimize` diversion so DEC-057
    manifests can bind bundled flows.
  - Why deferred (not rejected): out of SPEC-085's scope (`apply`/`web`), and the build cache/hash keys on
    the recipe — a terminal-optimize target needs its cache semantics thought through (the auto-decision's
    passthrough/format choice is input-dependent). Better as its own change than smuggled in. The current
    behaviour is a **typed error**, so the limitation fails loudly, not silently.

## Consequences

**Good**

- `web <inputs>` and `apply --recipe web <inputs>` are byte-identical on **both** the unpinned
  (auto-decision → AVIF/lossless-WebP) and pinned (`-o .png` / `--format`) paths — verified on the real
  corpus (`DSCF1154.JPG` → identical AVIF unpinned, identical downscaled PNG pinned).
- The recipe format gains a single, documented terminal capability without touching the pixel-op model;
  `gallery`/`product` reuse it for free at their own long edges.
- Precedence is unsurprising and backward-compatible: no existing file recipe changes behaviour.

**Bad / risky**

- **`build` can't run terminal-`optimize` recipes** — a real gap for anyone wanting a bundled modern-format
  flow in a manifest. It fails with a clear typed error, and the fix (wire the strip into `run_build`) is
  scoped as a follow-up.
- **The terminal-`optimize` apply path is sequential**, not the rayon batch the plain apply path uses (it
  reuses the `optimize`/`web` fan-out). Batch throughput on huge input sets is lower than a plain recipe;
  acceptable because the AVIF encode dominates wall-clock anyway.
- **A mid-recipe `optimize` step** (not last) fails as a generic `UnknownOperation("optimize")` rather than
  a "must be terminal" message — accurate but not maximally helpful; a dedicated error would read better.

## Validation

- `web_equals_apply_recipe_web` (unpinned) and `web_pinned_format_equals_apply_recipe_web_pinned` (pinned,
  `-o .png`) both assert byte-identical output; `bundled_recipe_names_resolve` /
  `apply_prefers_real_path_over_bundled_name` pin the resolution rule. Manually confirmed on the corpus.
- Revisit when `build`-manifest use of bundled flows is wanted (wire the terminal-optimize split into
  `run_build`), or if a mid-recipe `optimize` warrants its own "must be terminal" error.

## References

- Related specs: SPEC-085 (this decision's origin), SPEC-086 (`optimize --verify`), SPEC-088 (audit report).
- Related decisions: DEC-069 (the `Mode::Fast` decision + score this step invokes — DEC-070 is the recipe
  follow-through DEC-069 deferred), DEC-005 (the recipe TOML format this extends), DEC-057 (the
  build-manifest binding the limitation in §4 concerns), DEC-017 (auto-orient/strip the pixel steps reuse),
  DEC-048 (the content branch the flow preserves).
