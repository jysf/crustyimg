---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-076
  type: chore
  cycle: ship
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-008
  stage: STAGE-026
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-20

references:
  decisions: [DEC-067]
  constraints: []
  related_specs: [SPEC-075, SPEC-082, SPEC-100]

value_link: >
  Doubles the launch story: crustyimg as a zero-dependency JS/TS library (crustyimg-wasm) alongside the
  CLI — no native addon, runs in the browser/Node. The outward-facing npm publish, gated.

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
      tokens_total: 40000
      estimated_usd: 0.40
      note: >
        ran in a build session main loop, not a metered subagent — tokens_total is an
        order-of-magnitude ESTIMATE (autonomous-run-cost-estimates convention), not a
        harness-reported number. Two wasm-pack release builds, one smoke run, one
        publish dry-run, one full native gate (just check), and two small file edits
        (npm/README.md, npm/package.overrides.json).
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 80000
      estimated_usd: 2.00
      note: >
        Estimated order-of-magnitude (main-loop verify on Opus 4.8, 1M ctx). Dominated by
        two full `just check` runs (crate compile + test suite); wasm rebuilds cargo-cached.
        Adversarial: identity from a fresh build, both required negative controls fired
        (override removal → normalize warning returns; smoke version-assert mutation → RED),
        README caveats graded live against src/wasm.rs, nothing published.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.30
      recorded_at: 2026-07-20
      note: >
        orchestrator main loop — PR #107, CI CLEAN (full 3-OS matrix), squash-merge
        (0d3f936), bookkeeping. No DEC (identity already lockstep from SPEC-075's finalize
        + the 0.5.0 crate; the one real fix — canonical repository.url — is too small for a
        decision record). Publish stays maintainer-fired + permanent; ships the npm README +
        override to main. crustyimg-wasm not yet on npm.
  totals:
    tokens_total: 120000
    estimated_usd: 2.70
    session_count: 3
---

# SPEC-076: publish crustyimg-wasm to npm (gated)

## Context

The wasm library was built and gated at SPEC-075 (DEC-067): the pure-Rust engine compiles to wasm and
`just wasm-npm-pkg`/`just wasm-npm-smoke` produce a near-publishable `crustyimg-wasm` package (client-side,
no native addon, no lifecycle script, zero deps), but **`npm publish` was deliberately deferred** to this
spec. For the r/rust-first launch the maintainer decided (2026-07-20) to publish it: it makes crustyimg a
**dual-surface** tool — a CLI *and* a zero-dependency JS/TS library (`info`/`transform(recipe)`/`optimize`/
`optimizeDetailed`+score/`score`) that runs in the browser, Node, Deno, and bundlers — the direct answer
to sharp's native/libvips addon and the abandoned `@squoosh/cli`. `crustyimg-wasm` is **free on npm**
(404, verified 2026-07-20).

**Publishing to npm is outward-facing and effectively permanent** (npm unpublish is heavily restricted).
The actual `npm publish` is **[MAINTAINER-AUTHORIZED]** — this spec prepares everything and stops at the
gate; the maintainer fires the publish (as with the crate tag).

## Goal

Publish `crustyimg-wasm` to npm as a good JS/TS citizen: correct package identity + a usage README for the
npm page, lockstep version (0.5.0), a green pack→install→run smoke, and the gated publish — plus update
the crustyimg README's wasm section to the real `npm install` once it's live. **No `src/`/engine change.**

## Inputs

- **Files to read:** `specs/done/SPEC-075-*` (the package shape + `just wasm-npm-pkg`/`wasm-npm-smoke`
  recipes, DEC-067); `pkg/package.json` (currently emitted as name `crustyimg` v0.4.0 — the raw wasm-pack
  output; **confirm the packaging recipe sets the intended identity `crustyimg-wasm` and the version**);
  `src/wasm.rs` (the JS API surface to document: `init`, `info`, `transform(input, recipe_toml, out_format)`,
  `optimize`, `optimizeDetailed`, `score`, `version`); `README.md` (the wasm section to update on publish);
  `RELEASING.md` (for the lockstep-version + gate discipline); the live demo (a real usage reference).

