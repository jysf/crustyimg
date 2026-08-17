# SPEC-119 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-119-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. **11 ACs** (10 at framing; AC-7b added 2026-08-16 when Call 4
      was revised), 4 design calls, 9 pre-written tests. The multi-frame sweep ran at design and
      changed the scope: **three formats (GIF, APNG, animated WebP), not one**, so complexity
      moved S → M.
- [x] **build** — 2026-08-16, Sonnet. PR #176, 16/16 CI green, 12 files, +1204/−82.
      **$51.24 / 143,470,855 tokens / 61 min** — the most expensive build of the wave, 4.3× the
      cheapest, driven by message count rather than duration.
      Went beyond the spec in one good way: `lint` reads `Image::is_animated_input` instead of
      re-decoding, so the linter and the pixel path **cannot** disagree — that divergence was the
      root cause. AVIF settled by reading `avif-parse`. Chose a separate `format/animated-input`
      rule over broadening the GIF rule.
      ⚠ Its DEC shipped as DEC-092, colliding with SPEC-120's; **renumbered to DEC-093** by the
      orchestrator in `d66a357`.
- [x] **verify** — 2026-08-16, Opus. **⚠ PUNCH LIST** (3 items, all documentation).
      $30.99 / 47,880,996 tokens / 28 min. Headline: *"the implementation is correct and I could
      not break it"* — all 11 ACs re-driven against fixtures built OUTSIDE the repo and validated
      with decoders it doesn't ship, plus 96 byte-identity pairs vs `main`.
      P1 `responsive`/`apply`-plain/`build`-plain still flatten silently, so the Goal is not met;
      P2 the new api-contract paragraph is the exhaustive-sounding short list its own neighbour
      warns against; P3 `lint --max-warnings 0` returns a FALSE GREEN in directory mode on
      animated WebP — the shape CI uses, and the shape Call 1 was accepted on.
      Also measured the detector's cost (+3.4 ms on animated GIF, controls flat), and caught
      itself in the reverting-source-does-not-rebuild trap mid-run, then re-drove every headline
      claim against a correctly-rebuilt binary.
      ⚠ Its cycle-advance + cost commit `c920cb9` is **local on a detached HEAD, unpushed**.
- [x] **punch list** — 2026-08-16, `d9f2d8c`. All five items, documentation only; no `src/` or
      `tests/` change, `cycle:` left at verify. Went beyond the brief: found that `info` warns
      for truncated-JPEG but **not** animated-input, so the two warnings have genuinely different
      verb sets.
- [x] **verify (re-read)** — **done by the orchestrator, 2026-08-16, NOT a separate session.**
      A knowing deviation from the cycle model, recorded rather than drifted into: the punch list
      changed no source, CI was green, and the first verify had already re-driven all 11 ACs,
      three controls, 96 byte-identity pairs and the full matrix. A fresh session would have cost
      ~$10–30 to confirm that a documentation patch says true things. The prompt
      (`prompts/SPEC-119-verify-reread.md`) was written and is retained unused.
      **All four items closed by reading source, not by trusting the report:**
      · **P1** — `## Goal` amended with the exact wired list; `## Known residual` added at the
        spec's `:523`, cross-referencing the STAGE-046 `[M]` rather than re-filing it. ✅
      · **P2** — both verb maps verified against source. The two warnings emit at the *same five*
        sites (`ops.rs:341/440` in `run_pixel_op`, `optimize.rs:1500/1553` in
        `run_optimize_autodecide`, `build.rs:458` in `build_one`); truncated-JPEG has **one
        extra**, `report.rs:257` in `run_info`. That is exactly the documented asymmetry.
        `run_apply` splits at `optimize.rs:56`: the terminal-`optimize` branch warns on **both**
        its pinned (`:73` → `run_pixel_op`) and unpinned (`:87` → autodecide) paths, while a plain
        recipe falls through at `:100` to its own inline loop that never reaches `run_pixel_op` —
        so the "silent" claim is correct for the right reason. ✅
      · **P3** — `IMAGE_EXTENSIONS` has `gif` and `png`, and **no** `webp`; the GIF/APNG
        exemption in the qualifier holds. ✅
      · **The `info` claim — TRUE, and worse than documented.** `run_info` checks
        `is_truncated_jpeg()` at `report.rs:253` and never calls `is_animated_input()`. Beyond
        the missing warning, its report is internally inconsistent: `file_size_bytes` covers all
        frames while `decoded_bytes`/`width`/`height`/`color_type` come from frame 1.
        **Verdict (b): true but itself a defect** — filed `[S]` on STAGE-046 in `33e3d82`.
      ⚠ Method note, **CORRECTED 2026-08-16**: I recorded that `git show`/`git cat-file` had
      returned truncated file contents and blamed `rtk`. **That was wrong and the tooling was
      fine.** `git show "$B:src/cli/ops.rs"` resolved to `git show "$B"` — it printed the
      *commit*, not the file. The varying counts (206, then 7) were that commit's diff, then a
      merge commit's header after `update-branch`; nothing was truncated. Proven three ways: the
      literal ref returns all 1547 lines, `cat-file -p <blob-sha>` returns 1547, and
      `rtk proxy` — the no-filter escape hatch — returned the *same* 7, which alone rules `rtk`
      out. **The real lesson: read the output before diagnosing it.** Counting lines told me
      "truncated"; reading seven lines of `commit/Merge:/Author:/Date:` told me the truth.
- [ ] **ship** — **at ship:** append the verify cost entry (`agent: claude-opus-5`,
      `tokens_total: 47880996`, `duration_minutes: 28.3`, `estimated_usd: 30.99`, breakdown
      `{input: 538, output: 179350, cache_creation: 461199, cache_read: 47239909}`), compute
      `cost.totals` (**191,351,851 / $82.23**), run `just cost-audit`.
