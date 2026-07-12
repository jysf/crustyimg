---
# Maps to ContextCore project.* semantic conventions.
project:
  id: PROJ-007
  status: shipped                   # proposed | active | shipped | cancelled
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: 2026-07-12

value:
  thesis: >
    Turn crustyimg from a tool that RUNS image transforms into one that BUILDS an
    asset tree incrementally and verifiably — "a Makefile for images." Today `apply
    --recipe` re-does all the work on every run and offers no reproducibility
    guarantee; this wave adds a declared `build` (sources × recipes → outputs), a
    content-addressed cache so a re-run does only the work that changed, and a
    committed lockfile that pins the build so CI can assert outputs are reproducible
    — plus `--watch` for the dev inner loop. This is the roadmap's Wave 2 and the
    "build-native & verifiable, no service, no CDN" half of the territory thesis:
    deterministic, reviewable, local. It is also near-ideal agent/automation work —
    turborepo/Bazel-shaped, clear pass/fail.
  beneficiaries:
    - "Frontend/content devs with an asset tree (many images × recipes) who want fast incremental rebuilds"
    - "Build-step / Makefile / GitHub-Actions shops that want image outputs reviewed + verified like code"
    - "CI (a committed lockfile → 'outputs match' gate; cache hits → fast pipelines)"
    - "The maintainer/agents (a deterministic, well-specified, clear-pass/fail surface)"
  success_signals:
    - "`crustyimg build` runs a declared build; a second run with no changes does ZERO work (full cache hit) and says so"
    - "Changing one source (or one recipe param, or the crustyimg version) rebuilds ONLY the affected outputs"
    - "A committed lockfile pins the build; `build --check`/`--frozen` fails in CI when outputs would drift"
    - "`build --watch` re-runs only affected targets on file changes"
    - "Same inputs + recipes + crustyimg version → stable, reproducible outputs (the lockfile is meaningful)"
  risks_to_thesis:
    - "Encoder byte-reproducibility across platforms/versions is UNVERIFIED — it decides whether the lockfile can pin OUTPUT hashes or only cache keys (the load-bearing probe)"
    - "Cache-key correctness is the classic hard problem: miss one output-affecting input (a param, quality, the version) → stale outputs shipped silently"
    - "Scope creep into a general build system — this must stay 'a Makefile for images', not Bazel"
    - "`--watch` reliability (debounce, rename/atomic-write editors, large trees) is fiddly cross-platform"
---

# PROJ-007: Reproducible build (build + cache + lockfile + `--watch`)

## What This Project Is

The **reproducible-build wave** — roadmap Wave 2. crustyimg already runs an ordered
TOML recipe over one image or a batch (`apply --recipe`, decode-once pipeline, rayon
batch). This wave wraps that in a declared, cached, verifiable **build**: you declare
targets (source globs × a recipe → an output directory + name template), and
`crustyimg build` resolves them, runs them through the existing pipeline, and — via a
**content-addressed cache** — does only the work that actually changed. A committed
**lockfile** pins the resolved build so a re-run (or CI, or a teammate) is
**reproducible and checkable**, and `--watch` gives the fast dev inner loop. The
pitch is "a Makefile for images, verifiable" — the local, deterministic, no-service
counterpart to a delivery CDN's per-request `f_auto` (`docs/territory.md`).

## Why Now

- **It's the best daily-driver feature and the cleanest agent work left before 1.0.**
  The input-reach wave (PROJ-009) broadened *what* crustyimg reads; this wave makes
  *running it over a real asset tree* fast and trustworthy. It is turborepo/Bazel-shaped
  — deterministic, well-specified, clear pass/fail — so it is exactly the kind of work
  that goes fast and reviews cleanly (roadmap sequencing rationale, 2026-07-07).
- **It completes the "build-native & verifiable" leg of the identity.** `docs/territory.md`
  stakes the claim on *deterministic, reproducible recipes reviewed like code* for the
  build/CI/pre-deploy layer. Without incremental + lockfile, "reviewed like code" is
  aspirational; this wave makes it real.
- **The load-bearing unknown is small and probeable now.** Whether our encoders are
  byte-reproducible (across a re-run, and across platforms) decides the lockfile design —
  a design-time probe (encode the same input twice; diff bytes) resolves it before we
  commit to hashing outputs vs. only cache keys.

## Success Criteria

- `crustyimg build` runs a declared build end to end; a no-change re-run is a **full
  cache hit** (near-zero work) and reports hits/misses/rebuilt.
- A change to a single source, recipe param, or the crustyimg version **rebuilds only
  the affected outputs** (correct, minimal invalidation).
