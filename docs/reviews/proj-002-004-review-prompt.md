# Review prompt — PROJ-002 → PROJ-004 (self-contained, balanced)

Hand this to a fresh strategy/review session. It embeds everything the reviewer needs — assume
they have NO access to this repo or its files.

---

You are a senior product/strategy reviewer for **crustyimg**. Your job is to **critically review
the plan for PROJ-002 → PROJ-004 and give specific, actionable feedback — challenge it, don't
rubber-stamp it.** This is planning/scoping review only: do not write code or framed specs. You
have no access to the codebase; everything you need is below.

## What crustyimg is
A shipped (v0.2.1), open-source (MIT OR Apache-2.0), **pure-Rust, zero-system-deps, single-binary**
image CLI. Shipped commands: view, info, resize, thumbnail, shrink, convert, optimize, responsive,
auto-orient, watermark, strip, clean, set, edit, apply, copy-metadata. It already has an
**outcome-driven engine**: perceptual auto-quality (binary-search encoder quality against the
**SSIMULACRA2** metric to hit a visual target), a `--max-size` byte budget with a
dimension-reduction fallback, WebP output (default, pure-Rust), AVIF output (feature-gated,
pure-Rust). It has input hardening (decode limits, path/symlink guards, no-panic) and a `diff`
command (SSIMULACRA2 + a `--fail-under` CI gate on exit code 7). Architecture: a **load-once
pipeline** (decode → ordered Operations → encode-once via a per-format Sink) with TOML recipes
(a serialized Operation list with a byte-stable round-trip).

## The direction (decided with evidence in a prior scoping session)
- Position crustyimg as an image **OPTIMIZATION ENGINE** — "declare intent → produce the best
  artifact under a measurable quality/size target → explain the decision → emit a manifest" —
  **NOT a general image editor.** Extends the existing wedge "set the look, not the number."
- **Territory:** "the pre-deploy image asset layer" — local, deterministic, safe on untrusted
  input, no service and no Node runtime. Competes with ImageMagick (cryptic + unsafe-by-default,
  the ImageTragick/GhostScript CVE lineage), sharp/squoosh (Node + native), rimage (C-backed +
  narrow), Cloudinary/imgix `f_auto` (a paid per-request service), Lighthouse (needs a deployed URL).
