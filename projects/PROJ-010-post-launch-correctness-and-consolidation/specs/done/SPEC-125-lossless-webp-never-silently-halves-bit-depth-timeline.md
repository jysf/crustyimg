# SPEC-125 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-125-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-18. 7 ACs, 3 settled design calls, 4 failing tests.
      Promoted from the STAGE-042 backlog item that **SPEC-121's punch-list cycle filed and
      measured** — `convert --format webp` halves a 16-bit source on the **default path, no
      feature flag**, and prints `ssim 100.0` while doing it.
      ⚡ **The second defect is the dangerous one.** SSIM is computed on 8-bit renderings, so the
      scorer **structurally cannot see** the loss it reports on. With SPEC-090's honest-size line
      the output reads as *"86% smaller, pixel-perfect"* at the moment half the depth was thrown
      away — a metric that cannot see a defect converts silence into positive reassurance.
      Call 1 widens SPEC-121's Call 3 diagnostic from a two-format list to the rule *"the target
      cannot hold the source's depth"*, **derived from encoder capability, not hard-coded** — and
      states the `Tiff`/`Png` 16-bit-capable assumption as a **prior to check**, not a conclusion.
      Call 2 forbids a bare `ssim 100.0` across a depth change; AC-6 pins DEC-019's search path so
      the fix cannot perturb `optimize`'s candidate selection.
      ⚠ **Blocked on SPEC-121 merging** — it owns the sink diagnostic this widens.
      📌 **The backlog item this promotes lives on SPEC-121's branch, not `main`.** Mark it
      `[x] **SPEC-125**` and bump STAGE-042's count **at SPEC-121's ship**, not before — editing
      STAGE-042 on `main` now would conflict with PR #181.
- [x] **build** — 2026-08-21, PR #185. **Sonnet**, own worktree,
      branch `fix/spec-125-lossless-webp-never-silently-halves-bit-depth`. Gate cleared (PR #184
      merged before branching). All 7 ACs met — see the spec's own `## Build Completion`.
      Call 1's set widened beyond the spec's own candidate list, MEASURED not assumed: BMP/lossless
      WebP/AVIF warn, PNG/TIFF stay silent (prior held), GIF/ICO excluded for two different
      non-depth reasons — GIF hard-errors instead of downgrading, ICO's own `image`-decoder
      round-trip is broken independent of depth (filed to STAGE-042, not fixed here). Call 2's SSIM
      qualifier lands without crossing the DEC-019 boundary — it reads the already-decoded images'
      own depth, never touches the scorer. DEC-097 records the full measured table. Also caught: the
      spec's own Context repro command was wrong (`convert` never prints an ssim line; that is
      `web`'s report) — corrected in STAGE-042's backlog entry, not restated here.
- [x] **verify** — 2026-08-21, **Opus**, read-only against `main` (base `f35e28a`), no commits.
      **⚠ PUNCH LIST — 3 items, all record-level, none blocking a tag.** Cost **$23.30** (full
      session; recorded $21.96 at 202 of 208 messages — the cycle measured itself twice 90 s apart,
      saw the drift, and predicted a 1–3 % residual; actual was 6.1 %). Mixed-model, priced per
      message: 206 Opus + 2 Sonnet.
      ⚡ **AC-6 is CLEAN, and that is the headline** — this is a reporting fix, byte-for-byte. Driven
      over a 34-file corpus at both commits, 102 runs per binary, `diff -r` exit 0 — **with a working
      positive control** (the same sweep against `--no-default-features` produced 150 differing
      files, so the harness can go red [[a-harness-that-exercises-nothing-reports-green]]). Then the
      test the build never ran: **the corpus is entirely 8-bit, so `scored_source_depth` is `None`
      throughout it and AC-6 is a no-op there by construction.** Verify hand-seeded four 16-bit PNGs
      (raw zlib+struct, independent of the code under test) and re-ran everything — bytes, exit
      codes, winner and candidate list all identical; only the new JSON key and the new warning line
      appear. Plus the code read: `scored_source_depth` is computed strictly after the winner is
      fixed, and its only two readers are render sites.
      **AC-2's table re-derived with independent container probes** — hand-written PNG/BMP/TIFF/
      WebP/AVIF header parsers rather than `image`, the dependency whose behaviour is the claim
      [[verify-wasm-output-with-an-independent-decoder]]. Exhaustiveness driven both through the CLI
      (8 formats accepted, everything else rejected) and the `pub` library surface (the 7 remaining
      `ImageFormat` variants all `Err`). Nothing is in the "holds the depth" bucket by omission.
      **A finding in the change's favour the build never claimed:** at `f35e28a` a 16-bit photo
      through `web` printed one warning — **for JPEG, the candidate that LOST** — while the AVIF
      downgrade that actually shipped was silent. The widened set closes that.
      **The 3 punch-list items were applied by the orchestrator on `main` at `c5efcf2`.** The one
      that mattered: **the ICO exclusion's universal claim was false in five places including
      shipped source** — `Rgba8` round-trips fine — and **DEC-097 contradicted itself**, since its
      own mechanism sentence and its own proposed fix both depend on `Rgba8` working. ⚠ The
      STAGE-042 entry, the artifact a maintainer rules from, was correct from the start; the claim
      was overstated only where it was restated.
      ⚖ **Verify made one factual slip**, worth recording because it is the same class it was
      catching: it wrote that SPEC-124's Build Completion carries no file inventory. It does. The
      **conclusion** was right, though — the *template* has no such field, so both builds added it
      by hand and SPEC-124's still listed 8 of 9. Fixed at the source: `projects/_templates/spec.md`
      now requires it, built from `git diff --name-only` rather than recall.
- [x] **ship** — 2026-08-21. `cycle: ship`, archived. **$107.88** total (build $84.58 + verify
      $23.30, both re-derived). Verify's 3 items applied on `main` at `c5efcf2`; **maintainer
      approved shipping on that basis.**
      ⚡ **Blocks nothing, but needs a ruling before it can be specced: the ICO round-trip defect.**
      `image`'s own decoder cannot read back its own encoder's output for any colour type except
      `Rgba8` — including plain 8-bit RGB, with no depth question involved. More severe than the bug
      this spec fixed, found only because Call 1 said *derive the set, don't copy it*, and a real
      fix changes output bytes. **warn / fix / accept is yours.**
