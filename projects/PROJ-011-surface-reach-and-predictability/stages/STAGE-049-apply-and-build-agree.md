---
stage:
  id: STAGE-049
  status: shipped
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: 2026-09-03

value_contribution:
  advances: >
    The half of the thesis that must land first: before a recipe can carry a
    format, `apply` and `build` have to agree on what a format means. They do not
    today, and a `*.build.lock` therefore pins bytes the `apply` spelling of the
    same recipe cannot reproduce.
  delivers:
    - "`apply --format` is honoured on a multi-input batch, not silently ignored"
    - "`apply` and `build` produce byte-identical output for the same recipe and input"
    - "A test asserting that agreement, so it cannot drift back"
  explicitly_does_not:
    - "Add format or quality to `Recipe` — that is STAGE-050, and it depends on this"
    - "Touch the `-o`-extension pin, `--explain`'s silence, or any other invocation defect (PROJ-010)"
    - "Build the full `(command × output-flag)` or `bundled_recipe × entry_point` matrices (PROJ-010, SPEC-118)"
---

# STAGE-049: `apply` and `build` Agree

## What This Stage Is

`apply --recipe r.toml` and a `build` manifest naming the same recipe should produce the same
bytes for the same input. They do not. When this stage ships they do, and a test says so.

## Why Now

**First, because it settles the semantics STAGE-050 then has to match.** `Recipe` is about to gain
a `format` field. Adding it on top of two paths that already disagree about what format a recipe
produces would bake the disagreement into the schema rather than fix it.

**Driven on `main` at `232c9cf`:**

| invocation | output format |
|---|---|
| `apply` **1** JPEG, no `--format` | **PNG** — the source format is changed |
| `apply` **2** JPEGs, no `--format` | JPEG — the source format is preserved |
| `apply` **1** JPEG, `--format png` | PNG ✅ honoured |
| `apply` **2** JPEGs, `--format png` | **JPEG — the flag is silently ignored** |

**`apply`'s multi-input path does no format resolution at all** — it preserves the source format,
ignoring both the single-input default and an explicit `--format`. The two paths disagree in
**both** directions, which is why an earlier audit reported it as two findings (F6 and F7). It is
one defect. No warning, no error, exit 0.

⚠ **`--name-template` is not the discriminator** — an explicit `{stem}.{ext}` behaves identically.
It is purely single-input vs many.

📌 **Controls already run, and they are what make this conclusive:** `resize`, `thumbnail` and
`watermark` given the same two inputs and the same `--format` all wrote the requested format
correctly. **The defect is specific to `apply`** — the one verb whose entire purpose is running a
recipe over a batch — not to the batch path in general.

## Success Criteria

- `apply --format X` is honoured for **1 input and for N inputs**, identically.
- `apply` and `build` produce **byte-identical output** for the same recipe, input and settings.
- The chosen default is **stated and justified**, not inherited by accident from whichever path
  someone read first.
- A regression test covers both, and **fails on today's `main`** before the fix.

## Scope

### In scope
- `apply`'s multi-input format resolution.
- The `apply` / `build` default-format disagreement for a plain pixel recipe.
- A targeted encode-identity test: `apply` vs `build`, and `-o` vs `--out-dir`, same recipe.

### Explicitly out of scope
- `Recipe` gaining `format` / `quality` fields — STAGE-050, and it depends on this landing first.
- The `-o`-extension pin, `--explain`'s silence, ICO/TIFF, `IMAGE_EXTENSIONS` — all PROJ-010.
- The full conformance matrices. This builds only the assertion its own change needs.

## Spec Backlog

- [x] **SPEC-126** (shipped on 2026-09-03) — **`apply` and `build` agree on output format.**
  PR #187 → `dd60ef5`. **$68.82** across build ($38.16, Sonnet), verify ($15.64, Opus) and a
  sixth **re-approve** cycle ($15.02, Opus) added because the orchestrator applied verify's own
  punch list. **The design call was settled by measurement:** `apply` at one input was the sole
  outlier across six sibling paths, so it moved to preserve the source and `build` did not move.
  ⚠ **Not released** — batches with STAGE-050 as PROJ-011's single lockfile migration.

**Count:** **1 shipped** / 0 in flight / 0 pending — re-derive with a grep you just ran.

## Design Notes

- ⚠ **This is byte-changing on a shipped verb**, so it batches into PROJ-011's single lockfile
  migration. It must not ship alone in its own release.
