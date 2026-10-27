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
- [x] **design** — `prompts/SPEC-134-design.md`. Opus, 2026-10-25, $7.30 (in `cost.sessions`).
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
- [x] **fix** — prompt: `prompts/SPEC-134-fix.md`. Sonnet, 2026-10-26, draft PR #202 at
      `a3547cc`: three doc fixes; `name_pinned_format` reads only the template at N inputs (new
      `resolved_input_count`); 5 rows added (41 → 46), 4 RED before / green after, negative control
      flips exactly those 4; Fixed entry + DEC-105 bullet; matrix green; CI 16/16. ~$1.29.
- [x] **re-verify** — prompt: `prompts/SPEC-134-reverify.md`. Opus, 2026-10-26, ~$1.99. **⚠ PUNCH
      LIST at `a3547cc`**: behaviour, docs, CI all correct (bug gone on 3 verbs; one-input order
      kept; glob/dir-of-one, stdin, missing-path edges right). Gap: every N-input row uses literal
      paths, so `resolved_input_count` → `inputs.len()` (M2) stays green while the mislabel returns
      for `'in*.jpg'`. Double resolution judged acceptable (backlog item filed); `meta`'s duplicate
      N-input branch: optional cleanup, taken into fix 2.
- [x] **fix 2** — prompt: `prompts/SPEC-134-fix2.md`. Sonnet, 2026-10-26, draft PR #202 at
      `ca8a961`: 3 glob rows (RED under M2, exactly those) + 2 `meta` N-input rows (46 → 51);
      `meta` pre-flight calls `name_pinned_format(global, all.len())`; matrix green; CI 16/16.
      ~$0.72.
- [x] **re-verify 2** — prompt: `prompts/SPEC-134-reverify2.md`. Opus, 2026-10-26, ~$1.02. **✅
      APPROVED at `ca8a961`.** M2 → exactly the 3 glob rows RED; `ops.rs` cleanup equivalent in
      every case (driven on `meta strip`); full suite 1039/0; CI green. Branch then updated with
      `main` (#205, `Cargo.toml` only) to `7bfcbe9`; PR #202 squash-merged as `7034d94`,
      2026-10-26 PDT.
- [x] **ship** — prompt: `prompts/SPEC-134-ship.md`. Sonnet, 2026-10-27. Reflected; recorded the 3
      unrecorded verify sessions plus ship in `cost.sessions`. PR #202 merged 2026-10-26 PDT
      (`7034d94`). Total cost $34.43 across 9 sessions.
