---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-075
  type: story
  cycle: design                    # frame | design | build | verify | ship
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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

*Filled in at the end of the build cycle.*

- **Branch:**
- **PR:**
- **All acceptance criteria met?**
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
