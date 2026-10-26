# SPEC-134 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-134-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-134-frame.md`). **GO, M** (filed [S]). Premise confirmed
      and wider: every encoding verb ignores a template extension, and `web`/`optimize`/`apply
      --recipe web` write AVIF into `.png`/`.webp`; `build` already pins on terminal-`optimize`
      targets (DEC-087), so `apply` and `build` disagree on the same recipe + template. The rule
      being reversed lives in `docs/api-contract.md:490`, not in DEC-087.
- [ ] **design** — `prompts/SPEC-134-design.md`
- [x] **build** — prompt: `prompts/SPEC-134-build.md`. **Opus**, draft PR #202 at `635adfb`,
      $11.43, completed 2026-10-26. 22/22 ACs; 41-row table (28 RED baseline rows); NC-1…NC-10
      flipped; CI 16/16 green. DEC-105 updated at build for AC-22. NC-7 has two readings (verify
      judges).
- [x] **verify** — prompt: `prompts/SPEC-134-verify.md`. Opus, $5.14, 2026-10-26. **⚠ PUNCH
      LIST at `635adfb`**, code sound on every check: bytes identical where nothing pins (24 runs),
      caches survive the upgrade (6 cached, 2 pinned rebuilt), 14 rows sniffed with `file`, meta
      refusals right, NC-7 both shapes caught, 7/7 own mutations caught. Punch: (1) CHANGELOG
      understates byte changes on web/optimize; (2) the new exit 2's template trigger missing from
      cli-reference and `web` help; (3) one api-contract sentence false at N inputs. Plus (4),
      ruled in by the maintainer 2026-10-26: at N inputs with `-o`, the pin check reads `-o` while
      the write uses the template (JPEG / AVIF inside `in.png`; also on 0.7.x). Verify cost goes
      into `cost.sessions` at ship.
- [ ] **fix** — prompt: `prompts/SPEC-134-fix.md`. Sonnet, same draft PR #202; then a short
      re-verify.
- [ ] **ship**
