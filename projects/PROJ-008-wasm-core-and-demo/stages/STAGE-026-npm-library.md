---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-026
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
shipped_at: null

value_contribution:
  advances: >
    Turns the proven, sized WASM core (STAGE-025) into an installable npm library — image
    optimization in a JS toolchain with NO native addon, the sharp answer to `sharp`'s ABI/CI
    friction and the abandoned `@squoosh/cli`. The distinct adoption artifact the WASM wave
    promises alongside the demo page.
  delivers:
    - "An installable, typed npm package over the WASM build: `info` / `transform` / `optimize` run client-side, no native addon, no backend"
    - "A settled package identity (name/scope), wasm-pack target(s), versioning policy, and README — decided in a DEC"
    - "A publish-READY package proven by `npm pack` + a fresh-install smoke test (actual `npm publish` gated on explicit maintainer approval — it's outward-facing)"
  explicitly_does_not:
    - "Build the demo page (STAGE-027) — this is the library it consumes"
    - "Publish to npm without explicit maintainer approval (publishing a package is irreversible/outward-facing)"
    - "Add engine features or change the WASM surface (STAGE-025 shipped it); this is packaging + identity"
    - "Touch the native build / released binary"
---

# STAGE-026: npm-packaged library

## What This Stage Is

Package the shipped WASM core (STAGE-025) into an **installable, typed npm library** that runs
the engine **client-side with no native addon**. STAGE-025 already produces a near-publishable
artifact — `just wasm-build` emits `pkg/` with a `package.json` (name/version/types/files/license),
a clean typed `crustyimg.d.ts` (`ImageInfo`, `info`, `transform`, `optimize`), the `.wasm`, and
JS glue. So this stage is the **decisions + packaging polish** that make that `pkg/` a real,
installable, correctly-identified npm package — not new engine work. Its output is a package a JS
developer can `npm install` and call in the browser (and the artifact STAGE-027's demo consumes).

## Why Now

- **STAGE-025 de-risked and sized the core** (it compiles, AVIF encodes, 1.33 MB brotli) — the npm
  library is now packaging over a proven, measured artifact, not a hope.
- **wasm-pack already emits ~90% of the package** (`--target web`, a valid `package.json` + `.d.ts`)
  — the remaining work is identity (the crate name `crustyimg` may need scoping/distinguishing on
  npm), target choice (`web` vs `bundler` vs multiple), versioning, an npm-appropriate README, and
  proving install-and-run — plus deciding the publish gate.
- **It's a distinct adoption artifact** — the roadmap's "sharp without the native addon" pitch — and
  **STAGE-027's demo wants a package to consume**, so it comes first.

## Success Criteria

- `npm pack` produces a tarball that **installs into a fresh project and runs a real
  `transform`/`info` client-side** (Node ESM smoke test and/or a headless browser import), with **no
  native addon** and no backend.
- Package **identity is settled**: name (scoped or distinguished from the `crustyimg` *crate* if the
  bare name collides/confuses), the wasm-pack **target(s)** (`web` and/or `bundler`), a **versioning
  policy** (track the crate's semver? the roadmap's 0.6.0 line?), and an npm-facing README — all
  recorded in a DEC.
- The packaged `.wasm` is the **size-profiled `just wasm-build`** artifact (1.33 MB brotli), not a
  bare `cargo build --target wasm32` (which silently ships +109 KB — the STAGE-025 footgun); the
  package/CI path enforces that.
- The package is **publish-ready** — but actual `npm publish` is **gated on explicit maintainer
  approval** (outward-facing/irreversible); the stage ships the dry-run + tarball + the publish
  recipe, not necessarily a live publish.
- Native build / released binary unchanged; `just deny` unchanged; pure-Rust posture intact.

## Scope

### In scope
- Finalize `package.json` (name/scope, `main`/`module`/`types`, `files`, target(s)); an npm README;
  a versioning policy; a JS/Node (+ optional headless-browser) install-and-run smoke test; an
  `npm pack` dry-run; a `just` recipe for the package build (through `just wasm-build`); a DEC for
  identity + target + versioning + publish policy.

### Explicitly out of scope
- The demo page (STAGE-027); a live `npm publish` without approval; engine/API changes; a broad
  multi-bundler test matrix beyond what a real consumer needs; the native build.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-075 (shipped 2026-07-13, PR #84 `125a590`, DEC-067) — **npm package shape + identity +
  install smoke test.** `crustyimg-wasm`, `--target web` (one artifact), lockstep versioning
  (enforced), publish gated (tooling stops at `npm pack`). `just wasm-npm-pkg` (depends on
  `wasm-build` so the size profile can't be bypassed) + `just wasm-npm-smoke` (pack → fresh-install →
  bare-specifier import in a separate Node process → `initSync` → `info`/`transform` to
  png/jpeg/webp/avif, dims decoded back). 8-file tarball, no native addon / lifecycle script / deps.
  Verify hardened the size guard STRUCTURALLY (strip fingerprint 42 B vs 980,292 B, not a size band).
  Native untouched. Cost $2.10.
- [x] SPEC-076 (shipped 2026-07-20, PR #107 `0d3f936`, no DEC) — **`crustyimg-wasm@0.5.0` prepped for
  npm publish (gated).** Identity was already lockstep (SPEC-075's finalize + the 0.5.0 crate → raw
  wasm-pack now emits v0.5.0, not v0.4.0); the one real fix was a canonical `repository.url` override so
  the dry-run is warning-clean. npm README brought current (`optimizeDetailed`/`score` + Caveats).
  `wasm-npm-smoke` + `npm publish --dry-run` green (8 files / 2.0 MB, zero deps). **The actual `npm publish`
  remains [MAINTAINER-AUTHORIZED] + permanent — merge ships readiness, not the package.** Two gated
  follow-throughs owed once publish is live: demo npm link (`demo/index.html:168`) + the README
  "isn't on npm yet" flip.

**Count:** 2 shipped / 0 active. SPEC-075 SHIPPED 2026-07-13 (package installs + runs client-side,
DEC-067). SPEC-076 SHIPPED 2026-07-20 (identity + npm README + dry-run gate). **Stage content-complete;
the gated `npm publish` is the maintainer's to fire — hold the stage active until the package is live on
npm, then close deliberately (status→shipped + shipped_at + reflection).**

## Design Notes

- **wasm-pack output is the starting point** (`pkg/`, `--target web`): `package.json` already has
  name `crustyimg` / version 0.4.0 / `main`+`types`+`files` / MIT-OR-Apache / keywords. Decide
  whether to keep `--target web`, add `bundler`, or ship both; and whether the npm name should be
  scoped (`@jysf/crustyimg`) or suffixed (`crustyimg-wasm`) to avoid confusion with the CLI crate.
- **Publish is outward-facing** — `npm publish` is effectively irreversible (unpublish is
  restricted) and public. Frame the stage to stop at publish-ready + a dry-run unless the
  maintainer explicitly approves a live publish; never publish on inference.
- **The size footgun is load-bearing here:** the 1.33 MB profile lives in `just wasm-build`'s env
  vars, not `[profile.release]`. The package build MUST go through `just wasm-build`, or it ships
  +109 KB heavier. This is also the STAGE-025 "wasm CI job" carry — a publish/CI path is where it
  gets enforced.
- **API surface is already typed and shipped** (STAGE-025's `src/wasm.rs` + the generated `.d.ts`);
  this stage should not redesign it, only wrap/document for JS ergonomics if needed.

## Dependencies

### Depends on
- STAGE-025 (shipped) — the WASM build + `just wasm-build`/`wasm-size` + the sized `pkg/` output.
- External: `wasm-pack` 0.15.0 (installed); an npm registry account for a live publish (maintainer's).

### Enables
- STAGE-027 (demo page) — consumes this package (or the local `pkg/`).
- The "sharp without the native addon" adoption pitch (Track B).

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
