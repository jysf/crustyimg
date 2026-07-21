---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-083
  type: chore
  cycle: verify
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
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 400000
      duration_minutes: null
      estimated_usd: 3.6
      note: >
        Main-loop verify session on Opus 4.8 (no metered subagent) —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. Scope:
        re-derived every published cell from the harness JSON (per-photo,
        per-bucket, per-core), re-ran the run1/run2 determinism diff, and drove
        the load-bearing prose claims against the real tools (squoosh/sharp/cwebp
        output dimensions, `web` vs `convert -q` byte identity, ImageMagick and
        cwebp quality readouts, RAW extension list, dist default features).
        Outcome: ⚠ PUNCH LIST — 4 substantive + 4 minor.
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 700000
      duration_minutes: null
      estimated_usd: 5.0
      note: >
        Second build (fix) session on Opus 4.8 clearing the verify punch list —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. Scope: read
        @squoosh/lib's resizeWithAspect to find the real aspect semantics, fixed
        the squoosh/sharp/cwebp resize dialects, added the harness dimension guard
        + `--self-test` + `--q-from`, proved the guard with an end-to-end negative
        control (re-injected the old squoosh call, watched the run fail), re-ran
        the full harness three times (~70 min wall), re-derived every published
        cell mechanically from the fresh JSON, re-measured the resampler claim,
        md5-verified the `web` == `-q 80` identity, and rewrote the affected
        BENCHMARKS.md / DEC-080 / README prose. Mostly waiting on encodes.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 450000
      duration_minutes: null
      estimated_usd: 4.0
      note: >
        Re-verify (second verify pass) on Opus 4.8 (no metered subagent) —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. Scope: an
        INDEPENDENT three-pass re-run of the harness (run1/run2 identical
        `--runs 3`, run3 per-core via `--q-from`, ~30 min wall) and a mechanical
        re-derivation of every published cell from that fresh JSON; the guard
        proven by two end-to-end negative controls (the shipped squoosh bug and
        an independent sharp `--fit fill` distortion, both exit 3) plus matching
        positive controls and an EXIF-Orientation=6 false-positive fixture; the
        12 sampled encodes reproduced byte- and score-exact from the DOC's own
        commands; `web` == `convert -q 80` md5-confirmed on all 8 photos; the
        sharp-resampler ~82 outlier reproduced against a common reference.
        Outcome: ⚠ PUNCH LIST — 1 substantive + 2 minor, all doc prose; every
        number re-derived and no re-measurement required.
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 120000
      duration_minutes: null
      estimated_usd: 0.9
      note: >
        Third build pass — PROSE ONLY, on Opus 4.8 (no metered subagent) —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. No
        benchmark was re-run and no number changed. Scope: reproduced the
        ImageMagick claim's failure mode directly (PNG reference write fails,
        AVIF encode succeeds), checked what `web`/`optimize` actually print,
        read the harness's reference path to describe it accurately, and
        rewrote the affected prose in BENCHMARKS.md / DEC-080 / this spec /
        the timeline / one harness docstring line. Cheap: no encodes.
  totals:
    tokens_total: 3270000
    estimated_usd: 24.5
    session_count: 5
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

### Second build pass — clearing the verify punch list (2026-07-20)

Verify came back NOT CLEAN. One finding forced a full re-run, so every published
number below is re-derived from a fresh three-pass benchmark, not patched.

**What was wrong, and what changed:**

1. **`@squoosh/cli` was benchmarked distorted (blocking).** The harness passed both
   `width` and `height` to `--resize`; squoosh's `resizeWithAspect` stretches to the
   box when given both and only derives the missing axis when given one. Six of the
   eight photos were squashed to a square. **Own-reference scoring hid it perfectly**
   — the distorted encode scored against the distorted reference still landed in the
   82 band, so the quality column offered zero protection. Fixed by constraining the
   long axis only.
2. **The harness now measures what it claims.** "Same pipeline for every tool" was an
   unchecked claim; nothing in the harness looked at output shape. Every reference
   and every grid output is now measured against the source long edge and aspect
   ratio (orientation-insensitive, so `web`'s EXIF-baked transpose isn't a false
   positive); a violation flags the row and exits 3. **Negative control:** the old
   squoosh call was re-injected via a patch-in driver and the guard failed the run
   end-to-end, reproducing the exact poisoned published cell (DSC_0163 squoosh
   245 KB · 82.5) while flagging it. `--self-test` covers eight shapes — including
   both real bugs — with no corpus and no tools installed.
