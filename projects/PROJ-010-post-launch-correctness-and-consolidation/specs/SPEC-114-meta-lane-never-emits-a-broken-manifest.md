---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-114
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

- [ ] **AC-1.** `meta set` on a file with a valid manifest **emits no manifest**, and c2patool
      reports **"No claim found"** rather than Invalid. **Fails today** (Invalid,
      `assertion.dataHash.mismatch`).
- [ ] **AC-2.** It **warns on stderr**, naming what was removed; **exit stays 0**. Assert the
      message.
- [ ] **AC-3.** `meta clean` — **confirm the inferred transition first**, on a fixture with **both**
      a valid manifest and real GPS, then fix. Record what you observed before the fix.
- [ ] **AC-4.** `meta copy` — **driven both ways**: signed donor and signed recipient. Neither may
      produce a retained-but-invalidated manifest.
- [ ] **AC-5.** `meta strip` (JPEG) still removes APP11, and **a test pins it by name** so narrowing
      `0xE1..=0xEF` fails rather than silently regressing. The doc comment names JUMBF.
- [ ] **AC-6.** **PNG determined and handled.** Drive `meta strip` and `meta set` on a signed PNG.
      If `caBX` survives an `eXIf` rewrite, PNG gets the same treatment. **Report the finding either
      way** — "PNG is unaffected" is a claim needing evidence.
- [ ] **AC-7.** **A file with no manifest is untouched.** Byte-identical output for every `meta`
      verb on unsigned input — the did-not-break-the-lane control.
      [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-8.** **A negative control**: revert the drop, confirm AC-1 goes RED, restore. Prove the
      revert reached the built artifact.
- [ ] **AC-9.** Clean **full matrix**, fresh per-leg `CARGO_TARGET_DIR`, sequential, through
      `rtk proxy` from the first leg; `Compiling crustyimg` in each log. **Then read the CI legs.**

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

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **What did AC-3 and AC-4 actually show** (the two INFERRED paths)?
- **What did AC-6 show for PNG?**
- **New decisions emitted:**
- **Deviations from spec:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
