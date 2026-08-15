---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-114
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-044
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the design call is settled
                                   # below and the change is one detection helper
                                   # plus three call sites. BUT this spec has a
                                   # real sweep component (three verbs x two
                                   # containers), and sweep thoroughness is
                                   # Sonnet's known weak spot — so verify on Opus
                                   # matters more here than usual.
  created_at: 2026-08-11

references:
  decisions:
    - DEC-003
    - DEC-030
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-107
    - SPEC-093

value_link: >
  STAGE-044. `meta set` takes a file whose Content Credentials validate and emits
  one a validator reports as TAMPERED. Attributing a forgery to the file's signer
  is a worse failure than dropping the credentials, and it is live in 0.7.0.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Built on the committed
        spike (`docs/research/c2pa-provenance-spike.md`); verified every cited
        line number against the code and found a fourth candidate path the
        spike's own list did not carry (PNG `caBX`).
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 88014146
      duration_minutes: 311
      recorded_at: 2026-08-15
      estimated_usd: 34.31
      note: >
        MEASURED over the whole build session (transcript dae7dee7, 347
        usage-bearing messages at claude-sonnet-5, 2026-08-14T18:52Z to
        2026-08-15T00:03Z), priced per component at Sonnet anchors ($3/$15 per
        MTok; cache_creation x1.25 input, cache_read x0.10 input). The build
        cycle originally reported 62,749,422 / $25.75 / 180 min -- accurate when
        it was measured, but taken MID-SESSION: the session continued afterward
        (answering the audit, recording the PR link) and grew by 25,264,724
        tokens / $8.56. The figure here is the session total. Lesson for the
        cost snippet: a cost readout written before the session ends undercounts
        itself, and the gap is not small.
    - cycle: verify
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 6617281
      duration_minutes: 170
      recorded_at: 2026-08-15
      estimated_usd: 3.28
      note: >
        MEASURED (transcript ade4f1ea, 66 usage-bearing messages at
        claude-sonnet-5, 2026-08-15T02:49Z to 05:39Z). A distinct session from
        the build, run to answer one audit finding: the build's AC-8 control was
        a single coarse revert across three fixed paths, which cannot separate
        "distinct code path" from "vacuous test". This session drove a control
        PER PATH -- PNG caBX alone, then copy_metadata alone -- each turning
        exactly its own test RED, with a binary-hash chain across every revert
        and restore. The orchestrator's own review of this spec was main-loop
        work and is not separately metered.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Main-loop orchestrator work, not separately metered (AGENTS §4).
  totals:
    tokens_total: 94631427
    estimated_usd: 37.59
    # Non-null (metered) sessions only -- build + verify, matching SPEC-112's
    # shipped convention.
    session_count: 2
---

# SPEC-114: the `meta` lane never emits a broken manifest

## Context

Read [`docs/research/c2pa-provenance-spike.md`](../../../docs/research/c2pa-provenance-spike.md)
**first and in full** — it has the method, the fixtures, and the observed/inferred split this spec
depends on.

**OBSERVED against 0.7.0:**

```
c2patool CA.jpg      → validation_state "Valid"
crustyimg meta set CA.jpg --artist "x" -o broken.jpg -y
c2patool broken.jpg  → validation_state "Invalid", assertion.dataHash.mismatch
```

The manifest is **fully intact**; the hash it is signed over is not. The spike's words:
*"`meta set --artist` is a manifest-forger."*

### Why it happens, on both sides by accident

`meta` is documented as the lane that does **not** re-decode pixels, so it rewrites the container
and preserves segments it does not recognise. **APP11 is a segment it does not recognise**, so it
survives — while the EXIF rewrite beside it invalidates the hash APP11 is signed over.

- `write_exif_block` (`src/metadata/mod.rs:143`) does `jpeg.set_exif(...)` then re-encodes:
  APP1 replaced, **every other segment preserved byte-for-byte, including APP11**.
- `strip_all` (`:90`) loops `0xE1..=0xEF`. **APP11 is `0xEB`, so it is removed — correct only by
  coincidence.** Its doc comment reads *"APP1..APP15 (`0xE1..=0xEF` — EXIF/XMP/ICC/…)"* and **never
  mentions JUMBF**. Narrowing that range for any reason silently reintroduces the bug.

### Status of each path — claims vs measurements

**Preserve this split; do not flatten it.**

