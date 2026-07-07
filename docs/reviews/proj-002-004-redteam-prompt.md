# Red-team prompt — PROJ-002 → PROJ-004 (self-contained, adversarial)

Hand this to a fresh session when you want an adversarial stress-test instead of a balanced review.
It embeds everything the reviewer needs — assume they have NO access to this repo or its files.

---

You are a skeptical, adversarial reviewer red-teaming the plan for **crustyimg** PROJ-002 → PROJ-004.
**Assume this plan will fail, stall, or get out-competed. Your job is to build the strongest possible
case AGAINST it** — find the fatal flaw, the wrong assumption, the competitor that eats its lunch, the
scope that sinks it — and only THEN concede what genuinely survives the attack. Do not be balanced or
polite; be right. This is planning/scoping critique only: do not write code. You have no access to the
codebase; everything you need is below.

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

## The direction being defended (decided with evidence in a prior scoping session)
- Position crustyimg as an image **OPTIMIZATION ENGINE** — "declare intent → produce the best
  artifact under a measurable quality/size target → explain the decision → emit a manifest" —
  **NOT a general image editor.** Extends the existing wedge "set the look, not the number."
- **Territory:** "the pre-deploy image asset layer" — local, deterministic, safe on untrusted
  input, no service and no Node runtime. Competes with ImageMagick (cryptic + unsafe-by-default,
  the ImageTragick/GhostScript CVE lineage), sharp/squoosh (Node + native), rimage (C-backed +
  narrow), Cloudinary/imgix `f_auto` (a paid per-request service), Lighthouse (needs a deployed URL).
- **Demand-validated framing (load-bearing):** (a) format recommendation is a **SILENT auto-decide
  default** inside `optimize` — "the local `f_auto`" — not an advisory report (Cloudinary `f_auto`
  adoption is cited as proof people want the tool to CHOOSE); (b) `lint` must be **source-file /
  no-URL / deterministic exit code**, because Lighthouse owns the page/URL "next-gen formats" audit
  and can't lint a source asset tree; (c) `explain` is concise polish; (d) classification is an
  **internal enabler only**.
- **Hard constraints:** permissive-only default deps (MIT/Apache/BSD/Zlib); pure-Rust; zero system
  deps; new untrusted-input surfaces bounded/no-panic; the maintainer is **separately** building a
  web-content/site tool crustyimg must **NOT** absorb — crustyimg feeds it via a **manifest** (the seam).

## The near-term ladder (a project ≈ one focused wave)
PROJ-002 engine core → PROJ-003 planner → PROJ-004 lint → PROJ-005 web-asset → PROJ-006 geometry →
PROJ-007 formats. **You are red-teaming 002–004.**

### PROJ-002 — "the optimization engine core" (FRAMED; ships 0.3.0)
Focused engine core; planner deferred to 003. Two stages:
- **STAGE-011 Analysis foundation** — a NEW shared, **computed-once, immutable `Analysis`** context
  (peer module; NOT on the Operation trait; NEVER a recipe step), one pass over the decoded buffer:
  histogram, entropy, edge density, alpha coverage, **capped** unique-colour count, dominant colour +
  a deterministic **no-ML internal classification** (photo/graphic/icon/document/ui-screenshot →
  three optimization buckets). Lands standalone, unit-tested, **no CLI change, zero new dependency**,
  bounded/no-panic.