## Outputs

- **The published package** (gated): `crustyimg-wasm@0.5.0` on npm — but **only the maintainer runs
  `npm publish`**; the build stops at a verified `npm pack` + `npm publish --dry-run`.
- **Package identity nailed** — name `crustyimg-wasm`, version **lockstep with the crate (0.5.0)**, correct
  `main`/`module`/`types`/`files`, `repository`/`homepage`/`license`, keywords, and an npm `README.md` in
  the package (the npm page): what it is (client-side, no native addon), an honest capability + caveats
  note (`--target web` needs `await init()`, single-threaded, AVIF encodes / decode via the browser), and
  a **minimal working JS/TS snippet** (`init()` → `optimize`/`transform` → bytes + score).
- **`README.md` (crustyimg) wasm section** — updated to the real `npm install crustyimg-wasm` + the snippet
  (replacing the honest "isn't on npm yet" note) — **applied only when the publish is authorized/done**, so
  the repo never claims an install that isn't live.
- **Files modified:** the packaging recipe/`package.json` template if the identity/version needs fixing;
  the crustyimg README wasm section (on publish).

## Acceptance Criteria

- [ ] `just wasm-npm-pkg` produces a tarball named **`crustyimg-wasm`** at **0.5.0** (lockstep with the
      crate), with a package `README.md`, correct `main`/`types`/`files`/`repository`/`license`, and the
      size-profiled `.wasm` (the DEC-066 `just wasm-build` artifact, not a bare build).
- [ ] `just wasm-npm-smoke` is green: `npm pack` → fresh-install in a separate dir → a bare-specifier
      import runs `init()` + `optimize`/`transform` client-side with **no native addon and no lifecycle
      script**.
- [ ] `npm publish --dry-run` is clean (the shape a real publish uses).
- [ ] The package README is honest — the caveats (`init()`, single-threaded, AVIF decode via browser) are
      stated; no capability is overclaimed.
- [ ] **The actual `npm publish` is NOT run by the build** — it stops at the dry-run and hands the exact
      command to the maintainer. (When authorized + published, the crustyimg README wasm section is updated
      to the real `npm install` in the same or a follow-up change.)
- [ ] `just validate` green; no `src/`/engine/behavior change.

## Failing Tests

- `just wasm-npm-smoke` green (pack → fresh-install → `init()`+`optimize` in a separate Node process),
  asserting the resolved package name is `crustyimg-wasm@0.5.0` and no native addon / lifecycle script.
- `npm publish --dry-run` exits 0 with the expected file list.
- A structural check that the packaged `.wasm` is the size-profiled build (the SPEC-075 strip-fingerprint
  guard: the `name` section is stripped) — the published bytes must be the tuned artifact.

## Implementation Context

### Decisions that apply
- `DEC-067` — the `crustyimg-wasm` identity, `--target web` + explicit `init()`, lockstep version, and the
  publish-gated posture. This spec is the authorized publish DEC-067 deferred.

### Prior related work
- `SPEC-075` (shipped) — built the package + the `just wasm-npm-pkg`/`wasm-npm-smoke` tooling (publish
  deferred here). `SPEC-082`/`SPEC-100` — the README this updates on publish.

### Out of scope
- Any `src/`/engine/wasm-surface change (the bindings are frozen — SPEC-079); new JS API.
- Cutting the crate release (SPEC-099/the 0.5.0 crate cut is separate); the wasm package version just
  tracks it.
- A full JS SDK / TypeScript wrapper beyond the generated `.d.ts` + a usage snippet.

## Notes for the Implementer
- **The publish is the maintainer's to fire** — prepare, dry-run, and STOP. Print the exact `npm publish`
  command (and the npm 2FA/OTP note if applicable) for the maintainer. Do not run `npm publish`.
- **Lockstep version:** the package must be **0.5.0** to match the crate; if the crate 0.5.0 cut hasn't
  happened yet, coordinate (this can publish at 0.5.0 alongside/after the crate).
- **Honesty on the npm page:** state the caveats plainly (`init()`, single-threaded, AVIF decode via the
  browser) — the same candor as the demo/README. Don't sell it as a server-side sharp replacement.