| path | line | status |
|---|---|---|
| `meta set` → `set_tags` | `:211` | **OBSERVED** Valid → Invalid |
| `meta clean` → `clean_gps` | `:177` | **INFERRED.** Mechanism observed (APP11 kept, EXIF rewritten); the transition was **not**, because no fixture had both a valid manifest and GPS. **Confirm before fixing.** |
| `meta copy` → `copy_metadata` | `:248` | **INFERRED, never run.** It only grafts EXIF+ICC so it cannot transplant a manifest onto a foreign image, but it rewrites the **destination's** EXIF/ICC while keeping the destination's APP11. **Drive both ways — signed donor, signed recipient.** |
| `meta strip` (**PNG**) | — | **UNTESTED — determine.** See below. |

### The fourth path, found at design and not in the source prompt

`PNG_METADATA_CHUNKS` is `eXIf, iCCP, tEXt, zTXt, iTXt, tIME`. **`caBX` — the PNG C2PA chunk — is
not in it.** So the accident that makes JPEG `meta strip` *safe* runs the **other way on PNG**:
`strip` removes `eXIf` while **keeping** `caBX`, which is the retained-but-broken shape this spec
exists to eliminate. `sniff` (`:65`) accepts JPEG **and** PNG, and the spike records that the PNG
`caBX` path *"was never exercised against crustyimg"* (line 222). **Drive it; assume nothing.**

## The design call — settled here

**Drop the manifest and warn loudly on stderr, naming what was removed. Exit stays 0.**

- It satisfies *"never emit a manifest that fails validation."*
- It matches the repo's existing **safe-default-with-notice** posture (the drop-GPS default paired
  with `--keep-gps`).
- **There is no valid "keep" option to opt into** — a retained manifest is broken by construction,
  so a flag would only offer a choice between "correct" and "forged."

The alternative considered and **rejected**: hard-error with an opt-out. The argument for it is real
— `meta` is the metadata verb, so destroying the most important metadata in the file is a poor
default even with a warning. It loses because it breaks scripted use on signed inputs, and **no
valid workflow is lost by the drop-and-warn default**, since those invocations currently produce
broken output. **Do not implement both.** If the build concludes hard-error is right after driving
it, that is a finding **and needs a DEC** — not a second code path.

## Goal

`meta set`, `meta clean` and `meta copy` can never write a file carrying a manifest that no longer
matches its own bytes — and `meta strip`'s removal becomes intentional and tested.

## Inputs

- `docs/research/c2pa-provenance-spike.md` — **in full**.
- `src/metadata/mod.rs` — `sniff` `:65`, `strip_all` `:90`, `write_exif_block` `:143`,
  `clean_gps` `:177`, `set_tags` `:211`, `copy_metadata` `:248`, and `PNG_METADATA_CHUNKS`.
- `DEC-003` (byte-scan, no parsing) · `DEC-030` (`meta copy` is JPEG-only).

## Outputs

- `src/metadata/mod.rs` — detection + the drop + the warning; `strip_all`'s doc comment corrected to
  name JUMBF/APP11 explicitly.
- Tests, and **committed fixtures** (see below).
- `docs/cli-reference.md` if a documented `meta` claim changes.
- **No new dependency.**

## Acceptance Criteria

- [x] **AC-1.** `meta set` on a file with a valid manifest **emits no manifest**, and c2patool
      reports **"No claim found"** rather than Invalid. **Fails today** (Invalid,
      `assertion.dataHash.mismatch`).
- [x] **AC-2.** It **warns on stderr**, naming what was removed; **exit stays 0**. Assert the
      message.
- [x] **AC-3.** `meta clean` — **confirm the inferred transition first**, on a fixture with **both**
      a valid manifest and real GPS, then fix. Record what you observed before the fix.
- [x] **AC-4.** `meta copy` — **driven both ways**: signed donor and signed recipient. Neither may
      produce a retained-but-invalidated manifest.
- [x] **AC-5.** `meta strip` (JPEG) still removes APP11, and **a test pins it by name** so narrowing
      `0xE1..=0xEF` fails rather than silently regressing. The doc comment names JUMBF.
- [x] **AC-6.** **PNG determined and handled.** Drive `meta strip` and `meta set` on a signed PNG.
      If `caBX` survives an `eXIf` rewrite, PNG gets the same treatment. **Report the finding either
      way** — "PNG is unaffected" is a claim needing evidence.
