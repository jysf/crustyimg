# Repo tooling / methodology backlog (non-release)

> A queue of **repo-infrastructure and spec-driven-template improvements** that are wanted in the
> repo but are **not** crustyimg product features — they don't ship in the binary, the npm package,
> or a release. Product/feature direction lives in `docs/roadmap.md`; this list is for the
> `justfile` / `scripts/` / `projects/_templates/` / front-matter-convention layer.
>
> These are picked up opportunistically (a `chore`-type spec, or just done directly on `main` as
> tooling), not sequenced into product waves. Add items freely.

---

## Queued

### 1. Port the lifetime-report commands from `zany-animal-slots`

Bring the whole-repo "lifetime report" tooling into crustyimg. Source:
`~/PSeven/experiments/zany-animal-slots/scripts/lifetime-report.sh` (~8.5 KB) + three `just`
recipes. It complements the existing `report-daily` / `report-weekly` tooling with an
all-time, whole-repo rollup.

- **`just lifetime-data`** → `scripts/lifetime-report.sh data` — prints the whole-repo **Lifetime
  Data Report**: every project / stage / spec / decision / release. **Rule-based, deterministic,
  no LLM.**
- **`just lifetime-report`** → `scripts/lifetime-report.sh prompt` — the same history wrapped in a
  synthesis **prompt** for an LLM to narrate.
- **`just lifetime-save`** → writes the data report to `reports/lifetime/YYYY-MM-DD-HHMMSS.md`
  (timestamped **to the second**, so repeated runs never overwrite).

Port notes: adapt the script's project/stage/spec/decision discovery to crustyimg's layout
(`projects/PROJ-*/`, `decisions/DEC-*.md`, `CHANGELOG.md`/releases); `mkdir -p reports/lifetime`;
keep it POSIX/`bash`-portable per the shell conventions. Small; no new dep. Likely a `chore` spec
or a direct-to-`main` tooling commit.

### 2. Add an `activity:` field to project (and maybe stage) front-matter

Add a human-facing **`activity`** field to the brief front-matter, distinct from the coarse
`status` that tooling keys on. Model it on `bragfile000`'s PROJ-006 brief
(`~/PSeven/experiments/bragfile000/projects/PROJ-006-agent-native-depth-core/brief.md`), which
documents it as:

```yaml
project:
  status: active            # coarse; what tooling keys on (active/shipped/archived/cancelled)
  # activity = the type of work CURRENTLY active within the project. Human-facing detail.
  #   Suggested vocabulary (extend as needed): requirements | design | build | test | blocked
  activity: requirements
```

- **Why:** `status` alone (active/shipped) doesn't say *what kind* of work is live. `activity` gives
  a glanceable "we're in requirements / blocked / testing" without reading the whole brief.
- **Vocabulary — TO FINALIZE (the maintainer flagged the list is not settled).** bragfile000 uses
  `requirements | design | build | test | blocked`. crustyimg's spec **cycle** model is
  `frame | design | build | verify | ship` — so reconcile: a candidate project-level set is
  `requirements | design | build | test | verify | review | blocked | shipped` (+ maybe
  `maintenance`). Pick the vocabulary when this is built; keep it an open string with a documented
  suggested set (bragfile000 does not hard-enum it), not a rejected-on-parse enum.
- **Where:** `projects/_templates/project-brief.md` (the template + the explanatory comment); decide
  whether stages/specs also carry it (specs already have `cycle`, which overlaps — likely
  project-level only, or stage-level, to avoid duplicating `cycle`).
- **Tooling touch:** `just validate` (the front-matter YAML check) must accept the new field;
  optionally `just status` could surface `activity` next to `status`. Backfill existing active
  briefs (currently just PROJ-008). Small; no new dep.

---

## Done

*(move items here with the commit/PR when completed)*

- **Make the cross-tool benchmark refreshable without an LLM** — the harness already runs
  standalone (`just bench-compare --corpus DIR` costs wall-clock, not tokens). What costs tokens
  is everything *after*: transcribing output into `BENCHMARKS.md`, re-checking cells, and updating
  the derived prose. Four changes push that to ~zero: (1) the harness **emits the markdown blocks
  verbatim**, not JSON a model reformats; (2) **generated-region markers** in `BENCHMARKS.md`
  (`<!-- BEGIN GENERATED: per-photo -->`) plus a `just bench-refresh` that rewrites them in place;
  (3) the harness **computes every derived claim** — the smallest-AVIF tally, the speed ranges,
  the worst-case ratio, the score span, `web`'s median, median savings — since those are exactly
  the values that broke repeatedly during SPEC-083; (4) a **`--check` mode** that re-runs, diffs
  against what's published, and exits non-zero naming the moved cells. Pair with a cheap **input
  tripwire**: fail when a benchmark-relevant constant changes (`FAST_LOSSY_QUALITY`, the AVIF
  speed preset, the 2048 downscale default) so staleness surfaces mechanically instead of being
  found by a reader. Principle: *the harness owns the numbers and every claim derived from them;
  the doc holds only narrative that doesn't depend on specific values.*
  **Do this BEFORE encoder threading**, which will invalidate every time column in the doc.
- **CI hygiene, both surfaced merging PR #108:** (a) the workflow appears to trigger on both
  `push` and `pull_request`, so every PR runs the full 3-OS matrix **twice** — doubles cost and
  doubles the chance a network flake blocks a merge; (b) the `cargo-deny` action pulls a Docker
  Hub base image, so a required check can fail for reasons unrelated to this repo (it did:
  `dial tcp ... i/o timeout`, three retries, while the same SHA passed cargo-deny in the duplicate
  run). Consider a non-Docker invocation or a pinned/mirrored image.
- **Stop DCO sign-off recurring** — a verify-cycle commit has landed without `-s` three times now
  (most recently blocking PR #108 until `git rebase --signoff main`). It keeps happening because
  verify sessions commit punch lists as an afterthought. Mechanical fix: a local pre-push hook, or
  make `-s` explicit in the verify prompt's commit instruction rather than relying on the house
  convention being remembered.
