# Launch readiness — before pointing public attention (Show HN) at the live demo

> The demo is live at **https://jysf.github.io/crustyimg/**. This is the checklist that must clear
> before we point a Show HN (or any public attention) at it. HN interacts with two things: **the
> demo** (must be flawless on *their* browser/device) and **the repo/README** (must tell a coherent,
> non-overclaimed story). HN is unforgiving of rough edges and overclaims — and generous to candor.
>
> A Show HN launches the **demo + the pitch** (and optionally the npm lib + CLI). It does **not**
> require 1.0. Snapshot: 2026-07-13, **de-staled 2026-07-26 at the STAGE-028/029 close-out.**
>
> ⚠ This file sat at its 2026-07-13 snapshot while **five of its blockers shipped**, so for two
> weeks it read far redder than reality — while being the document the go/no-go keys on. If a doc
> gates a decision, the spec that clears an item must tick it.

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
- [ ] **Mobile** — ⚠ STILL OPEN, the remaining cross-browser blocker. iOS Safari + Android Chrome
      were undrivable in verify (no simulator/SDK). **Load the live page on a real phone** (module
      Worker + AVIF encode + `.avif` input + layout) before the Show HN. HN has heavy mobile traffic.
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
- [ ] **Hostile / edge inputs in the browser** — HN *will* drop huge / garbage / unsupported files.
      The decode caps + clear error messages must hold on the live page — no hangs, no cryptic
      failures. (Hold natively; confirm in the browser.)

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
      0.5.0. NOT re-verified end-to-end since the 0.6.0 cut — worth one pass before the post.**
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

**As of 2026-07-26, every repo-side item is done.** SPEC-078 ✅ → STAGE-029 ✅ (closed, 9 specs) →
README front-door ✅ + BENCHMARKS ✅ + AVIF-in-the-binary ✅ (STAGE-028, closed) → `crustyimg-wasm`
published ✅ → 0.6.0 live on crates.io / brew / Releases ✅.

**What remains is maintainer-only and needs no repo work:**

1. **Mobile real-device test** — the one genuine remaining blocker. Now also covers the RAW
   preview gate: does a phone survive a ~60 MP decode? The answer decides whether platform-aware
   gating is ever worth building (until then the global 60 MP gate stands).
2. **Hostile / edge inputs confirmed on the live page** — holds natively; browser confirmation
   was never recorded either way.
3. **Re-verify the install one-liners at 0.6.0**, if the post mentions the CLI.
4. **ROADMAP read + post draft** — ⚠ the draft needs a **CLI-vs-demo RAW split** fix: the CLI
   reads RAW, and since SPEC-103 the demo does too, behind a stated 60 MP gate. State it honestly.
5. **The go/no-go itself** — moved here from STAGE-028 at that stage's close-out.

## Owners / pointers

- SPEC-078 (demo Worker + AVIF + cross-browser) — `projects/PROJ-008-.../specs/SPEC-078-*.md`.
- SPEC-076 (gated `npm publish`) — `projects/PROJ-008-.../stages/STAGE-026-npm-library.md`.
- Positioning / pitch — `docs/territory.md` (the wedge); roadmap Track B (`docs/roadmap.md`).
