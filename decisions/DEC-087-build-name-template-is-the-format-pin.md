---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-087
  type: decision
  confidence: 0.88
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

created_at: 2026-08-07
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/build.rs"
  - "src/cli/common.rs"
  - "src/cli/optimize.rs"
  - "src/cli/ops.rs"
  - "src/wasm.rs"
  - "src/recipe/mod.rs"
  - "docs/api-contract.md"
  - "docs/data-model.md"

tags:
  - build
  - recipe
  - optimize
  - cache
  - correctness
---

# DEC-087: `build`'s name template is the format pin; `edit --save-recipe` records the bake

## Decision

Two calls, one spec (SPEC-111):

1. **A `build` target's name template is its format pin**, copying `apply`'s existing
   pinned-format rule (`optimize.rs:80-102`) instead of inventing a second one. A template
   naming a **literal extension** (`name = "{stem}.png"`) pins that format and skips the
   fast AVIF-aware decision; a template using **`{ext}`** (including the default
   `{stem}.{ext}`) lets the decision choose, same as `apply --recipe web`.
2. **`edit --save-recipe` records `auto-orient` explicitly** as the saved recipe's first
   step, whenever it is not already there from `--auto-orient` — matching what the shared
   CLI prefix always bakes (DEC-086), and how `recipes/web.toml` already names the step.

A third, load-bearing implementation fact discovered while driving decision 1: the
content-addressed build cache's key must fold in the format PLAN (not just the recipe),
or two targets sharing one recipe file could serve each other's cached bytes on a hit.
See Consequences.

## Context

STAGE-039's D-2: `build` could not run any recipe the binary ships with — every bundled
recipe (`web`/`gallery`/`product`) ends with the reserved terminal `optimize` step, which
`build_pipeline` does not recognize as a registry op, so `build` died with
`unknown operation 'optimize'` on all three, both bound by bundled name and by file path.
`apply --recipe web` on the identical recipe already worked (SPEC-085) — it strips the
terminal step and reroutes to the fast decision. Driven on a release build at `3dd8fa7`,
one PNG source: `build` with `recipe = "web.toml"` exits 1; `build` with `recipe = "web"`
(by name) exits 1, identically; `apply --recipe web` on the same source exits 0 and writes
a real AVIF; `build` with a plain pixel recipe (no terminal step) exits 0. The two controls
pin the fault to exactly the terminal marker, not to `build`'s recipe handling generally.

**The trap this spec exists to name:** stripping the terminal step alone is a *worse* bug
than the one it fixes. `encode_one` (`src/cli/common.rs`) hardcoded
`img.source_format()` — "no `--format` override in batch path v1." Strip-and-stop makes
`build` run the pixel pipeline and then write the result in the SOURCE format, silently
discarding the modernization the recipe exists to perform. Today's bug fails loudly (exit
1, nothing written); the naive fix fails quietly (exit 0, a file written under the wrong
format). `EXT_SENTINEL` (`build.rs`) and `lock_output_path`'s `ext` parameter already
anticipated a post-decode extension — only `encode_one`'s format CHOICE needed to become
conditional.

SPEC-110 also introduced a second, independent defect this spec closes: it made `edit`
bake EXIF orientation unconditionally via a shared CLI prefix (DEC-086) but did not
record that bake in `--save-recipe` output. A saved recipe is captured from `ops` — the
flag-driven op list — never from the prefix. So `edit --invert` on an `Orientation=6`
source bakes and returns display-correct dimensions, but replaying the SAME saved recipe
via `apply` does not bake, and returns the pre-bake dimensions. DEC-086 names this as
introduced by SPEC-110, not pre-existing — before that decision, `edit` never baked
either, so a saved recipe and its own replay agreed (both unbaked).

## Alternatives Considered

### Decision 1 — the format pin

- **A new `--format`/manifest-level format key.**
  - What it is: give `build` (or the manifest schema) an explicit format override,
    independent of the name template.
  - Why rejected: the CLI surface is frozen (STAGE-030); a manifest-level `[output]`
    table would subsume this but is a much larger schema change and not launch work.
    Filed as explicitly out of scope for this spec.
