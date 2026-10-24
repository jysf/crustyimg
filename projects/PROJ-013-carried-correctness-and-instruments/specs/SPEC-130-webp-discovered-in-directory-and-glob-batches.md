---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-130
  type: bug
  cycle: verify
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-013
  stage: STAGE-052
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5
  created_at: 2026-10-23

references:
  decisions:
    - DEC-002   # image-ness by extension, never by decoding (Source must not decode)
    - DEC-021   # WebP decode is a default-feature, pure-Rust input
    - DEC-093   # animated input warns and proceeds; `lint --max-warnings 0` is the strict gate
  constraints:
    - single-image-library
    - test-before-implementation
    - every-public-fn-tested
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs:
    - SPEC-004   # source resolution (where IMAGE_EXTENSIONS was born)
    - SPEC-019   # WebP decode in the default build
    - SPEC-058   # the `.avif` precedent: a default-decodable input added to the allow-list
    - SPEC-119   # found this gap; its lint tests work around it by naming `.webp` files directly

value_link: >
  Batch verbs stop silently returning less than they were given: a directory or glob of
  `.webp` files is processed like any other image, and `lint --max-warnings 0` on a
  directory stops being a false green for animated WebP — the gate DEC-093 rests on.

cost:
  sessions:
    - cycle: design
      date: 2026-10-23
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: "main-loop, not separately metered; design+build ran in one session (see Build Completion)"
    - cycle: build
      date: 2026-10-23
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: "same interactive session as design; ship must fill this from the session transcript (dedupe by .message.id) before cost-audit"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-130: WebP discovered in directory and glob batches

## Context

`IMAGE_EXTENSIONS` in `src/source/mod.rs` is the allow-list that directory listings and glob
matches filter through. It names 30+ extensions — the raster formats, `avif`, `svg`, eleven RAW
families, `heic`/`heif` — and **not `webp`**, although WebP decode is a default-feature input
(SPEC-019, DEC-021) and WebP is a format crustyimg writes by default (`web`, `optimize`).

Reproduced on `main` at `3dd58dc` (2026-10-23) and on the released 0.7.1 binary:

```
crustyimg info in/a.webp                          → works (single file is not extension-filtered)
crustyimg resize in/ --max 8 --out-dir o/         → exit 3 "input not found or unreadable"
crustyimg resize 'in/*.webp' --max 8 --out-dir o/ → exit 3 "input not found or unreadable"
```

In a mixed directory the `.webp` files are skipped **silently** — the batch succeeds on
everything else.

Two items on STAGE-052's backlog carry this:

1. "`webp` is missing from `IMAGE_EXTENSIONS`, so directory and glob discovery silently skips
   `.webp` files" — the defect.
2. "PRIORITY: the `IMAGE_EXTENSIONS` gap silently defeats the strict gate that a maintainer
   decision rests on" — its consequence: `lint --max-warnings 0 <dir with anim.webp>` exits `0`,
   while the same file named directly exits `7`. DEC-093 accepted "animated input warns and
   proceeds" on the argument that `lint --max-warnings 0` is the strict path, and
   `docs/api-contract.md` carries a qualifier saying that argument fails in directory mode.

This spec closes both.

## Goal

Add `webp` to the source allow-list so directory and glob inputs discover `.webp` files (any
case), and prove it at the resolver, at a batch verb, and at `lint --max-warnings 0` in
directory mode.

## Inputs

- **Files to read:** `src/source/mod.rs` (the list and both filtered branches);
  `tests/source.rs` (resolver test style); `tests/lint.rs` (the two SPEC-119 tests whose doc
  comments describe the gap as current); `tests/common/mod.rs` (`animated_webp`,
  `webp_lossless`-style fixtures); `docs/api-contract.md` §animated input (the qualifier).
- **Related code paths:** every caller of `source::resolve` — `src/cli/ops.rs`,
  `src/cli/optimize.rs`, `src/cli/report.rs` (`info`, `lint`), `src/cli/build.rs`.

## Outputs

