---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-071
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-15
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/mod.rs"
  - "src/analysis/decide.rs"
  - "src/quality/mod.rs"
  - "tests/cli.rs"
  - "docs/*"

tags:
  - optimize
  - verify
  - shrink
  - taxonomy
  - ssimulacra2
  - explain-json
---

# DEC-071: `optimize --verify` (opt-in score) + removing the `shrink` verb

## Decision

The two-tier finish of the STAGE-030 taxonomy freeze (DEC-069/DEC-070). Surface + deletion only —
no engine or decision change.

1. **`optimize` gains an opt-in `--verify` flag.** The keep-dimensions default stays lean and
   **score-free** (SPEC-084 acceptance #4: scoring a full-resolution winner costs ~107 ms/MP —
   too much to run on the verb everyone runs). `--verify` turns the single SSIMULACRA2 readout back
   on **for that run**, reusing the existing `score_winner_once` plumbing (SPEC-085's `always_score`
   path): it threads `verify` straight into `run_optimize_autodecide`'s score flag. The flagship
   `web` still scores **always** (SPEC-085) — its winner is downscaled, so the score is cheap;
   `--verify` is `optimize`-only.

2. **The score is reported on the human channel AND, under `--verify`, in the JSON explain.** The
   summary/human-`--explain` lines carry ` · ssim NN.N` (existing SPEC-084 plumbing). The
   `crustyimg.optimize.explain/v1` JSON gains a trailing **`"ssim":NN.N`** field — emitted **only when
   the winner was actually scored** (`web`, or `optimize --verify`), so a non-verify run's JSON stays
   byte-identical (the field rides `ExplainTrace.verify_score`, `None` → omitted). This is a
   backward-compatible extension of the pinned `/v1` schema, not a version bump: existing consumers of
   a non-verify run see no change. The score never appears on the format-pinned path (`-o x.ext` /
   `--format` bypass the auto-decision and its score, exactly as before).

3. **The `shrink` verb is REMOVED — a hard cutover, no alias, no deprecation.** `web` (downscale +
   modernize to AVIF) does what `shrink --max` did and better; `optimize` (keep dims + the opt-in
   perceptual/byte-budget searches) owns `shrink --target/--ssim/--max-size`. So `Commands::Shrink`,
   `run_shrink`, `shrink_auto_config`, and `DEFAULT_SHRINK_MAX` are deleted. Shared helpers were
   renamed to neutral names (`shrink_params` → `resize_max_params`, `DEFAULT_SHRINK_QUALITY` →
   `DEFAULT_LOSSY_QUALITY`) since `optimize`/`web`/`responsive` still use them. The stale
   `run_optimize` doc-comment ("perceptual target / visually-lossless by default") is corrected — the
   default is the fast fixed-quality decision; the searches are opt-in.

## Migration mapping (shrink → web / optimize)

| Old | New |
|---|---|
| `shrink <in>` / `shrink --max N` | **`web <in>`** (downscale to a web default + modernize to AVIF), or **`optimize --max N`** to keep dimensions except the long-edge bound |
| `shrink --target T` / `shrink --ssim S` | **`optimize --target T`** / **`optimize --ssim S`** (the opt-in perceptual search) |
| `shrink --max-size SIZE` | **`optimize --max-size SIZE`** (the opt-in byte-budget search) |
| `shrink -q Q -o x.jpg` | **`optimize -q Q -o x.jpg`** (pinned re-encode) — or just `optimize` for the fast default |

## Why

- **The proof should be opt-in on the primitive, always-on on the flagship.** `optimize` is the honest
  byte-primitive: fast, never bigger, keeps dims. Making it score every run would tax the common path
  for a number most runs don't need. `--verify` is the "show me it's good" switch; `web` shows it by
  default because it can afford to.
- **One intent per verb.** `shrink` overlapped both `web` (its downscale-and-modernize job, now better
  with AVIF) and `optimize` (its quality/size searches). Removing it collapses the ~20→14-verb surface
  the STAGE-030 benchmark called for, before the Show HN — no user but the maintainer, so no alias.
- **Extending JSON only under `--verify` keeps the pinned schema honest.** SPEC-084 deliberately kept
  the score off the `/v1` JSON; `--verify` is the explicit opt-in that turns it on, and gating the field
  on "was it measured" means non-verify consumers are untouched.

## Consequences

- `crustyimg shrink …` now exits **2** (unknown subcommand). Docs that taught `shrink` redirect to
  `web`/`optimize` per the table above; dated historical artifacts (session handoffs, blog, research,
  prior specs/DECs) keep their factual past references.
- The `optimize.explain/v1` JSON may now carry a trailing `"ssim"` field on a `--verify`/`web` run —
  consumers must tolerate an optional key (they already parse a growing object).
- Follow-up from DEC-069 still open: the native(85)/wasm(80) AVIF-quality divergence (untouched here —
  no wasm change).
