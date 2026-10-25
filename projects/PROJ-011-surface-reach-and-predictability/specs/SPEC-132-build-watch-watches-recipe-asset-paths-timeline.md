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
- [ ] **design** — `prompts/SPEC-132-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