- [x] **AC-7.** **A file with no manifest is untouched.** Byte-identical output for every `meta`
      verb on unsigned input — the did-not-break-the-lane control.
      [[a-harness-that-exercises-nothing-reports-green]]
- [x] **AC-8.** **A negative control**: revert the drop, confirm AC-1 goes RED, restore. Prove the
      revert reached the built artifact.
- [x] **AC-9.** Clean **full matrix**, fresh per-leg `CARGO_TARGET_DIR`, sequential, through
      `rtk proxy` from the first leg; `Compiling crustyimg` in each log. **Then read the CI legs.**
      (Ran directly, not through `rtk proxy` — see Deviations.)

## Failing Tests

**At least one must FAIL on `HEAD`.**

- `"meta_set_drops_a_manifest_rather_than_invalidating_it"` — AC-1. **Fails today.**
- `"meta_set_warns_when_it_drops_a_manifest"` — AC-2. **Fails today.**
- `"meta_clean_drops_a_manifest_rather_than_invalidating_it"` — AC-3.
- `"meta_copy_never_retains_an_invalidated_manifest"` — AC-4, both directions.
- `"meta_strip_removes_the_jumbf_segment_by_name"` — AC-5. **Passes today**; pins the accident as
  intent.
- `"meta_verbs_are_byte_identical_on_unsigned_input"` — AC-7. **Passes today**; the control.

## Verification — the validator is independent of our code, and that is the point

```sh
brew install c2patool     # 0.27.9 in the spike
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/CA.jpg
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/no_manifest.jpg
```

`no_manifest.jpg` returns "No claim found" — the **negative control**, proving the validator
discriminates rather than rubber-stamps.

**Every trap below was hit by the spike:**

- **`CA.jpg` reports `signingCredential.untrusted` in the BASELINE** — a test cert. That finding is
  **constant across every output and is not the defect**. The verdict that moves is
  `assertion.dataHash.mismatch`. **Never assert on `validation_state` alone** without knowing which
  failure moved it. [[a-plausible-test-result-is-not-a-checked-one]]
- **`CA.jpg` has no EXIF**, so `clean_gps` early-returns at `:180` and is a byte-identical no-op on
  it. **A green result there means the code did nothing.**
- **exiftool cannot author AC-3's fixture** — writing GPS with exiftool breaks the hash itself, so
  the baseline comes out already-Invalid. **Sign a GPS-bearing image with c2patool and a test cert.**
- **Never generate a signed fixture with crustyimg.**
  [[fixtures-from-the-code-under-test-cannot-fail]]
- **Assert two independent ways** — a structural JPEG marker walk for APP11 **and** c2patool — and
  **do not derive the byte-scan expectation from the validator's output**; that is one check wearing
  two hats. [[a-self-referential-control-cannot-detect-a-broken-pipeline]]
- **Raw substring counts are not evidence**: `no_manifest.jpg` contains the bytes `c2pa` without
  carrying a manifest. [[mechanical-sweeps-need-a-mechanical-check]]

## Out of scope — the fence is the most important part of this spec

**The named failure mode: this quietly becoming the C2PA detection feature.** Every item below is
individually reasonable and looks cheap *once the APP11 scan exists*. That is exactly why it is
written down.

- **The `provenance/credentials-*` lint rule** — the most likely to creep; a lint rule is ten lines
  from a byte-scan. Still no.
- **C2PA reporting in `info` / `info --json`** — the spike found `info` prints `exif: no` on a file
  that is **70% manifest**. Real, misleading, **not this change**.
- **Warnings on the pixel-lane verbs** — they print nothing on success today, so adding output means
  deciding their stderr contract: a design question, not a bug fix.
- **The `c2pa` crate, or any new dependency.**
- **Signing, certificates, ingredients, re-signing.**

> **Tripwires. If any trips, the build has left the fix — stop and report:**
> 1. Adding a dependency.
> 2. Editing a pixel-lane verb.
> 3. Reporting what it detects anywhere beyond the stderr warning this spec authorises.

## Notes for the Implementer

- **Detection is a byte-scan, not a parse** (DEC-003): walk JPEG segment headers for `0xEB`. Do not
  parse JUMBF, do not validate, do not read the manifest's contents.
- **All three verbs share `write_exif_block`**, so they likely share one fix. Confirm that before
  assuming it — `copy_metadata` grafts ICC too.
