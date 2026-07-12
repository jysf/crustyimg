---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-071
  type: chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # four small, independent, localized point-fixes (lint / cache / cli / cli) each with a test + a docs sync (api-contract limits + cli-reference --watch) — no single item is hard; the batch runs as ONE build/verify/ship cycle, proportionate to the size of the tail

project:
  id: PROJ-007
  stage: STAGE-024
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-11

references:
  decisions: [DEC-034, DEC-050, DEC-052, DEC-058, DEC-060, DEC-025]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-062, SPEC-064, SPEC-067, SPEC-068, SPEC-070]

value_link: "STAGE-024's small-fix tail, batched — the point-fixes the LEAD review + the fuzz/peak-memory specs surfaced, done in one proportionate cycle rather than four spec lifecycles."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-11
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        A deliberate BATCH: the STAGE-024 backlog's high-value items (threat model, fuzz gate,
        peak-memory) each got a full adversarial spec; the remaining small correctness/UX tail is
        disproportionate to that cadence, so four localized point-fixes ride one cycle. Each fix
        grounded firsthand: lint `TruncatedOrCorrupt` (`src/lint/mod.rs:576`), cache off-by-53
        (`store_bounded:361` vs `read_entry:393/402` + the documenting test at `:855`), the exit-code
        value-assertion gap (`code()` is already compiler-exhaustive — test-completeness only), and
        the `--watch` global-flag no-op on non-build subcommands (`src/cli/mod.rs:717/720`).
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 190000
      estimated_usd: 3.10
      duration_minutes: 45
      recorded_at: 2026-07-11
      notes: >
        Main-loop build (not separately metered) → order-of-magnitude ESTIMATE, not a measured
        total. Five items landed in one cycle as framed. The batch's cost was dominated by ONE
        item: fix 1's spec-predicted `LimitsExceeded` arm turned out to be insufficient on the
        real binary — `lint` decoded by BYTES while holding a path, so RAW never reached its
        extension-routed decoder (SPEC-061/DEC-055) and every valid `.nef` linted as "truncated
        or corrupt". Root-caused and fixed via `Image::decode_path` (one line, an existing
        documented contract). The other four items were mechanical and cheap. The "drive the real
        binary" instruction paid for itself outright: the type-level unit test was green while
        the shipped behavior was still wrong.
  totals:
    tokens_total: 190000
    estimated_usd: 3.10
    session_count: 1
---

# SPEC-071: STAGE-024 hardening cleanup (batch)

## Context

STAGE-024's three high-value items — the threat-model review (SPEC-068), the decoder fuzz gate
(SPEC-069), and the peak-memory cap (SPEC-070) — each got a full adversarial spec and each caught a
real defect. What remains on the stage backlog is a **long tail of small, self-contained
correctness/UX point-fixes**, most surfaced *by* those specs' verify passes. Running each through
its own frame/build/verify/ship lifecycle (~4 sessions, ~$3–5 apiece) is disproportionate to a
three-line fix. This spec **batches four such fixes into one cycle**. Each is independent, localized,
and testable; none touches a public contract or schema. The genuinely larger tail items (pre-decode
format-sniff, `out`-dir canonicalize-containment, the unusual-filename `.to_str()` sweep, cache-key
build-profile, `--max-pixels`) are deliberately **left as their own specs** — this batch is only the
small stuff.

## Goal

Land these four fixes in one build/verify/ship cycle, each with a regression test:

1. **`lint` `LimitsExceeded` false-diagnosis** — a valid-but-too-large image gets called "truncated
   or corrupt … re-export a valid image"; make `lint` stop false-diagnosing it (mirror the existing
   `CodecNotBuilt` special-case). *(From SPEC-070 due-diligence; the
   [IMAGE_EXTENSIONS-exposes-every-decode-caller] lesson, 4th time.)*
2. **Cache `CACHE_ENTRY_MAX_BYTES` off-by-53** — `store_bounded` bounds the payload while `read_entry`
   bounds the whole frame (payload + 53-byte header + ext), so a near-cap payload is stored but
   permanently unreadable (silent miss). Bound the frame consistently. *(Confirmed in SPEC-068.)*
