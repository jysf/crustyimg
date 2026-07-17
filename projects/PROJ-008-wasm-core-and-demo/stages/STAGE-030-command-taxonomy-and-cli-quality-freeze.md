---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-030
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-14
shipped_at: null

value_contribution:
  advances: >
    The last thing between crustyimg and a 1.0-worthy launch: freeze a CLI surface people can trust
    and make the verb everyone runs deliver the measured win. A benchmark over a real photo corpus
    proved the current default optimizes the least-valuable variable (24% in 16.5 s, 2/8 passthrough,
    49 s on a 47 MP photo) while a downscale→AVIF "web" path gets 98% in 2.7 s. This stage ships that
    flow as the flagship and cuts the surface from ~20 verbs to ~14, each with one clear intent —
    before the README and Show HN document it, so we never relaunch with a changed CLI.
  delivers:
    - "A `web` flagship verb: downscale + content-aware modernize (AVIF for photos / lossless-WebP for graphics) + never-bigger + strip + orient — the 98%/2.7s path, one command"
    - "A fast, AVIF-aware DEFAULT engine decision (fixed-quality single-encode, not the 9–74 s byte-budget search) with a first-class 'kept, already optimal' passthrough"
    - "`optimize` demoted to an honest byte-primitive (best format at good quality, keep dimensions, never-bigger, --verify); the perceptual search becomes opt-in via --target/--ssim/--max-size"
    - "A cleaner ~14-verb surface: `convert --to`, a `meta` group (strip/clean/set/copy), bundled recipes (web/gallery/product/…), `shrink` removed (its behavior folded into `web`, upgraded with AVIF)"
    - "A unified audit report + --json/--timing across lint/optimize/web, and a committed benchmark corpus + harness (no telemetry)"
  explicitly_does_not:
    - "Add a backend/service, a new codec, or ML — the territory guardrails stand"
    - "Touch the wasm surface (SPEC-079 shipped the wasm twin of the engine decision) or add a CLI --speed flag (DEC-020)"
    - "Do the README/BENCHMARKS authoring (SPEC-082/083, STAGE-028) — this stage produces the surface + numbers those will document"
    - "Preserve backward compatibility: this is a HARD CUTOVER (rename/remove/merge, no aliases, no deprecation) — done now because the surface has no dependents but the maintainer"
---

# STAGE-030: command taxonomy & CLI-quality freeze

## What This Stage Is

The pre-launch surface freeze. A measured benchmark (2026-07-14 — a real 8-photo corpus, 0.7–47 MP,
5 cameras, + an AVIF quality sweep; harness in `scratchpad/bench/`) settled the "is the hero
`optimize`, `shrink`, or modernize-to-AVIF?" question with numbers: the hero is a **`web`** flow —
downscale to a web size, then modernize by content (AVIF for photos, lossless-WebP for graphics),
never bigger. It gets **98% median savings in 2.7 s and is size-insensitive** (2–5 s from 0.7→47 MP),
where today's `optimize` default gets **24% in 16.5 s** (2/8 passthrough, up to 49 s).

So this stage (a) makes the **default engine decision fast and AVIF-aware** (SPEC-084 — the native
twin of the wasm surface SPEC-079 already shipped), (b) ships **`web`** as the flagship verb built on
bundled recipes (SPEC-085), (c) **redefines `optimize`** into an honest keep-dimensions byte-primitive
and **removes `shrink`** (SPEC-086), and (d) **cleans the surface** to ~14 one-intent verbs
(`convert --to`, a `meta` group, an elevated audit pillar) with a unified report + committed bench
(SPEC-087/088, opt. 089). It is a **hard cutover** — no aliases, no deprecation — because crustyimg
has no dependents but the maintainer, and relaunching with a changed CLI later is strictly worse.

## Why Now

- **Freeze before you document.** The README (SPEC-082) and BENCHMARKS (SPEC-083) must show the
  *final* surface and these numbers. Maintainer decision (2026-07-14): **freeze the CLI first, then
  launch.** So STAGE-030 lands before STAGE-028.
- **The win is measured, not intuited** — a real corpus + a q-sweep, not a hunch. The root cause is
  understood: the "faceplant" is the byte-budget AVIF search re-encoding rav1e (9–74 s) plus a
  conservative visually-lossless target — both fixable by admitting AVIF at *fixed* quality and making
  the search opt-in.
- **The enabling fact is already true.** Native AVIF *decode* shipped (DEC-058), so the
  `Mode::SizeBudget`-only AVIF gate in `decide.rs` is vestigial on native — the only 2× win is being
  excluded from the default path for a reason that no longer holds.
- Maintainer decision (2026-07-14): **fold into PROJ-008** as a pre-launch stage (not a new PROJ-010 —
  AGENTS.md frames a project only after the prior ships, and this is the same launch push).

