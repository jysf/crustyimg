---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-028
  status: active                  # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-13
shipped_at: null

value_contribution:
  advances: >
    The payoff of the whole WASM wave: get the demo + library in front of people without
    embarrassing the project. Groups the launch-specific work (the front-door story, honest proof,
    the publish, the Show HN go/no-go) so it has a home and a close, instead of loose ends after
    the demo works.
  delivers:
    - "A README front door that points at the live demo + the npm lib with an honest pitch (today it's CLI-only — no demo/wasm/live-link)"
    - "Honest, equal-quality benchmarks (BENCHMARKS.md) vs squoosh/sharp — the numbers HN will scrutinize"
    - "A cleared launch-readiness checklist (docs/launch-readiness.md) and a go/no-go for the Show HN"
  explicitly_does_not:
    - "Do the demo feature work (SPEC-078, incl. cross-browser) or the npm publish mechanics (SPEC-076) — this stage times + depends on them, it doesn't re-do them"
    - "Cut 1.0, build a docs site, or the full CLI-quality pass (man pages/SBOM/signed releases) — those are broader Track-B/1.0 work, not gates for a demo-centric Show HN"
    - "Add engine/WASM features"
---

# STAGE-028: launch readiness

## What This Stage Is

The capstone of PROJ-008: turning "the demo works and the lib installs" into "we can point public
attention (a Show HN) at it without regret." It owns the launch-specific work that isn't a demo
feature — the **README front door**, **honest benchmarks**, and the **Show HN go/no-go** — against
the checklist in [`docs/launch-readiness.md`](../../../docs/launch-readiness.md). It depends on, but
does not re-do, SPEC-078 (the demo, incl. the cross-browser pass) and SPEC-076 (the gated npm
publish); it times them together with the front-door story so the launch lands as one coherent moment.

## Why Now

