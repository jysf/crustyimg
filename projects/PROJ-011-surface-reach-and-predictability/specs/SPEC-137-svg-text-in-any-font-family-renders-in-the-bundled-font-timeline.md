# SPEC-137 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-137-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-28 (`prompts/SPEC-137-138-frame.md`, shared with SPEC-138). **GO, S.**
      Premise confirmed on `main`, 0.8.0 and 0.7.1: any family but exact `Go` draws 0 px of ink,
      exit 0 (generics, named, unknown, lists, `go` lowercase; a mixed `<tspan>` loses half). A
      probe build proved the fix: five `fontdb` `set_*_family("Go")` calls, since usvg appends
      `Serif` as a last resort. Every row draws, and the working forms stay byte-identical.
      Recommend silent fallback (a warning is [M], its own spec).
- [x] **design** — 2026-10-28 (`prompts/SPEC-137-design.md`). 10 ACs, 7 tests (4 red on `main`, 3
      green guards). Driven: `set_serif_family` alone fixes every row (usvg appends `serif`), so the
      other four setters get a unit row. One pinned pixel hash, identical on x86-64 Linux and arm64
      macOS. fontdb builds with no features: no system-font loader. DEC-054 amendment worded, written at build; DEC-107 released.
- [ ] **build** — prompt: `prompts/SPEC-137-build.md`.
- [ ] **verify** — prompt: `prompts/SPEC-137-verify.md`.
- [ ] **ship** — prompt: `prompts/SPEC-137-ship.md`.