- **The default is a real choice, not a bug fix with an obvious answer.** `build` preserving the
  source format is the conservative behaviour and the one a lockfile already pins; `apply`
  defaulting to PNG is defensible for a pixel recipe with no terminal `optimize`, since PNG is the
  lossless target. **Whichever wins, the other changes** — say which and why in the DEC.
- 📌 The `-o` vs `--out-dir` half of the test is cheap to add here and worth it: it is the same
  class of divergence, and PROJ-010 owns the *ruling* on the `-o` pin but not the *assertion* that
  a recipe run agrees with itself.

## Dependencies

### Depends on
- Nothing. This is the entry point of PROJ-011.

### Enables
- **STAGE-050** — `Recipe` cannot gain a `format` field until the two paths agree on what one means.

## Stage-Level Reflection

**Built vs planned.** One spec against a plan of one, all four success criteria met with
evidence rather than assertion: `--format` honoured at both arities (two tests, split apart
at re-approve precisely so the arities fail independently), `apply` and `build` byte-identical,
the default stated and justified in DEC-098, and a regression test confirmed RED on `main`
before the fix and re-driven in **both** revert directions afterwards. **$68.82** and three
cycles became four — a sixth cycle type, `re-approve`, was invented mid-stage.

**Speed and shape.** The stage did what it said and nothing else. The design call was settled
by measuring all six sibling paths rather than by argument, which is why the fix is one rule
reused from `ops::output_format_for` instead of a second special case — and why `build`, the
path a lockfile already pins, did not have to move at all.

**Emergent behaviour, and it is the finding of the stage.** Every defect came from a review
cycle and none from the build — but unlike previous waves, **none of the reviews found the code
wrong.** The code was correct at `f8deb55` and stayed correct through two independent reviews.
What they found, four times over, was records overstating what had been measured. That is a
different failure mode than this repo has been tracking, and it is the one worth carrying
forward.

### Did the stage stay inside its own exclusions?

Yes, but one deserves stating plainly rather than being ticked off. The stage said it would
not *"touch the `-o`-extension pin"* — and it did not touch that **ruling**, which is about a
*recognized* extension pinning the format on `web`/`optimize` and still sits open in PROJ-013
STAGE-037. But SPEC-126 did change `-o` **behaviour** on three single-input invocations
(no `--format`, extensionless path, unrecognised extension — all `4` → `0`). Adjacent, not the
same thing, and ruled a conformance fix because `resize` and `thumbnail` already behaved that
way. ⚠ **The exclusion held in substance; the wording did not anticipate the neighbourhood.**
A future stage writing this exclusion should say *the pin ruling*, not *the `-o` extension*.

The other two exclusions held cleanly: `Recipe` gained no `format`/`quality` field (STAGE-050's
job, and it now inherits settled semantics rather than a disagreement), and neither full matrix
was built.

### What this stage hands to STAGE-050

The thing it was created to produce: **`apply` and `build` now agree on what a recipe's output
format is**, so a `format` field can be added to `Recipe` on top of one rule instead of two. The
`*.build.lock` that pinned bytes the `apply` spelling could not reproduce no longer does.

📌 Two items were filed out of this stage rather than absorbed: a literal `--name-template`
extension does not pin the format for a plain pixel recipe (STAGE-050, and it is the same
family as the open `-o` pin ruling — worth ruling on together), and
`scripts/decisions-audit.sh` cannot parse an inline-array `affected_scope`, which makes DEC-015
invisible to `--changed` (PROJ-013 STAGE-047).

### Proposed process change, on evidence from this stage

⚠ **A punch list applied by whoever received it is unreviewed work, and CI cannot see it.**
`re-approve` cost $15.02 — 22 % of the build — and returned three real items, one of which was
an overstatement in the orchestrator's own backlog filing. All sixteen CI legs were green
throughout, because all three findings were prose. **Recommend AGENTS §15 name this case:** when
the orchestrator applies a punch list itself rather than dispatching one, a read-only re-approve
follows before merge. It should not be automatic for every punch list — it is the *self-applied*
case that needs it.

📌 Also worth a rule, from the pattern behind all four record defects: **when a claim covers N
items, quote the same evidence field for all N.** An item that cannot produce that field is the
one that was never checked. Every overstatement this stage produced is visible in its own
sentence by that test.
