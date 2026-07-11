# SPEC-069 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — run the never-executed decoder fuzz gate (AVIF/SVG/RAW/HEIC), STAGE-024 #1 (High),
  the roadmap's pre-1.0 gate. The `fuzz/` harness EXISTS (detached workspace, 4 targets, run+seed
  recipes in `fuzz/Cargo.toml`, seed fixtures in `tests/fixtures/<fmt>`) and was never run (no
  nightly/cargo-fuzz in prior envs). So the spec = **run it** (nightly + `cargo fuzz`, seeded, a
  documented budget) → **triage** every crash (our bug / loose cap / upstream-guarded) → **convert
  each finding to a deterministic regression test in the NORMAL suite** (minimized bytes →
  `Image::from_bytes`/`raw_preview` → typed error, no panic; the per-PR durability move) → a
  `just fuzz` recipe + committed seed corpus + a written **run record** + an always-on
  `fuzz_corpus_never_panics` smoke → **DEC-062** (gate policy: mechanism, budget, triage,
  regression-conversion, CI decision, HEIC best-effort). **A clean run is a first-class result — don't
  manufacture findings.** 3 default targets REQUIRED; HEIC best-effort (opt-in, C libheif). No new
  default dep (fuzz tooling stays detached/dev; repo-root Cargo/deny untouched). Framing, 2026-07-10.
- [ ] **build** — install nightly + `cargo-fuzz` (+ `brew libheif` for HEIC); `cargo +nightly fuzz
  build` then run each target seeded to the budget floor (`-runs=1000000` or `-max_total_time=600`);
  triage + fix/guard + `cargo fuzz tmin`-minimize each crash → regression test + committed fixture;
  add `just fuzz` + the corpus smoke + the run record + DEC-062; tick the roadmap gate. Gates: default
  + lean + clippy + fmt + `just deny` (unchanged) + `just validate`; repo-root Cargo/lock/deny diff empty.
- [ ] **verify** — fresh session. Confirm the run recipe reproduces (a short `just fuzz` run surfaces
  nothing new); confirm each crash regression FAILS without its fix (mutation-check) and passes with it;
  confirm `fuzz_corpus_never_panics` runs in normal `cargo test` across 3-OS CI; confirm no new default
  dep and the run record matches what actually ran. A clean-run result is verifiable via the recipe + smoke.
- [ ] **ship** — merge PR; build/verify/ship cost sessions + totals + reflection; archive to done/.
  STAGE-024 backlog: SPEC-069 shipped → the #1 (High) surface closed; remaining backlog items follow
  (off-by-53, format-sniff, cache-key profile, etc.). PROJ-007 continues until STAGE-024 completes.
