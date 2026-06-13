# Migrating to v5.2 — Reports, Cost Tracking, Business Value

If you've been using an earlier version of this template and already
have a populated project (brief, stages, shipped specs), read this
once. Otherwise you can skip it — new scaffolds from `just new-spec`
and `just new-stage` carry the v5.2 shape automatically.

## TL;DR

- **Nothing breaks.** Old specs without `cost:` or `value_link:` still
  advance through cycles, archive, and show up in `just status`.
- **Reports degrade gracefully** on pre-v5.2 data. Missing fields show
  as "*(not set)*" or "missing cost data" rather than crashing.
- **You don't need to backfill.** Old specs are left alone; new ones
  are born with the v5.2 shape.
- **Adding value/cost to an existing project is a copy-paste job** if
  you want to — see blocks below.

## What's new

1. **Business value structure** in project-brief and stage
   front-matter (`value:`, `value_contribution:`), plus a lightweight
   `value_link:` on specs.
2. **Self-reported AI cost** in each spec (`cost:` block with per-
   cycle session entries).
3. **Two new commands:** `just report-daily`, `just report-weekly`
   that aggregate the above into `reports/daily/` and
   `reports/weekly/`.
4. **`feedback/`** is now the known home for downstream user feedback
   captures (not a required workflow, just a convention).

See `CHANGELOG.md` for the full list.

## Backfilling an existing project (optional)

If you want reports to say something interesting about an existing
project, the minimum-effort path is to add a `value:` block to the
project brief and a `value_contribution:` block to each active stage.
Specs can stay as-is.

### project brief

In the front-matter, right after `shipped_at: null` and before the
closing `---`, paste:

```yaml
value:
  thesis: "one sentence — what this wave of work delivers"
  beneficiaries:
    - "who benefits"
  success_signals:
    - "observable outcome 1"
    - "observable outcome 2"
  risks_to_thesis:
    - "what could make this wrong"
```

### stage

In the front-matter, right after `shipped_at: null` and before the
closing `---`, paste:

```yaml
value_contribution:
  advances: "which part of the project's thesis this stage advances"
  delivers:
    - "user-visible capability"
  explicitly_does_not:
    - "what this stage is NOT trying to do"
```

### spec (only if you want value_link)

At the bottom of the front-matter (before the closing `---`):

```yaml
value_link: "one sentence on what this spec contributes to its stage"
```

For plumbing specs: `value_link: "infrastructure enabling STAGE-XXX's X"`.

### spec cost block (only for future cycles)

If you want cost tracking on specs that already shipped, you can
paste a retroactive `cost:` block — but reports will treat it as
"missing cost data" unless agents or humans populate `sessions[]`.
Easier: let new specs start with it, and let old ones stay bare.

If you do want to backfill, the block looks like:

```yaml
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
```

## What reports show on pre-migration data

Running `just report-daily` and `just report-weekly` against an
existing project with no value/cost populated will produce a report
that:

- Shows **"Project thesis: *(not set — project brief has no
  value.thesis)*"** instead of crashing.
- Counts all existing specs under **"Specs with no cost data yet"**.
  Not a problem — just visibility.
- Shows **"value_link population: 0 with, N without"** — also fine;
  `value_link` is optional.

See `reports/daily/` and `reports/weekly/` in the template for
sample output from the shipped example project.

## If something does crash

The reports are deliberately pure bash + awk + `date` — no yq, no
python. If a shipped cost block or front-matter is malformed, the
awk parsers will likely silently ignore it rather than crash. If
you hit an actual error, the most likely causes:

- A stage file with broken YAML (unclosed quotes, weird indentation)
- A cost block with indentation that doesn't match the template
  (session scalar fields must be at 6-space indent)

Check `scripts/_lib.sh` for the exact patterns the parsers expect.

## What's deliberately out of scope for v5.2

- **Monthly reports** — daily + weekly first; monthly later if
  useful.
- **Narrative generation in reports** — reports are quantitative
  snapshots. Ask Claude to narrate on demand.
- **Budget tracking, caps, alerts** — cost is about visibility,
  not control.
- **Admin API integration for authoritative cost** — self-reporting
  works across Claude.ai, Claude Code, API, and third-party agents
  (Ollama, etc.).
- **Pricing tables** — agents self-estimate in USD.

See `KNOWN_LIMITATIONS.md` for the durable list.