- **`build` always auto-decides; there is no pin.**
  - What it is: every terminal-`optimize` target runs the fast decision, full stop —
    a literal-extension template just gets whatever bytes the decision produces, named
    with the literal extension regardless of what they actually are.
  - Why rejected: reintroduces the exact "AVIF-in-a-`.png`" bug the pinned-format rule
    exists to prevent on `apply`. A hand-authored `name = "{stem}.png"` target is a clear
    signal of intent — a real PNG — that this alternative would silently violate.
- **The name template is the pin (chosen).** The template already distinguishes "I named
  a literal extension" from "I want the real extension" (`build.rs`'s pre-existing
  `EXT_SENTINEL` collision-detection logic already made exactly this distinction, for a
  different reason — over-detecting collisions pre-decode). Reusing it as the FORMAT
  decision, not just the collision-detection decision, is one rule instead of two, and
  it is the `build` shape of a distinction `apply` already draws from `-o`/`--format`.
  This is the lesson SPEC-110 paid three build cycles for (a design table that omitted
  one verb cost a full extra cycle) — do not repeat it by inventing a second format rule
  where one already exists.

### Decision 2 — the recipe divergence

- **(a) `edit --save-recipe` records `auto-orient` explicitly (chosen).** A recipe is a
  record of what happened; if `edit` baked orientation, the recipe should say so.
  `recipes/web.toml` already names `auto-orient` as an explicit first step — bundled
  recipes already follow this convention; saved recipes now match it.
- **(b) `apply`/`build` gain the same implicit prefix the CLI verbs have.**
  - Why rejected: this makes a recipe no longer a complete description of its own
    behavior — a hand-written recipe would silently gain a step it never names. That is
    exactly the "implicit behavior nobody can state" pattern DEC-086 spent three cycles
    removing from the pixel lane (the seven-verb orientation bug was caused by exactly
    this kind of unstated per-caller behavior). Reintroducing it at the recipe layer
    the moment after removing it at the verb layer would be self-defeating.

## Consequences

- **Positive:** `build` can now run every recipe the binary ships with, both by bundled
  name and by file path — STAGE-039's last launch-gating repo item. `apply` and `build`
  share one format-decision rule (`split_terminal_optimize`, reused not copied) instead
  of two that could drift. A recipe saved by `edit` is now a complete, replayable
  description of the **pixel steps** `edit` ran — including `auto-orient` — closing
  DEC-086's own named follow-up. Quality (`-q`) is not a recipe field, so it is not part
  of that description: replaying a saved recipe reproduces the same pixel operations in
  the same order, not necessarily the same encode quality the original invocation used.
- **Negative:** a `build` target whose name template names a literal extension that is
  NOT a recognized image format (e.g. `name = "{stem}.txt"`) now fails at prepare time
  (exit 4, before any input is touched) rather than the template being silently
  interpreted some other way — a new failure mode for a config shape nobody exercised
  before (the whole terminal-`optimize`-through-`build` path did not exist). `edit`'s
  saved recipe now has one more step than the flag list alone would suggest, though this
  matches what actually ran.
- **Negative, pre-existing class, newly triggerable:** `build` has never cleaned its
  `out` tree — it only ever writes the path the current run decides on, never removes a
  target's PRIOR output. A `{ext}`/Decide target whose source content changes such that
  the fast decision picks a different format than last time (e.g. `photo.avif` last run,
  `photo.webp` this run because the source changed) leaves the stale file behind; `out`
  now holds both. This gap predates this spec, but Decide's whole point is a
  content-dependent extension, so this spec is what makes the flip — and the orphan —
  actually reachable. Not fixed here (cleaning `out` is a scope decision of its own, not
  implied by "run the recipe correctly"); named so it isn't silently inherited. `--check`
  at least surfaces it loudly (an unexpected file in a supposedly-clean tree) rather than
  masking it.
