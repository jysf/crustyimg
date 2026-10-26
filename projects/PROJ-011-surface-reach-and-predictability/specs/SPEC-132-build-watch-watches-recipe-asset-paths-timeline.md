# SPEC-132 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-132-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-132-frame.md`). **GO, S.** Premise confirmed on a
      release build of `main`: editing a watermark `image` or `font` asset outside every watched
      dir triggers 0 rebuilds (source-edit control 1, idle control 0); an asset beside its recipe
      is caught by accident. Must touch `src/cli/build.rs`: the watch set is recomputed only on
      manifest edits, so a re-pointed asset is also missed.
- [x] **design** — 2026-10-25 (`prompts/SPEC-132-design.md`). **S, 12 ACs, 13 failing tests + 1
      control.** Re-measured on a release build of `main`: re-pointed `image`/`font` and an asset
      dir created mid-session are also never watched (3 independent conditions, one negative
      control each). Watch set gains asset dirs (shallow); recompute after every rebuild,
      re-register on a changed set or a now-existing failed root. DEC-060 amended (no DEC-103).
- [x] **build** — prompt: `prompts/SPEC-132-build.md`. **Sonnet**, branch
      `feat/spec-132-watch-recipe-assets`. Draft PR #201, ~$2.03 (measured before the Linux fix
      and merge turns, so low), completed 2026-10-25. 12/12 ACs, C1–C3 flipped. ⚠ **Out-of-spec
      change:** a `make_watcher` filter dropping inotify open/read-close events, because
      `build --watch` re-triggered itself on Linux (~216 attempts / 45 s), reproduced on
      unmodified `main`, so the bug is in released 0.7.x. Verify judges it.
- [x] **verify** — prompt: `prompts/SPEC-132-verify.md`. Opus, 2026-10-26. **Two independent
      verify sessions ran** (the first, c50ceccb, ~$4.79, never reported back; the second,
      613a8d01, $5.22) and **both returned ⚠ PUNCH LIST at `1a18a01`**: (1) no CHANGELOG Fixed
      entry for the Linux self-trigger, (2) DEC-060 lacks the event-kind rule. Linux loop
      measured: 207 idle builds / 45 s before, 0 after; all five edit kinds still rebuild once;
      macOS/Windows unaffected. Optional: 3 test strengthenings (surviving mutations M4, M5,
      AC-8(d)). Both verify costs go into `cost.sessions` at ship (not now: the PR branch also
      edits that list, so a `main` edit would conflict at merge).
- [x] **fix** — prompt: `prompts/SPEC-132-fix.md`. Sonnet, 2026-10-26, draft PR #201 at
      `1e88c8d` (`775a4c9`): CHANGELOG Fixed entry, DEC-060 event-kind amendment, AC-8(d) /
      AC-7(d) / AC-5 tests tightened; tests only in `src/`; CI 16/16 green. $0.96 (in the spec's
      `cost.sessions` on the branch). The reporting session did not re-run the mutations itself.
- [x] **re-verify** — prompt: `prompts/SPEC-132-reverify.md`. Opus, 2026-10-26. **✅ APPROVED
      at `1e88c8d`.** Fix scope tests-only; M4 and M5 each turn exactly their new test red; M3's
      narrow form (empty only for a non-string value) turns only AC-8(d) red; AC-5's count reads
      1 on unmutated code (not vacuous); `build_watch.rs` 3/3 clean runs; CI every leg green.
      ~$1.31. PR #201 squash-merged as `7eca94b`, 2026-10-26.
- [~] **ship** — prompt: `prompts/SPEC-132-ship.md`. Sonnet. Records 4 unrecorded sessions'
      costs (2 verify, 1 no-op fix, re-verify) from their transcripts.
