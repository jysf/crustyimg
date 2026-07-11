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
- [x] **build** — installed rustup nightly + cargo-fuzz (+ brew libheif). Ran the 4 targets seeded to
  `-max_total_time=600`. **SVG + RAW clean of crashes; all AVIF findings upstream `avif-parse` 2.1.0**
  (not re_rav1d/our glue): 2 fixed at the boundary (`box_sizes_fit` + `catch_unwind` + `frame_size_limit`)
  with mutation-checked regressions, 1 documented upstream residual (F-AVIF-3). Durability: minimized
  crash fixtures + `tests/fuzz_regressions.rs` + always-on `fuzz_corpus_never_panics` smoke; `just fuzz`
  recipe; run record `docs/research/proj-009-fuzz-run.md`; DEC-062; roadmap ticked. **Deviation (sound):**
  fuzzed `-O`/release for AVIF (upstream `debug_assert!`s never fire in prod) — but ran RAW in debug,
  which hid F-RAW-1. → **PR #76**, 26/26 3-OS CI green, no new default dep. Est. ~280k tok. 2026-07-10.
- [x] **verify** — fresh adversarial session. PASS on valid-AVIF-no-regression, the 2 AVIF regressions
  (mutation-checked), F-AVIF-3's contract (typed error, ~8.5 MB RSS — the "OOM" is an ASAN malloc-hook
  artifact), the 3-OS durability smoke, and the empty dep surface. **FOUND the overclaim (SPEC-068 class):**
  `raw_preview` recorded "CLEAN" but the canonical `-O` `just fuzz raw_preview` reproducibly OOMs on
  F-RAW-1 — a crafted embedded JPEG (SOF 16384×9776) peaks ~1.9 GB past the DEC-034 caps (`max_alloc`
  bounds single-alloc, not peak). Doc-honesty punch list. Est. ~200k tok. 2026-07-10.
- [x] **ship** — confirmed F-RAW-1 on the real binary (782 B .nef → `info` peaks ~1.93 GB); applied the
  doc-honesty punch list on-branch (scoped the raw verdict to parity with F-AVIF-3 across run record +
  roadmap + spec; SVG stays legitimately CLEAN; commit 123b87e, CI green); squash-merged **PR #76** → main
  (**7bd18fc**); filled build/verify/ship cost sessions + `cost.totals` (570k tok / ~$5.13, 4 sessions,
  labelled estimates §4) + ship reflection; timeline; **STAGE-024 marks SPEC-069 shipped** + files the
  F-RAW-1 peak-memory follow-up (**→ SPEC-070, Medium, user-prioritized next**); archived to `done/`;
  cost-audit + validate green; brag + memory. **SPEC-069 SHIPPED — the #1/High untrusted-binary surface
  is now fuzzed + durable.** PROJ-007 continues until STAGE-024 completes. 2026-07-10.