3. **Portrait sources fixed for sharp and cwebp.** Confirmed the old arguments
   really produced 2048×3068 from a 4016×6016 source for both (2.2× the pixels).
   sharp now gets the full `resize E E --fit inside` box (byte-identical output on
   landscape, so no published number moved from this); cwebp pins whichever axis is
   long. A full six-tool run on a portrait source passes the guard.
4. **The per-core table is genuinely controlled.** New `--q-from` re-times *the same
   encodes* the main table matched, so only the thread count changes; previously the
   band was re-picked under one thread and sharp landed at a different quality. The
   table now prints the matched score for both arms.
5. **DEC-080's calibration was factually wrong** and the error flattered the setup:
   `crustyimg web` is byte-identical to `convert --format avif -q 80` (md5-verified
   here on three photos; byte counts identical on all eight), **not** `-q 85`, and it
   lands 73.5–79.0 / median **75.2**, not "≈79–82". So 82 is ~7 points *above*
   crustyimg's real default, and the stated rationale ("the band centre is
   crustyimg's real operating point") was unfounded. Re-justified honestly: 82 is the
   band every tool's grid can bracket and a quality worth shipping; anchoring on
   crustyimg's own default would have dragged four competitors down to a band chosen
   to suit crustyimg's fast preset. The RESULTS stood — BENCHMARKS.md already
   reported the honest ~75 and disclosed the tune-up.
6. **Prose corrections:** "3–8× faster" → 3–9× (sharp) and 4–14× (ImageMagick);
   "none of the competitors ship a perceptual quality readout" was false (`magick
   compare -metric DSSIM/SSIM`, `cwebp -print_ssim`, both confirmed) → narrowed to
   SSIMULACRA2, reported as part of the encode and gateable via `diff --fail-under`;
   matched-score span 79.0–83.6 (was prose "79–83.5" against a table showing 83.6);
   the documented squoosh and cwebp commands now match what the harness runs, with
   `E = min(2048, long edge)` spelled out; README "2 to 5 seconds" → "about one to
   five seconds" (the doc's own small-bucket `web` row is 967 ms).
7. **The resampler claim was re-measured, and it changed.** DEC-080 said cross-tool
   downscales are 92–95 similar, so the resampler is a second-order effect. With every
   downscale now aspect-correct: 90.9–94.5 on the photos sampled, but sharp's
   lands at **~82** against the others on one 24 MP photo. Own-reference scoring is
   doing real work — against a shared reference that gap would have been charged to
   sharp's encoder. Corrected in both DEC-080 and the doc.

**The re-run (identical config, no hand-edits):** run 1 and run 2 both `--runs 3
--warmup 1` (the first pass compared `--runs 3` against `--runs 1`, which is not a
like-for-like reproducibility claim); run 3 per-core with `--q-from run1.json`.
Dimension check PASSED on all three. Determinism: **141/141** deterministic fields
identical run1≡run2; wall-time drift median 1.6%, 5 of 47 over 5%, max 19.6% on a
416 ms measurement — the doc now says that instead of "≤ ~2%".

**What moved in the results:** the smallest-AVIF tally **flipped from sharp 5 /
IM 2 / squoosh 1 to sharp 4 / IM 2 / squoosh 2** — aspect-correct squoosh is
markedly smaller (DSC_9952 26→21 KB, DSC_0974 415→181 KB, DSCN3478 520→422 KB) and
now wins two photos, one of them a 0.4% edge over sharp on the 47 MP Leica.
crustyimg's worst case against the smallest widened from ~1.5× to ~1.7×. Every
bucket median, per-photo cell and per-core row was re-derived from the fresh JSON
and then **mechanically cross-checked** against it (47 per-photo + 18 bucket +
8 per-core cells, all match). The headline is unchanged: crustyimg is neither the
smallest nor the fastest, and per core it is still faster on four of eight.