- **The demo is live** (https://jysf.github.io/crustyimg/) and the lib is proven — the artifacts a
  Show HN would point at exist. What's missing is the *presentation*: the repo's front door is
  CLI-only, there are no honest numbers, and the launch pieces aren't sequenced.
- **HN is unforgiving of overclaims and rough front doors** — the demo (SPEC-078) and its
  cross-browser story are the flawless-experience half; this stage is the coherent-story half.
- Framed **proposed** (not active): the active work is finishing SPEC-078 (verify → ship). This
  stage is picked up once the demo feature work lands, so the launch pieces are the last mile.

## Success Criteria

- Every **blocker** in `docs/launch-readiness.md` is cleared (or consciously waived) before the Show
  HN: SPEC-078 shipped incl. cross-browser; README front door; npm-publish decision; edge-input
  handling.
- The **README** leads with what crustyimg is, links the live demo, states "no server / your image
  never leaves your browser," is honest about scope, and lists install paths (cargo / brew / npm).
- **BENCHMARKS.md** exists with an honest, equal-quality methodology and reproducible numbers.
- A recorded **go/no-go**: the checklist is green (or waived with reasons), then launch.

## Scope

### In scope
- The README front-door rewrite; BENCHMARKS.md; the post narrative + a GIF/screenshot; sequencing
  the launch (demo live + `crustyimg-wasm` published + Show HN, pointing at the live URL); keeping
  `docs/launch-readiness.md` current as the checklist.

### Explicitly out of scope
- SPEC-078's feature/cross-browser work; SPEC-076's publish mechanics (this stage *triggers* the
  publish, on maintainer approval); 1.0 / docs site / CLI-quality pass; engine features.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

> **Sequencing (maintainer decision 2026-07-14): STAGE-030 — the CLI taxonomy/quality freeze — lands
> BEFORE this stage.** The README + BENCHMARKS must document the *final* surface and the measured
> numbers, so they are framed only after the freeze. The two specs below are **scaffolded as reserved
> stubs (SPEC-082/083)** and stay in `frame` until STAGE-030 ships.

- [x] SPEC-082 (shipped 2026-07-20, PR #105, ~$7.5) — **README front door.** Command-first hook + live
  browser-demo link + why-vs-sharp/squoosh (measured quality / pure-Rust zero-dep / `web` / browser) +
  honest de-staled install + the 98% headline number. Verified by running all 48 fenced commands (+
  negative control); an independent voice read confirmed it doesn't read AI-written. **Ships to crates.io
  on the 0.5.0 cut.**
- [x] SPEC-100 (shipped 2026-07-20, PR #106) — **README: surface RAW + recipes; note CI action inputs.**
  Added 2 "Why crustyimg" bullets (RAW embedded-preview / declarative recipes) + an action-inputs line;
  the CI section already existed (DEC-051), so not duplicated. Docs-only, claims verified against `src/`.
  Rides to crates.io at 0.6.0.
- [x] SPEC-083 (shipped 2026-07-21, PR #108 `0465c67`, DEC-080, ~$40.5 / 12 sessions) —
  **BENCHMARKS.md + `scripts/bench-compare.py`/`just bench-compare`.** Iso-quality (SSIMULACRA2 ~82
  band, one scorer) cross-tool comparison vs sharp / ImageMagick / `@squoosh/cli` / cwebp over 8 real
  photos (0.7–47 MP). **Honest headline: crustyimg is neither smallest nor fastest** — sharp 4 / IM 2 /
  squoosh 2 on size (crustyimg 0, but within ~1.7× on every photo), 3–9× and 4–14× slower on the clock —
  **but per core it's a wash (faster on 4 of 8), so the gap is threading, not the encoder.** `web`'s
  default lands 80.8 median, ~97% smaller. Two exit-3 guards (dimension + operating-point) +
  `--self-test` 24/24. README's stale 98% corrected to 97%. **5 build / 4 verify passes caught five real
  defects** (squoosh squashed on 6 of 8, portrait mis-sizing, per-core not iso-quality, `web` rows at the
  wrong operating point, a guard advertising reach it lacked) — four invisible to number-checking alone.
- [ ] (coordination, not a spec) — the **Show HN go/no-go**: `docs/launch-readiness.md` blockers
  green, `crustyimg-wasm` published (SPEC-076, on approval), post drafted → launch.

**Count:** 3 shipped (SPEC-082 README front-door, SPEC-100 RAW/recipes/CI, SPEC-083 BENCHMARKS) / 0
active + the launch go/no-go. **0.5.0 SHIPPED 2026-07-20** (frozen CLI + caret + README, live on
crates.io/brew/Release, demo footer 0.5.0); `crustyimg-wasm@0.5.0` live on npm. **Remaining before the
r/rust launch, in order:** (1) **demo pass** — favicons + `site.webmanifest` subpath fix + SPEC-101
(SSIMULACRA2 explainer links, FF/Safari `color-mix()` band check, a visible re-convert signal) + the
**mobile/cross-browser device gate**, batched as ONE browser session; (2) **AVIF in the distributed
binary** — BENCHMARKS.md has to tell readers the flagship path needs `cargo install --features avif`,
so a `brew install` user can't reproduce the headline (own decision + DEC); (3) **public ROADMAP.md**
(themes + honest non-goals, no dates); (4) the go/no-go + drafting the post. **Deferred to first
post-launch:** encoder threading (the benchmark proves per-core parity, so it's the biggest real win —
but it likely trades byte-reproducibility, which PROJ-007 built around, so it needs a probe + a DEC),
then the benchmark-refresh tooling that makes re-running it cost wall-clock instead of tokens.

## Design Notes

- **This stage is mostly docs + coordination, not code.** The risky/technical launch work
  (cross-browser, AVIF, responsiveness) lives in SPEC-078 by design; keep it there.
- **Honesty is the strategy.** The demo README's candor (WebP lossless-only, AVIF encode-not-decode)
  is the model — carry it into the README + benchmarks + post. HN rewards it and punishes the opposite.
- **Lean on the free/private/scalable story** — static, no-backend: an HN spike is free, un-DDoS-able,
  and "your image never leaves your machine" is real.
- **Technical positioning: sync core, no async runtime (DEC-006) — a real differentiator worth a README
  line (→ SPEC-082).** crustyimg is a **synchronous** CLI: no `tokio`/`async-std` anywhere (0 in
  `Cargo.lock`), because image work is CPU-bound and embarrassingly parallel per file — async buys
  overlapping *waits*, and there are none. Batch parallelism is `rayon` data-parallelism across inputs
  with real `--jobs` control (`ThreadPoolBuilder` + `par_iter`, `src/cli/mod.rs` ~1250). The payoff is
  **instant startup + no async coloring**, which lands directly against Node-based tooling (the
  sharp/`@squoosh/cli` pain this wave answers) — pair it with the existing "no native addon" story.
  The decision has *aged well and been re-validated*, which is the credible part: `build --watch`
  (DEC-060) deliberately uses `notify`'s thread + an `std::sync::mpsc` channel rather than a runtime,
  and the wasm target has no threads at all. **Framing note for SPEC-082:** state it as a *consequence*
  (fast startup, one static binary), not as async-bashing — the honesty voice above. Don't overclaim a
  benchmark we haven't run; if we want a startup number, it belongs to SPEC-088's harness / SPEC-083.
  Origin: maintainer asked 2026-07-16 whether the missing `tokio` was a gap — it isn't; DEC-006 (conf.
  0.95, PROJ-001) cut it from the prototype on purpose and the rationale still holds.

## Dependencies

### Depends on
- SPEC-078 (demo Worker + AVIF + cross-browser) shipped; SPEC-076 (`crustyimg-wasm` publish, gated on
  maintainer approval); the live Pages deploy (proven 2026-07-13).

### Enables
- The Show HN / public launch — the adoption moment PROJ-008 was building toward.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
