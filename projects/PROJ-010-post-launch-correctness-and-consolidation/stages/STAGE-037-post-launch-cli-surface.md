---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-037                     # stable, zero-padded within the project
  status: on_hold                   # proposed | active | shipped | cancelled | on_hold
  # PARKED 2026-08-10, by its own criterion rather than a new one. This stage has always
  # said "pull it when there's an adoption signal or a maintainer decision to broaden the
  # surface — not on the launch clock." 0.7.0 has shipped and no adoption signal exists
  # yet, so the condition is unchanged and the honest status is on_hold rather than
  # proposed — `proposed` reads as queued work. Revive when someone other than the
  # maintainer asks for `convert --to` or for more bundled recipes.
  priority: low                     # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-010                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-20
shipped_at: null

# Re-homed 2026-07-26 from PROJ-008 STAGE-032, where it was framed but never
# started. Content unchanged; the ID and parent project moved. No spec shipped
# under the old number, so nothing is split across projects.

value_contribution:
  advances: >
    Adds convenience on top of the frozen CLI surface without reopening it — post-launch quality-of-life
    verbs and recipes that the STAGE-030 freeze deliberately left out so 1.0 could ship on a stable core.
  delivers:
    - "A `convert --to <fmt>` convenience surface (explicit one-shot format conversion), alongside the
      existing engine/`web`/`optimize` verbs."
    - "Additional bundled recipes (social/archive presets) on top of the shipped web/gallery/product set."
  explicitly_does_not:
    - "Change, rename, or remove any of the ~14 verbs frozen in STAGE-030 — this is additive only."
    - "Add a new codec, engine capability, backend/service, or ML — the PROJ-008 territory guardrails stand."
    - "Block the launch — this stage is post-launch, pulled only when there's a reason to."
---

# STAGE-037: post-launch CLI surface enhancements

## What This Stage Is

The home for additive CLI conveniences that STAGE-030's surface freeze deliberately deferred. The
taxonomy froze at ~14 one-intent verbs so 1.0 could launch on a surface with no dependents but the
maintainer and no relaunch risk. This stage holds the optional-but-nice surface work that can land
*after* launch without touching that frozen core: an explicit `convert --to` conversion verb and extra
bundled recipes (social/archive presets). It is strictly additive — nothing here renames, removes, or
re-specs a frozen verb.

## Why Now

Not now — **proposed and deferred by design.** STAGE-030 shipped SPEC-092 out of scope because a
convenience rename plus extra recipes are exactly the kind of surface that should not gate a launch or
churn a just-frozen CLI. Captured here as a real stage (rather than an out-of-scope brief bullet) so the
work has a spec backlog to grow into when pulled. Pull it when there's an adoption signal or a maintainer
decision to broaden the surface — not on the launch clock.

## Success Criteria

- `convert --to <fmt>` exists as an explicit one-shot conversion, consistent with the frozen verb set,
  with no change to any existing verb's behavior or output (byte-identity for unchanged paths).
- Any new bundled recipes follow the shipped recipe registry conventions (file-path-wins precedence,
  plain behavior-first headers per the recipe-header guard, SPEC-096).
- All gates green (native + `--features avif` + lean).

## Scope

### In scope
- The `convert --to` convenience verb; additional social/archive bundled recipes; the in-repo doc/test
  updates those require.

