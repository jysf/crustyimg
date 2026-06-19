# SPEC-035 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-035-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Limits policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-036**. Read the recipe loader
  + the resize op: the functional validation (version/unknown-op/param) already exists;
  the gap is resource bounding. Caps: recipe text ≤ 64 KiB + ≤ 1024 steps, typed
  `RecipeError::TooLarge`/`TooManySteps` (exit 1), at the `from_toml` choke point + a CLI
  pre-read file-size guard. Noted the resize-upscale-bomb (op-param bound) as an explicit
  out-of-scope follow-up. std-only, no new dep. Updated SECURITY.md + api-contract. Build
  prompt at `prompts/SPEC-035-build.md`.
- [ ] **build** (Sonnet) — make the failing tests pass; see `prompts/SPEC-035-build.md`.
- [ ] **verify** (Opus, Explore) — independent read-only review + gate re-run (incl. lean).
- [ ] **ship** — pause for the user before merge.
