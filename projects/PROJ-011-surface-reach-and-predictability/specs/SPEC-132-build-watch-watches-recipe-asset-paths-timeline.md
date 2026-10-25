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
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