- A committed **lockfile** makes the build reproducible; a `--check`/`--frozen` mode is
  a CI gate that fails on drift, with a clear diff.
- `build --watch` re-runs only affected targets on file changes (debounced, robust to
  editor atomic-writes).
- Everything stays **local + deterministic + no service** (no CDN, no network cache) and
  pure-Rust-default; `just deny` green; the lean build unaffected.

## Scope

### In scope
- A **`build` command** driven by a declared build (targets: source globs × a recipe →
  output dir + name template), over the existing pipeline/batch. **(STAGE-020)**
- A **content-addressed cache** — a local store keyed by a hash of (input bytes +
  resolved recipe/params + crustyimg version + output config); skip-unchanged;
  hit/miss/rebuilt reporting. **(STAGE-021)**
- A **reproducibility lockfile** (source→output content hashes + resolved config) +
  `build --check`/`--frozen` for CI. **(STAGE-022)**
- **`--watch`** — a file-watching inner loop that rebuilds only affected targets. **(STAGE-023)**
- The **determinism decisions** these require (what is in the cache key; whether the
  lockfile pins output hashes or only cache keys; encoder-reproducibility policy).

### Explicitly out of scope
- The **web-asset manifest / placeholders / favicon** (Wave 4 / PROJ-005) — a build could
  later *emit* a manifest as one output type, but the manifest artifact + SSG integration
  are a separate wave.
- **Remote / distributed / networked cache** — local only (the no-service, no-CDN guardrail,
  `docs/territory.md`). No cache server, no cloud.
- **WASM / demo page** (Wave 3), **geometry** (Wave 5), **smart-crop/auto-color** (post-1.0 beta).
- A general-purpose build system / arbitrary task graph — this is "a Makefile for images",
  scoped to image targets over recipes, not shelling out to arbitrary commands.
