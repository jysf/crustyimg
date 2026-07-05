---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-044
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-07-04
supersedes: null
superseded_by: null

affected_scope:
  - src/sink/mod.rs
  - src/cli/mod.rs

tags:
  - cli
  - output
  - ux
  - sink
---

# DEC-044: `--out-dir` auto-creates the target directory (safely)

## Decision

A batch `--out-dir DIR` **creates `DIR` (and parents) if it does not exist**, rather than
failing. Creation happens in the **`Sink::Dir` write path** (`src/sink/mod.rs`), so every
batch command — resize/thumbnail/shrink/convert/optimize/auto-orient/watermark/strip/
clean/set and `apply --recipe` — behaves consistently; `run_responsive`'s existing
explicit `create_dir_all` is deduplicated into the same path.

- **Safety unchanged:** creating the user's chosen output directory is `mkdir -p`-style
  intent, not a traversal vector — the DEC-035 per-file guard (`safe_join`, which rejects
  an expanded output *name* containing `..`, a separator, or an absolute path, and refuses
  symlinked destinations) is untouched and still validates every file written into `DIR`.
- **Genuine failures get a clear error:** if `DIR` can't be created (a *file* already
  exists at that path, or permission is denied), return a distinct
  `SinkError::OutDirCreate { path, source }` → **exit 5**, replacing the previous cryptic
  `SinkError::Io("could not write output")`.

## Context

Reported via real usage (PATCH-001): `crustyimg shrink *.jpeg --max 1600 --out-dir web/`
failed for every input with the opaque message *"could not write output"* — because the
`web/` directory didn't exist and the batch fan-out never created it. Yet `responsive`
*did* auto-create its `--out-dir` (an inconsistency), so the codebase already established
auto-create as the intended behavior.

The design question was create-vs-error. Auto-create wins: (1) `responsive` already does
it, so uniformity resolves toward the existing precedent; (2) a dedicated `--out-dir` flag
is an explicit statement of intent — creating it fulfills that, it doesn't guess; (3) a
batch tool whose job is producing output files shouldn't require a manual `mkdir` first.
The typo-safety argument for erroring is weak because the *current* behavior already
errors — just with a useless message; the real bug is the message, and a clear error path
is kept for the genuine can't-create cases.

## Alternatives Considered

- **Keep erroring, but improve the message** ("output directory does not exist: DIR") —
  safer against typos, but inconsistent with `responsive` and less ergonomic; rejected in
  favor of auto-create + a clear error for *genuine* failures.
- **Create the dir at each CLI dispatch site** (in `require_out_dir_for_batch`) — works,
  but duplicates the call across commands; putting it in `Sink::Dir` is DRY and covers
  `apply` + future batch commands automatically.
- **Guard/restrict which out-dirs may be created** (e.g. forbid absolute paths) — over-
  restrictive; the user controls their own invocation and could already write to an
  existing such dir. The per-file `safe_join` name guard is the correct, sufficient
  boundary.

## Consequences

- **Positive:** `--out-dir` "just works" across all batch commands; the confusing
  "could not write output" is replaced by success (or a clear typed error). Removes a
  real first-use papercut (surfaced the day v0.1.0 shipped).
- **Negative:** the tool now creates a directory the user named — but that is the
  explicit intent of the flag, and typo-creating `web-jbg/` is a minor, self-evident
  cost vs. the friction removed.
- **Neutral:** no change to the security posture — DEC-035's output-name traversal /
  symlink guards are unchanged and still apply to every file written. First user of the
  DEC-043 patch lane.