- **Files modified:**
  - `src/source/mod.rs` — `"webp"` added to `IMAGE_EXTENSIONS`; comment updated.
  - `tests/source.rs` — resolver tests for directory + glob WebP discovery.
  - `tests/cli.rs` — a batch-verb test over a WebP-only directory.
  - `tests/lint.rs` — a directory-mode strict-gate test; the two SPEC-119 doc comments that
    describe the gap as current are corrected.
  - `docs/api-contract.md` — the directory-mode qualifier is replaced with the fixed behaviour.
  - `projects/PROJ-013-…/stages/STAGE-052-….md` — both backlog items point at this spec.
- **New exports:** none.

## Acceptance Criteria

- [ ] **AC-1** `resolve(<dir>)` on a directory holding only `.webp` files returns every one of
      them, sorted; a non-image sibling (`.txt`) is still skipped.
- [ ] **AC-2** Discovery is case-insensitive: `B.WEBP` is discovered alongside `a.webp`.
- [ ] **AC-3** `resolve("<dir>/*.webp")` returns the WebP files (glob branch, not just the
      directory branch).
- [ ] **AC-4** A batch verb over a WebP-only directory succeeds end to end:
      `crustyimg resize <dir> --max 2 --out-dir <out>` exits `0` and writes one output per input.
      (Was exit `3`.)
- [ ] **AC-5** `lint --max-warnings 0 <dir>` exits `7` on a directory containing only an
      animated `.webp`, and `0` on a directory containing only a static `.webp` — the strict
      gate holds in directory mode.
- [ ] **AC-6** Every caller of `source::resolve` was audited for a new failure mode from
      admitting `.webp`; the result is recorded in Build Completion.
- [ ] **AC-7** No doc or test comment still describes the gap as current
      (`grep -rn "webp.*IMAGE_EXTENSIONS\|IMAGE_EXTENSIONS.*webp" src tests docs` reviewed).
- [ ] **AC-8** Negative control: with `"webp"` removed from the list again, AC-1, AC-3, AC-4
      and AC-5's animated half go RED; restoring it turns them green. The flip is the evidence.

## Failing Tests

- **`tests/source.rs`**
  - `directory_discovers_webp_case_insensitively` — asserts AC-1 + AC-2: a dir with `a.webp`,
    `B.WEBP`, `note.txt` resolves to exactly stems `["B", "a"]` (byte-order sort).
  - `glob_discovers_webp` — asserts AC-3: `<dir>/*.webp` over `x.webp`, `y.webp` resolves to
    stems `["x", "y"]`.
- **`tests/cli.rs`**
  - `resize_batch_over_webp_only_directory` — asserts AC-4: exit 0, and the out-dir holds
    exactly two files whose dimensions are bounded by `--max`.
- **`tests/lint.rs`**
  - `max_warnings_zero_fails_on_animated_webp_in_directory_mode` — asserts AC-5: `7` for the
    animated directory, `0` for the static one, and the animated run's stdout reports
    `format/animated-input` (so the `7` is the rule firing, not some other failure).

## Implementation Context

### Decisions that apply

- `DEC-002` — Source decides image-ness by extension, never by decoding. The fix stays inside
  that rule: one more extension, no sniffing.
- `DEC-021` — WebP decode is default-on and pure-Rust (`image`'s `webp` feature is in the base
  feature list for native and wasm builds, `Cargo.toml`), so admitting `.webp` cannot reach a
  "codec not built" path in any build, including `--no-default-features`.
- `DEC-093` — names `lint --max-warnings 0` as the strict gate. AC-5 is what makes that true in
  directory mode.

### Constraints that apply

- `single-image-library` — Source must not call the pixel decoder. Unchanged.
- `test-before-implementation` — the tests above fail on `main` (exit 3 / empty resolve).
- `one-spec-per-pr` — the unrelated `fill`/`cover` doc correction found alongside this goes in
  its own PR.

### Prior related work

- `SPEC-058` — added `avif` to the same list with a directory-discovery test
  (`tests/input_avif.rs::directory_source_discovers_avif`); this follows that precedent.
- `SPEC-119` — found the gap (recorded in DEC-093) and worked around it in `tests/lint.rs`.
  Those single-file tests stay: they still prove the rule fires on a named file. Only their
  comments change.

### Out of scope (for this spec specifically)

- Any other missing extension (`jxl`, `qoi`, `svgz`, …) — none is a decodable input today.
- Output-into-input-dir re-processing (`web dir/ --out-dir dir/` then re-running picks up its
  own outputs). That hazard already exists for `.jpg`/`.png` outputs; WebP joins it, nothing new.
