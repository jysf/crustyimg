---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-083
  type: chore
  cycle: design
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-008
  stage: STAGE-028
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-14

references:
  decisions: [DEC-074]
  constraints: []
  related_specs: [SPEC-088, SPEC-082]

value_link: >
  The credibility spine of the launch — honest, equal-quality, reproducible numbers vs sharp/squoosh/
  ImageMagick. The claims r/rust and HN will actually scrutinize.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 1600000
      duration_minutes: null
      estimated_usd: 11.0
      note: >
        Main-loop build session on Opus 4.8 (no metered subagent) —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. Scope:
        design-time probing (quality→SSIMULACRA2 grid calibration for 5 tools,
        diff dimension + AVIF-search limits), building crustyimg --features avif,
        installing + pinning 4 competitors (incl. a Node-16 shim for archived
        squoosh), writing the ~540-line cross-tool harness + a ~200-line DEC + a
        ~300-line BENCHMARKS.md, three full timed benchmark passes over 8 photos
        (background), reproducibility + per-core analysis, and README/justfile
        wiring. ~1.6M tokens mixed at Opus list rate (~80/20 in/out); midpoint
        recorded. Verify/ship should replace with real subagent_tokens or /cost.
  totals:
    tokens_total: 1600000
    estimated_usd: 11.0
    session_count: 1
---

# SPEC-083: honest benchmarks (BENCHMARKS.md)

## Context