- **Demand-validated framing (load-bearing):** (a) format recommendation is a **SILENT auto-decide
  default** inside `optimize` — "the local `f_auto`" — NOT an advisory "you should use X" report
  (Cloudinary `f_auto`'s heavy adoption proves people want the tool to CHOOSE); (b) `lint` must be
  **source-file / no-URL / deterministic exit code**, because Lighthouse already owns the page/URL
  "next-gen formats" audit and structurally can't lint a source asset tree; (c) `explain` is
  concise polish; (d) image classification is an **internal enabler only** (no user-facing demand).
- **Hard constraints:** permissive-only default deps (MIT/Apache/BSD/Zlib); pure-Rust; zero system
  deps; any new untrusted-input surface stays bounded/no-panic; the maintainer is **separately**
  building a web-content/site tool that crustyimg must **NOT** absorb — crustyimg feeds it via a
  machine-readable **manifest** (the manifest is the seam).

## The near-term ladder (a project ≈ one focused wave; framed for real only once the prior ships)
PROJ-002 engine core → PROJ-003 planner → PROJ-004 lint → PROJ-005 web-asset/manifest → PROJ-006
geometry/effects → PROJ-007 formats. **You are reviewing 002–004.** (002 is fully framed; 003 is a
roadmap direction + a design brief; 004 is a tentative draft — review each at its altitude.)

### PROJ-002 — "the optimization engine core" (FULLY FRAMED; ships as 0.3.0)
Scope decision made: the **focused engine core**, planner deferred to 003. Two stages:
- **STAGE-011 Analysis foundation** — a NEW shared, **computed-once, immutable `Analysis`** context
  (a peer module of the pixel pipeline, NOT on the Operation trait and NEVER a recipe step),
  extracted in ONE pass over the decoded buffer: colour histogram, luma entropy, edge density,
  alpha coverage, **capped** unique-colour count, dominant colour — plus a deterministic **no-ML
  internal classification** (photo / graphic-logo / icon / document / ui-screenshot) collapsed to
  three optimization buckets (lossy / lossless-flat / mixed-safe). Lands **standalone** with unit
  tests, **no CLI behaviour change, zero new dependency**, bounded/no-panic (a new untrusted-input
  surface). SPEC-046 = the layer; SPEC-047 = the classifier + a labeled fixture corpus.
- **STAGE-012 Auto-decide & explain** — format auto-decision INSIDE the existing `optimize` command
  (fires only when the user hasn't pinned `--format`): read the Analysis, shortlist ≤3 candidate
  formats via a deterministic decision tree, drive the **already-shipped SSIMULACRA2 search** across
  them, ship the smallest artifact that meets the target while **never emitting one larger than the
  source**; plus `--profile web|docs|preserve` (`preserve` = today's behaviour, so the change is
  strictly additive) and `--explain` (a concise, auditable decision trace). Composes the existing
  search — **no new search math, no new default dependency**. AVIF competes only in byte-budget mode
  (no decoder → can't be perceptually scored). SPEC-048 auto-decide, SPEC-049 explain.
- **Rationale:** builds the shared Analysis foundation every later project reads AND ships the
  differentiating headline ("optimize now picks the format and explains it"), on the cheapest +
  most-differentiating slice, zero new deps.

### PROJ-003 — the planner (roadmap direction + design brief; NOT yet framed)
The GENERAL goal-solver: the user declares a goal (`max_size` and/or `min_quality`/target, a profile,
allowed formats) and the engine chooses **format × quality × dimensions**, then explains it. It
**generalizes STAGE-012's decision engine** — the stated intent is "one decision engine, two entry
points" (`optimize` = auto-decide default; a goal-driven `plan` = explicit constraints). Objective
precedence: `max_size` is HARD, `min_quality` is SOFT (conflict → size wins + explain says so).
Bounded search: ≤3 formats × the existing ≤8-iteration quality binary-search × a **lazy** dimension
axis (only runs if quality alone can't fit). Reuses the existing search verbatim; new logic is only
classify → shortlist → pick-winner.

### PROJ-004 — image lint (TENTATIVE draft breakdown)
"Clippy for image assets": a **source-file, no-URL, deterministic, pass/fail** linter for an asset
tree, reusing the existing exit-code-7 gate. Positioning: Lighthouse's four image audits +
Lighthouse-CI budgets all run IN-BROWSER against a deployed page and structurally cannot lint a
source asset tree; `--maxkb` git-gates are format-blind; crustyimg owns the pre-deploy / no-URL /
format-aware slice. Rule catalog: **shipped-capability rules** (GPS/metadata leak [the privacy
moat], orientation-not-baked, oversized-bytes, wrong-colorspace, truncated/corrupt, animated-gif) +
**engine-backed rules** that need PROJ-002 (legacy-format via an equal-SSIMULACRA2 probe,
excessive-jpeg-quality, indexed-png-opportunity). A configurable **savings-threshold gate** (default
4 KiB / 10%, borrowed from Lighthouse) keeps it quiet for CI. Config: `.crustyimg-lint.toml`
(ruff-style select/ignore + per-file-ignores, eslint-style per-rule severity, per-glob budgets).
Output: human grouped-by-file (fix line = a runnable crustyimg command) + `--json`; opt-in **SARIF**
for GitHub code-scanning. Candidate stages: **A** lint core + shipped-capability rules (no PROJ-002
dependency — could start early); **B** engine-backed rules (needs PROJ-002); **C** CI/adoption — two
GitHub Actions (`setup-crustyimg` composite installer + `crustyimg-action` default lint mode emitting
native `::error file=,line=::` **PR annotations** + a job summary + an exit code; opt-in
optimize/commit-back that's fork-safe or delegates to autofix.ci) + a pre-commit hook. The "drop 3
lines in your workflow → image problems annotated on the PR" moment is the identified
highest-leverage adoption move. ~9 specs.

### Dependencies / sequencing
Both PROJ-004 and PROJ-005 depend ONLY on PROJ-002's Analysis layer and are independent of each
other (freely reorderable). PROJ-003 generalizes PROJ-002. PROJ-004's STAGE-A has no PROJ-002
dependency. New default deps: PROJ-002 = none; PROJ-004 = none; PROJ-005 (later) adds a few
permissive crates.

## Evaluate and give pointed feedback on
1. **Thesis & positioning** — is "optimization engine / local f_auto / pre-deploy asset layer" the
   right wedge? Where is it weakest? Is the demand-validated reframe sound (format-rec = silent
   auto-decide, not advisory; lint = source-file/no-URL)?
2. **PROJ-002 scope** — is the "focused engine core" (Analysis + auto-decide + explain; planner
   deferred) the right first-wave size — too big, too small? Is Analysis-layer-first the right
   sequencing? Any risk in the STAGE-011 → STAGE-012 split, or in landing the Analysis layer
   standalone?
3. **PROJ-003 (planner)** — does "one decision engine, two entry points" hold (auto-decide in 002,
   generalized solver in 003)? Real risk of building the format loop twice? Should the planner be
   folded INTO 002 instead of deferred?
4. **PROJ-004 (lint)** — is source-file/no-URL/deterministic genuinely defensible vs Lighthouse +
   `--maxkb` gates? Is the rule catalog right (too many / too few / wrong severities)? Is the A→B→C
   stage split sound? Is the GitHub Action the right adoption bet?
5. **Sequencing & dependencies** — is 002 → 003 → 004 the right order? Since 004 and 005 depend
   only on 002 and are independent, should anything reorder? Anything on the critical path that's
   under-planned?
6. **Risks & gaps** — the single biggest risk to this plan; anything missing or over-engineered;
   any settled decision that should be revisited.

## Deliverable
A structured review: a one-word verdict per section (**hold / adjust / concern**) with a sentence
of why; then the **top 3 things to change** and the **top 3 things that are right**. Be blunt — name
the weakest link. If you'd scope or sequence it differently, say so and why.