- Sniffing content to decide image-ness.

## Notes for the Implementer

- The change is one string. The work is the audit (AC-6) and making the tests prove something:
  check that each test fails on `main` before trusting it.
- Use `tests/common`'s WebP builders for fixtures; do not commit binaries.

---

## Build Completion

- **Branch:** `feat/spec-130-webp-batch-discovery`
- **PR (if applicable):** opened against `main` (link in the timeline once merged)
- **All acceptance criteria met?** yes
- **New decisions emitted:** none — one more extension under DEC-002's existing rule.
- **Files this diff touches** (from `git diff --name-only main`):
  - `src/source/mod.rs` — `"webp"` added to `IMAGE_EXTENSIONS`, with a comment saying why.
  - `tests/source.rs` — `directory_discovers_webp_case_insensitively`, `glob_discovers_webp`.
  - `tests/cli.rs` — `resize_batch_over_webp_only_directory`.
  - `tests/lint.rs` — `max_warnings_zero_fails_on_animated_webp_in_directory_mode`; two SPEC-119
    doc comments that described the gap as current now point at the directory-mode test.
  - `CHANGELOG.md` — an `[Unreleased]` / Fixed entry, including the `lint` behaviour change.
  - `docs/api-contract.md` — the directory-mode qualifier on `lint --max-warnings 0` replaced
    with the fixed behaviour.
  - `projects/PROJ-013-…/stages/STAGE-052-….md` — both backlog items point at SPEC-130.
  - `projects/PROJ-013-…/specs/SPEC-130-…{,-timeline}.md` — this spec + timeline (new).
- **Negative control (AC-8):** the four tests were run on the unfixed tree first —
  `directory_discovers_webp_case_insensitively` (`got []`), `glob_discovers_webp`
  (`NotFound(".../*.webp")`), `resize_batch_over_webp_only_directory` (exit 3 "input not found"),
  `max_warnings_zero_fails_on_animated_webp_in_directory_mode` (rule never fired). Adding the one
  string turned all four green. Only one condition changed, so the flip is attributable to it.
  The lint test puts a static PNG beside the animated WebP so its pre-fix state is the real
  false green (exit 0), not an unrelated exit 3; its static half asserts `1 scanned` so a skipped
  file cannot pass as clean.
- **Caller audit (AC-6):** every caller of `source::resolve`, driven over a WebP-only directory
  on the fixed build: `resize`, `thumbnail`, `auto-orient` (WebP out), `convert --format png`,
  `optimize`, `web` ("kept webp … already optimal"), `info`, `lint`, `apply --recipe`, `build`
  (2 rebuilt, then 2 cached on re-run — the cache keys on the lowercased `webp` extension like any
  other). All exit 0. No caller has an `Err(_)` arm that WebP reaches differently from PNG:
  WebP decode is in `image`'s base feature list for native and wasm, so no build — including
  `--no-default-features` — hits "codec not built". Output re-encoding to WebP uses the
  pure-Rust lossless encoder, as it already did for a single named `.webp`.
  **User-visible change:** `lint <dir>` now scans `.webp` files, so a CI job linting a tree with
  WebP assets can newly report findings on them. That is the point of the fix; the CHANGELOG
  entry says so.
- **Deviations from spec / process:**
  - Design and build ran in **one** session, and the spec is on the build branch rather than
    committed to `main` first. This session was dispatched from another one to fix the bug and
    open a PR, and could not commit to `main` directly. Verify should run in a fresh session.
  - PROJ-013 and STAGE-052 are `proposed`, not `active`. The spec was homed there because that
    is where both backlog items live; activating the stage is the maintainer's call.
- **Follow-up work identified:**
  - DEC-093's "Found, not fixed" note is now historical; DECs are stable records, so it was
    left alone — the live claim in `docs/api-contract.md` was updated instead.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing in the spec; the build manifest's `out` vs `name` template cost two tries while
   driving `build` for the audit (`out` is a directory, the template is `name`).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-002 already says image-ness is by extension; this was an omission from the list.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The stage note to "audit each caller and its `Err(_)` arm" was the right
   instinct; driving each verb was faster and stronger than reading each arm.