The launch claims crustyimg makes images dramatically smaller, fast, at measured quality. r/rust and HN
will not take that on the README's word — they want **numbers they can reproduce**, and they punish
cherry-picking and unfair comparisons hard. This spec produces `BENCHMARKS.md`: an honest, **equal-quality**,
**reproducible** comparison of crustyimg vs the tools people actually use — **sharp** (Node/libvips, the
incumbent), **`@squoosh/cli`** (the demo's spiritual sibling), and **ImageMagick** — on **size and speed**.

We already have a committed, offline, crustyimg-only benchmark (`just bench` / `scripts/bench.py`, SPEC-088,
DEC-074, CC0 corpus). This extends that discipline to a **cross-tool** comparison: same inputs, matched
quality, measured the same way, with the methodology and caveats stated so a skeptic can re-run it and get
the same answer.

**The bar is credibility, not favorable numbers.** If crustyimg loses on an axis (e.g. raw encode speed vs
libvips on a resize), the doc says so. Honesty is the strategy (it's the whole project's voice).

## Goal

Write `BENCHMARKS.md` backed by a reproducible cross-tool harness: crustyimg vs sharp / squoosh / ImageMagick
on size + speed, at **matched quality** (not "smallest wins"), on a real photo corpus — with the exact
commands, versions, and machine stated so anyone can reproduce it. No cherry-picking; caveats explicit.

## Inputs

- **Files to read:** `scripts/bench.py` + `just bench` (SPEC-088 — the existing corpus/harness to extend or
  mirror); the STAGE-030 benchmark findings (`stages/STAGE-030-*` / the memory of the 8-photo 0.7–47 MP
  corpus: `web` ≈ 98% median / 2–5 s) for the crustyimg side; `README.md` (the headline number this doc
  substantiates); `docs/cli-reference.md` (the exact crustyimg invocations).
- **External tools (install for the harness):** `sharp` (Node), `@squoosh/cli` (Node, note it's archived),
  `imagemagick` (`magick`). Pin + record their versions.
- **Corpus:** a real photo set spanning small→large. **Reference corpus (the one STAGE-030 measured):**
  `~/PSeven/experiments/crustimg_redo_plus/_incoming0/` — 8 real photos, 0.7–47 MP, 5 cameras (`.JPG`/
  `.jpeg`/`.png`; ignore the `.mcp.json`). The doc's headline numbers must come from a real `--corpus`,
  NOT the committed CC0 fixtures (all <2048px — the SPEC-088 carry). Do NOT commit large real photos; the
  harness takes `--corpus <dir>`. State the corpus's provenance/size distribution in the doc (not the files).
- **The uniform scorer (already in the 0.5.0 binary):** `crustyimg diff A B` computes an **SSIMULACRA2**
  score of B vs A (`docs/cli-reference.md` §diff, exit 7 on `--fail-under`). This is the matched-quality
  anchor — score EVERY tool's output with the SAME scorer against the SAME original, so the quality column
  is one consistent metric, not each tool's self-reported quality.

## Outputs

- **Files created:**
  - `BENCHMARKS.md` — the human-facing doc: methodology (matched-quality definition, machine/versions,
    exact commands), a results table (size + speed, crustyimg vs each tool, per size bucket), the honest
    caveats (where crustyimg loses; what "matched quality" means; single-thread vs libvips threading), and
    a "reproduce it yourself" section (the harness command + how to point it at your own corpus).
  - A **cross-tool harness** (extend `scripts/bench.py` or a sibling, e.g. `scripts/bench-compare.py`) that
    runs crustyimg + the competitors on a `--corpus`, matches quality (see below), and emits the table.
- **Files modified:** `README.md` — the headline number links to / is consistent with BENCHMARKS.md (light;
  don't duplicate the whole table).

## Acceptance Criteria

- [ ] `BENCHMARKS.md` exists with: an explicit **matched-quality** methodology (how quality is equalized
      across tools — e.g. target the same SSIMULACRA2/target quality, not just smallest file), the tool
      **versions + machine** used, the **exact commands** per tool, a size+speed results table, and stated
      **caveats** (including at least one axis where crustyimg does not win, if the data shows one).
- [ ] The numbers are **reproducible**: a documented harness command regenerates the table from a
      `--corpus <dir>`; running it twice gives materially the same result. No hand-edited numbers.
- [ ] **No cherry-picking** — the corpus spans small→large real photos; the methodology is fixed before
      the numbers are read; every crustyimg claim in the README is consistent with the doc.
- [ ] Honest scope: if a comparison isn't apples-to-apples (e.g. AVIF vs a tool without AVIF), it's labelled,
      not silently dropped.
- [ ] `just validate` green; no `src/`/behavior change (docs + a bench script).

## Failing Tests

Benchmarks are empirical, so verification is **reproducibility + honesty**, not a unit test:

- The harness runs end-to-end on a sample corpus and regenerates the `BENCHMARKS.md` table; a second run
  matches within noise. Capture both.
- The matched-quality claim is checked: the tools' outputs are actually at comparable quality (score them),
  not "crustyimg tuned to win." Show the quality column, not just size.
- Every competitor command in the doc runs (versions pinned); every crustyimg command runs on the 0.5.0
  binary (extend the SPEC-082 command-sweep discipline).
- Grep: no README benchmark claim contradicts BENCHMARKS.md.

## Implementation Context

### Decisions that apply
- `DEC-074` — the committed offline bench (`just bench`, `scripts/bench.py`, CC0 corpus, `--corpus`); this
  extends its discipline to competitors. Keep the "no telemetry, offline, reproducible" posture.

### Prior related work
- `SPEC-088` (shipped) — the crustyimg-only bench harness + corpus this builds on. **Carry:** the committed
  corpus can't show the big wins (all <2048px) — headline numbers MUST come from a real `--corpus`.
- `SPEC-082` (shipped) — the README headline number this substantiates.

### Out of scope
- Any `src/`/engine change or new benchmark *feature* in the CLI (this is a doc + an external-comparison
  script). Committing large real photos (use `--corpus`). Micro-benchmarks (that's `just bench-micro`).

## Notes for the Implementer
- **Matched quality is the whole credibility question.** "Smallest file" is meaningless without equal
  quality. **The method: score every tool's output with ONE scorer — `crustyimg diff <original> <output>`
  (SSIMULACRA2) — against the same original, and report that quality column next to size + time.** Don't
  trust each tool's self-reported `-q`; a q80 JPEG and a q80 AVIF are not the same quality. Two credible
  framings, pick and state one: (a) **iso-quality** — drive each tool to a target SSIMULACRA2 band (e.g.
  ~90 "high") and compare bytes+time at matched quality; or (b) **the honest scatter** — a fixed sensible
  setting per tool, plot/table size-vs-quality so the reader sees the trade, no single "winner." Iso-quality
  is the stronger claim if the harness can hit the band; the scatter is the honest fallback. **Show the
  quality column either way.** A reviewer who spots an unfair comparison discredits the whole launch.
- **This is judgment-bound, not mechanical** — the methodology choice, the fairness of each competitor
  invocation, and the caveats are the deliverable. Expect a **DEC** recording the methodology (scorer,
  matched-quality definition, tool set + versions, corpus provenance) so the numbers are defensible and the
  method is fixed *before* the numbers are read (no post-hoc tuning to win).
- **Fair competitor commands.** sharp (libvips) and ImageMagick are general resizers, not AVIF-first — when
  comparing AVIF output, use each tool's best AVIF path if it has one, and **label** where a tool can't do
  AVIF at all (don't silently drop it, don't claim a win it can't contest). cwebp is present locally; sharp/
  `@squoosh/cli`/ImageMagick install via npm/brew (node+npm present). Pin + record every version.
- **State the machine, versions, and exact commands** — reproducibility is the point; a number without its
  command is a boast. Single-thread caveat: crustyimg is single-threaded on the wasm/demo path; libvips
  threads by default — note the CPU/thread context so the speed comparison is read fairly.
- **Report losses honestly** — if libvips out-throughputs us on a plain resize, say so; the credible doc is
  the one that admits where it's beaten. Honesty voice ([[comments-plain-no-spec-refs]]), no marketing.
- `@squoosh/cli` is archived/unmaintained — note that (it's context, not a dunk).
- **The q85-AVIF "high" story (STAGE-029 carry):** crustyimg's fast-AVIF default lands ~80 SSIMULACRA2
  ("high", a touch below "visually lossless") — tell this honestly in the quality story so it reads as a
  deliberate size/quality trade, not a defect.
- Build **after 0.5.0** so the crustyimg side reflects the shipped 0.5.0 surface/engine (it is; 0.5.0 live).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-083-honest-benchmarks` (off `main` @ 260dc02)
- **PR (if applicable):** none — build cycle; handed to verify, not merged.
- **All acceptance criteria met?** yes
  - BENCHMARKS.md with matched-quality methodology, machine + pinned versions,
    exact per-tool commands, size+speed+quality tables per size bucket, and stated
    caveats incl. two axes where crustyimg loses (smallest; wall-clock speed). ✓
  - Reproducible: `just bench-compare --corpus <dir>` regenerates the tables; two
    runs matched (every deterministic field — chosen quality, bytes, score —
    identical run-to-run; wall-times moved ≤ ~2%). No hand-edited numbers (tables
    built from the harness JSON). ✓
  - No cherry-picking: 8 real photos 0.7–47 MP; methodology fixed (DEC-080) and the
    grids calibrated before the full numbers were read; every README benchmark line
    is consistent with the doc (grep-checked — README makes no "beats competitor"
    size/speed claim). ✓
  - Honest scope: cwebp (no AVIF) labelled a WebP-only format-context row, not
    dropped or claimed as an uncontested win. ✓
  - `just validate` green (224 front-matter blocks); no `src/`/behaviour change
    (a doc + an external-comparison script + a justfile recipe + a README link). ✓
- **New decisions emitted:**
  - `DEC-080` — cross-tool benchmark methodology (iso-quality at an SSIMULACRA2
    ~82 band; one scorer = `crustyimg diff`; own-reference encode-fidelity scoring;
    fixed per-tool grids picked-nearest-band; tool set + pinned versions; corpus
    provenance).
- **Deviations from spec:**
  - **Framing:** chose **(a) iso-quality** (the stronger claim) — the probe
    confirmed every tool's quality→score grid is smooth and the 82 band is hittable.
    Added beyond the minimum: the real one-command `crustyimg web` operating point
    (fixed fast-AVIF, lands ~75 "high"), a labelled WebP-only cwebp row, and a
    single-thread (per-core) table isolating the threading question.
  - **The distributed 0.5.0 binary has NO AVIF.** AVIF is a compile-time feature
    off by default (crates.io / Homebrew / Releases build default features). The
    flagship path benchmarked here needs `cargo install crustyimg --features avif`
    (still pure Rust). This wasn't called out in the spec; it's the doc's central
    honesty pivot and is stated prominently.
  - **Corpus provenance corrected:** the brief said "5 cameras / Sony". EXIF shows
    6 models across 4 brands (Fujifilm X100F; Nikon P1100/D3300/D750; Leica Q2
    Monochrom; Apple iPhone 15) and **no Sony**. Corrected in the doc + DEC.
  - **The honest result:** at matched quality crustyimg is neither smallest (sharp
    wins size on 5/8) nor fastest (sharp/ImageMagick are 3–8× faster on wall-clock,
    being multi-threaded). Reported straight; per-core it's a wash vs single-thread
    libvips, so the value framing rests on portability + measured quality + RAW +
    wasm, not raw compression/speed superiority.
  - **ImageMagick errored** on the 47 MP Leica ("Incorrect data in iCCP" — a
    malformed source colour profile that crustyimg/sharp/squoosh tolerate).
    Reported honestly; that one cell is excluded (magick is n=4 in the large bucket).
- **Follow-up work identified:**
  - **Distributed-binary AVIF friction (launch blocker candidate):** a plain
    `cargo install crustyimg` / `brew install` user gets a binary that can't produce
    AVIF. Either ship an AVIF-enabled release channel or make the `--features avif`
    requirement much louder in install docs — an r/rust reader will hit this.
  - **Single-threaded native AVIF encode:** the wall-clock loss is entirely
    threading; a multi-threaded rav1e path on native would close it (ties to the
    parked `par_iter run_pixel_op` / perf item).
  - **Harness fragility:** `@squoosh/cli` is archived and runs only on Node 16; if
    it stops building, its row degrades to "NOT RUN" (labelled, not silent).
  - Optionally add a Linux/Windows machine run for cross-platform speed context
    (the single-machine caveat is stated in the doc).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec said the crustyimg side "reflects the shipped 0.5.0 surface" but
   didn't flag that the shipped binary has **no AVIF** (it's `--features avif`, off
   by default). The flagship photo path simply isn't in a default `cargo install`,
   which reframed the entire doc and forced the `--features avif` disclosure. The
   spec also leaned toward a "substantiate the headline / crustyimg wins" posture,
   while the honest data shows crustyimg is neither smallest nor fastest —
   reconciling the launch-credibility goal with the real losses was the main judgment.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two load-bearing facts, found only by probing: `crustyimg diff` **requires
   identical dimensions** (which forces own-reference scoring), and crustyimg's
   perceptual `--ssim` search **doesn't apply to AVIF** (so iso-quality needs a
   `convert -q` grid, not the native search). Also the corpus provenance in the
   brief was factually wrong (Sony / 5 cameras) — worth verifying from EXIF up front.

3. **If you did this task again, what would you do differently?**
   — Probe the distributed-binary AVIF status, the `diff` dimension constraint, and
   the corpus's real EXIF **first**, before designing the tables — each reshaped the
   method. And budget the wall-clock better: the timed encode runs dominate the
   session (~40 min for three full passes), so I'd pick the run count up front and
   avoid running anything else on the machine while timing.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