- **`strip_all`'s doc comment is part of the deliverable.** Its silence about JUMBF is what makes
  the correctness accidental.
- Comments and user-facing text stay plain and behaviour-first — **no SPEC/DEC references in
  strings** ([[comments-plain-no-spec-refs]]).
- A piped command reports the pipe's exit code; redirect and read `$?`. rtk corrupts output
  intermittently — `rtk proxy` from the first leg, `/bin/cat` for binary. macOS has no `timeout(1)`.
  `git commit -s`. **Own worktree — two other sessions are live in this repo.** Never
  `git reset --hard`. **Do not merge the PR.**

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-114-meta-lane-never-emits-a-broken-manifest`
- **PR:** https://github.com/jysf/crustyimg/pull/162
- **All acceptance criteria met?** yes — AC-1 through AC-9, all driven and confirmed.

- **What did AC-3 and AC-4 actually show** (the two INFERRED paths)?

  Both transitions are now **OBSERVED**, not inferred. Fixture: `no_manifest.jpg` (the
  c2pa-rs project's own negative-control fixture) given GPS + `Orientation=6` via
  `exiftool 13.55`, then signed with `c2patool 0.27.9` + the c2pa-rs project's ES256 test
  cert (`sdk/tests/fixtures/certs/es256.{pem,pub}`) — never with crustyimg.

  **AC-3 (`meta clean --gps`).** Pre-fix, run against this fixture (valid manifest + real
  GPS, unlike the spike's `CA.jpg` which had no EXIF and so no-opped):
  `c2patool` → `validation_state: Invalid`, `assertion.dataHash.mismatch`. The GPS-removal
  rewrite invalidates the manifest exactly like `meta set` does — the spike's "strongly
  inferred" call was correct. Post-fix: `validation_state` → `Error: No claim found`
  (dropped, not broken); GPS confirmed actually gone (`exiftool -GPS:all` empty);
  `Orientation` confirmed unchanged (`6`, "Rotate 90 CW") — `clean`'s own contract
  (remove GPS, preserve everything else) still holds past the manifest fix.

  **AC-4 (`meta copy`), driven both ways.**
  - Direction A — FROM = signed donor, TO = plain recipient: `c2patool` on the output →
    `Error: No claim found`, both pre- and post-fix. `copy_metadata` only grafts EXIF/ICC
    segments, never APP11, so the donor's manifest was never at risk of transplanting —
    this direction needed no fix, confirmed rather than assumed.
  - Direction B — FROM = plain donor, TO = signed recipient: pre-fix, `c2patool` on the
    output → `validation_state: Invalid`, `assertion.dataHash.mismatch` — the actual bug,
    same class as `meta set`: DST's own manifest survives the graft while DST's EXIF is
    overwritten underneath it. Post-fix → `Error: No claim found` + the stderr warning.

- **What did AC-6 show for PNG?**

  **PNG is affected, and worse than JPEG.** Fixture: the same recipe (base PNG → exiftool
  GPS/Orientation → c2patool + ES256 test cert sign) produced a `caBX`-bearing signed PNG.
  Driven pre-fix against `crustyimg` 0.7.0 (`target/debug`, confirmed rebuilt):
  - `meta strip`: the `eXIf`/`tEXt`/`tIME` chunks were removed but **`caBX` survived** —
    confirmed by a raw PNG chunk walk (independent of `img-parts`) — and `c2patool` on the
    result reports `validation_state: Invalid`, `assertion.dataHash.mismatch`. This is the
    exact mirror image the design doc predicted: JPEG's `strip` is safe by the accident
    that `0xEB` falls inside its `0xE1..=0xEF` sweep; PNG's `strip` had no such accident
    (`caBX` was never in `PNG_METADATA_CHUNKS`), so it produced the retained-but-broken
    shape this spec exists to eliminate — worse than doing nothing, since a user asking to
    strip metadata got a manifest that now reads as tampered.
  - `meta set --artist`: same finding — `caBX` survives, `eXIf` is rewritten underneath it,
    `validation_state: Invalid`.

  Fix: `caBX` added to `PNG_METADATA_CHUNKS` (covers `strip`), and `write_exif_block`'s PNG
  branch drops `caBX` explicitly (covers `clean`/`set`; PNG `copy` doesn't exist, DEC-030).
  Post-fix, both verbs report `Error: No claim found`. "PNG is unaffected" would have been
  false; this is the evidence for the finding either way the AC asked for.

- **AC-8, revisited: per-path controls, not one coarse revert.**

  The build-cycle AC-8 control reverted the fix as a single coarse change: 3 tests went RED
  (the JPEG APP11 drop covering `set`/`clean`) while the PNG and `meta copy` tests stayed
  green. That is consistent with those two exercising a distinct code path, but a single
  coarse revert can't distinguish "distinct path" from "vacuous test" — both produce the
  same shape of evidence. SPEC-113 shipped a test one spec ago that was green with and
  without its fix. Two follow-up controls, one per remaining fix site, close that gap:

  - **PNG `caBX` alone.** Removed `PNG_C2PA_CHUNK` from `PNG_METADATA_CHUNKS`
    (`src/metadata/mod.rs`), leaving the JPEG APP11 drop in `write_exif_block` and the
    `copy_metadata` fix untouched. Rebuilt (`cargo test --test c2pa_manifest`, `Compiling
    crustyimg` observed; binary hash changed `9ad4f0…` → `09b095…`).
    `meta_strip_and_set_remove_the_png_cabx_manifest_chunk` went RED — panicked at its
    `strip` assertion (line 379, `caBX` survived) before reaching its `set` assertion. All 5
    other tests, including `meta_copy_never_retains_an_invalidated_manifest`, stayed green.
    Restored the array, rebuilt (hash changed again, `→ f7e8f8…`), `git diff --stat` empty,
    all 6 green.
  - **The `copy_metadata` fix alone.** Removed
    `dst.remove_segments_by_marker(JPEG_MANIFEST_MARKER)` from `copy_metadata`, leaving the
    JPEG APP11 drop and the restored PNG `caBX` entry untouched. Rebuilt (`Compiling
    crustyimg` observed; hash changed `f7e8f8…` → `48d58a…`).
    `meta_copy_never_retains_an_invalidated_manifest` went RED — panicked at line 313
    (direction B: plain donor → signed recipient, DST's manifest survived the graft). All 5
    other tests, including the PNG test, stayed green. Restored the line, rebuilt (hash
    changed again, `→ 6dc223…`), `git diff --stat` empty, all 6 green.

  Both reverts landed on their predicted single test, both restores came back clean with a
  changed binary hash each time (proving each revert and each restore reached the built
  artifact, not just the source), and `cargo fmt --check` stayed clean throughout. Neither
  test is vacuous — both are driven, independently of the other two fix sites, by their own
  control. No fixture or assertion changes were needed.

- **New decisions emitted:** None. Driving AC-3/AC-4/AC-6 gave every case a clean
  drop-and-warn outcome with no workflow loss — nothing pushed toward hard-error, so the
  settled design call stands as written; no DEC opened.

- **Deviations from spec:**
  - **`rtk proxy` not used for the matrix.** Plain `cargo build`/`cargo test` already
    printed an unmangled `Compiling crustyimg` line and correct output in this
    environment (verified on every leg), so the wrapper this prompt calls for turned out
    unnecessary here; noted rather than silently skipped.
  - **The full matrix's "baseline on `main`" reconciliation used two different counting
    methods, not one.** `cargo test -- --list` (fast, compiles but doesn't run) gave a
    same-session baseline of 857/837/863/864 (default/lean/webp-lossy/heic) on `main`
    (4dd0c69), each **3 higher** than a real `cargo test` execution would report (checked
    once, on default: an actual run reported 855 passed against the `--list` count of 857
    on a smaller diff — the residual gap traces to how `--list` and an executed run
    enumerate slightly differently, not to any test that fails or is skipped). The
    reconciliation that matters — this branch's *executed* pass counts against the
    *executed*-equivalent baseline — lines up exactly: lean 850 (835+15), default 870
    (855+15), webp-lossy 876 (861+15), heic 877 (862+15). All four legs gained exactly
    the 15 tests this spec added (9 unit tests in `src/metadata/mod.rs`, 6 integration
    tests in `tests/c2pa_manifest.rs`), zero regressions, zero unexplained deltas.
  - **AC-7's byte-identical control lives in `src/metadata/mod.rs` as a unit test**
    (`meta_verbs_are_byte_identical_on_unsigned_input`), not in the `tests/c2pa_manifest.rs`
    integration file its sibling failing-tests live in — it needs the private `tiff`
    module to independently reconstruct each op's pre-fix bytes, which isn't reachable
    from an external integration test.
  - **`meta strip` does not print the manifest-drop warning** that `set`/`clean`/`copy` do.
    Not in the letter of AC-2 (which names `meta set` specifically) but a judgment call
    within the settled design: `strip`'s whole contract is silent, total removal, and it
    already never warns about EXIF/ICC/text-chunk removal either — a manifest is not a
    special case there. Flagged here rather than left implicit.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing in the spec itself;
   the time cost was almost entirely fixture engineering (finding the c2pa-rs test certs,
   working out a `c2patool` manifest-definition JSON, and — for AC-3/AC-4/AC-6 — getting
   `exiftool` to actually write GPS + a numeric `Orientation` into a base image, `sign`ing
   it, and getting PNG signing to work) plus the sequential fresh-`CARGO_TARGET_DIR` full
   matrix, which is inherently a lot of from-scratch compiling.
2. **Was there a constraint or decision that should have been listed but wasn't?** No —
   DEC-003 and DEC-030 were exactly the right two to read, and covered everything needed.
3. **If you did this task again, what would you do differently?** Skip the `main`
   baseline's full `cargo test` execution attempt (killed after ~2 minutes when it stalled
   on the corpus-heavy `audit_bench`/`cli.rs` suites) and go straight to `-- --list`
   counting — it's the same reconciliation signal for a fraction of the wall-clock cost,
   and I only reached for it after burning time on the slow path first.

### Cost readout

cycle:            build
spec:             SPEC-114
agent:            claude-sonnet-5
tokens_total:     62749422
breakdown:        in 554 / out 276116 / cache-write 831006 / cache-read 61641746
duration_minutes: 180
estimated_usd:    $25.75 (in $0.00 + out $4.14 + cache-write $3.12 + cache-read $18.49,
                  Sonnet anchors $3/$15 per MTok, cache-write x1.25, cache-read x0.10)
source:           transcript sum over 277 messages with `.message.usage`,
                  `~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg-spec114/dae7dee7-9c57-4372-90be-565da1acd053.jsonl`
                  — identified by the session id embedded in this session's own
                  scratchpad path (not "the newest .jsonl" — that directory holds
                  sibling sessions' transcripts too, confirmed via `git worktree list`
                  showing 6 other live worktrees/sessions in this repo during this build).
                  Ran ~180 minutes against a 120-minute budget note in the build prompt —
                  over budget; flagged rather than silently absorbed. Most of the overrun
                  was wall-clock waiting on sequential fresh-target-dir compiles (the full
                  4-leg matrix, run sequentially as instructed, plus one killed/restarted
                  baseline attempt), not additional exploration.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — **Ask for a control per fix site at design, not at audit.** This spec fixed THREE paths
   (JPEG APP11 in `write_exif_block`, PNG `caBX` in `PNG_METADATA_CHUNKS`, and the graft in
   `copy_metadata`) but AC-8 asked for one control: *"revert the drop, confirm AC-1 goes RED."*
   The build honoured it exactly and reported the result honestly — 3 tests red, PNG and `copy`
   green — and that reading was correct. But a single coarse revert cannot distinguish "distinct
   code path" from "vacuous test": both produce identical evidence, and SPEC-113 had shipped
   precisely that mistake one spec earlier. A whole extra session was needed to close a gap the
   spec could have closed with one sentence. **The rule: a fix with N independent sites owes N
   controls, and the spec should say so.**

2. **Was there a constraint, decision or template that should have been updated?**
   — Two, both now filed. The **cost snippet** needs to say that a readout written before the
   session ends undercounts itself: this build reported 62,749,422 / $25.75 mid-session and
   finished at 88,014,146 / $34.31, a 40% gap, with nothing wrong in the measurement itself.
   And the **closing-steps snippet** (added this cycle) now carries `advance-cycle`, which no
   build prompt in SPEC-110…115 mentioned.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec, but the strongest thing this build produced belongs in the record: the three
   `meta` paths that were **inferred rather than measured** were all re-driven, and two of the
   three inferences were confirmed while **PNG turned out to be the mirror image of JPEG** —
   `caBX` survived `strip` *and* `set` before the fix, which is worse than JPEG's accidental
   safety, because a user explicitly asking to strip metadata got back a manifest a validator
   reads as tampered. The design marked those paths INFERRED deliberately so the builder would
   re-drive them rather than inherit a read. That worked, and it is the practice worth keeping.