### Explicitly out of scope
- Any change to the STAGE-030 frozen verbs; new codecs/engine features/backends/ML; anything that would
  force a CLI relaunch.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [M] ⚡ **`apply --recipe` ignores `--format` on a multi-input batch, and
  disagrees with itself on the default. Re-driven on `main` at `232c9cf`, 2026-08-22.** From the
  read-only CLI-surface audit (F6 + F7 — **one defect, not two**).

  | invocation | output format |
  |---|---|
  | `apply` **1** JPEG, no `--format` | **PNG** — the source format is CHANGED |
  | `apply` **2** JPEGs, no `--format` | JPEG — the source format is PRESERVED |
  | `apply` **1** JPEG, `--format png` | PNG ✅ honoured |
  | `apply` **2** JPEGs, `--format png` | **JPEG — the flag is SILENTLY IGNORED** |

  **`apply`'s multi-input path does no format resolution at all.** It preserves the source format,
  ignoring both the default-PNG rule the single-input path applies *and* an explicit `--format`.
  The two paths disagree in **both** directions, which is why the audit saw it as two findings.
  No warning, no error, exit 0. **`--name-template` is not the discriminator** (an explicit
  `{stem}.{ext}` behaves identically) — it is purely single-input vs many.

  ⚠ **Controls run by the audit:** `resize`, `thumbnail` and `watermark` given the same two inputs
  and the same `--format` all wrote the requested format correctly. **This is specific to `apply`**
  — the one verb whose entire purpose is running a recipe over a batch — not to the batch path.

  ⚡ **Consequence beyond tidiness (audit's F7):** a `build` lockfile pins bytes the `apply`
  spelling of the same recipe cannot reproduce, so two commands the docs present as interchangeable
  are not. **Decide which default is intended and make the other match** — that is the design call,
  and it is byte-changing on a shipped verb, so it wants a migration.

  📌 Same class as the `--json`-with-a-pinned-format case the project deliberately raises as a
  usage error (exit 2) — **except here it is silent**, which the report contract explicitly forbids.

- [ ] (not yet written) — [S] **`--explain` goes silent exactly where `--json` raises a usage
  error.** Re-driven on `main` at `232c9cf`:

  ```
  optimize photo.png --explain human -o out.avif -y   → exit 0, stdout 0 B, stderr 0 B
  optimize photo.png --json          -o out.avif -y   → exit 2, "--json/--timing report the
                                                        auto-decision and are unavailable with
                                                        a pinned format"
  ```

  `docs/cli-reference.md:441` states the contract: a report flag under a pin is *"a **usage error
  (exit 2)** — not a flag that silently does nothing."* **Human `--explain` violates it in exactly
  the case the contract calls out.** Drop the `-o` pin and `--explain` emits its full report, so the
  feature works — it fails silently only on the pinned path. Likely one line, at the guard `--json`
  already reaches (`reject_audit_without_autodecide`).

- [ ] (not yet written) — [M] ⚡ **A recognized `-o` extension silently switches `web`/`optimize` from
  auto-decide to pinned-convert — changing the quality, the score, and the report.** Reported by the
  maintainer against 0.7.0 (homebrew) while optimizing a photo for jysf.org; **re-driven on `main`
  at `4514345`**, 200×200 RGBA PNG:

  | command | bytes | report |
  |---|---:|---|
  | `web photo.png -o a.avif` | **6,459** | **silent** |
  | `web photo.png --out-dir od1/` | **8,932** | prints the summary |
  | `web photo.png --out-dir od2/ --format avif` | **6,459** | silent |
  | `convert photo.png --format avif` (default q) | **6,459** | — |

  ⚠ **The report's framing is the right symptom and the wrong cause.** It is **not** `-o` vs
  `--out-dir`, and it is **not** two code paths with independently-specified encode settings:
  `--out-dir --format avif` is byte-identical to `-o a.avif`. The real axis is **pinned vs
  auto-decided**, and `run_web` (`src/cli/optimize.rs:770-785`) treats *a recognized extension on
  `-o`* as a format pin exactly as `--format` does. One deliberate rule, applied consistently at
  three sites (`:66`, `:633`, `:772`).

  **What is already correct and documented:** the pin rule and the missing report are both in
  `docs/cli-reference.md:144` and `:441` — and the contract is stricter than the reporter saw
  (`--json`/`--timing` under a pin is a usage error, exit 2, not a silent no-op).

  ⚡ **What is NOT documented, and is the actionable gap:** pinning also moves the encode from
  `FAST_LOSSY_QUALITY = 85` (the auto decision) to `AVIF_DEFAULT_QUALITY = 80` (`convert`'s
  default). **Nothing user-facing says a filename extension changes the quality.** That is exactly
  the reporter's own question 3, and the answer is: deliberate, consistent, undocumented.

  **The design question worth a ruling, not just a doc fix:** `-o out.avif` reads to a user as
  *where the file goes*, not *what codec settings to use* — yet it silently changes three things at
  once (quality, scoring, reporting). This is the same shape as the defects STAGE-046 spent a wave
  fixing: **a mode switch the user cannot see.** Options: warn when an extension triggers a pin;
  require `--format` to pin and treat `-o`'s extension as destination only; or document it loudly
  and leave the behaviour. ⚠ Any change here is **byte-changing on the flagship verb** and wants a
  migration, so it is a 0.7.2+ item, not a tag blocker.

- [ ] (not yet written) — [M] ⚡ **`watermark` cannot be expressed in a recipe, and neither can the
  output format. Recipe coverage measured on `main` at `92a60b7`, 2026-08-21.** **Maintainer decided
  2026-08-21 that watermark must be recipe-expressible** — which is also this stage's own unpark
  criterion (*"a maintainer decision to broaden the surface"*), so **the `on_hold` status above is
  now stale and wants a ruling.**

  **What a recipe can express — the whole list.** The built-in registry
  (`src/operation/registry.rs:80-83`) holds **four** constructors: `identity`, `invert`, `resize`,
  `auto-orient`. A fifth name, `optimize`, works in the bundled recipes but is **not a registered
  operation** — it is a terminal marker (`OPTIMIZE_STEP_OP`, `src/recipe/mod.rs:302`) special-cased
  by the recipe machinery. Driven: every other name returns `error: unknown operation '<x>'`.

  | CLI capability | in a recipe? |
  |---|---|
  | `resize`, `auto-orient`, `edit --invert` | ✅ |
  | `optimize` (and `web`, as resize + the marker) | ✅ via the terminal marker |
  | **`watermark`** — image/text, font, size, colour, gravity, opacity, scale, margin, tile | ❌ **the whole verb, ~10 params** |
  | **output format and quality** | ❌ **structural — see below** |
  | `thumbnail --size --square` | ❌ no `thumbnail` op; `--square`'s crop semantics may not reduce to a `resize` mode |
  | `meta` strip / clean / set | ❌ container lane (DEC-003) — possibly by design, but a recipe cannot say "strip GPS" |
  | `responsive` | ❌ output multiplicity, not an operation |

  ⚠ **The output-side gap is the one nobody asked about and it is structural.** `Recipe`
  (`src/recipe/mod.rs:176`) has exactly four fields — `version`, `name`, `description`, `steps`.
  **There is no format field and no quality field.** So `convert --format webp -q 80` is not
  expressible as a recipe at all; the only way a recipe reaches an encoder decision is the
  `optimize` marker, which *chooses* the format automatically. A user who wants "always WebP at
  q80, applied to a directory" cannot write that recipe today. **This is a bigger hole than
  watermark and should be scoped with it, not after it.**

  📌 **Only `edit` has `--save-recipe`** (driven across all 12 verbs). So the one path that
  *emits* a recipe covers only the three ops it can perform — which is why the gap has stayed
  invisible: nothing that can save a recipe can do anything a recipe cannot express.

  ⚠ Not a gap, but a documentation error found while checking: **AGENTS.md:452's glossary defines
  Gravity as anchoring "a watermark or crop region"** — there is **no crop capability** anywhere in
  the CLI (`crop` is not a verb, not an `edit` flag, and not a registry op). The glossary describes
  something that does not exist.

- [x] **SPEC-092 — CANCELLED 2026-08-23 (maintainer).** `convert --to` rename + social/archive
  recipes. Deferred from STAGE-030 on **2026-07-20**, making it the oldest open item in PROJ-010.
  **This stage's own unpark criterion is "an adoption signal or a maintainer decision to broaden
  the surface"** — 0.7.0 and 0.7.1 have both shipped and **no adoption signal has appeared**.
  ⚠ **A rename with no demand is churn on a frozen CLI surface** (STAGE-030), and it was carrying a
  PROJ-010 success signal, so it had the power to keep the project open indefinitely without anyone
  choosing that.
  📌 **The reason to cancel rather than keep deferring:** a deferred item and a cancelled one look
  identical in a backlog, but only one of them stops binding. This was blocking PROJ-010's closure
  by accident. **Revisit trigger:** a real user request for `convert --to`, or a decision to
  broaden the CLI surface — at which point re-file it fresh rather than reviving this entry.
  Optional convenience surface; additive on top of the frozen ~14-verb core.

**Count:** 0 shipped / 0 active / 0 pending / **1 cancelled (SPEC-092)** — re-derive with a grep you just ran

## Design Notes

- **Additive-only discipline.** Unlike STAGE-030 (a hard cutover), this stage may not rename or remove a
  shipped verb — the surface is frozen. New verbs/recipes sit alongside the existing set.
- Recipe headers must stay plain and behavior-first (enforced by `bundled_recipe_headers_are_plain`,
  SPEC-096).

## Dependencies

### Depends on
- STAGE-030 (PROJ-008, shipped 2026-07-20) — the frozen ~14-verb surface and recipe registry this
  stage extends.

### Enables
- Nothing blocks on this; it is pure post-launch convenience.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
