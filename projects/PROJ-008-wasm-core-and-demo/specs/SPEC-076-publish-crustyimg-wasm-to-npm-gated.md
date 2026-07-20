---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-076
  type: chore
  cycle: design
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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
