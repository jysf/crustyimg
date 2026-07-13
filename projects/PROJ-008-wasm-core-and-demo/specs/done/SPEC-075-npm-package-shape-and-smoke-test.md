---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-075
  type: story
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

project:
  id: PROJ-008
  stage: STAGE-026
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # separate build session
  created_at: 2026-07-12

references:
  decisions:
    - DEC-064    # the wasm target-cfg boundary + wasm-bindgen surface (what we package)
    - DEC-066    # the size profile lives in `just wasm-build` (must package THAT, not bare cargo)
  constraints:
    - pure-rust-codecs-default
  related_specs:
    - SPEC-072   # the wasm surface + `just wasm-build`/`pkg/`
    - SPEC-074   # the 1.33 MB size-profiled artifact + the +109 KB footgun

value_link: >
  Makes STAGE-025's proven WASM core an installable, identified npm package that runs
  client-side with no native addon — the "sharp without the native addon" artifact, and the
  package STAGE-027's demo consumes.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop; grounded in a design-time probe of
        `just wasm-build`'s pkg/ (2026-07-12): wasm-pack 0.15.0 emits a valid package.json +
        typed .d.ts (info/transform/optimize/version/ImageInfo); the npm names `crustyimg`,
        `crustyimg-wasm`, `@jysf/crustyimg` are ALL unpublished (404 — bare name is free); and
        the `--target web` package needs an explicit `init()`/`initSync` before use.
    - cycle: build
      interface: claude-code
      tokens_total: 130000
      duration_minutes: 40
      estimated_usd: 1.20
      note: >
        ran in the orchestrator main loop, not a metered subagent — tokens_total is an
        order-of-magnitude ESTIMATE (~80/20 in/out at Opus 4.8 list rates, no cache discount),
        not a harness-reported number.
    - cycle: verify
      interface: claude-code
      tokens_total: 95000
      duration_minutes: 30
      estimated_usd: 0.90
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE (verify ran in the main loop, not a metered subagent —
        see the autonomous-run-cost-estimates lesson). Fresh adversarial session in an
        isolated worktree, 2026-07-13. Dominated by two full wasm-pack release builds (the
        profiled artifact + a stock-profile one to measure the guard's failure case), the
        pack/install/run smoke, the finalize-script drift mutations, and the native gate
        sweep (build/lean/test/clippy/deny/validate).
  totals:
    tokens_total: 225000        # build 130k + verify 95k (design null, un-metered main loop)
    estimated_usd: 2.10         # LABELLED ESTIMATE, not a meter read (§4)
    session_count: 3
---

# SPEC-075: npm package shape + identity + install smoke test

## Context

First spec of STAGE-026 (npm library). STAGE-025 shipped the WASM core and `just wasm-build`
already emits a **near-publishable `pkg/`**: a valid `package.json` (name `crustyimg`, v0.4.0,
`main`/`module`/`types`/`files`, MIT-OR-Apache) + a clean typed `crustyimg.d.ts`
(`info`, `transform`, `optimize`, `version`, `ImageInfo`) + the `.wasm` + JS glue + LICENSEs. So
this spec is **decisions + packaging polish + a proof it installs and runs**, not new code.

A **design-time probe (2026-07-12)** grounded the two load-bearing choices:
- **The npm name is free.** `crustyimg`, `crustyimg-wasm`, and `@jysf/crustyimg` are all `404`
  (unpublished) — so the bare name isn't taken; identity is a *clarity* decision (distinguish the
  browser lib from the CLI crate on crates.io?), not a forced scoping.
- **`--target web` requires explicit instantiation.** The package's default export is an async
  `__wbg_init(module_or_path)` (+ `initSync`) — the consumer must `await init(...)` (browser:
  fetch the `.wasm`; Node: feed the bytes) before calling `info`/`transform`. This shapes both the
  smoke test and how STAGE-027's demo loads it, and is the crux of the **target** decision
  (`web` vs `bundler` vs `nodejs` vs shipping more than one).

## Goal