- **Neutral / a finding this spec's implementation surfaced, not just its design:** the
  build cache's key (`crate::build::cache::compute_key`) is documented as depending only
  on the recipe hash, quality, and input — explicitly NOT the output destination, because
  "identical inputs produce identical bytes wherever they land." That invariant assumed
  the output format was a pure function of (recipe, input), which held before this spec.
  It does not hold for a terminal-`optimize` target: the SAME recipe file can be Pinned
  (via one target's literal-extension template) or Decided (via another target's `{ext}`
  template), producing DIFFERENT bytes for the same input. Left alone, two such targets
  sharing a recipe file would collide in the cache — one target's entry could serve the
  other's bytes on a hit. Fixed by folding the format PLAN into the target's effective
  recipe hash (`target_recipe_hash`, `src/cli/build.rs`) — but ONLY for terminal-`optimize`
  targets; a plain pixel recipe (the only shape that could ever build before this spec)
  hashes via the untouched `crate::build::cache::recipe_hash`, so no prior cache entry or
  committed lockfile line goes stale from this change alone. `cache.rs`'s own module doc
  is not amended by this DEC (the field being hashed there is still opaque to it); this
  note lives here because it is this spec's finding, not `cache.rs`'s original design.
- **Named, not fixed (sweep finding, "the sweep wins" — see Validation):** `src/wasm.rs`'s
  `transform` function builds a pipeline from an arbitrary caller-supplied recipe the same
  unstripped way `build` did before this spec — a bundled/terminal-`optimize` recipe
  handed to it via the wasm surface would hit the identical `unknown operation 'optimize'`
  failure. This is the SAME defect class SPEC-111 fixes for `build`, found by the required
  sweep of every `.build_pipeline(` call site. It is explicitly out of scope for this spec
  ("Out of scope: … wasm … work") and is not fixed here.
  > ⚠️ **Closed by the Amendment below (SPEC-112).** The demo reasoning that justified
  > leaving this open was correct; the README's claim about `transform` was not — see the
  > amendment for why the two came apart.

  Also named: `build`'s new terminal-`optimize`/auto-decide path reaches
  `optimize_decide_one` — the same seam that triggers SPEC-107's truncated-JPEG stderr
  warning on `apply` — but this spec's `encode_one_optimize_decided` wrapper discards that
  signal rather than threading it through, so `build` does not yet warn where
  `apply --recipe web` would for the identical input. Filed as a follow-up, not an AC of
  this spec.

## Validation

Right if: `build` completes and writes real output for a target bound to each bundled
recipe, by name and by path (AC-1); the bytes match what `apply --recipe <name>` produces
for the same input, asserted on bytes not extension (AC-2); a literal-extension template
pins a real file of that format (AC-3); a genuinely unknown terminal op still fails
(AC-4, negative control — driven: mutating the strip to drop the last step
unconditionally turns `build_still_rejects_an_unknown_terminal_op` RED, confirmed against
the rebuilt binary via a changed test-binary hash, not just a re-run); `optimize`
anywhere but last still fails (AC-5); a plain pixel recipe is byte-identical to before
(AC-6, checked directly against `apply` on the same input, not merely re-asserted);
the lockfile/cache name the real decided extension and a cache hit reproduces the same
path as the miss that filled it (AC-7); `edit --invert --save-recipe` on an
`Orientation=6` JPEG replays via `apply` to the SAME dimensions the direct `edit` produced
(AC-8), and the saved recipe names `auto-orient` explicitly, asserted on the TOML itself
(AC-9); `--check`/`--frozen`/`--locked` all pass on a terminal-`optimize` target with a
decided extension (AC-10, driven for all three; `--watch` not separately driven — it
re-enters the same `run_build` this spec changes, and touches none of the watch-specific
debounce/exclusion logic, so the marginal risk was judged not to justify the added
harness). Revisit if: a real user needs a manifest-level format override independent of
the name template (then build the `[output]`-table spec this decision declined to build
early).
> **Closed, 2026-08-09 (SPEC-112).** The other trigger named here — *"if `src/wasm.rs`'s
> `transform` gets a caller that hands it a bundled recipe"* — fired: `README.md` already
> promised exactly that caller, so the gap got its own spec. See the Amendment below.

## References

