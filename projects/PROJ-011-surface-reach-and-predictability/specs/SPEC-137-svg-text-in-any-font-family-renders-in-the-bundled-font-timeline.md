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
- [x] **build** — 2026-10-28 (`prompts/SPEC-137-build.md`, Sonnet, ~$2.26). Draft PR #217 at
      `fb09421`, CI green on every leg. Commit 1 `51156ab` (tests only). 4 red / 3 green baseline,
      all six controls flipped. Unchecked: the wasm row on unmodified code, and the per-OS CI log
      lines for the system-font row.
- [x] **verify** — 2026-10-29 (`prompts/SPEC-137-verify.md`, Opus, ~$4.75). ⚠ PUNCH LIST at `fb09421`.
      Everything held: the baseline (the wasm row is RED on unmodified code), all six controls, and
      the full AC-1 row set driven under controls 1 and 4. M-a to M-d were caught, ImageMagick ink counts matched
      (main 0 / PR 738), and 0.7.1 and 0.8.0 draw 0 px. The one item: the Linux system-font row
      passes silently in CI, so the log can't show whether it found a font or skipped.
- [x] **punch list** — 2026-10-29 (`prompts/SPEC-137-punchlist.md`, Sonnet, ~$0.90). Code at
      `5cfd87f`, docs at `3de8ccf`. Docker controls: skip without the variable, panic with it, run
      with DejaVu installed. The orchestrator read CI run 40931258121: ubuntu `using DejaVu Sans`,
      macos `using Helvetica`, windows `using Arial`; all code legs green. PR #217 marked ready.
      The fix as prompted: On Linux the row panics
      when `CRUSTYIMG_REQUIRE_SYSTEM_FONT` is set; CI installs DejaVu and runs it with
      `--show-output` on all three OSes. The fix also corrects three text items in the spec.
      Note for release prep: a `build` cache made by a 0.8.0 binary serves blank SVG
      text until the 0.8.1 version bump changes the key.
- [ ] **ship** — prompt: `prompts/SPEC-137-ship.md`.