3. **Exit-code `is_total` value-assertion completeness** — `code()` is already compiler-exhaustive
   (no wildcard), so this is test-completeness, not a bug: assert the values for the
   `Metadata::Container` / `Metadata::Exif` (and confirm `Watch::Watch`) arms so no future *value*
   drift ships green. *(From SPEC-068's exit-code resize.)*
4. **`--watch` is build-only** — `--watch` is a global clap flag but only `Commands::Build` honors it;
   on any other subcommand it's a silent no-op. Reject `--watch` outside `build` with a usage error
   (exit 2). *(From SPEC-067 verify.)*
5. **Docs sync** — bring the user-facing docs in line with the wave's shipped behavior: (a)
   `docs/api-contract.md`'s decode-limits section is missing SPEC-070's **64 Mpix pixel cap**
   (DEC-063) and still calls `--max-pixels` merely "a planned follow-up"; (b) `docs/cli-reference.md`
   documents `build` but **not `--watch`** at all (SPEC-067 debt) — document it and its build-only
   restriction (which fix 4 now enforces). *(Raised pre-ship: docs should ship with the behavior they
   describe.)*

No new dependency; no new decision expected (each fix is mechanical, and item 4's reject-vs-document
choice is decided here: **reject**).

## Inputs

- **Fix 1 — lint:** `src/lint/mod.rs` — the `TruncatedOrCorrupt` rule's `check` (`:565-585`): `Ok(_)
  => None`, `Err(ImageError::CodecNotBuilt { .. }) => None` (SPEC-062, with the comment explaining
  "valid; we just cannot read it"), `Err(_) =>` the "truncated or corrupt … re-export" finding
  (`:577-583`). `LimitsExceeded` falls into that catch-all. `ImageError::LimitsExceeded` is in
  `src/error.rs`.
- **Fix 2 — cache:** `src/build/cache.rs` — `CACHE_ENTRY_MAX_BYTES` (`:86`), `store_bounded` bound
  `if bytes.len() > max` (`:361`), `read_entry` bounds `meta.len() > max` (`:393`) + `take(max+1)` +
  `buf.len() > max` (`:402-403`); the frame header = `MAGIC + 1 + 8 + 32 = 53` bytes + `ext`; the
  **documenting test** at `:855-868` (`HEADER = 53`) that currently pins the *asymmetry*.
- **Fix 3 — exit codes:** `src/cli/mod.rs` — `CliError::code()` (`:632`, compiler-exhaustive; the arms
  `Watch(_) => 1` `:665`, `Metadata::Container => 1` `:686`, `Metadata::Exif => 1` `:687`); the
  `exit_code_mapping_is_total` test (`:4738`) — check which variants it already asserts (`Watch` at
  `:4857`, `Metadata::UnsupportedFormat` at `:4962`) and add the missing `Container`/`Exif`.
- **Fix 4 — --watch:** `src/cli/mod.rs` — the dispatch `match &cli.command` (`:717`); only
  `Commands::Build { file }` (`:719`) reads `cli.global.watch` (`:720`); `GlobalArgs.watch` (`:130`,
  doc `:124` already says "`build` only"). `CliError::Usage` (exit 2) is the existing usage-error
  variant.
- **Test homes:** `src/lint/mod.rs` `#[cfg(test)]`; `src/build/cache.rs` tests (alongside `:855`);
  `src/cli/mod.rs` `exit_code_mapping_is_total` + a `tests/`/`#[cfg(test)]` driver for `--watch` on a
  non-build command.

## Outputs

- **Files modified:**
  - `src/lint/mod.rs` — add `Err(ImageError::LimitsExceeded(_)) => None` (mirror the `CodecNotBuilt`
    arm; a too-large image is valid, just outside the decode budget — "truncated or corrupt /
    re-export" is a false diagnosis with a useless remedy). Keep the `Err(_)` catch-all for genuine
    truncation/corruption.
  - `src/build/cache.rs` — in `store_bounded`, reject when the **frame** (`53 + ext.len() +
    bytes.len()`) exceeds `max`, so anything stored is readable by `read_entry`. Update the `:855`
    documenting test to assert the *consistent* behavior (a top-of-band payload is now refused at
    store, matching read) instead of pinning the asymmetry.
  - `src/cli/mod.rs` — (fix 3) add the missing value assertions to `exit_code_mapping_is_total`;
    (fix 4) guard the dispatch: if `cli.global.watch` and the command is not `Build`, return
    `CliError::Usage("--watch is only valid with `build`")` (exit 2) before running the command.
  - `docs/api-contract.md` — (fix 5a) update the "Decode resource limits" section (`:67-73`): add the
    64 Mpix total-pixel cap (SPEC-070/DEC-063) to the limit list, and revise the `--max-pixels`
    sentence from "a planned follow-up" to reference the concrete DEC-063 cap + the medium-format
    revisit trigger (still opt-in / filed, but no longer vaguely "planned").
  - `docs/cli-reference.md` — (fix 5b) add a `--watch` note to the `build [FILE]` section (`:251`):
    what it does (debounced rebuild loop) and that it is **build-only** (a usage error, exit 2,
    elsewhere — matching fix 4).
- **Files created:** none (tests live in existing modules).
- **New exports / decisions:** none expected.

## Acceptance Criteria

- [ ] **Fix 1:** `lint` on a valid-but-over-cap image (a >64 Mpix input, e.g. the SPEC-070 pixel-bomb
  fixture) produces **no** `size/truncated-or-corrupt` finding — driven on the real binary, not just a
  unit assertion. A genuinely truncated/corrupt image still triggers the rule (the catch-all is
  intact).
- [ ] **Fix 2:** a near-cap cache payload (in the top `53 + ext` band below `CACHE_ENTRY_MAX_BYTES`)
  is handled **consistently** — either it round-trips, or `store` refuses it up front; it is never
  stored-but-unreadable. The `:855` test asserts the fixed behavior and would fail against the old
  asymmetry.
- [ ] **Fix 3:** `exit_code_mapping_is_total` asserts a value for **every** `CliError` arm that maps a
  concrete code, including `Metadata::Container`, `Metadata::Exif`, and `Watch::Watch`; `code()` stays
  compiler-exhaustive.
- [ ] **Fix 4:** `crustyimg info --watch <file>` (and any non-`build` subcommand with `--watch`) exits
  **2** with a clear "`--watch` is only valid with `build`" message — driven on the real binary;
  `build --watch` still works.
- [ ] **Fix 5:** `docs/api-contract.md`'s decode-limits section names the 64 Mpix pixel cap
  (SPEC-070/DEC-063) and no longer calls `--max-pixels` merely "planned"; `docs/cli-reference.md`'s
  `build` section documents `--watch` + its build-only restriction. Docs match the shipped/this-PR
  behavior (no stale "fixed in v1" / "planned" claims about the limits SPEC-070 changed).
- [ ] Full gate matrix green: `cargo test` (default) + `cargo build --no-default-features` +
  `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` + `just deny` (unchanged) +
  `just validate`. No new dependency (`git diff main -- Cargo.toml Cargo.lock deny.toml` empty). No
  `unwrap` on recoverable paths.

## Failing Tests

Written before each fix; the fix makes them pass. Drive the real binary where the value is a
user-visible string/exit code (the wave's recurring lesson).

- **Fix 1 (`src/lint/mod.rs` + integration):** `"lint_does_not_call_a_too_large_image_corrupt"` — a
  `LintTarget` whose decode returns `LimitsExceeded` → the `TruncatedOrCorrupt` rule returns `None`;
  and drive `crustyimg lint <over-cap fixture>` → no `truncated-or-corrupt` error line. Plus keep a
  test that a truly corrupt image still fires the rule.
- **Fix 2 (`src/build/cache.rs`):** `"near_cap_payload_is_consistent_store_and_read"` — a payload sized
  in `[max-52-ext .. max]`: after the fix, `store` + `read_entry` agree (round-trips, or store refuses);
  fails against the current asymmetry (stored, then `read_entry` returns `None`). Rework the existing
  `:855` documenting test into this.
- **Fix 3 (`src/cli/mod.rs`):** extend `exit_code_mapping_is_total` with `assert_eq!` for
  `Metadata::Container`, `Metadata::Exif` (→ 1), confirm `Watch::Watch` (→ 1). (Pure test addition.)
- **Fix 4 (`tests/` or `#[cfg(test)]`):** `"watch_on_non_build_subcommand_is_usage_error"` — drive
  `info --watch` (and one more, e.g. `convert --watch`) → exit 2 + the build-only message; a control
  `build --watch` path is unaffected.

## Implementation Context

*Read this and re-confirm anchors. Each fix is independent — implement + test them one at a time.*

- **Fix 1 (lint):** the rule already documents the exact principle for `CodecNotBuilt` ("it is valid;
  we just cannot read it… reporting truncated/corrupt would be a false diagnosis with a destructive
  remedy"). `LimitsExceeded` is the same shape — a valid image outside our decode budget. Add the arm;
  do NOT invent a new lint rule (a dedicated "too-large" lint is out of scope — note it as a possible
  follow-up if you like). Keep the `Err(_)` catch-all for real truncation/corruption.
- **Fix 2 (cache):** the header is `ENTRY_MAGIC.len() + 1 (ext_len) + 8 (payload_len) + 32 (hash) =
  53`. `store_bounded` currently checks `bytes.len() > max`; change to reject when `53 + ext.len() +
  bytes.len() > max` (the frame), matching what `read_entry` will accept. This keeps the invariant
  "anything `store` writes, `read_entry` can read." A near-cap payload then fails at store (a warning,
  as store failures already are) rather than becoming a permanent silent miss. Update the `:855` test.
- **Fix 3 (exit codes):** `code()` at `:632` has no `_ =>` wildcard, so totality is compiler-enforced
  — this is only closing the *test's* value-assertion gaps so a wrong *code* can't ship green (the
  `is_total` test shipped incomplete twice before). Add the missing `assert_eq!`s; no production change.
- **Fix 4 (--watch):** guard in `run` (the `match &cli.command` at `:717`) — before dispatching, if
  `cli.global.watch && !matches!(cli.command, Commands::Build { .. })` return
  `Err(CliError::Usage("--watch is only valid with `build`".into()))`. This is the same shape as the
  SPEC-067 `--watch`×verify-mode usage guard. Update `GlobalArgs.watch`'s doc if needed (`:124` already
  says build-only). `build --watch` unchanged.
- **Fix 5 (docs):** `docs/api-contract.md:67-73` currently lists only DEC-034's per-dim ≤ 65 535 +
  alloc ≤ 512 MiB and says "Limits are fixed in v1; a `--max-pixels`/env override … is a planned
  follow-up." Add the DEC-063 total-pixel cap (**64 Mpix**, the peak-decode-memory bound) as a third
  limit, and revise the last sentence: the cap is deliberate (not "fixed in v1" hand-waving), the
  tradeoff (rejects > 64 MP medium-format/panoramas) is stated in DEC-063, and `--max-pixels` is a
  filed opt-in with a live revisit trigger (not vaguely "planned"). Keep it factual and short — link
  DEC-063. `docs/cli-reference.md`'s `build [FILE]` section (`:251-267`) has no `--watch`: add a short
  paragraph (debounced rebuild loop over the declared build; build-only — a usage error elsewhere per
  fix 4; `Ctrl-C` to stop). Match the doc's existing terse style; don't over-document.
- **Constraints:** `untrusted-input-hardening` (fixes 1–2 are on untrusted-input paths),
  `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `clippy-fmt-clean`, `ergonomic-defaults`
  (fix 4 turns a silent no-op into a clear error).

### Out of scope (their own specs / deferred)
- Pre-decode format-sniff (closes SPEC-065/066 residuals); `out`-dir canonicalize-containment (the
  symlink write-escape residual); the unusual-filename `.to_str()` sweep; cache-key build-profile
  completeness; `--max-pixels` opt-in; the full-pipeline peak envelope. A dedicated "too-large" lint
  rule. F-AVIF-3 (upstream). These are larger or decision-bearing — not batched here.

## Notes for the Implementer

- Implement + test each fix independently; they don't interact. A reviewer should be able to read
  four small, self-contained diffs.
- Drive the real binary for fix 1 (`lint` on an over-cap image) and fix 4 (`info --watch`) — the value
  is the string/exit code a user sees, which unit tests on types don't exercise (the wave's lesson).
- No new DEC expected; if a fix turns out to need a real decision, emit one and say so.
- Keep `just deny` untouched and the repo-root dependency diff empty.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-071-hardening-cleanup`
- **PR (if applicable):** #74
- **All acceptance criteria met?** yes
- **New decisions emitted:** none (no fix needed one; the one deviation below is a bug fix
  *against* an existing decision's documented contract, not a new choice)
- **Per-fix status:**
  - **fix 1 (lint `LimitsExceeded`) — done, and it uncovered a bigger defect.** Added the
    `Err(ImageError::LimitsExceeded(_)) => None` arm mirroring `CodecNotBuilt` (`src/lint/mod.rs`).
    But driving the real binary (as the spec demanded) showed `lint <pixel_bomb.nef>` **still** said
    "truncated or corrupt": `LintTarget::build` decoded with `Image::from_bytes` (content-sniffed)
    though it had the path, so RAW — which is *extension*-routed (SPEC-061/DEC-055) — never reached
    its decoder and failed with a generic `Decode` error, not `LimitsExceeded`. The arm alone could
    not satisfy the acceptance criterion. Root cause fixed by routing `LintTarget` through
    `Image::decode_path` — the seam whose own doc says *"every command that decodes a `Path` MUST
    route through here"*. **This also fixes a strictly worse latent bug: `lint` called every
    perfectly valid `.nef`/`.cr2` "truncated or corrupt (failed to decode); re-export a valid image"
    and failed CI with exit 7**, while `info` on the same file read it fine. (This is the
    previously-filed "lint-on-RAW" follow-up from SPEC-061, and the 5th instance of the
    IMAGE_EXTENSIONS-exposes-every-decode-caller lesson.)
  - **fix 2 (cache off-by-53) — done.** `store_bounded` now bounds the **frame**
    (`ENTRY_HEADER_BYTES + ext + payload`), not the payload, so the invariant "anything `store`
    writes, `read_entry` can read" holds. Introduced `ENTRY_HEADER_BYTES` (= 53) as the single
    source of truth, used by `write_entry`, `store_bounded`, and the test (it was an open-coded
    `+ 41` in `write_entry` and a `const HEADER` in the test). The `:855` documenting test became
    `near_cap_payload_is_consistent_store_and_read`, which **sweeps the whole boundary band** rather
    than pinning one point, and asserts store/read agree at every size.
  - **fix 3 (exit-code asserts) — done.** Added `assert_eq!`s for `Metadata::Container` and
    `Metadata::Exif` (both → 1) to `exit_code_mapping_is_total`. `Watch(_)` → 1 was already asserted
    (`:4856`), confirmed. No production change; `code()` remains compiler-exhaustive.
  - **fix 4 (`--watch` build-only) — done.** Guard at the top of `dispatch`: `cli.global.watch &&
    !matches!(cli.command, Commands::Build { .. })` → `CliError::Usage` (exit 2). Same shape as
    SPEC-067's `--watch` × verify-mode guard. `build --watch` unchanged.
  - **fix 5 (docs sync) — done.** `docs/api-contract.md`: decode limits are now a 3-item list
    including the 64 Mpix total-pixel cap (DEC-063, linked), checked pre-allocation; the "fixed in
    v1 / planned follow-up" hand-wave is replaced by DEC-063's actual stated tradeoff (>64 MP
    medium-format/panoramas are rejected; consumer/prosumer photography fits) and the honest status
    of `--max-pixels` (filed, deliberately not built, revisit trigger = a real >64 MP user).
    `docs/cli-reference.md`: a `--watch` subsection under `build` (debounced loop, self-trigger-proof,
    cache-incremental, survives a failing cycle, Ctrl-C, does not rewrite the lockfile, incompatible
    with `--check`/`--frozen`/`--locked`) ending on **build-only → exit 2 elsewhere**, matching fix 4.
- **Deviations from spec:**
  - **One extra production change, forced by fix 1's own acceptance criterion** (see fix 1 above):
    `LintTarget::build` now decodes via `Image::decode_path` instead of `Image::from_bytes`. The spec
    predicted the `LimitsExceeded` arm would be sufficient; on the real binary it was not, because
    `lint` was bypassing the extension-routing seam entirely. The change is one line against an
    existing, documented contract — no new decision. It is *why* the spec's "drive the real binary"
    instruction earned its keep: the unit test on `LintTarget::from_bytes` passed while the shipped
    behavior was still wrong.
  - Fix 4's message is `--watch is only valid with \`build\`: there is no rebuild loop to run for
    this subcommand` — the spec's string plus a clause saying why.
  - Fix 2's test is a boundary **sweep**, not the single-point test the spec sketched; it fails
    against the old asymmetry at multiple sizes, so it is strictly stronger.
- **Follow-up work identified:**
  - **A `lint` decode-seam audit.** Fix 1's root cause was `lint` constructing its decode outside the
    shared path seam. Worth a sweep for *other* callers that decode bytes while holding a path
    (`Image::from_bytes` call sites), plus a per-format `lint` smoke test mirroring the
    `info`-per-format test SPEC-061 added — the same class of bug will otherwise recur a 6th time.
  - **A `size/over-decode-budget` lint rule (Info/Warn).** `TruncatedOrCorrupt` now stays silent on
    both `CodecNotBuilt` and `LimitsExceeded`, which is correct but means `lint` says *nothing* about
    a file it could not inspect. The fully honest answer is a `meta/not-inspected`-style finding whose
    severity is not fixed at Error (noted in the code as a follow-up, as the SPEC-062 comment already
    did for `CodecNotBuilt`). Deliberately not invented here — the spec said not to.
  - Cache-slot accounting: an output whose *frame* exceeds the cap is now correctly refused, but
    silently. A `-v` note ("output too large to cache; rebuilding each run") would make the
    always-rebuild visible instead of mysterious.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing was unclear; one thing was *wrong*, and usefully so. Fix 1's Implementation Context
   asserted that `LimitsExceeded` "falls into that catch-all" in the `TruncatedOrCorrupt` rule — true
   for a content-sniffed image, but not for the `.nef` fixture the acceptance criterion names, which
   never reaches the RAW decoder from `lint` at all. The spec's anchors were all accurate; the causal
   claim behind them was not. It cost ~15 minutes and was caught by the spec's own "drive the real
   binary" instruction, which is exactly what that instruction is for.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-055/SPEC-061 (RAW extension-routing) belonged in fix 1's referenced decisions. The spec
   referenced DEC-052 (the `CodecNotBuilt` twin it was mirroring) but not the decision that governs
   *how `lint` gets its decoded image in the first place* — which is where the actual bug lived. The
   `affected_scope` of DEC-055 does not list `src/lint/mod.rs`, so `just decisions-audit --changed`
   would not have surfaced it either; worth adding.

3. **If you did this task again, what would you do differently?**
   — Drive the real binary **before** writing the unit test, not after. I wrote a passing unit test
   against `LintTarget::from_bytes` and only then discovered the shipped path did something else
   entirely — the unit test was green and meaningless for ~20 minutes. For any fix whose acceptance
   criterion is stated in terms of a real invocation, the first thing to run is that invocation: it
   tells you where the code actually goes, and the unit test can then be written at the right seam.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
