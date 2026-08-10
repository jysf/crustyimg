# Launch readiness — before pointing public attention (Show HN) at the live demo

> The demo is live at **https://jysf.github.io/crustyimg/**. This is the checklist that must clear
> before we point a Show HN (or any public attention) at it. HN interacts with two things: **the
> demo** (must be flawless on *their* browser/device) and **the repo/README** (must tell a coherent,
> non-overclaimed story). HN is unforgiving of rough edges and overclaims — and generous to candor.
>
> A Show HN launches the **demo + the pitch** (and optionally the npm lib + CLI). It does **not**
> require 1.0. Snapshot: 2026-07-13, de-staled 2026-07-26 at the STAGE-028/029 close-out,
> **de-staled again 2026-08-09 at the PROJ-010 launch-gating close-out.**
>
> ⚠ This file sat at its 2026-07-13 snapshot while **five of its blockers shipped**, so for two
> weeks it read far redder than reality — while being the document the go/no-go keys on. If a doc
> gates a decision, the spec that clears an item must tick it.
>
> ⚠ **It happened again, in the other direction.** Between 2026-07-26 and 2026-08-09 the whole
> PROJ-010 launch-gating wave shipped — the classifier blow-up, the hostile-input pass, and
> shipped-verb correctness — and this file recorded none of it, while still asserting "every
> repo-side item is done". Reading *greener* than reality is the worse failure: it hid the
> release-cut blocker below, which nobody had written down.

## Blockers — do NOT launch until these clear

