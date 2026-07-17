# SPEC-093 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-093-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-17. Origin: SPEC-089's verify flagged (out of scope) that `set`
  mangles EXIF Orientation (6 → 1536) and loses GPS precision; the orchestrator **independently
  reproduced it and found it is WIDER** — `meta clean --gps` (the PRIVACY verb, documented to preserve
  orientation) corrupts it too, so the bug is in the shared container lane (`src/metadata/tiff.rs`), not
  in `set`. Pre-existing (SPEC-026/027 era); the pre-move binary is identical. **Survived because ASCII
  tags are immune and every test checks only ASCII tags — plus byte-identity proofs compared against an
  equally-broken oracle ("identical to the old bytes ≠ correct bytes", SPEC-089's verify).** A clean
  orchestrator hypothesis (normalized-LE serialize vs big-endian input) was **REFUTED at framing**: both
  `MM` and `II` inputs corrupt identically → the bug is unconditional and undiagnosed. DEC at build.
- [x] build — worktree session (PR #94, commit 7a4b089). Fix: `Tiff::byte_order` preserved through
  `serialize` (DEC-076, amends DEC-046). Diagnosed the real mechanism empirically and **corrected the
  spec's own framing**: the "REFUTED — do not re-derive" byte-order hypothesis was CORRECT. 8 new tests
  behind a serialize-independent fixture builder, all mutation-tested; graded end-to-end with exiftool.
- [x] verify — independent worktree session (2026-07-17). **VERDICT: CLEAN.** Resolved the build↔orchestrator
  mechanism contradiction with driven evidence in favor of the **build**: built genuinely-MM (`4d4d002a`)
  and genuinely-II (`49492a00`) fixtures from a no-EXIF base, verified both the raw TIFF magic **and** the
  stored Orientation value bytes (II encodes `06 00` = correct). Pre-fix: MM → 1536, **II → 6 (clean
  no-op)**; post-fix both → 6 with byte order preserved (post_mm magic MM, post_ii magic II). The
  orchestrator's contradicting "verified-II pre-fix → 1536" was an artifact — reproduced the trap: reusing
  one `-o out.jpg` across runs returns the first (corrupt) output because the tool refuses to overwrite.
  Mechanism is **complete, no second bug**: `meta copy` pre-fix already preserved MM correctly (segment
  graft, never reaches the writer), proving only the parse→serialize path was affected. Re-verified: all 8
  tests fail under in-place mutation (exactly 8; copy/strip tests correctly pass), fixture builder is
  genuinely `serialize`-independent, GPS precision (50.4957) and thumbnail LONG both preserved, gates green
  (test 745/758, clippy, fmt, no-default-features, `just validate`), CI all-pass incl. browser smoke, DEC-076
  vs DEC-075 no collision (075 reserved by open SPEC-090/091), api-contract "orientation" claim now true, two
  follow-ups correctly deferred.
- [x] orchestrator re-test (on verify's report) — **I was WRONG.** Re-tested the mechanism, reused an `-o`
  output filename under a no-overwrite CLI, read stale MM output, and reported "verified-II→1536" while
  accusing the build of the stage's own failure mode. Verify's negative control (unique filenames, byte
  order built from a genuinely-empty base) proved the build correct: pre-fix MM→1536, II→6 no-op; post-fix
  both→6, order preserved. Second time in one spec I let a plausible test stand in for a checked one
  (framing's `-ExifByteOrder` refutation was the first). Lesson: a negative control is not optional.
- [x] ship — squash-merged PR #94 (**d14a13a**) 2026-07-17. CI hit the **SPEC-091 re_rav1d DisjointMut
  flake** on a *required* `avif` check (BLOCKED a clean PR); confirmed it's the flake (same commit passed
  on the sibling trigger; the SPEC-093 diff touches only `src/metadata` + docs), rerun cleared it → CLEAN.
  Bookkeeping: cycle→ship, 3 cost sessions with `model:` (build Opus $3.00 / verify Opus $2.15 / ship
  $0.70 ≈ **$5.85**), timeline, DEC-076, archive, STAGE-030, memory + brag. Lessons banked incl. my own
  unverified re-test. The flake blocking a required check escalates SPEC-091 (maintainer to sequence).
