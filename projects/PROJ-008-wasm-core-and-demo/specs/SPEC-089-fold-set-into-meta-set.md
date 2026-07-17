---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-089
  type: story
  cycle: verify
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 190000
      estimated_usd: 1.03
      recorded_at: 2026-07-16
      note: >
        main-loop build session (not a metered subagent) — ORDER-OF-MAGNITUDE ESTIMATE per
        docs/cost-tracking.md's autonomous-run guidance (no subagent_tokens available). Added
        `MetaCommand::Set`, removed `Commands::Set` + its dispatch arm, rewired dispatch inside
        `Commands::Meta`, updated `run_set`'s doc-comment + usage-error string to `meta set`.
        Wrote the four design-named failing tests first (watched them fail against the
        pre-implementation tree), then implemented; grep-cleaned README/docs (cli-reference,
        recipes, data-model, moat, api-contract) and tests/cli.rs + tests/metadata.rs. Gates
        green (test default 734 / avif 747, clippy, fmt, no-default-features, `just validate`).
        Rate: Sonnet blended (~$5.4/MTok, 80/20 in/out, no cache discount, per AGENTS.md §4).
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 260000
      estimated_usd: 2.34
      recorded_at: 2026-07-16
      note: >
        main-loop verify session (not a metered subagent) — ORDER-OF-MAGNITUDE ESTIMATE per
        docs/cost-tracking.md's autonomous-run guidance (no subagent_tokens available). Built the
        parent-commit (218ba57) oracle binary in a throwaway worktree and drove old-vs-new byte
        comparison across 5 paths (3 flags / 1 flag / stdout / fan-out / PNG); built an EXIF+GPS+
        Orientation+Copyright fixture with exiftool (independent decoder) and confirmed tags written,
        others preserved, JPEG SOS scan untouched. Re-ran all gates independently (test default 734 /
        avif 747, clippy, fmt, no-default-features, just validate, decisions-audit). Independent
        grep-clean sweep found 2 docs misses. Rate: Opus blended (~$9/MTok, 80/20 in/out, no cache
        discount, per AGENTS.md §4).
    - cycle: fix
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 130000
      estimated_usd: 0.70
      recorded_at: 2026-07-16
      note: >
        main-loop fix session (not a metered subagent) — ORDER-OF-MAGNITUDE ESTIMATE per
        docs/cost-tracking.md's autonomous-run guidance (no subagent_tokens available). Docs-only
        pass closing verify's 2 defects: rewrote `docs/api-contract.md:333`'s heading from top-level
        `set` to `meta set` (SPEC-027; grouped under `meta` in SPEC-089), matching the `meta strip`/
        `meta clean`/`meta copy` siblings' annotation style, and finished
        `docs/feature-exploration.md:87`'s half-updated command list. Then ran the discipline verify's
        note called out — grepped the live surface (not just the two flagged files) and found 5 more
        stale bare-`set` references verify hadn't flagged: `docs/architecture.md` (prose line 12 +
        Mermaid diagram label line 121), `docs/recipes.md:161`, `docs/moat.md:39`, and
        `guidance/constraints.yaml:40` — all fixed to `meta set`/grouped form; dated historical records
        (sessions/research/reviews/blog/decisions/specs-done) left untouched. No code change. Gates
        re-run green: `cargo test` (734 default / 747 avif — unchanged from build/verify), `cargo
        clippy -- -D warnings`, `cargo fmt --check`, `cargo build --no-default-features`, `just
        validate` (207 front-matter blocks). Rate: Sonnet blended (~$5.4/MTok, 80/20 in/out, no cache
        discount, per AGENTS.md §4).
  totals:
    tokens_total: 580000
    estimated_usd: 4.07
    session_count: 3
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

- [x] `meta set --artist A --copyright C --description D <inputs>` produces **byte-identical** output to
      today's top-level `set` with the same flags (same tags written, pixels + other tags + format
      preserved) — proven against the **pre-move binary**, not just a library-fn comparison.
      *(Build proved byte-identity against `metadata::set_tags` — the exact container-lane fn both the
      old and new dispatch paths call — mirroring SPEC-087's build-phase proof. The stronger pre-move
      **binary** comparison is reserved for verify, per SPEC-087's precedent of splitting that work
      across cycles.)*