- [x] **Ship SPEC-078** — ✅ done (2026-07-13, PR #86). In-browser AVIF conversion works both ways
      off the main thread; the explain readout is honest. (STAGE-027 complete.)
- [x] **STAGE-029 — demo launch quality** — ✅ **done (closed 2026-07-26, 9 specs).** Shipped speed
      10, the intent-led redesign, the SSIMULACRA2 readout, RAW input, and the classifier fix.
      Original framing below. (the blocker as framed 2026-07-13 after a measured perf
      investigation). The demo is live but mis-serves the common photo case: a 12 MP photo → AVIF is
      **~33 s** at the hardcoded rav1e speed 6 (a silent spinner reads as a *hang*), the default
      output is lossless-WebP which makes photos *bigger*, and "Auto" falls back to a slow JPEG
      search (~4–11 s, ~13 %). Ship STAGE-029 before pointing traffic here: **speed 10** (measured
      3.6× faster), an **intent-led "make it smaller" redesign** (Auto picks AVIF for photos, a
      never-bigger guard, an offered resize, megapixel-keyed warnings + a live timer), and the
      **SSIMULACRA2 score readout** (the differentiator vs squoosh). See
      `projects/PROJ-008-.../stages/STAGE-029-demo-launch-quality.md`.
- [x] **Desktop cross-browser** — ✅ done (SPEC-078 verify, 2026-07-13): driven CLEAN in **Chrome 150,
      Firefox 150 (real Gecko), Safari 26.5 (real WebKit)** via three separate clients; all three do
      module Worker + `instantiateStreaming` + `createImageBitmap`-decodes-AVIF, all responsive
      through a real ~3.1 s AVIF encode.
- [x] **Mobile** — ✅ **done (SPEC-101, device gate PASS, maintainer-decided).** iOS Safari +
      DuckDuckGo (both WebKit) driven end-to-end on a real iPhone: drop → convert → read score →
      download all worked, including a real photo dropped straight from the Photos library.
      Android Chrome (Blink) — **not tested (no device); accepted on maintainer judgment** as a
      launch-readiness call, not a build blocker (the demo is static/no-backend and would degrade
      gracefully). Original text below.
      (was: ⚠ STILL OPEN, the remaining cross-browser blocker. iOS Safari + Android Chrome
      were undrivable in verify (no simulator/SDK). **Load the live page on a real phone** (module
      Worker + AVIF encode + `.avif` input + layout) before the Show HN. HN has heavy mobile traffic.)
- [x] **README front door** — ✅ **done (SPEC-082, PR #105; extended by SPEC-100, PR #106).**
      Command-first hook, live demo link, the no-server privacy line, honest scope, and install
      paths; all 48 fenced commands were run to verify. Original text below.
      (was: README.md is **CLI-only today: no mention of the demo, the wasm, or
      `crustyimg-wasm`, and no live-demo link.** That's the page HN clicks through to. Add: the
      one-line pitch, the live demo link, "no server — your image never leaves your browser," honest
      scope, and install paths (cargo / brew / npm).
- [x] **Decide the npm story** — ✅ **done (SPEC-076).** `crustyimg-wasm` is published on npm
      (0.5.0), so the post may claim `npm install crustyimg-wasm`. Original text below.
      (was: `crustyimg-wasm` is **unpublished (404)**. If the post says
      `npm install crustyimg-wasm`, publish it first (SPEC-076, gated on maintainer approval). If
      not, don't claim it.
- [x] **Hostile / edge inputs** — ✅ **done (SPEC-107): driven, not assumed.** A committed
      corpus (zero-byte, text-as-image, truncated JPEG/PNG/AVIF, a forged pixel-count bomb, an
      empty-OBU AVIF) is driven through the native CLI; the equivalent hostile shapes (zero-byte,
      non-image text, a truncated JPEG, an oversize PNG, plus the SAME committed empty-OBU AVIF)
      are separately driven through headless wasm. Both surfaces agree: no hang, no panic, no
      OOM on any input, and every input now gets a clear, typed message (the one live defect
      found — a truncated JPEG silently succeeding on `web` — is fixed, DEC-085). See
      `tests/hostile_inputs.rs` + `tests/wasm_roundtrip.rs`'s AC-7 cases.
      **Genuinely browser-specific and still open:** whether the demo *surfaces* these errors
      legibly in the UI (the engine returns a typed message; whether the page renders it
      readably is untested), and how a phone behaves on the largest inputs (the decode caps are
      global, not device-aware — see the RAW/60 MP note in "Critical path" below). Left on the
      board for the mobile device pass. Original text below.
      (was: HN *will* drop huge / garbage / unsupported files. The decode caps + clear error
      messages must hold on the live page — no hangs, no cryptic failures. (Hold natively;
      confirm in the browser.))
- [ ] **Cut a release — the shipped CLI predates every PROJ-010 fix.** ⚠ **NEW BLOCKER, added
      2026-08-09.** `v0.6.0` was tagged **2026-07-24**. Everything PROJ-010 fixed landed after
      it: the 18.5× classifier blow-up, the silently-succeeding truncated JPEG, seven verbs
      returning sideways images, and a `build` that could not run any bundled recipe — plus the
      demo's RAW support. **63 commits and 13 `src/` files** separate the tag from `main`.
      The **demo is fine** (`.github/workflows/pages.yml` redeploys from `main` on `src/**`), but
      `brew install crustyimg`, `cargo install crustyimg` and the Releases downloads all still
      hand out the binary PROJ-010 exists to fix. A post that mentions the CLI would point HN at
      the broken build. **Cut 0.7.0** — minor, not patch: orientation baking changes output
      dimensions and `edit --save-recipe` changes its output shape, both behaviour changes on
      shipped verbs. Follow `RELEASING.md`; the maintainer fires the tag push.
      *Nobody had this written down until the PROJ-010 close-out — see the second staleness note
      at the top of this file.*
- [ ] **The README promises something `transform()` cannot do.** README:34–36 tells readers to
      "start from a bundled `web`/`gallery`/`product`" recipe and says "the same recipe TOML runs
      in the browser demo too, via the wasm `transform()` binding." It does not: every bundled
      recipe ends with the reserved terminal `optimize` step, and `wasm::transform` is the one
      call site that still does not strip it — driven, it returns `unknown operation 'optimize'`.
      The demo works only because `demo/worker.js` hand-builds a different, terminal-step-free
      recipe. This was filed as out-of-scope on the grounds that the shipped demo never reaches
      it — true of the demo UI, but the README sends readers down exactly that path, and
      `crustyimg-wasm` is a **published npm package**. Fix `transform` (the same strip helper;
      its format is always caller-pinned via `out_format`, so no decision is needed) or correct
      the README. **Fix before the 0.7.0 cut** — the README renders on the crates.io crate page.

## Strengtheners — harden the reception (do if time allows)

- [x] **Honest numbers** — ✅ **done (SPEC-083, PR #108, DEC-080).** `BENCHMARKS.md` ships an
      iso-quality comparison over 8 real photos, and publishes the losses: crustyimg wins size 0
      of 8 and is slower on the clock, but per core it is a wash — the gap is threading, not the
      encoder. Original text below. (was: a `BENCHMARKS.md` (none today): size/speed vs squoosh / sharp under an
      **equal-quality rule**. HN scrutinizes perf claims; honest ones land, hand-wavy ones get dunked.
- [ ] **A GIF/screenshot + the post narrative** — the "I built…", and the crisp *why, not squoosh*
      (squoosh-cli is **abandoned** — that's the wedge).
- [~] **CLI install one-liners verified** (cargo binstall / brew / released binary) — **0.6.0 is
      live on crates.io / brew / Releases, and SPEC-082 verified every fenced README command at
      0.5.0. NOT re-verified end-to-end since the 0.6.0 cut. 0.7.0's release commit is prepared
      but untagged, so the channels still serve 0.6.0 — do this pass once the tag has fired.**
      (was: if the post
      mentions the CLI, it must install cleanly.

## Already handled — strengths to lean on in the pitch

- **Static, client-side, no backend** → an HN spike costs nothing, can't be DDoS'd, no rate limits,
  and *"your image never leaves your machine"* is a real privacy story. State it — it's a selling
  point, not just a non-problem.
- **Honest scope discipline** — the demo README already owns its limits (WebP lossless-only,
  AVIF-encode-not-decode, the `file://` explanation). Carry that candor into the README + the post.
- **Safe on untrusted input** — the decoder fuzz gate ran (PROJ-007 / DEC-062); a real trust story.
- **Permissive licensing** (MIT/Apache), pure-Rust, single static binary.

## Don't block on

1.0; the full CLI-quality pass (man pages, `--help` examples, SBOM/signed releases); a docs site /
cookbook; Wave-4 manifest / Wave-5 geometry. These strengthen adoption over time but are not
gates for a demo-centric Show HN.

## Critical path

**As of 2026-08-09, the correctness work is done and two repo items remain.** SPEC-078 ✅ →
STAGE-029 ✅ → README front-door ✅ + BENCHMARKS ✅ + AVIF-in-the-binary ✅ → `crustyimg-wasm`
published ✅ → 0.6.0 live ✅ → **PROJ-010 launch-gating complete ✅** (STAGE-034 classifier,
STAGE-035 hostile input, STAGE-039 shipped-verb correctness — 5 specs, 4 decisions).

**Still repo work, both blockers above:**

0. **Cut 0.7.0**, and **fix `wasm::transform` first** — ✅ **repo work done 2026-08-10; the tag is
   the only thing left, and it is yours.** `wasm::transform` now runs the bundled recipes
   (SPEC-112, PR #144) so the README's claim is true, and the release commit
   `chore(release): v0.7.0` is prepared on `chore/release-0-7-0` with the full `RELEASING.md`
   gate green (test 841/0, clippy, fmt, lean build, cargo-deny, `publish --dry-run`, plus
   `just wasm-test` 37/37 by hand — no CI leg runs that suite). **Not tagged**: `git tag -a
   v0.7.0` + the push are maintainer-authorized and fire crates.io, Homebrew and the Release
   page in one go.
   **npm correction:** the package is `crustyimg-wasm` and the registry has **only 0.5.0
   (2026-07-21)** — the 0.6.0 cut never republished it, so npm is two minors behind, not one, and
   its sole release predates the `transform` fix. (`pkg/package.json` is a gitignored `wasm-pack`
   artifact, not a maintained file — the earlier note here read the working tree instead of the
   registry.) Republish at 0.7.0 is decided and recorded on STAGE-040; the publish is
   maintainer-gated.

**What remains after that is maintainer-only and needs no repo work:**

1. **Mobile real-device test** — ✅ **done (SPEC-101)**: iOS Safari + DuckDuckGo PASS on a real
   iPhone; Android Chrome untested, accepted on maintainer judgment. Still genuinely open: does a
   phone survive the ~60 MP RAW-preview decode specifically — RAW landed in the demo via
   SPEC-103/104, after SPEC-101's device pass, so it has never been on-device tested. The answer
   decides whether platform-aware gating is ever worth building (until then the global 60 MP gate
   stands).
2. **Hostile / edge inputs confirmed on the live page** — ✅ **native + headless wasm done
   (SPEC-107)**, holds and is now driven, not assumed (the one defect found is fixed, DEC-085).
   Still genuinely open: does the demo *surface* these errors legibly in the UI, and how a phone
   behaves on the largest inputs — both fold into item 1's device pass.
3. **Re-verify the install one-liners at 0.7.0** — after the tag fires, not before; 0.7.0 is not
   on any channel until then. If the post mentions the CLI, it must install cleanly.
4. **ROADMAP read + post draft** — ⚠ the draft needs a **CLI-vs-demo RAW split** fix: the CLI
   reads RAW, and since SPEC-103 the demo does too, behind a stated 60 MP gate. State it honestly.
5. **The go/no-go itself** — moved here from STAGE-028 at that stage's close-out.

## Owners / pointers

- SPEC-078 (demo Worker + AVIF + cross-browser) — `projects/PROJ-008-.../specs/SPEC-078-*.md`.
- SPEC-076 (gated `npm publish`) — `projects/PROJ-008-.../stages/STAGE-026-npm-library.md`.
- Positioning / pitch — `docs/territory.md` (the wedge); roadmap Track B (`docs/roadmap.md`).
