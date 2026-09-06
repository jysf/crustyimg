# SPEC-129 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-129-<cycle>.md`.

## Instructions

- [x] **design** — 2026-09-06. 10 ACs, 7 settled design calls, 8 failing tests.
      ⚡ **The bug is a one-line consequence of SPEC-128's deliberate seam.** `set_resolved_bytes`
      is a side-channel field the `Serialize` impl skips (`src/operation/mod.rs:123-133`), so
      `to_toml` still emits the path (DEC-100's highest-consequence guard). `target_recipe_hash`
      then hashes `to_toml()`, so two `logo.png`s with different bytes at the same path hash to
      the same key. Reproduced at SPEC-128 verify with a verified control; the fix is one helper
      that folds the resolved bytes' content hashes into `target_recipe_hash` from the SAME side
      channel `resolve_recipe_assets` already populated.
      📌 **DEC-058's own "Revisit if" clause named exactly this case** — *"an output-affecting
      input is discovered outside the seven"*. Call 5 departs from its recommended remedy
      (bump `CACHE_SCHEMA_VERSION`) with a written reason: the new input applies only to
      asset-bearing recipes, and bumping the schema would over-invalidate every existing
      plain-recipe cache entry — a regression in DEC-058's own no-change-hit headline. AC-4 is
      the pin on that.
      ⚡ **Call 1 was ruled by precedent, not preference.** The source input is content-hashed
      unconditionally on every build (`cache_key_for` at `src/cli/build.rs:319-346`) — the
      DEC-058 discipline says a stat-based key is under-invalidation, which is a silent
      wrong-answer failure. mtime survives `cp -p`, `rsync`, `touch -r` and coarse-resolution
      filesystems; size survives lossless recompression and the exact edit this bug is about.
      Content hashing matches the input side and adds one SHA-256 pass per target — dominated by
      the decode+encode a hit exists to skip.
      ⚠ **AC-6's byte-identical sweep is the same 41/41 shape SPEC-128 did.** The verb roster is
      the same, the discipline (sweep by call graph, not by file) is the same, and the positive
      control is the exact SPEC-128-verify reproduction (`main` caches → serves stale; branch
      rebuilds → new bytes). Do not re-derive the corpus boundary; state it in DEC-101.

- [ ] **build** — prompt: `prompts/SPEC-129-build.md`. **Sonnet**, own worktree, branch
      `feat/spec-129-build-cache-asset-hash`.

- [ ] **verify** — prompt: `prompts/SPEC-129-verify.md`. **Opus**, read-only, own worktree.

- [ ] **ship** — merge, punch list, reflect, archive.