- **STAGE-012 Auto-decide & explain** — format auto-decision inside `optimize` (only when `--format`
  isn't pinned): read Analysis → shortlist ≤3 formats via a deterministic decision tree → drive the
  **already-shipped SSIMULACRA2 search** across them → ship the smallest artifact meeting the target,
  never larger than source; `--profile web|docs|preserve` (`preserve` = today's behaviour) + a concise
  `--explain` trace. **No new search math, no new default dependency.** AVIF competes only in
  byte-budget mode (no decoder → unscoreable).
- Rationale: builds the shared Analysis foundation every later project reads AND ships the headline
  ("optimize picks the format and explains it"), cheapest + most-differentiating slice, zero new deps.

### PROJ-003 — the planner (roadmap direction + design brief; not framed)
General goal-solver: declare `max_size`/`min_quality`/profile/allowed-formats → engine chooses
**format × quality × dimensions** → explains. **Generalizes STAGE-012** ("one decision engine, two
entry points": `optimize` = auto-decide, `plan` = explicit goal). `max_size` HARD, `min_quality` SOFT
(conflict → size wins). Bounded search (≤3 formats × ≤8-iter quality × lazy dimension). Reuses the
existing search; new logic = classify → shortlist → pick-winner.

### PROJ-004 — image lint (TENTATIVE draft)
Source-file, no-URL, deterministic, pass/fail linter (reuses exit-7). Positioning: Lighthouse's image
audits run in-browser against a deployed page and can't lint a source tree; `--maxkb` gates are
format-blind; crustyimg owns pre-deploy/no-URL/format-aware. Rules: shipped-capability (GPS/metadata
leak, orientation-not-baked, oversized-bytes, wrong-colorspace, truncated/corrupt, animated-gif) +
engine-backed (legacy-format via equal-SSIMULACRA2 probe, excessive-jpeg-quality,
indexed-png-opportunity). Savings-threshold gate (4 KiB / 10%). Config `.crustyimg-lint.toml`
(select/ignore/severity/budgets). Output: human grouped-by-file + `--json` + opt-in SARIF. Stages:
A core + shipped rules (no PROJ-002 dep), B engine-backed rules (needs PROJ-002), C CI/adoption — two
GitHub Actions (`setup-crustyimg` installer + `crustyimg-action` lint mode with native PR annotations,
job summary, exit code; opt-in fork-safe commit-back / autofix.ci) + a pre-commit hook. "Drop 3 lines
→ image problems annotated on the PR" is the claimed highest-leverage adoption move. ~9 specs.

### Dependencies / sequencing
004 & 005 depend only on 002's Analysis layer, independent of each other. 003 generalizes 002. 004
STAGE-A has no 002 dependency. New default deps: 002 none; 004 none; 005 a few permissive crates.

## Attack it — the red-team task
Make the strongest case that this is the wrong plan. Address, at minimum:
1. **Kill the thesis.** Why is "optimization engine / local f_auto / pre-deploy asset layer" the
   wrong wedge or a non-market? Who doesn't care? Is "the local f_auto" a real need or a solution in
   search of a problem now that Hugo/Zola/frameworks do their own optimization and Cloudinary/CDNs
   handle it at delivery?
2. **Name the competitor that wins.** Who out-competes crustyimg here (sharp, libvips, imgproxy,
   Cloudinary f_auto, rimage, Lighthouse CI, the frameworks themselves) and why does the user pick
   them instead? Is the differentiation (pure-Rust, safe, explained, local) something users actually
   switch for?
3. **Break PROJ-002.** Where does auto-decide silently produce WORSE results, erode trust, or
   surprise users? Is deterministic no-ML classification reliable enough to drive format choice, or
   will its gray-zone failures (gradient UI, photo-of-a-document) make `optimize` untrustworthy? Is
   the Analysis layer over-engineered for a first wave?
4. **Break PROJ-004.** Is "source-file image lint" a real, felt pain with budget/attention, or a
   vitamin nobody adds to CI? Does Lighthouse/`--maxkb`/existing optimizers already cover 90% of it?
   Will the rule set be noisy enough to get muted? Is the GitHub Action adoption bet realistic for a
   brand-new tool with no distribution?
5. **Attack the sequencing.** Is 002→003→004 wrong? Is the whole ladder too slow to reach something
   people adopt? Should adoption (the Action) or the manifest (SSG integration) come BEFORE the
   engine polish? Is the "1 project/week" cadence a fantasy given STAGE-011's real cost?
6. **Find the over-build and the settled-decision that's actually wrong.** What here is gold-plating?
   What "decided" thing (deferring the planner, no new deps, classification-as-internal, the
   web-content-tool boundary) is a mistake?

Then, and only then: **concede what genuinely survives** the attack — the parts that are actually
right and worth keeping.

## Deliverable
Lead with **the single most likely way this plan fails** (one paragraph). Then the **3 strongest
objections, ranked**, each with the concrete failure scenario and what would have to change to
de-risk it. Then a short, honest **"what survives"** — the parts the attack couldn't break. No
hedging; commit to the critique.