## Success Criteria

- The **default** decision on a photo picks **AVIF at fixed quality via a single encode** (no 9–74 s
  search), beats the source substantially, and reports a once-computed SSIMULACRA2 score; on a graphic
  it stays **lossless**; when nothing beats the source it **passes the original through** (never bigger).
- **`web <inputs>`** exists and delivers the measured flow (downscale + content-modernize + never-bigger
  + strip + orient), size-insensitive, in a few seconds — `web == apply --recipe web`.
- **`optimize`** is an honest keep-dimensions byte-primitive with `--verify`; the perceptual/byte-budget
  **searches are opt-in** (`--target`/`--ssim`/`--max-size`). **`shrink` is gone.**
- The surface is ~**14 verbs**, each with one intent: `convert --to`, a `meta` group, bundled recipes,
  the audit pillar (lint/info/view/diff) elevated, a unified report + `--json`/`--timing`.
- A **committed benchmark corpus + harness** (seeded from `scratchpad/bench/`, no telemetry).
- The default AVIF quality is **validated on eyeballs** (a small diverse corpus + the q-sweep), not
  hardcoded blind. All gates green (native + `--features avif` + lean).

## Scope

### In scope
The engine default-decision change; the `web` verb + bundled-recipe registry; the `optimize`
redefinition + `shrink` removal; the `convert --to` rename; the `meta` consolidation; the unified
audit report + `--json`/`--timing`; the committed bench corpus/harness; all in-repo doc/test updates
the cutover requires.

