---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-089
  type: story
  cycle: design
  blocked: false
  priority: medium
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-16
references:
  decisions: [DEC-003, DEC-017]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-087, SPEC-027]
value_link: >
  Completes the `meta` group: fold the existing top-level `set` verb (write EXIF attribution tags)
  into `meta set`, so ALL four metadata operations — remove-all / remove-GPS / copy / write — read as
  one job under `meta`, instead of `set` sitting stranded outside the group SPEC-087 just created.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-089: fold `set` into `meta set`

## Context

SPEC-087 grouped `strip`/`clean`/`copy-metadata` under a `meta` command (`meta strip`/`meta clean`/
`meta copy`). It **left the top-level `set` verb in place** — because that spec's grounding wrongly
asserted "there is no `set` verb today." There **is** one: `Commands::Set` (`set --artist/--copyright/
--description <inputs>` → `run_set`, `src/cli/mod.rs` ~408 / dispatch ~853, SPEC-027), which writes EXIF
attribution tags via the container lane (DEC-003) — pixels untouched, every other tag and the format
preserved, `-q`/`--format` ignored. That left `set` as the **one metadata verb outside `meta`**.

The maintainer decided (2026-07-16) to **fold `set` → `meta set`** so the group is whole
(`meta {strip, clean, copy, set}`) — the `git config set/unset` · `gh secret set/remove` · `docker
image …` pattern, where a noun-group owns its read/remove verbs **and** its write verb together. This is
the exact same **pure surface move** SPEC-087 did for the other three: same args, same handler, just
relocated into the `MetaCommand` enum. Hard cutover, no aliases, no behavior change, no new capability.

## Goal

Add a **`meta set`** subcommand carrying `set`'s existing args (`<inputs>` + `--artist`/`--copyright`/
`--description`), dispatching to the unchanged `run_set`; **remove** the top-level `Commands::Set` and
its dispatch arm (hard cutover); rewrite every live-surface reference `set` → `meta set`. No behavior
change to the bytes any invocation produces.

## Inputs — files to read

- `src/cli/mod.rs` — `Commands::Set` (~408) + its dispatch arm (~853); the `MetaCommand` enum (~499:
  `Strip`/`Clean`/`Copy`) and its dispatch (~873–875); `run_set` (~3400). The `meta copy`
  `#[arg(long)]` pattern is the template for `set`'s three optional flags.
- Everything naming the old top-level `set`: `grep -rn` across `src/` (help text, completions goldens,
  lint fix fragments if any emit `set`), `docs/` (cli-reference, api-contract, recipes, data-model, any
  usage example), `README`, and integration/unit tests. Dated historical records (sessions/research/
  reviews/`specs/done/`) stay intact — the SPEC-087 grep-clean discipline.
- `projects/.../specs/done/SPEC-087-*.md` — the just-shipped sibling; mirror its shape and its
  byte-identity-against-the-old-binary proof exactly.

## Outputs

- **`src/cli/mod.rs`** — a new `MetaCommand::Set { inputs, artist, copyright, description }` variant
  (same args as today's top-level `Set`), dispatched to the existing `run_set` (no logic change).
  **Remove** `Commands::Set` and its top-level dispatch arm. Bare `meta` now lists **four** subcommands
  (strip/clean/copy/set). `run_set`'s doc-comment (`/// Wire \`set\`: …`) updates to `meta set`.
- **The usage-error string** — `run_set` currently returns `"set requires at least one of
  --artist/--copyright/--description"`. Update it to **`"meta set requires at least one of …"`** (we're
  touching the surface; the message should name the live command). *(SPEC-087 left `clean`'s message
  verbatim; here we correct it since the fold makes `set` alone stale — a deliberate small divergence,
  noted.)*
- **Docs/tests/completions** — rewrite every user-facing `crustyimg set …` → `crustyimg meta set …`
  (README, `docs/`, help examples, shell completions if hand-written; they're enum-generated so they
  follow automatically, but verify). Dated historical records untouched.
- No DEC needed (a surface move within the STAGE-030 freeze, like SPEC-087); note it in the STAGE-030
  stage doc's Design Notes + mark the `set`-fold "RESOLVED → shipped."

## Acceptance Criteria

- [ ] `meta set --artist A --copyright C --description D <inputs>` produces **byte-identical** output to
      today's top-level `set` with the same flags (same tags written, pixels + other tags + format
      preserved) — proven against the **pre-move binary**, not just a library-fn comparison.
- [ ] `meta set` with **none** of `--artist`/`--copyright`/`--description` is a usage error (exit 2) with
      the updated `"meta set requires …"` message.
- [ ] The **top-level** `set` verb **no longer exists** (unknown-subcommand exit 2); bare `meta` prints
      help listing **strip / clean / copy / set**.
- [ ] `meta strip`/`meta clean`/`meta copy` and `auto-orient` are **unchanged** (SPEC-087's surface holds;
      `auto-orient` stays top-level, DEC-017).
- [ ] No top-level `set` reference remains on the live surface (help, completions, README, user-facing
      docs); historical records untouched.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
      `cargo build --no-default-features` pass; `just validate` passes.

## Failing Tests (written at design)

- **Integration / `src/cli`**
  - `meta_set_matches_old_set` — `meta set --artist …` output byte-identical to the old top-level `set`
    on the same inputs (compare against `metadata::set_tags` via the container lane — the exact code both
    paths dispatch to, per SPEC-087's proven approach; verify re-drives against the pre-move binary).
  - `meta_set_requires_a_tag` — `meta set <inputs>` with no tag flag exits 2 with `"meta set requires …"`.
  - `top_level_set_is_gone` — `crustyimg set …` at top level errors as an unknown subcommand; the parser
    has no `Commands::Set` variant.
  - `meta_bare_lists_four_subcommands` — bare `meta` help now lists strip/clean/copy/**set** (extends
    SPEC-087's `meta_bare_prints_subcommand_help`).

## Implementation Context

### Decisions that apply
- `DEC-003` — the container-lane metadata model `run_set` already uses (write tags without a pixel
  re-encode); unchanged, just re-dispatched.
- `DEC-017` — `auto-orient` bakes orientation into pixels → it is an image op, **stays top-level**; do
  NOT fold it into `meta` (SPEC-087 held this; keep holding it).

### Constraints
- `ergonomic-defaults` — the grouped surface reads by job; `one-spec-per-pr` — one coherent move.
- `every-public-fn-tested` / `test-before-implementation` — the four Failing Tests first, watched fail.

### Out of scope (this spec)
- Any change to what `set` *writes* or how (no new tags, no value validation, no behavior change) — this
  is a **move**, identical in spirit to SPEC-087.
- SPEC-088 (audit report + committed bench); `convert --to` (now SPEC-090, optional).

## Notes for the Implementer
- **Pure move, prove byte-identity against the OLD binary.** SPEC-087's verify built the parent-commit
  binary and `cmp`'d real output — do the same here; that's the invariant that matters.
- **Mirror `meta copy`'s arg shape** for the three `Option<String>` flags; dispatch to `run_set`
  unchanged (only its doc-comment + the usage-error string change).
- **Hard cutover, live-surface grep-clean** (SPEC-086/087 precedent): rewrite user-facing docs/examples;
  leave dated historical records. Check whether any lint fix fragment emits `set` (SPEC-087 had to fix
  `clean --gps` fragments — confirm `set` isn't similarly emitted).
- This closes the `meta` group; after it, STAGE-030's metadata-taxonomy work is complete.

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