- **Only claim `npm install` in the repo README once it's actually published** — the current honest "isn't
  on npm yet" wording must not be replaced with a live-install claim before the publish is done.
- Confirm the `pkg/package.json` identity mismatch (raw `crustyimg` vs intended `crustyimg-wasm`) is
  resolved by the packaging recipe, not left to chance.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-076-npm-publish`
- **PR (if applicable):** none — build stops before PR, per spec (prepare + dry-run only)
- **All acceptance criteria met?** Yes, five of six directly; the sixth (the live `npm publish`) is
  correctly NOT done — it is gated to the maintainer.
  - **`just wasm-npm-pkg` → `crustyimg-wasm@0.5.0`.** Ran clean. The package identity the spec
    flagged as a risk (`pkg/package.json` raw-emitting `crustyimg` v0.4.0) was **already fixed**:
    SPEC-075's finalize script (`scripts/wasm-npm-finalize.mjs` + `npm/package.overrides.json`)
    merges the right name in, and the crate is already at 0.5.0 (the SPEC-099 cut shipped earlier
    today), so the finalized `pkg/package.json` came out correct on the first build — name
    `crustyimg-wasm`, version `0.5.0`, correct `main`/`module`/`types`/`files`/`homepage`/`license`/
    `keywords`, package `README.md` present, `.wasm` verified size-profiled (`name` debug section
    42 B, DEC-066 fingerprint).
  - **One real identity bug found and fixed:** `repository.url` was wasm-pack's plain
    `https://github.com/jysf/crustyimg` (copied from `Cargo.toml`, deliberately left un-overridden by
    SPEC-075 as "wasm-pack's value is right"). It isn't — `npm publish --dry-run` silently
    auto-corrected it to the canonical `git+https://github.com/jysf/crustyimg.git` form and warned
    about it. Fixed by adding an explicit `repository` override to `npm/package.overrides.json` so
    the committed `package.json` and what the registry would actually store agree; re-ran, the
    warning is gone.
  - **`just wasm-npm-smoke` green.** Pack → fresh install → bare-specifier import → `initSync` →
    `info`/`transform` for png/jpeg/webp/avif, all asserted (independent PNG IHDR parse, AVIF
    ISOBMFF box check). Asserts `crustyimg-wasm@0.5.0`, no `.node`/lifecycle script, zero transitive
    deps.
  - **`npm publish --dry-run` clean** (after the repository fix) — 8 files, 2.0 MB tarball, no
    warnings. Full output captured below for the maintainer.
  - **npm README honest + complete.** Found it stale against the current wasm surface: it documented
    `init`/`info`/`transform`/`optimize`/`version` but not `optimizeDetailed`/`score` (both real,
    shipped surface — SPEC-079/081, and what the live demo actually calls). Added a `## Caveats`
    section (explicit `init()` requirement, single-threaded/blocking, AVIF encode-only + can't
    self-score an AVIF output) and extended the snippet + API table to cover `optimizeDetailed` and
    `score`. Kept the copy plain — no spec/DEC references, per the project's user-facing-copy
    convention.
  - **`just validate` green** (224 front-matter blocks). Also ran the full native gate (`just
    check` = fmt-check + clippy + build + test, all green) as an extra check, since the spec's
    acceptance bar implies "nothing else broke" — `git diff --stat` confirms only `npm/README.md`
    and `npm/package.overrides.json` changed; no `src/` touch.

- **New decisions emitted:** None. This is packaging polish inside DEC-067's existing frame, not a
  new decision.

- **Deviations from spec:**
  - The spec's Inputs section assumed the `pkg/package.json` name/version mismatch still needed
    fixing; it did not (SPEC-075 + the already-cut 0.5.0 crate had resolved it). The only identity
    fix actually needed was `repository.url`'s non-canonical form, which the spec didn't anticipate.
  - Extended the npm README beyond the spec's literal ask (which named `optimize`/`transform` +
    `score`) to also document `optimizeDetailed`, since leaving a real, demo-used export
    undocumented would make the "honest, no capability overclaimed" bar cut both ways (a package
    README that hides a capability is its own kind of dishonesty).