- New **input or output formats** (PROJ-009 covered inputs; encoders are as shipped).

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-020 (shipped on 2026-07-08) — the `build` command + `crustyimg.build.toml` manifest
  (targets = source × recipe → out/name) + executor that prepares all targets then loops the shipped
  `apply_one`. The skeleton: "declare my asset build and run it." **No new dep** (SPEC-063, PR #69, DEC-057).
- [x] STAGE-021 (shipped on 2026-07-09 — SPEC-064, PR #70, DEC-058) — content-addressed cache (incremental
  rebuild): the cache key (over every output-affecting input) + local `.crustyimg/cache/` store (atomic,
  self-describing, verify-on-read, corrupt→miss) + skip-unchanged + `(C cached, R rebuilt)` reporting +
  `--no-cache`. The headline; **encoder-determinism experiment retired the nondeterminism risk** (byte-identical
  run-to-run/thread-count incl. AVIF+lossy-WebP on a fixed machine) → shipped as the *robust* half of
  verifiable (cache-correctness, deterministic-within-env), distinct from STAGE-022's *fragile* cross-arch
  byte-reproducibility. One pure-Rust dep (`sha2`). Keys on output-**byte** identity (not path), so the
  injective source→output constraint (DEC-057) is **not** resolved here and stays a STAGE-022 blocker.
- [x] STAGE-022 (shipped on 2026-07-09) — reproducibility lockfile + `build --check`/`--frozen` (the CI drift
  gate). **The "verifiable" leg — DONE.** **SPEC-065** (PR #71, bc13c4d): the injective source→output guarantee
  (reject same-output collisions at prepare, exit 2, discharging DEC-057's blocker). **SPEC-066** (PR #73,
  ce2fc69, DEC-059): the committed `crustyimg.build.lock` + `--check`/`--frozen`/`--locked`/`--strict` — pins
  each output's cache key (robust inputs, DEC-058) + records the observed output hash + env; `--check` fails on
  input drift (exit 7), cross-env byte variance informational unless `--strict`; perceptual/SSIMULACRA2 (shipped
  `diff`) stays the review-grade check. No new dep across either spec.
- [x] STAGE-023 (shipped on 2026-07-10 — SPEC-067, PR #74, DEC-060) — `--watch`: a debounced file-watching
  inner loop. A thin loop over the shipped `run_build` (the STAGE-021 cache makes a full re-run incremental,
  so "only affected rebuilds" is free — no dependency graph); watch manifest + recipes + source roots,
  **exclude own outputs/cache/lock so it never self-triggers** (`is_excluded` normalizes both sides — verify
  hammered it under symlinked `/tmp` + a whole-cwd `.` glob, no self-trigger); loop-resilient; Ctrl-C via
  default SIGINT. One new dep (`notify`, threads+mpsc, not async — DEC-006 holds), `just deny` green via 3
  scoped per-crate exceptions (notify CC0-1.0, inotify/inotify-sys ISC). Shipped deviation with teeth: a
  **two-tier WatchSet** (recursive source roots, shallow manifest/recipe dirs) — a recursive watch over the
  config dirs also covered `.crustyimg/` and overflowed Linux inotify (3-OS CI caught it).
- [x] STAGE-024 (shipped on 2026-07-12) — hardening & security sweep: the wave's closing correctness +
  security pass, queued LAST. Led with a threat-model / attack-surface review of PROJ-007's NEW untrusted-input
  surface (manifest, recipe, cache store, committed lockfile, `--watch`'s tree) against `untrusted-input-hardening`
  — **SPEC-068** (PR #75, DEC-061): threat-model note + inline tightenings (recipe top-level `deny_unknown_fields`
  closing a zero-step silent-passthrough footgun; an out-directory write-escape clamp) + a reprioritized backlog.
  **SPEC-069** (PR #76, DEC-062): ran the never-executed decoder fuzz gate — AVIF findings all upstream
  `avif-parse` (validating the pure-Rust decoder choices), boundary-fixed, converted to always-on per-PR
  regressions. **SPEC-070** (PR #78, DEC-063): closed the memory-amplification class (F-RAW-1) with a 64 Mpix
  pre-decode pixel budget. **SPEC-071** (PR #79): the small hardening tail as one batch (lint false-diagnosis,
  cache off-by-53, exit-code totality, `--watch` build-only, docs sync). 4 specs; no new default dep. **The
  larger decision-bearing tail (8 items) was triaged 2026-07-12 and DEFERRED to a post-1.0 maintenance pass**
  (see the stage's Spec Backlog) — the one security-flavored residual (canonicalize-contain-out) is an
  accept+document decision with a real usability tradeoff; the rest are polish / feature / fails-closed.

**Count:** 5 shipped / 0 active (STAGE-020+021+022+023+024 all shipped — build + cache + lockfile + `--watch`
delivered, and the closing hardening & security sweep done). **PROJ-007 CLOSED 2026-07-12.**

## Dependencies

### Depends on
- Shipped recipe/apply/pipeline surface: `src/recipe/` (Recipe TOML + `build_pipeline` + registry, DEC-005),
  `src/pipeline/` (decode-once executor), the `apply --recipe` batch path (SPEC-031, rayon, DEC-006),
  `src/source/` (glob/dir resolution), `src/sink/` (file/dir + name templates).
- PROJ-009 (input reach) — `build` ingests AVIF/SVG/RAW inputs like any other now.
- DEC-004 (pure-Rust default), DEC-034 (decode caps), the `untrusted-input-hardening` posture.
- A new hashing dep (e.g. `blake3`, permissive) + a file-watch dep (e.g. `notify`, permissive) — each
  gated behind a `DEC-*` at the stage that needs it (probe licenses + determinism first).

### Enables
- The SSG web-asset **manifest** (Wave 4) — it describes the outputs this build produces.
- **Reproducible releases / CI** for anyone using crustyimg in a build step; the "reviewed like code" story.
- Faster large-tree runs generally (the cache benefits `apply`/batch too).

## Project-Level Reflection

*Shipped 2026-07-12.*

- **Did we deliver the outcome in "What This Project Is"?** **Yes — in full.** `crustyimg build`
  runs a declared build (source globs × recipe → out dir + name template); a no-change re-run is a
  full cache hit that reports `(cached, rebuilt)`; changing one source / recipe param / the
  crustyimg version rebuilds only the affected outputs; a committed `crustyimg.build.lock` +
  `--check`/`--frozen`/`--locked` is a CI drift gate (exit 7); `--watch` gives the debounced dev
  inner loop. All of it stays local + deterministic + no-service + pure-Rust-default; `just deny`
  stayed green (the one new default dep, `sha2`, is pure-Rust; `notify` for `--watch` is behind a
  default-on feature the lean build drops). The load-bearing unknown — encoder byte-reproducibility
  — was resolved by a design-time experiment (byte-identical run-to-run and across thread counts on
  a fixed machine, incl. AVIF/rav1e + lossy-WebP), which is exactly what let the design **split
  "verifiable" into the robust half (cache key = inputs, cross-machine reproducible) and the fragile
  half (observed output hash + env, informational cross-env)** — the single most important design
  call of the wave. The closing STAGE-024 also delivered something the brief didn't originally
  scope: a **run** of the roadmap's pre-1.0 fuzz gate and a threat-model note, so the "reviewed like
  code / trust it in CI on files you didn't write" claim is earned, not aspirational.
- **How many stages did it actually take?** **5** (STAGE-020 build+manifest, 021 cache, 022
  lockfile, 023 `--watch`, 024 hardening & security sweep) across **9 specs** (SPEC-063..071). The
  original brief planned 4 build stages + a determinism-decisions thread; STAGE-024 (hardening &
  security sweep) was **appended mid-wave** after a self-review + code sweep — a deliberate expansion,
  not scope creep, and it turned out to be the highest-leverage stage for the trust claim.
- **What changed between starting and shipping?** Two things. (1) An external design-feedback review
  (2026-07-08) reshaped the "verifiable" leg from "pin output bytes" to "pin the robust cache key,
  *record* the fragile output hash + env" — the honest model given non-bit-identical encoders across
  arch/version. (2) The wave grew a fifth, security-focused stage; and the threat-model pass (SPEC-068)
  reprioritized its own tail, promoting the fuzz gate to #1 and deferring 8 lower-value items past 1.0.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **Drive the real binary with adversarial / hostile-serialized input — green exit-code and
    struct-level unit tests miss whole classes of defect** (user-facing strings, hostile committed
    files, cross-platform path handling, config-dependent behavior). This wave shipped **five**
    green-but-broken defects caught only by driving the CLI: SPEC-065 `{output:?}` Windows escaping,
    SPEC-066 stale `--strict` message + non-hex-digest panic, SPEC-068 out-of-tree write escape +
    symlinked-out-dir, SPEC-069 raw "clean" that OOMed under `-O`. **This belongs in AGENTS verify
    guidance as a first-class rule.**
  - **A "clean / contained / safe" verdict must be EARNED under the exact command + config it claims**
    (a driven attack that failed to break it, or a fuzz run under the shipped `-O` config) — an
    unearned "safe" is a defect, not prose.
  - **`IMAGE_EXTENSIONS` exposes every decode caller** (memory) recurred a 5th time — a new extension
    or `ImageError` variant needs an audit of every decode caller + every `Err(_)` catch-all.
  - **A spec that must prove a compiled-in const is load-bearing should put that const in the
    function signature** (SPEC-064 cache-key schema) so the test can reach it without a private shim.
  - **Ship user-facing docs WITH the behavior** — `--watch` and the 64 Mpix cap both left doc-debt
    that SPEC-071 had to sweep; add a pre-ship doc-drift check to the ship checklist.
  - **Cross-platform defects hide on macOS-green** (the two-tier WatchSet inotify overflow, the
    Windows `{:?}` escaping) — 3-OS CI caught them, local macOS did not; "macOS-green ≠ done."
  - **Batch the small tail; triage the large one.** SPEC-071 (4 fixes + docs in one cycle) is the
    proportionate cadence for a cleared small-fix tail; the decision-bearing tail gets a "does 1.0
    actually need this?" triage and an explicit defer, not a reflexive build-everything close.
- **What did we defer to the next project (post-1.0 maintenance backlog)?**
  - The 8-item STAGE-024 tail: canonicalize-contain-out (accept+documented write-escape residual, own
    spec w/ opt-out), pre-decode format sniff (fails-closed), full-pipeline peak envelope,
    `--max-pixels` cap-raise dial (revisit trigger live), lint decode-seam audit + not-inspected rule,
    unusual-filename `.to_str()` sweep, cache-key build-profile, orphan-output prune. Each pulled
    individually if usage surfaces the need (see the stage backlog + `docs/roadmap.md` post-1.0).
  - `build --gc` / `--cache-dir` / mtime fast-path / `--dry-run` plan preview / cache-hit rate in `-v`
    (STAGE-021 follow-ups); `--check --json`; F-AVIF-3 (upstream `avif-parse` parse-stage over-alloc,
    not boundary-fixable without vendoring).
  - Not in this wave by design: the web-asset manifest (Wave 4), WASM core + demo (Wave 3), geometry
    (Wave 5), remote/distributed cache (never — no-service guardrail).

---

*Lineage: this instantiates the roadmap's provisional "PROJ-007 (determinism)" as the concrete
build/cache/lockfile/`--watch` wave (Wave 2). Other things the drafts once parked under PROJ-007 — a
permissive quantizer (indexed-PNG), SVG favicons — are NOT part of this project: the quantizer lives in
`guidance/license-watchlist.yaml`, SVG shipped in PROJ-009, and favicons are Wave 4.*
