# SPEC-084 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-084-<cycle>.md`.

## Cycles

- [x] **design** (2026-07-14) — Adopted the strategy session's build-ready draft; corrected
  PROJ-010→PROJ-008/STAGE-030; validated the premise (SizeBudget-only AVIF gate vestigial since native
  AVIF decode, DEC-058). Refined acceptance #4 mid-flight (`cb1e262`) to forbid always-on scoring in
  the keep-dims default after measuring the score cost (~107 ms/MP).
- [x] **build** (2026-07-14, ~41 min) — `Mode::Fast` + `avif_admissible()` bucket predicate (immune to
  `MAX_SHORTLIST` truncation), single fixed-quality AVIF compare, `score_winner_once()`,
  `FAST_LOSSY_QUALITY=85` (AVIF_DEFAULT_QUALITY stays 80 → convert byte-identical). q85 validated on the
  real corpus + eyeballed. Deviation: made passthrough correctness-safe (raw only when the pipeline was
  a no-op AND no metadata — else ship the smallest processed candidate). Emitted DEC-069. Gates green.
- [x] **verify** (2026-07-14, ~24 min) — **NOT CLEAN.** Drove the real artifact + corpus. Confirmed the
  engine + common photo path + the passthrough privacy fix, but found: (1) BLOCKER — the metadata-forced
  fallback ships a lossless blow-up (6.2 MB from a 1.6 MB source) reported "0% smaller" (never-bigger +
  honesty violation, real repro); (2) always-on score wired into the keep-dims default (measured ~14% at
  24 MP) vs refined #4; (3) stale `--help`; (4) Fast-mode explain says "met the target". convert
  byte-identity + content branch + opt-in searches + truncation + hostile input verified clean.
- [x] **build (fix)** (2026-07-14, ~27 min) — Merged origin/main first (union-resolved refined #4).
  Fixed all 5: `fast_fallback_lossy_entry` (a compact lossy re-encode for a lossy source with no lossy
  candidate — never a lossless blow-up), honest negative savings ("N% larger", never clamped 0%), gated
  the default score off (helper kept for SPEC-085/086), honest `--help` + explain, corrected DEC-069's
  savings to median ~82%.
- [x] **verify (re-verify)** (2026-07-14, ~11 min) — **CLEAN.** Drove all 5 fixes on the binary
  (`detailed_jpeg_with_icc` → compact JPEG not a blow-up, ICC stripped; genuine larger case → "19%
  larger" / `-19`; default has no `· ssim` suffix; honest help/explain; DEC-069 corrected) + a regression
  spot-check (convert byte-identity, photo→AVIF, metadata-free passthrough, content branch, hostile
  input). Gates green. One minor non-blocking note: a stale internal `run_optimize` `///` comment
  (→ SPEC-086).
- [x] **ship** (2026-07-14) — **SHIPPED.** Clean squash-merge **PR #88** (`20c5fba`). DEC-069 recorded.
  Cost 645k tok / **$7.10** (per-session: build $2.30 + verify $2.10 + fix $1.65 + re-verify $1.05;
  design/ship main-loop null; session_count 6). Ship reflection (land framing refinements before the
  build branches; never-bigger must survive the metadata-strip fallback). STAGE-030 backlog: SPEC-084
  shipped (1/6). Follow-ups → SPEC-086 (doc-comment; the `--verify` gate) + the native/wasm AVIF-quality
  alignment. `just validate` + `just cost-audit` green.