### Explicitly out of scope
- README/BENCHMARKS authoring (SPEC-082/083); the wasm surface (SPEC-079); a CLI `--speed` flag
  (DEC-020); any backend/codec/ML; backward-compat aliases (deliberate — hard cutover).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`. Build order: **084 → {085, 086} → {087, 088}
→ 089 (opt)**.

- [x] SPEC-084 (shipped 2026-07-14, PR #88, DEC-069) — **engine: fixed-quality AVIF in the DEFAULT
  decision** via single-encode compare (`Mode::Fast`, not the search), q85 two-regime quality, a
  never-bigger + metadata-safe fallback, a score-winner-once helper (gated off the keep-dims default).
  Native twin of SPEC-079; converges the native/wasm Auto paths. Verified CLEAN after a fix pass
  (caught a never-bigger+honesty blow-up on the metadata-bearing graphic edge). $7.10.
- [x] SPEC-085 (shipped 2026-07-15, PR #89, DEC-070) — **`web` flagship verb** (= `optimize` + default
  downscale to 2048 + always-on score, reusing SPEC-084's engine) + bundled-recipe registry
  (web/gallery/product, file-path-wins precedence). **`web == apply --recipe web` DELIVERED** (not
  descoped) via a terminal-optimize recipe step (DEC-070). RAW highlight works. Verified CLEAN after a
  fix pass (the pinned `-o`/`--format` corner ignored the pin). $5.25.
- [x] SPEC-086 (shipped 2026-07-15, PR #90, DEC-071) — **redefine `optimize`** surface: added
  **`--verify`** (opt into the score-once; JSON gains an `"ssim"` field, non-verify byte-identical) +
  **removed `shrink`** (hard cut, no alias — `web` absorbs it) + fixed the stale `run_optimize`
  doc-comment. Independent verify CLEAN (byte-identity both ways; live surface `shrink`-clean). $3.30.
- [x] SPEC-087 (shipped 2026-07-15, PR #91, no DEC) — **`meta` group** consolidation: folded the 3
  existing metadata verbs (`strip`/`clean`/`copy-metadata`) into `meta strip`/`clean`/`copy`; a pure
  hard-cutover surface move (byte-identity proven against the OLD binary), auto-orient stays top-level.
  **Grounding correction (see Design Notes):** a top-level `set` verb (SPEC-027) DOES exist — it was left
  top-level per scope; **maintainer decided to fold `set` → `meta set`** in a follow-up so the group is
  whole. Complexity S.
- [x] SPEC-088 (shipped 2026-07-16, PR #92, **DEC-074**, ~$19.75 / 7 sessions) — **unified audit report**
  (`--json` + `--timing` on optimize/web/apply, `optimize.explain/v1` extended **additively + gated**;
  `lint` reconciled as the documented `lint --format json`, `docs/cli-reference.md` §"Audit surface") +
  a **committed bench** (`just bench` + `scripts/bench.py`, stdlib/offline/no-telemetry, `--corpus <dir>`
  capable; criterion → `just bench-micro`). Took **build → verify → fix → re-verify → fix2 → ship**; the
  engine was sound throughout (byte-identity 28/28 → 32/32 → 55/55 vs a pre-spec oracle) — every defect
  lived in the reporting layer. See the spec's Ship Reflection for the three banked lessons.
  **⚠ Two carries:** (1) the committed corpus **cannot** show `web`-vs-`optimize` (nothing exceeds the
  2048px bound → the verbs converge on every row; a >2048px real photo can't be both committed and lean —
  measured ~620 KB) — a `print_table` footer says so and auto-suppresses under `--corpus <real>`; **every
  SPEC-083 headline number must come from `--corpus <real>`.** (2) A **pre-existing `--features avif` test
  flake**: re_rav1d's debug-only `DisjointMut` overlap check panics under load (`disjoint_mut.rs:837`),
  reached because `web` always scores → decodes the AVIF winner. Reported-not-fixed; **under
  investigation** (disjointness is *unchecked in release* → possible UB in shipped builds).
- [x] SPEC-089 (shipped 2026-07-17, PR #93, no DEC, ~$4.67 / 4 sessions) — **folded `set` → `meta set`**;
  the metadata group is now whole (`meta {strip,clean,copy,set}`). Pure hard-cutover move; byte-identity
  proven against the pre-move oracle across 5 paths (3 flags / 1 flag / stdout / fan-out / PNG), confirmed
  with exiftool. **This was the near-controlled MODEL EXPERIMENT** — build on **Sonnet** ($1.03) vs its
  mirror SPEC-087's build on **Opus** ($2.30), verify held constant on Opus. Result: indistinguishable on
  the hard parts; Sonnet's self-awareness about its own proof was its *best* quality; it lost only on
  mechanical-sweep thoroughness (6 files vs 15). **The decisive datum: verify found 2 stale docs, then one
  mechanical `grep` found 5 MORE it had missed — 7 total. A mechanical sweep needs a mechanical check, not
  model judgment.** Same species as SPEC-088's defects, which happened on Opus → a **process gap the model
  choice widened, not a Sonnet defect**. `model:` is now recorded per cost session. See the Ship Reflection.
- [~] SPEC-090 (design — framed build-ready 2026-07-16) — **reconcile `web`'s never-bigger claim with its
  actual baseline.** SPEC-088's verify measured `web` shipping a file **14% larger** than a 3000px source;
  the pre-spec oracle reproduces it → **pre-existing SPEC-085 behavior**, honestly reported, but
  `pick_winner` enforces never-bigger against the **downscaled intermediate** while the recipe + docs
  promise "never shipping a larger file" (i.e. than the user's input). Genuine design tension: `web`'s
  dimension contract (≤2048px) means passthrough-the-original isn't an equivalent fallback. Decide
  claim-vs-behavior **with evidence**; DEC-075. `optimize`'s unconditional keep-dims guarantee is out of
  scope and unchanged. Complexity S, priority high (flagship promise).
- [~] SPEC-091 (design — framed build-ready 2026-07-17) — **AVIF decode thread policy**: `decode_obus`
  uses `Settings::new()` (dav1d default `n_threads=0` = ALL CORES) and never caps threads, so EVERY
  decoder spawns its own pool — inside a rayon batch that already parallelizes across files (DEC-006).
  Very likely what trips re_rav1d's `DisjointMut` overlap check (SPEC-088's flake). Severity BOUNDED:
  upstream's contract says provenanceless targets ⇒ wrong results, **NOT** memory unsafety — not a
  security fix; the bite is a wrong SSIMULACRA2 score + untrustworthy decode timings. Repro FIRST, then
  measure single + batch, then decide (`set_n_threads(1)` is the prior). **Land before SPEC-083** or its
  BENCHMARKS decode numbers are measured under oversubscription. DEC at build. Complexity S.
- [ ] SPEC-092 (optional / may fold) — `convert --to` rename + social/archive recipes.
- [~] SPEC-093 (design — framed build-ready 2026-07-17, **BUG, high**) — **the metadata write path
  corrupts numeric EXIF tags.** `meta clean --gps` (the PRIVACY verb, documented to preserve
  "orientation, copyright, ICC") rewrites **Orientation 6 → 1536**; `meta set` does the same AND degrades
  GPS to a plausible-but-wrong coordinate (50.4957 → 50.4843…). Shared container lane
  (`src/metadata/tiff.rs`), **pre-existing** (SPEC-026/027 era). Survived because **ASCII tags are immune
  and every test checks only ASCII tags**, and byte-identity proofs compared against an equally-broken
  oracle. Orchestrator's byte-order hypothesis REFUTED at framing (both MM and II corrupt) → mechanism
  undiagnosed. The test gap is as much the deliverable as the fix. Complexity M.

**Count:** 6 shipped (SPEC-084/085/086/087/088/089) / 3 in design (SPEC-090, SPEC-091, SPEC-093) / 1
pending (SPEC-092, optional `convert --to`). The taxonomy work is **done**: fast-AVIF default + `web` +
`optimize` demoted + `shrink` gone + a whole `meta` group + the measurable-honesty pillar. Remaining is
**quality/correctness**, in priority order: **093** (the metadata lane silently corrupts Orientation/GPS —
the privacy verb breaks its own documented promise; arguably a launch blocker), **090** (`web`'s
never-bigger measured against the wrong baseline), **091** (AVIF decode thread policy — must land before
SPEC-083). **091 and 093 are independent** (`src/image/avif.rs` vs `src/metadata/tiff.rs`); 090 touches the
decision path. **Then: reframe SPEC-080 (demo)** — the `web` hero it needs is shipped.
**Follow-up still filed out of SPEC-088** (not blocking): a regression test asserting
`bench/corpus/photo_forest_cc0.jpg` classifies `photograph`, so a future classifier change fails loudly
instead of silently falsifying the corpus README's class column. (The re_rav1d investigation is now
**SPEC-091**.)
## Design Notes

- **Hard cutover discipline.** Rename/remove/merge freely; no aliases, no deprecation, no CHANGELOG
  migration. Cost is in-repo docs + tests only. Every reference to a renamed/removed verb gets updated.
- **SPEC-087 `meta` group — built (no DEC).** A pure surface move within the freeze: `strip`/`clean`/
  `copy-metadata` → `meta strip`/`meta clean`/`meta copy` (nested clap subcommand; handlers unchanged;
  byte-identity proven). No decision record needed — it changes only the path, not behavior. **Grounding
  correction:** the SPEC-087 framing (and the backlog line above) said "no `set` verb exists today" — that
  is **wrong**; a top-level `set` (SPEC-027) exists. Build left `set` top-level per the spec's enumerated
  scope, which left it the one metadata verb *outside* `meta`. **RESOLVED (maintainer, 2026-07-15):** fold
  `set → meta set` in its own follow-up spec (a pure move mirroring SPEC-087) so the group is whole
  (`meta {strip,clean,copy,set}`) — the git-config / gh-secret pattern where a noun-group holds its
  read/remove *and* write verbs together. **Framed as SPEC-089 (2026-07-16), build-ready.**
- **Honesty guardrails (non-negotiable):** passthrough is a **green** result ("kept, already optimal"),
  not a failure; **never silently enlarge** (subsumes the Track-B `optimize` fix); downscale is
  **`web`'s** opinion, not `optimize`'s (optimize keeps dimensions); don't claim *visually-lossless* at
  a generous default — say the measured score.
- **The one judgment-bound number is the default AVIF quality** (two-regime: `web` generous ~q85–90;
  `optimize` can lower via `--target`). The q-sweep shows bytes are trivial across the range after a
  downscale, so be generous — but **validate on eyeballs**, and sanity-check the `-q`→perceptual
  mapping (q80 → ~77 is more aggressive than the label). Record the chosen value + rationale in DEC-069.
- **Content branch already works** (the bucket classifier): a screenshot → lossless WebP (AVIF on it is
  a 4× regression). Do **not** fire AVIF on graphics.
- **AVIF admission is a bucket predicate**, not shortlist membership (SPEC-079's `MAX_SHORTLIST`
  truncation lesson).
- **The SSIMULACRA2 score is informational, and its cost is dimension-linear — measured 2026-07-14
  (release): ~107 ms/MP** (1 MP 114 ms, 3 MP 330 ms, 12 MP 1.31 s, ~5 s at 47 MP; via `diff`, which
  decodes both — the in-flow score-once is a touch cheaper since the input is already decoded). Two
  consequences for the score-once readout: (1) the **fast default picks the winner by BYTES** (smallest
  beats source), so the score is **never needed for the decision** — it can be gated freely. (2) The
  gate: **`web` shows it always** (it scores the downscaled ~2–3 MP output → ~0.2–0.35 s, effectively
  free), **keep-dimensions `optimize` shows it only under `--verify`** (SPEC-086) — because scoring a
  full-res 12–47 MP image is 1.3–5 s. SPEC-084 provides the score-once *helper*; do NOT wire it
  always-on into the keep-dims default.

## Dependencies

### Depends on
- SPEC-079 (shipped) — the wasm twin of the default decision; SPEC-084 mirrors its shape and the two
  paths converge. DEC-058 (native AVIF decode) — what makes admitting/scoring AVIF sound on native.

### Enables
- STAGE-028 (README/BENCHMARKS, SPEC-082/083) — which document the frozen surface + these numbers.
- The demo (STAGE-029, SPEC-080) — its hero becomes the **`web`** flow; reframe SPEC-080 to mirror
  `web`'s definition once SPEC-085 lands.
- The Show HN launch, on a surface we won't have to change.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