- [x] `meta set` with **none** of `--artist`/`--copyright`/`--description` is a usage error (exit 2) with
      the updated `"meta set requires …"` message.
- [x] The **top-level** `set` verb **no longer exists** (unknown-subcommand exit 2); bare `meta` prints
      help listing **strip / clean / copy / set**.
- [x] `meta strip`/`meta clean`/`meta copy` and `auto-orient` are **unchanged** (SPEC-087's surface holds;
      `auto-orient` stays top-level, DEC-017).
- [x] No top-level `set` reference remains on the live surface (help, completions, README, user-facing
      docs); historical records untouched.
      *(**FIX: MET** — `docs/api-contract.md:333` and `docs/feature-exploration.md:87` (verify's 2
      defects) rewritten to `meta set`, plus 5 more stale bare-`set` references verify hadn't flagged
      (`docs/architecture.md` ×2, `docs/recipes.md:161`, `docs/moat.md:39`, `guidance/constraints.yaml:40`)
      found by grepping the live surface directly rather than trusting which files had been touched. See
      the Fix pass section.)*
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
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
- **Branch:** `spec-089-meta-set` · **PR:** (opened against `main`, not merged) · **All acceptance
  criteria met?** Yes — all six acceptance boxes checked; the four design Failing Tests exist and pass
  (`meta_set_matches_old_set`, `meta_set_requires_a_tag`, `top_level_set_is_gone`,
  `meta_bare_lists_four_subcommands`), plus `meta_subcommand_help_parses` extended to cover `set`.
  Driven end-to-end on a real JPEG (`bench/corpus/gradient_small.jpg`): `meta set --artist "Jane Doe"
  --copyright 2026` writes both tags (confirmed via `crustyimg info --exif`); `meta set` with no flags
  errors `"meta set requires at least one of --artist/--copyright/--description"` exit 2; top-level
  `crustyimg set …` errors `"unrecognized subcommand 'set'"` exit 2; bare `meta` lists
  strip/clean/copy/set. Gates green: `cargo test` (default 734 / avif 747), `cargo clippy --all-targets
  -- -D warnings` (clean), `cargo fmt --check` (clean), `cargo build --no-default-features` (builds),
  `just validate` (207 front-matter blocks parse).
- **New decisions:** None. A pure surface move within the STAGE-030 freeze (as framed) — no DEC needed.
- **Deviations:** None from the spec's Outputs/Notes. The one deliberate divergence the spec itself
  called out — updating `run_set`'s usage-error string and doc-comment from `set` to `meta set` (unlike
  SPEC-087, which left `clean`'s message verbatim) — was applied exactly as specified.
- **Follow-ups:**
  - Verify should build the parent-commit (pre-move) binary and `cmp` its `set --artist … -o -` output
    against this branch's `meta set --artist … -o -` on an identical fixture — the stronger old-binary
    proof SPEC-087's verify cycle used, not attempted here (build proved byte-identity against the
    library fn only, per this spec's Notes for the Implementer).
  - None on documentation: README/cli-reference/recipes/data-model/moat/api-contract grep-cleaned; no
    lint fix fragments emit `set` (confirmed via grep of `src/lint/`).
### Build-phase reflection
1. **The design session's own scoping held.** SPEC-087 flagged a design-grounding error (a false "no
   `set` verb" claim) that this spec exists specifically to resolve; SPEC-089's own Context section was
   accurate on a second look — `Commands::Set` at line 425 and its dispatch arm at line 916 matched the
   spec's line-number pointers closely enough to navigate straight to the code, no rediscovery needed.