- **Follow-up work identified:**
  - None new. The existing follow-ups from SPEC-075 stand: a CI job running `just wasm-npm-smoke`
    (nothing runs it automatically yet), and flipping the crustyimg README's "isn't on npm yet" line
    once the maintainer actually runs the publish below.

### The exact command for the maintainer

Run from a clean `pkg/` (i.e. `just wasm-npm-pkg` immediately before, so the `.wasm` and
`package.json` are fresh):

```bash
just wasm-npm-pkg   # rebuild + finalize pkg/ one more time, right before publishing
cd pkg
npm publish
```

- No `--access`/`--tag` flags needed — the name is unscoped and this is the first release, so
  npm's defaults (public, `latest`) are correct.
- **2FA/OTP:** if the maintainer's npm account has two-factor auth on publish (recommended, and
  likely already the case), `npm publish` will prompt for a one-time password interactively — pass
  `--otp=123456` non-interactively, or just answer the terminal prompt.
- After it succeeds: `npm view crustyimg-wasm` should show `0.5.0` instead of 404, and a follow-up
  change should flip the crustyimg `README.md` wasm section from "isn't on npm yet" to the real
  `npm install crustyimg-wasm` line (explicitly out of scope for this build, per spec).

### Cost estimate

Ran in the main-loop build session (not a metered subagent) — order-of-magnitude estimate only, per
the `autonomous-run-cost-estimates` convention: **~35–45k tokens, ~$0.35–0.45.** Cheap relative to
SPEC-075's build/verify because there was no new tooling to write — two wasm-pack release builds
(~20s each), one smoke run, one dry-run, one full native gate (`just check`, dominated by the crate
compile + test suite), and two small file edits.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing slowed the build down, but the spec's Inputs section stated the identity mismatch
   ("`pkg/package.json` currently emitted as name `crustyimg` v0.4.0") as still-live, when it had
   already been fixed by SPEC-075 plus the intervening 0.5.0 crate cut. Not a real obstacle (a
   two-minute check confirmed it), but worth noting: a spec drafted against an older repo state can
   describe a bug that's already gone, and the build still has to verify that rather than trust it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The npm README's honesty bar implicitly extends to *completeness*, not just caveat-wording —
   the spec's Inputs section named `optimizeDetailed`/`score` as part of the surface to document,
   but the Outputs/Acceptance Criteria only asked for an `init()`→optimize/transform→score snippet,
   not full API coverage. The stale README (missing two real, demo-used exports) wasn't something
   the acceptance criteria alone would have caught — I only found it by reading `src/wasm.rs` end to
   end against the current npm README, which the spec's Inputs list did point at but didn't force.

3. **If you did this task again, what would you do differently?**
   — Run `npm publish --dry-run` *before* editing the README, immediately after the first
   `wasm-npm-pkg` build — that's what surfaced the `repository.url` auto-correction warning, and
   finding it first would have let both fixes (identity + README) land in one pass through the
   build/smoke/dry-run loop instead of two.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Nothing on the mechanics — clean single-PR ship, CI green first try, no split needed.
   The framing carried a stale premise ("raw pkg/ emits crustyimg v0.4.0") that the 0.5.0
   crate cut + SPEC-075's finalize had already resolved; the honest read is that this spec
   is *prep-to-dry-run* only — the actual value (a valid npm link, the flipped README line)
   lands at the maintainer's `npm publish`, which this merge does NOT perform. Don't let the
   merge read as "shipped the npm package" — it shipped the *readiness*.

2. **Does any template, constraint, or decision need updating?**
   — No new DEC. DEC-067 (identity/target/versioning) still governs; the lockstep-version
   guard it established is what made identity a non-event here. The one real fix (canonical
   `repository.url` override) is a packaging detail, not a decision.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec, but two gated follow-through steps once the maintainer fires `npm publish`
   and it's confirmed live: (a) fix the demo's broken npm link (`demo/index.html:168`,
   currently 403); (b) flip the crustyimg README "isn't on npm yet" line to a real
   `npm install crustyimg-wasm`. Both are one-line edits gated on the publish being live —
   NOT before.