Turn `just wasm-build`'s `pkg/` into a correct, identified, installable npm package, and **prove
by `npm pack` + a fresh-install smoke test that it runs `info`/`transform` client-side with no
native addon** — recording identity / target / versioning / publish policy in a DEC. No live
`npm publish` (that's SPEC-076, gated).

## Inputs

- **Files to read:**
  - `pkg/` (from `just wasm-build`) — `package.json`, `crustyimg.d.ts`, `crustyimg.js` (the
    `init`/`initSync` pattern), `crustyimg_bg.wasm`.
  - `justfile` — `wasm-build`/`wasm-size`; add the packaging recipe here.
  - `src/wasm.rs` — the surface being packaged (don't change it).
  - `docs/repo-tooling-backlog.md` is unrelated; `docs/roadmap.md` Track B "sharp without the
    native addon" is the framing.
- **External:** `wasm-pack` 0.15.0, `npm`/Node (installed). The public npm registry (read-only
  name checks only — no publish).

## Outputs

- **Files created/modified:**
  - `decisions/DEC-067-*.md` — identity (final npm name), wasm-pack **target** (+ rationale re the
    `init` requirement), **versioning policy** (track the crate? the 0.6.0 roadmap line?), and the
    **publish policy** (gated on maintainer approval; SPEC-076).
  - A **packaging recipe** (`just wasm-pack` or extend `wasm-build`) that produces the final `pkg/`
    with the settled name/target, **through the size-profiled build** (DEC-066 — never bare
    `cargo build --target wasm32`, which ships +109 KB).
  - A **smoke test**: a small Node ESM (and/or headless-browser) script + a `just` recipe that
    `npm pack`s, installs the tarball into a temp project, `init`s, and runs `info` + a
    `transform` on a real fixture, asserting the output. Committed under `tests/` or `scripts/`.
  - `package.json` finalized (name/target/`files`/README pointer); an **npm-facing README** if the
    repo README (currently copied verbatim, 13 KB) is wrong for npm consumers.
  - `docs/research/proj-008-wasm-build.md` (or a package note) — the install-and-run recipe.
- **No change to:** `src/wasm.rs` / the engine; the native build; the pure-Rust posture.

## Acceptance Criteria

- [ ] `npm pack` produces a tarball that **installs into a fresh temp project and runs
      `info` + `transform` client-side** (Node ESM smoke via `initSync`/`init` with the `.wasm`
      bytes; optionally a headless-browser import) — output asserted (valid bytes / correct dims),
      **no native addon**, no backend, no network.
- [ ] Package **identity, target, and versioning are settled and DEC-067-recorded**: the final npm
      name, the wasm-pack target(s) (with the `init`-requirement rationale), and how the npm version
      relates to the crate/roadmap.
- [ ] The packaged `.wasm` is the **size-profiled `just wasm-build`** artifact (~1.33 MB brotli),
      verified (e.g. `just wasm-size` on the packaged file) — not a bare `cargo build`.
- [ ] **No live publish** — `npm publish` is not run; the tarball + a documented (gated) publish
      path is the deliverable (SPEC-076 owns the actual publish, on approval).
- [ ] Native build / released binary unchanged; `just deny` unchanged; `just validate` green.

## Failing Tests

Written now (design). The package "test" is an install-and-run smoke, driven by a `just` recipe:

- **`tests/npm_smoke.mjs`** (or `scripts/npm-smoke.sh` driving it), run via **`just wasm-npm-smoke`**:
  - `"packs, installs, and runs client-side"` — `npm pack` → install the tarball in a fresh temp
    dir → `import` the package, `await init()`/`initSync(bytes)`, call `info(png)` → assert
    width/height/format; call `transform(png, recipe, "png")` → assert the output decodes to the
    resized dims. Fails if the package can't install, can't instantiate, or the API is wrong.
  - `"no native addon / no postinstall build"` — assert the installed package has no `.node`
    binary and no build step (pure JS + `.wasm`).
- **Guard:** the smoke asserts the packaged `.wasm` size is the profiled ~1.33 MB (catches a
  regression to a bare `cargo build` that would ship +109 KB).

## Implementation Context

### Decisions that apply
- `DEC-064` — the wasm-bindgen surface (`info`/`transform`/`optimize`/`version`/`ImageInfo`) is what
  we package; don't redesign it here.
- `DEC-066` — the 1.33 MB size profile lives in `just wasm-build`'s env vars, NOT `[profile.release]`;
  the packaging recipe MUST go through it or silently ship +109 KB (the STAGE-025/026 footgun).

### Constraints that apply
- `pure-rust-codecs-default` — the package is pure-Rust→wasm + JS glue; no native addon, no C, no
  postinstall compile (that's the whole "not sharp" pitch).

### Prior related work
- `SPEC-072`/`SPEC-074` — the wasm surface, `just wasm-build`, and the sized artifact this packages.

### Out of scope (for this spec)
- **Live `npm publish`** and a release/CI workflow — **SPEC-076**, and gated on explicit maintainer
  approval (publishing is outward-facing/irreversible; npm unpublish is restricted).
- The demo page (STAGE-027); engine/API changes; a broad multi-bundler test matrix.

## Notes for the Implementer

- **The name is free** (probe: all candidates 404) — so DEC-067 is choosing the *clearest* name, not
  escaping a collision. Weigh bare `crustyimg` (available, matches the crate) vs `crustyimg-wasm` /
  `@jysf/crustyimg` (disambiguates the browser lib from the CLI). Recommend, decide, record.
- **Target ↔ smoke-test coupling:** `--target web` needs `await init(url)` in the browser and
  `initSync(fs.readFileSync(wasmPath))` in Node — so the Node smoke test must read the `.wasm` bytes
  and init explicitly. `--target bundler` auto-instantiates but needs a bundler to run; `--target
  nodejs` is CommonJS + auto-load (easy Node smoke, not browser). The demo (STAGE-027) is
  browser/client-side, so `web` (or `bundler`) is the likely ship; you may build a `nodejs` variant
  just for the smoke, or feed bytes to the `web` build. State the choice in DEC-067.
- **Do NOT publish.** Keep this spec to `npm pack` + install-and-run; the live publish is SPEC-076
  on approval.
- Next DEC id is **DEC-067**. Commit with `-s` (DCO is real). Drive the real install-and-run — a
  package.json that looks right but doesn't `init` in a fresh project is the failure mode here.

---

## Build Completion

- **Branch:** `feat/spec-075-npm-package`
- **PR:** #84

- **All acceptance criteria met?** Yes — all five, each driven rather than inspected.
  - **Packs, installs, runs client-side.** `just wasm-npm-smoke` is green: `npm pack` → install
    THAT tarball into a fresh temp project → a separate Node process imports the **bare specifier**
    `crustyimg-wasm`, `initSync({ module: bytes })` (bytes resolved through the package's own
    `./crustyimg_bg.wasm` subpath export — which is what proves the export map is real), then
    `info(png)` → 64×48 png / hasAlpha false, and `transform(png, resize-recipe, fmt)` for **png,
    jpeg, webp, avif**. The PNG output's dimensions are asserted **twice**: its IHDR is parsed in
    plain JS (a decoder the crate did not write) *and* the bytes are fed back through `info()`; both
    say 32×24. AVIF is checked at the container level only (`ftyp`/`avif`, 329 B) — this build
    encodes AVIF but cannot decode it (DEC-065), and saying so beats faking a round-trip.
  - **Identity / target / versioning settled → DEC-067.** `crustyimg-wasm`, `--target web` (one
    artifact), version in lockstep with the crate.
  - **The packaged `.wasm` is the size-profiled one.** 1,394,313 B brotli, and the smoke test's
    1.30–1.45 MB band **was mutation-tested against a real bare build**, not assumed: a stock-profile
    `wasm-pack build` (no DEC-066 env vars) lands at **1,503,817 B** — 54 KB above the ceiling, and
    exactly the +109 KB the spec warned about. The guard trips.
  - **No live publish.** Nothing in the tooling can reach `npm publish`; the finalize script also
    *rejects* any `package.json` carrying a lifecycle script, so the published artifact can't run
    anything on a consumer's machine either. Verified `npm view crustyimg-wasm` is still 404.
  - **Native unaffected.** `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
    `cargo build --no-default-features`, `cargo test` (716 passed), `just deny`, `just validate` —
    all green. No change to `src/` at all.

- **New decisions emitted:** **DEC-067** — npm identity (`crustyimg-wasm`), target (`web`, single
  artifact), versioning (lockstep with the crate, enforced by the finalize script), publish policy
  (gated; the tooling stops at `npm pack`).

- **Deviations from spec:** None material. Two shape choices the spec left open:
  - Recipes are **`just wasm-npm-pkg`** (finalize) + **`just wasm-npm-smoke`** (pack/install/run)
    rather than one `just wasm-pack` — the finalize step is worth being able to run and inspect
    without a five-minute pack cycle. `wasm-npm-pkg` **depends on `wasm-build`**, so the size profile
    (DEC-066) is not bypassable by accident.
  - The npm identity is committed as data (`npm/package.overrides.json` + `npm/README.md`) merged in
    by `scripts/wasm-npm-finalize.mjs`, rather than a hand-maintained `pkg/package.json` — `pkg/` is
    git-ignored and regenerated from scratch by every build, so an edit there could not survive and
    would be a lying source of truth.

- **Follow-up work identified:**
  - **SPEC-076** (already framed): the live publish, by hand, on maintainer approval.
  - **CI**: nothing runs `just wasm-npm-smoke` automatically. It needs Node + wasm-pack + the
    wasm32 target, and it is the only thing standing between "the package works" and "the package
    worked once, on my Mac". Folds naturally into the wasm CI job STAGE-025 already owes
    (carried from SPEC-074: a wasm CI job MUST build through `just wasm-build`).
  - **The `crustyimg` npm name is deliberately left unclaimed** for a future npx-distributed CLI
    (DEC-067). Worth a line in the roadmap so it isn't re-litigated.

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** — Nothing, genuinely; the design-time
   probe carried its weight. Knowing up front that all three names were free, and that `--target web`
   has no auto-instantiation, meant the two load-bearing decisions were *choices* on arrival rather
   than discoveries — and the spec's "you may build a nodejs variant just for the smoke, or feed
   bytes to the web build" hint pointed straight at the answer that avoids a second artifact.
2. **Was there a constraint or decision that should have been listed but wasn't?** — Not listed, but
   worth naming: the spec (and DEC-066) framed the size profile as a *build* concern, and it is
   really a *packaging* concern — the profile is only load-bearing at the moment the artifact leaves
   the repo. Making `wasm-npm-pkg` depend on `wasm-build` is what turns "remember to use the right
   recipe" into "there is no other recipe", and that dependency edge is the single most important
   line in this change.
3. **If you did this task again, what would you do differently?** — Build the bare-`cargo build`
   comparison **first**, not last. I picked the smoke test's size ceiling (1.45 MB) by arithmetic
   from the spec's "+109 KB" and only afterwards built the stock-profile artifact to check the guard
   actually trips it. It did (1,503,817 B, 54 KB clear) — but that ordering is a guard-band chosen by
   assumption and confirmed by luck, and SPEC-074's whole lesson was that the levers you *assume* are
   the ones that bite. Measure the failure case, then set the threshold.

---

## Reflection (Ship)

*Appended during ship (2026-07-13). Shipped via PR #84 (squash `125a590`, DEC-067). The build ran
in the SHARED checkout (collided with the orchestrator's HEAD); verify correctly used an isolated
worktree — that split is the process note below.*

1. **What would I do differently next time?** — The size-guard arc is the whole lesson. Build set a
   size *band* (a 1.30–1.45 MB ceiling), honestly self-flagged it as "chosen by assumption,
   validated by luck" (the SPEC-074 trap). Verify then proved the flag right — the band was
   load-bearing on ~4% of daylight either side of two *moving* endpoints — and **replaced it with a
   structural assertion**: `strip = true` (a DEC-066 lever) leaves an observable fingerprint (the
   wasm `name` debug section: **42 B profiled vs 980,292 B stock** — categorical, 4 orders of
   magnitude, immune to legitimate code growth), with the size number demoted to a `±5%` regression
   baseline. **The reusable move: to assert "was this built through the right recipe?", find the
   lever's fingerprint in the artifact and assert THAT — don't infer intent from a size threshold
   between two targets that both drift.** Also: run the build in a worktree from the start (it shared
   the checkout and moved the orchestrator's HEAD; verify's worktree is the pattern).
2. **Does any template, constraint, or decision need updating?** — DEC-067 records identity
   (`crustyimg-wasm` — the suffix reserves the bare `crustyimg` for a future `npx` CLI, the
   esbuild/esbuild-wasm precedent), target (`--target web`, one artifact), lockstep versioning
   (enforced — verify broke it 3 ways, all exit 1), and the gated publish. Banked as its own memory:
   **assert the build profile structurally, not by a size band.** The cost-per-session `estimated_usd`
   lesson from SPEC-073/074 landed — build and verify both recorded it, so the table shows $2.10.
3. **Is there a follow-up spec I should write now before I forget?** — (a) **SPEC-076** (the live
   `npm publish`, by hand, on maintainer approval) — the remaining STAGE-026 spec; (b) **the wasm CI
   job** is now doubly-owed — it must build through `just wasm-build` (SPEC-074) *and* run
   `just wasm-npm-smoke` (this spec: the package is proven on one Mac until CI runs it); (c) a
   roadmap line that the bare `crustyimg` npm name is deliberately unclaimed (so it isn't
   re-litigated). Trivial: two "brotli" numbers circulate for the same artifact (recipe `brotli -q 11`
   = 1,394,313 B vs Node `brotliCompressSync` = 1,394,631 B, a 318 B tooling diff) — harmless, noted.
