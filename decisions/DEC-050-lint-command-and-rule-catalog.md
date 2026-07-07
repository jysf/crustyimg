---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-050                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.8                    # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-004                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-07-06
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/lint/**
  - src/cli/mod.rs

tags:
  - lint
  - rule-catalog
  - severity
  - exit-codes
  - config
  - command-shape
---

# DEC-050: `lint` command shape — read-only source-file linter, pinned rule-id catalog, 3-severity model, exit-7 reuse

## Decision

`crustyimg lint [PATHS]…` is a **read-only, advisory, source-file** linter for an image asset tree
— `clippy`/`eslint`/`ruff` for images. It resolves inputs via the shipped `source::resolve`
(glob/dir/file), skips non-images, runs a catalog of deterministic rules, and reports **findings
grouped by file**, each carrying a **stable rule id**, a **severity**, a message, and a **runnable
`crustyimg` fix command** — then maps its outcome to exit codes, **reusing `CliError::CheckFailed`
(exit 7, DEC-025)**: `0` clean · `7` ≥1 `error` (or `warn` count over `--max-warnings`) · `2`
usage/bad config · `3` no inputs. `info` never affects the exit code. The design is fixed on five
points:

1. **Rule ids are a stability surface** — namespaced `area/name` strings (`privacy/gps-metadata-leak`,
   `format/legacy-format`, …), pinned like clippy lint names, so `select`/`ignore` configs stay
   valid across releases.
2. **Three severities** — `error` (wrong/leaking: fails CI) · `warn` (measured savings / correctness
   risk: fails only under `--max-warnings`) · `info` (opt-in polish: never fails).
3. **Read-only** — `lint` NEVER writes an image; every finding *names* a fix (`clean --gps`,
   `optimize --format webp`, `auto-orient`), running it is the user's/Action's choice.
4. **A decode failure is a *finding*** (`size/truncated-or-corrupt` → exit 7), not an abort (exit
   1) — the one deliberate divergence from the rest of the CLI: a linter reports a broken asset.
5. **Config + noise control** — `.crustyimg-lint.toml` auto-discovered walking up to the repo root;
   ruff-style `select`/`ignore` + `per-file-ignores`, eslint-style per-rule severity, per-glob
   `[[budget]]`; and a **savings-threshold gate** (default `min_bytes = 4096`, `min_percent = 10` —
   Lighthouse's own 4 KiB floor) so "could be smaller" rules stay quiet enough to leave on.

No new default dependency: the JSON (and later SARIF) report is hand-rolled, matching
`write_json`/`write_diff_json`.

## Context

PROJ-004 makes crustyimg a *checking* tool, not just a *fixing* one. The rule catalog will grow, so
its conventions must be pinned before rules proliferate. The design questions this DEC settles
(grounded in `docs/research/proj-002-design-lint.md`):

1. What is the command's contract — inputs, output shape, exit codes?
2. Are rule ids stable (a compatibility surface) or free to churn?
3. What severity model, and how does it map to CI pass/fail?
4. Where does config live and how is it discovered?
5. How do we keep the "could be smaller" rules from being noisy enough to get disabled?

Constraints in play: `no-new-top-level-deps-without-decision` (hand-rolled reports); `ergonomic-defaults`
(zero-config must be right); `untrusted-input-hardening` (a corrupt asset is a finding, never a
panic); DEC-025 (the exit-7 `CheckFailed` whose source comment already anticipated "reusable by the
future EXIF audit-linter").

## Alternatives Considered

- **Option A: a URL/page-based checker (Lighthouse-style).**
  - What it is: render the deployed page, measure images in context (DPR-aware responsive,
    offscreen/lazy checks).
  - Why rejected: needs a deployed URL + a layout — it can't run on a *source* file in CI before
    deploy, and it structurally can't see GPS/camera EXIF (CDNs strip it before the browser).
    That's the exact white space we're filling, not competing in.

- **Option B: a format-blind size gate.**
  - What it is: `check-added-large-files`-style — fail if any file exceeds N KiB.
  - Why rejected: format-blind and metadata-blind — it can't say "this PNG should be a WebP" or
    "this leaks a location." It's the thing `lint` *replaces*, format-aware.

- **Option C (chosen): a clippy/eslint/ruff-style source-file linter with pinned rule ids +
  3 severities + exit-7 + discovered config + a savings-threshold gate.**
  - What it is: the design above — deterministic, per-file, read-only, with a runnable fix per
    finding and CI-native exit codes.
  - Why selected: it owns the pre-deploy/no-URL/format-aware gap, folds in the privacy moat,
    reuses shipped plumbing (source resolve, exit-7, info/EXIF/metadata), adds no dependency, and
    its conventions are familiar to anyone who's used clippy/ruff.

## Consequences

- **Positive:** a one-binary, one-exit-code CI check (the deliberate contrast with Lighthouse-CI's
  server/URL); the privacy moat becomes enforceable; the rule-id + severity + config surface is a
  stable contract later rules just register against; `cargo deny` unchanged.
- **Negative:** rule ids are now a compatibility surface — renaming one is a breaking change to
  users' configs (intended: it's what makes `select`/`ignore` durable). The savings-threshold
  defaults are anchors that may need corpus tuning.
- **Neutral:** `lint` is read-only, so "fixing" always routes back through the existing commands
  (or a separate autofix Action) — a clean separation, but it means the linter and the fixer are
  two invocations.

## Validation

- **Right if:** `crustyimg lint content/` is quiet on a clean tree, fails CI precisely on real
  problems (a GPS leak, an over-budget or wrong-format asset), names a fix a user can run verbatim,
  and teams leave it on because the savings-threshold keeps it from crying wolf.
- **Revisit when:** the rule set grows enough to want rule *groups*/presets; or SARIF/Action
  adoption (STAGE-015) suggests a schema change; or corpus data says the 4 KiB / 10% floor is
  wrong.

## References

- Related specs: SPEC-050 (this command core), SPEC-051 (config), SPEC-052 (JSON report),
  SPEC-053 (shipped-capability rules), SPEC-054/055 (engine-backed rules)
- Related decisions: DEC-025 (`diff` `--fail-under` exit-7 `CheckFailed` — reused), DEC-004 (codec
  gating → the fix-suggestion guard), DEC-019/020/021/022 (which codecs a suggestion may name),
  DEC-003 (the metadata lane that *fixes* privacy findings)
- External docs: `docs/research/proj-002-design-lint.md` (rule catalog + Lighthouse-parity map +
  config/exit design), `docs/roadmap-draft-proj-004-005.md`
- Discussions: PROJ-004 framing session 2026-07-06
