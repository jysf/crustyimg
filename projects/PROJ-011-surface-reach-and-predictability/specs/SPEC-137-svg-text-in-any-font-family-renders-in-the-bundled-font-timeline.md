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
- [ ] **design** — prompt: `prompts/SPEC-137-design.md`.
- [ ] **build** — prompt: `prompts/SPEC-137-build.md`.
- [ ] **verify** — prompt: `prompts/SPEC-137-verify.md`.
- [ ] **ship** — prompt: `prompts/SPEC-137-ship.md`.