- Related specs: SPEC-063 (declared `build`, the manifest/target/lockfile shapes this
  spec threads the format decision through), SPEC-064 (the content-addressed cache this
  spec's `target_recipe_hash` finding amends), SPEC-065 (injective source→output,
  `EXT_SENTINEL`, and the deliberately-unexpanded `{ext}` collision key — read before
  touching output paths), SPEC-085 (the terminal `optimize` step and the fast decision
  this spec extends from `apply` to `build`), SPEC-110 / DEC-086 (introduced the
  `edit --save-recipe` divergence this spec closes; also the source of this spec's
  process warnings — a design table that omits one verb, or a sweep that is filed but
  not honored, both cost a full extra cycle).
- Related decisions: DEC-057 (manifest paths resolve relative to the process CWD —
  unchanged by this spec), DEC-058/DEC-059 (the cache key composition and lockfile
  contract this spec's cache-key finding amends), DEC-070 point 4 (self-documented this
  defect before it was fixed), DEC-081 (AVIF as a default feature — why AC-2's format
  assertion needs no feature gate on the default/lean split for the pin case, and does
  need one for the AVIF-bytes case).
- External docs: none.

## Amendment (2026-08-09, SPEC-112): the wasm exception is closed

**The exception this decision named and left open — `src/wasm.rs`'s `transform` builds an
arbitrary caller-supplied recipe unstripped, so a bundled/terminal-`optimize` recipe hits
`unknown operation 'optimize'` exactly as `build` did before this spec — is now closed.**
`transform` strips the terminal marker before `build_pipeline`, runs the remaining pixel
steps, and encodes to the caller's `out_format`, using the SAME `split_terminal_optimize`
helper `apply`/`build` call (not a copy).

**The call to leave it open was right about the demo, and wrong about the README.** This
decision's own text said the exception held because "the shipped demo never reaches it" —
`demo/worker.js`'s `geometryRecipe()` hand-builds a terminal-step-free recipe. That was, and
remains, true; SPEC-112 changes nothing about the demo (its AC-4 pins `transform`'s output
on a markerless recipe byte-identical to before this spec). What this decision did not
check was a second, independent claim: `README.md:34-36` — the launch front door, which
renders on the crates.io crate page — promises "the same recipe TOML runs in the browser
demo too, via the wasm `transform()` binding," including **starting from a bundled
`web`/`gallery`/`product`**. The demo happening to avoid the bug was never evidence that
claim was true for every other caller of the published `crustyimg-wasm` npm package. A JS
consumer who follows the README literally — resolve a bundled recipe, hand it to
`transform()` — hit exactly the failure this decision named and declined to fix. The demo
reasoning and the README claim are two different questions; SPEC-112 is the record of the
first being right while the second was wrong, corrected once the two were checked
separately rather than treated as one story.

**Implementation note: why `split_terminal_optimize` moved rather than just widened.** The
Note above this decision's own Implementation ("wasm ... work" out of scope) assumed
reaching this helper from `wasm` was a visibility question — `pub(super)` in `cli::optimize`
to `pub(crate)`. It is not, and SPEC-112 found out why: `src/lib.rs` compiles `cli` only for
`#[cfg(not(target_arch = "wasm32"))]` and `wasm` only for `#[cfg(target_arch = "wasm32")]`
(the SPEC-072 target split). The two module trees never coexist in one build — a
`cli`-hosted `pub(crate)` function simply does not exist in the wasm32 artifact `wasm`
compiles into, no matter how it is marked. `split_terminal_optimize` (and its
`OPTIMIZE_STEP_OP` constant) moved to `src/recipe/mod.rs` instead — one of the modules
`src/lib.rs` compiles for **both** targets (the "pure engine," per its own module doc) — and
stayed `pub(crate)` there, which now genuinely reaches every caller: `cli::optimize::run_apply`
and `cli::build::prepare_target` (native) and `wasm::transform` (wasm32) all call the one
function. This is also the more honest home for it: the helper's whole subject is a
*recipe's* terminal marker, not a *CLI* concern, which SPEC-111's own Note had already
observed without acting on.