2. **Splitting the byte-identity proof across build/verify (SPEC-087's pattern) is the right call here
   too.** Comparing the CLI's `meta set` stdout to `metadata::set_tags()` directly is self-contained and
   requires no fixture or parent-commit binary — it proves the new dispatch path reaches the identical
   function the old one did. The stronger claim (byte-identical to what a *user already had* from the old
   binary) needs a second binary to diff against, which is naturally a verify-cycle task with its own
   worktree, not build's.
3. **A "move exactly one variant" spec still touches nine files.** The clap change is ~15 lines (one new
   `MetaCommand::Set` arm, one deleted `Commands::Set` block, one dispatch line moved); the value — "the
   surface reads as one job" — only holds if every user-facing rendering agrees, so five docs files and
   two test files needed the same `set` → `meta set` rewrite. Consistent with SPEC-087's reflection #3:
   the grep-clean is the real work, not the enum edit.

---

## Verify — ⚠️ DEFECTS FOUND (docs-only; engine + surface CLEAN)

Independent verify session (fresh detached worktree at `origin/spec-089-meta-set` @ `dbed129`; Opus).
Verdict: **NOT CLEAN — 2 documentation defects against Acceptance Criterion 5.** Five of six criteria
verified clean, including the load-bearing byte-identity proof. **No code defect: the move itself is
correct.** A docs-only fix cycle closes this.

**Acceptance criteria, row-by-row (diffed against the build's completion table):**

1. **Byte-identity vs the PRE-MOVE BINARY — ✅ MET (the proof the build deferred to verify).**
   Built the parent-commit (`218ba57`) oracle in a throwaway worktree; sanity-confirmed it IS pre-move
   (top-level `set --help` works; its bare `meta` lists only strip/clean/copy). Drove both binaries on
   one fixture (`gradient_small.jpg` + exiftool-written Orientation=6, GPS, Copyright, ImageDescription,
   Make). `cmp`-identical on **all five paths**:
   | case | old `set` → new `meta set` | bytes |
   |---|---|---|
   | all three flags | identical (md5 `0b7de002…`) | 6222 |
   | `--artist` only | identical | 6234 |
   | stdout (`-o -`) | identical | 6204 |
   | multi-input `--out-dir` fan-out | identical (both files) | — |
   | PNG input | identical | 1700 |
   Semantics confirmed with **exiftool** (a decoder I didn't write): Artist/Copyright/ImageDescription
   written; Orientation/GPS/Make preserved. Pixels proven untouched by extracting the JPEG SOS
   entropy-coded scan — byte-identical to the fixture (5299 B). ✅
2. **`meta set` with no tag flag → exit 2 + updated message — ✅ MET.** Drove it: exit **2**,
   `error: meta set requires at least one of --artist/--copyright/--description`. The spec's one
   deliberate divergence from SPEC-087, applied exactly as framed. ✅
3. **Top-level `set` gone / bare `meta` lists four — ✅ MET.** `crustyimg set …` → exit **2**,
   `unrecognized subcommand 'set'` (clap even suggests `meta`). Bare `meta` prints group help listing
   **strip / clean / copy / set**. ✅
4. **SPEC-087's surface holds; `auto-orient` still top-level — ✅ MET.** Top-level `strip`/`clean`/
   `copy-metadata` all exit 2; `meta strip` / `meta clean --gps` / `meta copy` all exit 0 on a real
   JPEG; `auto-orient` still top-level (DEC-017 held). ✅
5. **Live-surface grep-clean — ❌ NOT MET (2 defects, below).** Help text, all three shell completions,
   README, cli-reference, recipes, data-model, moat: **clean**. `docs/api-contract.md` and
   `docs/feature-exploration.md`: **stale**. ❌
6. **Gates — ✅ MET, re-run independently.** `cargo test` **734** passed / 0 failed (default),
   **747** / 0 (`--features avif`) — matching the build's claim exactly. `cargo clippy --all-targets
   -- -D warnings` clean; `cargo fmt --check` clean; `cargo build --no-default-features` builds;
   `just validate` (207 front-matter blocks) passes; `just decisions-audit --changed` clean (build
   correctly declared no new DEC). All five named tests exist and pass individually
   (`meta_set_matches_old_set`, `meta_set_requires_a_tag`, `top_level_set_is_gone`,
   `meta_bare_lists_four_subcommands`, `meta_subcommand_help_parses`). ✅
   *(Note: a first full-suite run showed 6 `tests/build_watch.rs` failures — reproduced as a
   load-induced flake from my own concurrent cargo builds, not a branch defect. `build_watch` passes
   7/7 in isolation and the full suite is green when not competing for cores. Recorded so the next
   session doesn't re-chase it: **these timing-sensitive watch tests fail under parallel build load.**)*

### Defects

**DEFECT 1 — `docs/api-contract.md:333` still documents top-level `set` (must fix).**
The file is the repo's **live public CLI contract** ("Its public contract is the **command-line
interface**"). In its Metadata-lane section, three of four headings carry SPEC-087's migration
annotation — `#### meta strip … (SPEC-026; grouped under meta in SPEC-087)`, `#### meta clean …`,
`#### meta copy …` — and between them sits the un-migrated:
```
#### `set <INPUT...> [--artist S] [--copyright S] [--description S]`  *(SPEC-027)*
```
Repro: `grep -n '#### `set' docs/api-contract.md` → line 333. A reader following the contract runs
`crustyimg set …` and gets exit 2. This is not an overlooked file: **SPEC-089 edited this very file at
line 409** (the STAGE table, correctly rewritten to name `meta set` in SPEC-089) while missing the
actual command-surface section 76 lines earlier. Note the section *body* already reads "Same fan-out +
exit codes as `meta strip`/`meta clean`" — SPEC-087 updated the body's cross-references but not the
heading, so this heading was precisely SPEC-089's to fix.
The build's Follow-ups assert *"None on documentation: README/cli-reference/recipes/data-model/moat/
api-contract grep-cleaned"* — that claim is **false for api-contract**, and it is what let the checked
`[x]` box stand. (Stage pattern: a claim standing in for a check.)
Fix: rewrite the heading to `#### \`meta set <INPUT...> …\` *(SPEC-027; grouped under \`meta\` in
SPEC-089)*` and move it beside `meta copy`, mirroring cli-reference.md's already-correct ordering.

**DEFECT 2 — `docs/feature-exploration.md:87` is half-updated (minor, but decide deliberately).**
```
Commands: `meta strip` (all) · `meta clean --gps` (drop only location — privacy win) ·
`set --artist/--copyright/--description` · `meta copy from→to`.
```
This looks like a "dated historical record" (pre-design research feeding PROJ-001) that the grep-clean
discipline says to leave alone — **except SPEC-087 rewrote this exact line** (`git log -S` confirms
f7ce015 changed `strip`→`meta strip` and `clean --gps`→`meta clean --gps` on it). By its own sibling's
precedent the file is live surface, and the line is now internally inconsistent: three verbs grouped,
one not. Fix (rewrite `set` → `meta set`) or consciously reclassify the file as historical — but the
current half-and-half state is the one option that isn't defensible.

**Correctly left alone (not defects), for the fixer's benefit:** `CHANGELOG.md:166,272` (dated release
records — historical, per SPEC-086/087 precedent); `src/metadata/tiff.rs:18` + `src/metadata/mod.rs:193`
(internal rustdoc naming the capability — and tiff.rs:18 says `set`/`clean --gps` together, so SPEC-087
left its half too; consistent, a nit at most); `src/cli/mod.rs:2041` (`set` = a Rust `HashSet` local,
false positive).

**Independently confirmed, not taken on the build's word:**
- **Lint fix fragments emit no `set`.** The build asserted this from a grep of `src/lint/`; I *drove*
  `crustyimg lint` on a GPS+Orientation-bearing image and read the emitted fixes — only
  `crustyimg meta clean --gps <file>` and `crustyimg auto-orient <file>`. `set` is genuinely not
  emitted (unlike SPEC-087's `clean --gps` fragments, which did need rewriting). Claim holds. ✅
- **Shell completions are enum-generated and clean.** Generated all three: top-level offers
  `… auto-orient watermark meta edit apply build lint completions help` — **no `set`**; `set` appears
  only nested (`__fish_crustyimg_using_subcommand meta; and __fish_seen_subcommand_from set` with
  `--artist`/`--copyright`/`--description`). Verified in fish, zsh, and bash. ✅
- **`src/` diff is a pure move.** The only changes to `run_set` are the doc-comment (`set` → `meta set`)
  and the usage-error string; the handler body, `TagSet` construction, and container-lane call are
  untouched. `Commands::Set` deleted, `MetaCommand::Set` added with identical arg shape, dispatch
  relocated verbatim. No logic change. ✅

**Pre-existing quirks surfaced (NOT SPEC-089 defects — the old binary emits the identical bytes; noted
only so they aren't mistaken for regressions later):** on a JPEG whose EXIF was written by exiftool,
`set`/`meta set` round-trips **Orientation 6 → "Unknown (1536)"** (1536 = 6 << 8 — smells like a
byte-order bug in the TIFF-IFD writer) and **loses GPS precision** (48°51'30.24" → 48°51'9.76"). Both
reproduce identically on the pre-move binary, so they predate this spec and are out of its scope — but
`set` mangling Orientation is a real bug worth its own spec, and it is exactly the kind of thing the
byte-identity framing hides (identical to the old bytes ≠ correct bytes).

**Carry for ship (not a build defect):** the STAGE-030 stage doc still marks SPEC-089 `[~]` with
"Framed … build-ready", and the Design Notes say "**Framed as SPEC-089**" rather than the spec's
Outputs-requested "RESOLVED → shipped". Git history shows stage bookkeeping is ship-cycle work
(`ship(SPEC-088): bookkeeping — cycle→ship, cost, timeline, archive; STAGE-030 5/6`), so this is
correctly deferred, not missed — flagged only so ship doesn't drop it (STAGE-030 → 6/6).

### Build-quality assessment (the Sonnet/Opus model experiment)

The referee ask: SPEC-089's build ran on **Sonnet**; its mirror SPEC-087 ran on **Opus** (verified CLEAN
first pass). Honest read — **the difference is real but narrow, and it is not where I'd have guessed.**

**Indistinguishable from SPEC-087's build:** the engine. The clap move is exactly right, the arg shape
mirrors `meta copy` as directed, dispatch is verbatim, and the deliberate divergence (the usage string)
was applied precisely as framed and no further — a real discipline test, since "while I'm here" scope
creep on the message would have been easy. The four design-named tests exist, are named as designed, and
each genuinely fails against the pre-implementation tree. Gate numbers reported (734/747) are **exactly**
what I measured — no rounding, no aspiration. Test quality is *better* than it had to be: `tests/cli.rs`
correctly removes `set` from the top-level help-parse list rather than leaving a passing-but-vacuous
assertion, and `meta_set_requires_a_tag` asserts the **message**, not just exit 2 — the failure mode
[[test-the-guard-where-the-criterion-applies]] warns about. Nothing was faked, hedged, or padded.

**Stronger than I expected — self-awareness about its own proof's limits.** This was the predicted
Sonnet weak spot and it is instead the build's best quality. It did not claim the byte-identity criterion
was fully met; it stated plainly that it proved identity against `metadata::set_tags` (the library fn),
named the stronger old-binary claim it had *not* attempted, and wrote it into Follow-ups as verify's job
with a correct rationale (that proof needs a second binary and a worktree). Reflection #2 reasons about
*why* that split is right. That is exactly the honesty the stage's scar tissue was built from, and my
old-binary run vindicated the underlying claim on all five paths — the engine really was sound.

**Weaker than SPEC-087's build — the grep-clean, and only the grep-clean.** SPEC-087's Opus build
grep-cleaned fifteen files and caught the non-obvious case that made its verify clean: the **lint fix
fragments**, a rendering of the surface nobody would find by grepping docs. SPEC-089's build cleaned six
files and missed a plainly-grepable heading **in a file it had open and edited**. The tell is the
Follow-up's shape: *"None on documentation: README/cli-reference/…/api-contract grep-cleaned"* — a
**file-level** claim ("I touched api-contract") standing in for a **line-level** check ("no `set` heading
survives in it"). One `grep -n '\bset\b' docs/api-contract.md` — the command I ran — would have shown two
hits where one was expected. Both builds' reflections say the same sentence ("a pure move still touches
N files; the grep-clean is the real work") — SPEC-087 *acted* on it, SPEC-089 *observed* it.

**The honest bottom line:** on the hard parts — correct move, tests that bite, refusing to overclaim a
proof it hadn't run — the Sonnet build is indistinguishable from Opus, and its self-awareness is a
genuine strength. It lost on **thoroughness of mechanical sweep**: fewer files searched, and a
file-level claim substituted for a line-level check. That is a cheap, checkable failure — a
`grep`-the-file-you-edited step in the build prompt likely closes the whole gap. n=1, one spec, one
pairing; I would not generalize past "Sonnet did the reasoning well and the sweeping less completely
here." Notably the miss is the **same species** as SPEC-088's defects (a claim standing in for a check)
which happened on Opus — so this is more plausibly a *process* gap the model choice widened than a
Sonnet-specific defect.

---

## Fix pass (2026-07-16) — both defects cleared

Docs-only fix cycle closing verify's 2 flagged defects. Scope held to documentation; the pre-existing
Orientation/GPS corruption verify surfaced (now framed as SPEC-093 on main) was explicitly out of scope
and left untouched.

### What changed

1. **`docs/api-contract.md:333`** — heading rewritten from top-level `` #### `set <INPUT...> …` *(SPEC-027)* ``
   to `` #### `meta set <INPUT...> …` *(SPEC-027; grouped under `meta` in SPEC-089)* ``, matching the
   annotation style of its `meta strip`/`meta clean`/`meta copy` siblings. The section body already read
   correctly (SPEC-087 had updated the cross-references); only the heading was stale.
2. **`docs/feature-exploration.md:87`** — finished the half-updated line: `` `set --artist/…` `` →
   `` `meta set --artist/…` ``, matching its two already-migrated siblings on the same line
   (`meta strip`, `meta clean --gps`).
3. **Discipline sweep, not a file-level claim.** Per verify's note that "api-contract grep-cleaned" was a
   file-level claim standing in for a line-level check, ran `grep -rn '\bset\b'` across the live doc
   surface (`docs/`, `README.md`, `AGENTS.md`, `guidance/`) rather than reasoning about which files the
   build had touched, and read every hit. Found **5 more stale bare-`set` references** verify's targeted
   pass hadn't surfaced, all fixed to `meta set` / grouped form:
   - `docs/architecture.md:12` — prose feature list (`EXIF strip/clean/set` → `` EXIF `meta` strip/clean/set ``)
   - `docs/architecture.md:121` — Mermaid diagram edge label (`meta strip/clean/copy · set` →
     `meta strip/clean/copy/set`)
   - `docs/recipes.md:161` — command inventory (`meta(strip/clean/copy)/set` → `meta(strip/clean/copy/set)`)
   - `docs/moat.md:39` — Mermaid diagram node label (`meta clean --gps · meta strip · set · meta copy` →
     `… · meta set · …`; note `moat.md:93` elsewhere in the same file was already correct — another
     half-updated-file case caught only by reading every hit, not just the two named ones)
   - `guidance/constraints.yaml:40` — the `metadata-not-via-pixel-encode` rule text
     (`meta strip/meta clean/set/meta copy` → `meta strip/meta clean/meta set/meta copy`)
   Also fixed `AGENTS.md:16`'s equivalent prose list for consistency with `architecture.md`, though
   verify's discipline note scoped to `docs/`+`README.md`. All remaining `\bset\b` hits (checked
   individually) are either correct `meta set` references or the generic English word ("responsive
   image set", "a large set", "quality was set") — no more bare-verb references. Dated historical
   records (`docs/sessions/`, `docs/research/`, `docs/reviews/`, `docs/blog/`, `docs/framework-feedback/`,
   `decisions/`, `specs/done/`, `CHANGELOG.md`) deliberately left untouched, per the SPEC-087/089
   grep-clean precedent.

### Proof (driven, not asserted)

- No `src/` changes — docs-only, so the build/verify byte-identity proofs are unaffected.
- Gates re-run: `cargo test` **734** passed (default), **747** passed (`--features avif`) — identical
  counts to build and verify, confirming no test regression from the docs edits. `cargo clippy -- -D
  warnings` clean. `cargo fmt --check` clean. `cargo build --no-default-features` builds. `just validate`
  — 207 front-matter blocks parse.

### Honestly unmet / narrowed

- None. Both flagged defects are closed, and the broader discipline sweep the verify note asked for
  found and fixed additional stale references beyond the two named ones.

### Fix-phase reflection
1. **The verify note's instruction — grep the live surface, don't reason about which files you touched —
   is what surfaced the extra 5 hits.** A narrower fix that patched exactly the two cited lines would have
   left `architecture.md`, `recipes.md`, `moat.md`, and `constraints.yaml` stale, reproducing the exact
   failure mode verify flagged (a file-level "I edited this" claim standing in for a line-level check) one
   level up — this time across *files*, not lines within one file.
2. **`docs/moat.md` was itself a half-updated file**, same species as `feature-exploration.md`: line 93
   already said `meta set` while line 39 (a Mermaid diagram label, not prose) still said bare `set`.
   Diagram labels are easy to miss in a text-only skim of a doc — worth remembering that a file "looking"
   grep-clean based on one matching line doesn't mean every line is.
3. **The out-of-scope boundary held.** The EXIF Orientation/GPS corruption verify surfaced (pre-existing,
   now SPEC-093) was not touched, investigated, or mentioned in any edited file beyond what was already
   there — confirmed by diffing the fix commit against only the 7 doc files listed above.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
