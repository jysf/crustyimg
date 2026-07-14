# Launch readiness — before pointing public attention (Show HN) at the live demo

> The demo is live at **https://jysf.github.io/crustyimg/**. This is the checklist that must clear
> before we point a Show HN (or any public attention) at it. HN interacts with two things: **the
> demo** (must be flawless on *their* browser/device) and **the repo/README** (must tell a coherent,
> non-overclaimed story). HN is unforgiving of rough edges and overclaims — and generous to candor.
>
> A Show HN launches the **demo + the pitch** (and optionally the npm lib + CLI). It does **not**
> require 1.0. Snapshot: 2026-07-13.

## Blockers — do NOT launch until these clear

- [ ] **Ship SPEC-078** — the headline (in-browser **AVIF** conversion) must actually work. Today
      the demo says *"AVIF output is not here yet."* Sending HN to a demo with its headline feature
      disabled underdelivers on the pitch. (STAGE-027's last spec; Web Worker + AVIF + `.avif` input
      + explain.)
- [x] **Desktop cross-browser** — ✅ done (SPEC-078 verify, 2026-07-13): driven CLEAN in **Chrome 150,
      Firefox 150 (real Gecko), Safari 26.5 (real WebKit)** via three separate clients; all three do
      module Worker + `instantiateStreaming` + `createImageBitmap`-decodes-AVIF, all responsive
      through a real ~3.1 s AVIF encode.
- [ ] **Mobile** — ⚠ STILL OPEN, the remaining cross-browser blocker. iOS Safari + Android Chrome
      were undrivable in verify (no simulator/SDK). **Load the live page on a real phone** (module
      Worker + AVIF encode + `.avif` input + layout) before the Show HN. HN has heavy mobile traffic.
- [ ] **README front door** — README.md is **CLI-only today: no mention of the demo, the wasm, or
      `crustyimg-wasm`, and no live-demo link.** That's the page HN clicks through to. Add: the
      one-line pitch, the live demo link, "no server — your image never leaves your browser," honest
      scope, and install paths (cargo / brew / npm).
- [ ] **Decide the npm story** — `crustyimg-wasm` is **unpublished (404)**. If the post says
      `npm install crustyimg-wasm`, publish it first (SPEC-076, gated on maintainer approval). If
      not, don't claim it.
- [ ] **Hostile / edge inputs in the browser** — HN *will* drop huge / garbage / unsupported files.
      The decode caps + clear error messages must hold on the live page — no hangs, no cryptic
      failures. (Hold natively; confirm in the browser.)

## Strengtheners — harden the reception (do if time allows)

- [ ] **Honest numbers** — a `BENCHMARKS.md` (none today): size/speed vs squoosh / sharp under an
      **equal-quality rule**. HN scrutinizes perf claims; honest ones land, hand-wavy ones get dunked.
- [ ] **A GIF/screenshot + the post narrative** — the "I built…", and the crisp *why, not squoosh*
      (squoosh-cli is **abandoned** — that's the wedge).
- [ ] **CLI install one-liners verified** (cargo binstall / brew / released binary) — if the post
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

**SPEC-078 → cross-browser/mobile hardening (in SPEC-078) → README front-door → (publish
`crustyimg-wasm` if the post mentions it, SPEC-076) → launch.**

## Owners / pointers

- SPEC-078 (demo Worker + AVIF + cross-browser) — `projects/PROJ-008-.../specs/SPEC-078-*.md`.
- SPEC-076 (gated `npm publish`) — `projects/PROJ-008-.../stages/STAGE-026-npm-library.md`.
- Positioning / pitch — `docs/territory.md` (the wedge); roadmap Track B (`docs/roadmap.md`).