- **All acceptance criteria met?** yes
  - BENCHMARKS.md with matched-quality methodology, machine + pinned versions,
    exact per-tool commands, size+speed+quality tables per size bucket, and stated
    caveats incl. two axes where crustyimg loses (smallest; wall-clock speed). ✓
  - Reproducible: `just bench-compare --corpus <dir>` regenerates the tables; two
    IDENTICALLY-configured runs matched on every deterministic field (chosen
    quality, bytes, score — 141/141). Wall-times moved: median 1.6%, 5 of 47 over
    5%, max 19.6% on a 416 ms measurement. No hand-edited numbers (tables built
    from the harness JSON, then mechanically cross-checked against it). ✓
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
    wins size on 4/8, ImageMagick 2, squoosh 2) nor fastest (sharp is 3–9× and
    ImageMagick 4–14× faster on wall-clock,
    being multi-threaded). Reported straight; per-core it's a wash vs single-thread
    libvips, so the value framing rests on portability + measured quality + RAW +
    wasm, not raw compression/speed superiority.
  - **The 47 MP Leica has no ImageMagick cell — a limit of this method, not of
    ImageMagick.** magick encodes that file to AVIF fine (rc 0, 2048×1367); what it
    won't write is the lossless PNG *reference* the scoring needs ("Incorrect data
    in iCCP" — the source's embedded colour profile trips magick's PNG writer;
    `-strip` makes it succeed). Stated as our limitation; that cell is excluded
    (magick is n=4 in the large bucket).
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
  - **sharp's resampler on one 24 MP photo** lands ~82 SSIMULACRA2 against every
    other tool's downscale of the same file (the rest cluster 90.9–94.5), and it isn't
    `fastShrinkOnLoad` (disabling it is byte-identical). Own-reference scoring makes
    it harmless for this doc, but the cause is unexplained and worth a look if the
    downscale ever becomes the thing being compared.

### Third build pass — prose only (2026-07-21)

Re-verify re-derived every published number independently (157/157 fresh cells,
230/230 against the run JSONs, tally and per-core reproduced, the dimension guard
proven to fire on a distortion it was never shown) and found **no number wrong**.
The findings were all wording. Nothing was re-measured and no table cell moved;
the diff is prose in `BENCHMARKS.md`, `DEC-080`, this spec, the timeline, and one
docstring line in the harness.

1. **The ImageMagick caveat was false and any reader could disprove it.** The doc
   said magick "refused the 47 MP Leica outright" and was "less tolerant of odd
   inputs" than the tools that "read it without complaint". Checked here:
   `magick L1024678.JPG -resize '2048x2048>' -quality 70 out.avif` returns 0 and
   writes a valid 2048×1367 AVIF. What fails is `... ref.png` — magick's PNG
   writer, on that source's embedded ICC profile ("Incorrect data in iCCP";
   `-strip` fixes it) — and that PNG is *our* scoring reference, not the
   benchmarked pipeline. Restated as a limit of this method in all three places
   (doc, this Build Completion, timeline); the "less tolerant" characterization and
   the contrast with the other tools are gone, because neither was earned.
2. **"cwebp is larger than every AVIF tool here" (twice) contradicted the doc's own
   table** — cwebp beats ImageMagick on DSC_2011 (166 vs 167 KB) and DSC_9952
   (65 vs 105 KB). Replaced with the measured, checkable claim: ~1.2×–3.0× the
   *smallest* AVIF on every photo, and the largest median in all three buckets.
3. **One resampler range, matching the measurement.** BENCHMARKS.md said 92–94,
   DEC-080 said 91–94; measured is 90.87–94.53. Both now say **90.9–94.5**, as does
   this spec.
4. **Nits.** The score-readout bullet claimed `web` and `optimize` "report it as
   part of the encode" — verified here that `optimize` is score-free without
   `--verify` and `web -o FILE` prints nothing (the `· ssim` line comes with
   `--out-dir`/default naming), so the sentence now names the forms that actually
   print it. TL;DR "it's the slowest" is now the precise version (all three bucket
   medians, 7 of 8 photos — squoosh is slower on the 47 MP). The harness docstring
   cited a two-process disclosure that existed in neither doc; the disclosure is now
   in BENCHMARKS.md (crustyimg's tuned path is two commands and its time sums both)
   and the docstring cites only that.

`just validate` green, `--self-test` green, no `src/` change, no number changed.
Handed back for a short prose-only re-verify — NOT merged.

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

### Second-pass reflection (the fix cycle)

1. **What was unclear in the spec that slowed you down?**
   — Nothing new; the punch list was specific enough to act on directly. The real
   cost was wall-clock: any finding that changes an input forces a full re-run, so
   a single wrong CLI flag cost ~70 minutes of re-measurement.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That every tool spells "downscale the long edge" in a different dialect, and
   two of the four dialects silently do something else (squoosh stretches when given
   both axes; sharp and cwebp constrain only the axis you name). A cross-tool
   benchmark needs the shared pipeline asserted from day one, not assumed — and the
   assertion has to be structural, because the obvious guard (the quality column)
   provably cannot see the failure.

3. **If you did this task again, what would you do differently?**
   — Write the output-shape assertion *before* the first timed run. Every number in
   the first pass was correct in the sense of "the harness reported what it
   measured"; it was wrong because nobody checked what the harness was measuring.
   Also: read the competitor's own resize source rather than trusting its flag names
   — `resizeWithAspect` answered in thirty seconds what the whole benchmark got wrong.

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
