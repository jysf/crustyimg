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
- [~] **verify** — prompt: `prompts/SPEC-132-verify.md`. **Opus**, read-only. Leads with the
      out-of-spec Linux watcher filter.
- [ ] **ship**
