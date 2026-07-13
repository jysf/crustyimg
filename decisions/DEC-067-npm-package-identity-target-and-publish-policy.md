---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-067
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
supersedes: null
superseded_by: null

affected_scope:
  - "npm/**"
  - "scripts/wasm-npm-finalize.mjs"
  - "tests/npm_smoke.mjs"
  - "justfile"

tags:
  - wasm
  - npm
  - packaging
  - release
  - naming
---

# DEC-067: The npm package is `crustyimg-wasm`, `--target web`, versioned in lockstep with the crate, and published only by hand

## Decision

Publish the WASM library to npm as **`crustyimg-wasm`** — a single `wasm-pack --target web`
artifact, whose **npm version is the crate version verbatim**, built by `just wasm-npm-pkg`
(which routes through the size-profiled `just wasm-build`, DEC-066) and proven by
`just wasm-npm-smoke`. **`npm publish` is never run by a recipe, a script, or an agent**: it is a
deliberate, maintainer-approved act (SPEC-076).

## Context

STAGE-025 left `wasm-pack` emitting a nearly-publishable `pkg/`. Four things it *cannot* decide for
us, and each was open going into SPEC-075:

1. **The name.** wasm-pack copies it from `Cargo.toml`, so `pkg/` claims the name **`crustyimg`** —
   the CLI's name — and inherits the CLI's description ("A fast Rust CLI to view and transform
   images"), which is the wrong artifact and the wrong audience for someone typing `npm install`.
2. **The target.** `web` / `bundler` / `nodejs` produce incompatible JS glue and different
   instantiation contracts, and the choice reaches into STAGE-027's demo and into the smoke test.
3. **Versioning.** Does npm get its own line, or the crate's?
4. **Publishing.** Who runs it, and when?

A design-time probe (2026-07-12, re-confirmed at build) established the ground truth that makes
this a *clarity* decision rather than a forced one: **`crustyimg`, `crustyimg-wasm`, and
`@jysf/crustyimg` are all unpublished (404)**. No collision is pushing us anywhere. And the
`--target web` package has **no auto-instantiation**: its default export is an async
`__wbg_init(module_or_path)` (plus `initSync`), so a consumer *must* init before calling `info` or
`transform`.

## Alternatives Considered

### The name

- **Option A: bare `crustyimg`.**
  - What it is: the free, obvious name; matches the crate and the CLI.
  - Why rejected: it spends the one name a **future npx-distributed CLI** would want. `npm install
    crustyimg` reads as "give me the crustyimg tool", and npm is a completely normal way to ship a
    prebuilt CLI binary (esbuild, biome, swc all do it). If the bare name resolves to a *browser
    library that exports `info()`*, that user gets something they cannot run and we have no name
    left for the thing they wanted. The confusion is silent and permanent — npm names are, in
    practice, unrecoverable.

- **Option B: `@jysf/crustyimg` (scoped).**
  - What it is: scope-disambiguated; guarantees no collision forever.
  - Why rejected: it disambiguates the *owner*, which is not the axis anyone is confused about, and
    it says nothing about *which* crustyimg artifact it is — a scope would leave the same
    CLI-vs-library ambiguity inside it. It also adds friction (`--access public` on every publish)
    to buy nothing we need, since the names are free.

- **Option C (chosen): `crustyimg-wasm`.**
  - What it is: the suffix names the artifact — the crustyimg engine, compiled to WebAssembly.
  - Why selected: it is *self-describing at the install line*. It also matches the strongest
    precedent for exactly our shape: **`esbuild` (the tool) vs `esbuild-wasm` (the same engine as
    portable wasm)**; likewise `@resvg/resvg-wasm`. And it **leaves `crustyimg` free** for a future
    npm-distributed CLI, keeping `npm i -g crustyimg` → the tool and `npm i crustyimg-wasm` → the
    library, an unambiguous split we get for the price of five characters.

### The target

- **Option A: `bundler`.**
  - What it is: wasm-pack's default; auto-instantiates via a bare `import ... from
    './crustyimg_bg.wasm'`.
  - Why rejected: **it cannot run without a bundler.** STAGE-027's demo is a static, client-side
    page ("watch it just work"); a `bundler` package would force a build toolchain into a demo whose
    entire point is that there isn't one — and into every consumer who just wants a `<script
    type="module">`.

- **Option B: `nodejs`.**
  - What it is: CommonJS, auto-loads the `.wasm` off disk. Trivial to smoke-test.
  - Why rejected: it doesn't run in a browser, which is the whole wave. Optimizing the packaging
    decision for the convenience of the *test* would be exactly backwards.

- **Option C: ship two or three targets** (e.g. `web` + `nodejs`), in one package or several.
  - What it is: the maximal-compatibility answer.
  - Why rejected: each target carries **its own full copy of the 5.7 MB `.wasm`** — this is
    DEC-065's "one artifact" reasoning again (a second engine is a second engine, whether it arrives
    as a lazy chunk or a second entry point). It also doubles the surface every future change has to
    keep honest, and it buys nothing: `web` **already runs in Node** (proven — `tests/npm_smoke.mjs`
    drives it through a real `npm install`), because `initSync({ module: bytes })` takes bytes from
    anywhere.

- **Option D (chosen): `web` only, one artifact.**
  - What it is: an ES module the consumer instantiates explicitly.
  - Why selected: it is the only target that runs **unbundled in a browser** (the demo), works
    **through every bundler** (Vite/webpack/Rollup all hand you the `.wasm` URL to pass to `init`),
    **and** works in **Node** via `initSync` with the bytes. One `.wasm`, three homes.
  - The cost, accepted and documented: **explicit `init()` is mandatory** — there is no
    call-it-and-it-works path. We consider that honest rather than merely tolerable: a 1.33 MB
    download should be a thing the page decides to do, at a moment of its choosing, not a hidden
    side effect of an import.

### Versioning

- **Option A: an independent npm version line** (so packaging-only fixes can ship without a crate
  release).
  - Why rejected: two version lines for one engine means `crustyimg-wasm@0.5.1` and `crustyimg
    0.5.1` are no longer the same code, and nobody can tell from the outside. The freedom it buys
    (shipping a README typo without a crate patch) is not worth that.

- **Option B (chosen): lockstep — the npm version IS the crate version.**
  - Why selected: `crustyimg-wasm@x.y.z` is, by definition, crate `x.y.z` compiled to wasm.
    wasm-pack already copies `version` from `Cargo.toml`, so this is the *default* behaviour and
    there is nothing to maintain; `scripts/wasm-npm-finalize.mjs` refuses to override `version` and
    **fails the build if `pkg/`'s version has drifted from `Cargo.toml`**, so lockstep is enforced
    rather than hoped for. Package-only fixes ride the next crate patch (they are cheap). The
    roadmap's 0.6.0 line therefore ships as `crustyimg-wasm@0.6.0`. Pre-1.0, a minor bump may break
    the API — the standard cargo/npm reading of 0.x.

### Publishing

- **Option A: a `just wasm-npm-publish` recipe / a CI release job.**
  - Why rejected (for now): `npm publish` is **outward-facing and effectively irreversible** —
    npm's unpublish window is narrow (72h, and blocked outright once anything depends on you), so a
    wrong name, a stale `.wasm`, or an accidental `0.4.0` is not a mistake we can take back. A
    one-keystroke path to it is a hazard, and an agent that can reach it is a bigger one.

- **Option B (chosen): the tooling stops at `npm pack`.**
  - What it is: `just wasm-npm-pkg` finalizes `pkg/`; `just wasm-npm-smoke` packs the tarball,
    installs it into a throwaway project and runs it. Neither ever invokes `npm publish`, and the
    finalize script **rejects a `package.json` carrying any lifecycle script**, so nothing in the
    published artifact can publish or compile anything either.
  - The live publish is **SPEC-076**, on explicit maintainer approval, run by a human.

## Consequences

- **Positive:** `npm install crustyimg-wasm` gets a package that installs on any OS with **no
  native addon, no postinstall compile, and zero transitive dependencies** — proven by a real
  pack → fresh-install → run, not by inspection. The bare `crustyimg` name stays available for a
  CLI. Version drift between the crate and the package is structurally impossible.
- **Negative:** consumers must call `init()`/`initSync()` before anything else — the one piece of
  ceremony, and the first thing the npm README explains. The `-wasm` suffix is four characters of
  extra typing forever. We are also *not* squatting `crustyimg` on npm, so in principle someone else
  could take it (a low risk for a name this specific, and squatting to prevent it is impolite —
  SPEC-076 can revisit if it ever matters).
- **Neutral:** `pkg/` stays git-ignored and fully regenerated on every build; the npm identity lives
  in `npm/package.overrides.json` + `npm/README.md` (committed) and is merged in after wasm-pack, so
  a hand-edit inside `pkg/` cannot survive and cannot be mistaken for the source of truth.

## Validation

The decision is right if:
- `just wasm-npm-smoke` stays green — it *is* the validation: pack → fresh `npm install` → `init` →
  `info`/`transform` → decode the output back, plus the no-native-addon and profiled-`.wasm`-size
  assertions.
- STAGE-027's demo consumes the package **as published** (a plain `<script type="module">` +
  `await init(url)`), with no bundler and no special-casing. If the demo needs to reach around the
  package, the target was wrong.

Revisit if: a real consumer needs `bundler`-style auto-instantiation badly enough to justify a
second artifact; or a native-CLI-over-npm distribution actually materialises and wants the bare name
(the split is already reserved for it); or `wasm-pack` grows first-class multi-target output that
shares one `.wasm`.

## References

- Related specs: SPEC-075 (this), SPEC-076 (the gated live publish), SPEC-072/SPEC-074 (the wasm
  surface and the sized artifact being packaged)
- Related decisions: DEC-064 (the wasm-bindgen surface we package), DEC-065 (AVIF encode-only on
  wasm, and the one-artifact argument), DEC-066 (the size profile lives in `just wasm-build` — the
  packaging recipe MUST route through it)
- External: [`esbuild` vs `esbuild-wasm`](https://www.npmjs.com/package/esbuild-wasm) — the naming
  precedent; [wasm-pack target docs](https://rustwasm.github.io/docs/wasm-pack/commands/build.html#target)
