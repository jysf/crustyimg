---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-044                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: critical
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-10
shipped_at: null

value_contribution:
  advances: >
    PROJ-010's thesis is that a shipped verb does what its name says on an ordinary
    input. `meta set` currently takes a file carrying valid Content Credentials and
    emits one a validator reports as TAMPERED — manifest intact, hash broken. That is
    the same defect class as the rest of this project, on a lane nobody swept, and it
    is live in the released binary.
  delivers:
    - "`meta set`/`clean`/`copy` can never emit a retained-but-invalidated C2PA manifest"
    - "`meta strip`'s removal of APP11 becomes intentional and tested rather than incidental"
    - "the PNG `caBX` question answered rather than assumed"
  explicitly_does_not:
    - "Build the C2PA feature — no lint rule, no `info` reporting, no pixel-lane warnings"
    - "Add the `c2pa` crate, or any dependency"
    - "Touch signing, certificates, ingredients, or re-signing"
---

# STAGE-044: the `meta` lane cannot emit a broken manifest

## What This Stage Is

A bug fix on a shipped verb, sourced from a driven spike
([`docs/research/c2pa-provenance-spike.md`](../../../docs/research/c2pa-provenance-spike.md), run
against 0.7.0). **It is not the C2PA feature work**, and the scope fence below is the most important
part of this stage.

The spike asked "does crustyimg drop Content Credentials, or carry them broken?" and found **both**,
split in a way that does not follow the lane boundary anyone assumed:

| path | behaviour | status |
|---|---|---|
| pixel lane (`web`, `optimize`, `convert`, `auto-orient`) | re-encodes; manifest gone entirely | **OBSERVED** |
| `meta strip` (JPEG) | drops it cleanly at the container level | **OBSERVED** |
| **`meta set`** | **carries it and breaks it** | **OBSERVED** |

> `c2patool CA.jpg` → `Valid`
> `crustyimg meta set CA.jpg --artist "x" -o broken.jpg -y`
> `c2patool broken.jpg` → `Invalid`, `assertion.dataHash.mismatch`

The spike's own words: *"`meta set --artist` is a manifest-forger."* Not hypothetically — today, on
a shipped verb, in the release binary.

## Why Now

- **It is live in 0.7.0**, which is what `brew install` hands out as of 2026-08-10.
- **It is the same defect class as everything else in PROJ-010** — a shipped verb quietly producing
  a wrong artifact on an ordinary input — on the one lane the project never swept. STAGE-034/039
  covered the pixel lane, STAGE-043 covers the pinned path; this is the **metadata lane**.
- **The failure mode is worse than "loses data."** Dropping credentials is a shrug; emitting a
  manifest that a validator reads as *tampering* attributes a forgery to the file's signer. In 2026,
  provenance tooling is exactly the thing a launch audience checks.
- **The cause is an accident on both sides, which is why nothing caught it.** `meta` is documented
  as the lane that does not re-decode pixels, so it rewrites the container and preserves segments it
  does not recognise. APP11 is one it does not recognise, so it survives — while the EXIF rewrite
  beside it invalidates the hash APP11 is signed over. `strip_all` (`src/metadata/mod.rs:90`) is
  *correct only by coincidence*: it loops `0xE1..=0xEF`, APP11 is `0xEB`, and the doc comment says
  *"APP1..APP15 (EXIF/XMP/ICC/…)"* — **JUMBF is never mentioned**. Narrowing that range for any
  reason silently reintroduces the bug.

## Success Criteria

- **`meta set`, `meta clean` and `meta copy` never emit a retained-but-invalidated manifest.**
  Proven with `c2patool`, an independent validator, not by reading the diff.
- **`meta strip`'s APP11 removal is intentional and tested**, so narrowing the marker range fails a
  test instead of silently regressing.
- **The PNG question is answered, not assumed** — see Design Notes; this stage added it to the
  spike's list.
- **At least one new test FAILS on today's `HEAD`.** If every new test passes before the fix, the
  tests do not cover the bug.
- **No new dependency.** Detection is a byte-scan for the JPEG APP11 marker — no parsing (DEC-003).

## Scope

### In scope
- `meta set`, `meta clean`, `meta copy` — never emit a broken manifest.
- Making `meta strip`'s APP11 drop intentional + tested.
- The minimal detection those need: a JPEG APP11 marker scan, byte-scan only.
- The PNG `caBX` determination.

### Explicitly out of scope — each is separately gated
- **The `provenance/credentials-*` lint rule.**
- **C2PA reporting in `info` / `info --json`.** The spike found `info` prints `exif: no` on a file
  that is 70% manifest. It is real, it is misleading, and it is **not this change**.
- **Warnings on the pixel-lane verbs.** They print nothing on success today, so adding output means
  deciding their stderr contract — a design question, not a bug fix.
- **The `c2pa` crate.** Nothing here needs it.
- Signing, certificates, ingredients, re-signing.

> **If the build finds itself adding a dependency or editing a pixel-lane verb, it has left the
> fix.** Stop and report.

## Spec Backlog

- [ ] (not yet framed) — **The `meta` lane cannot emit a broken manifest.** One spec; the three
  verbs share `write_exif_block` (`src/metadata/mod.rs:143`), so they share the fix. Complexity
  **S–M**.

**Count:** 0 shipped / 0 active / 1 pending (not framed)

## Design Notes

### The one design call

**What should `meta set`/`clean`/`copy` do when APP11/JUMBF is present?**

**Recommendation: drop the manifest and warn loudly on stderr, naming what was removed.** It
satisfies "never emit a manifest that fails validation"; it matches the repo's existing
safe-default-with-notice posture (the drop-GPS default with `--keep-gps`); and **there is no valid
"keep" option to opt into, because a retained manifest is broken by construction.**

The alternative worth weighing is **hard-error with an explicit opt-out**. For: `meta` is the
metadata verb, so silently destroying the most important metadata in the file is a poor default even
with a warning. Against: it breaks scripted use on signed inputs — though note **no valid workflow
is lost**, since those invocations currently produce broken output.

**Pick one. If the alternative wins, record it as a DEC. Do not implement both.**

### An asymmetry the spike did not list — PNG may be the mirror image

`PNG_METADATA_CHUNKS` (`src/metadata/mod.rs`) is `eXIf, iCCP, tEXt, zTXt, iTXt, tIME`. **`caBX` — the
PNG C2PA chunk — is not in it.** So the accident that makes JPEG `meta strip` *safe* runs the other
way on PNG: `strip` would remove `eXIf` while **keeping** `caBX`, which is the retained-but-broken
shape this stage exists to eliminate.

The spike explicitly records that the PNG `caBX` read path *"was never exercised against
crustyimg"* (line 222), and `sniff` (`:65`) accepts JPEG **and PNG**. So PNG `meta strip` is a
**fourth candidate path**, not covered by the three the source prompt named. **Drive it; do not
assume either way.** If it holds, PNG needs the same treatment and possibly `caBX` added to the
strip list.

### Verification, and the traps this bug sets

The validator must be **independent of our code** — that is the whole point.

```sh
brew install c2patool     # 0.27.9 in the spike
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/CA.jpg
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/no_manifest.jpg
```

`no_manifest.jpg` returns "No claim found" — use it as the **negative control**, so the validator is
shown to discriminate rather than rubber-stamp. Every trap below was hit by the spike:

- **`CA.jpg` reports `signingCredential.untrusted` in the BASELINE.** It is signed with a test cert.
  That finding is constant across every output and is **not the defect**. The verdict that moves is
  `assertion.dataHash.mismatch`. **Do not assert on `validation_state` alone** without knowing which
  failure moved it. [[a-plausible-test-result-is-not-a-checked-one]]
- **`CA.jpg` has no EXIF**, so `clean_gps` early-returns (`:180`) and is a byte-identical no-op on
  it. **A green result there means the code did nothing.** [[a-harness-that-exercises-nothing-reports-green]]
- **exiftool cannot author the `clean` fixture**: writing GPS with exiftool breaks the hash itself,
  so the baseline comes out already-`Invalid`. Sign a GPS-bearing image with c2patool and a test
  cert instead.
- **Do not generate any signed fixture with crustyimg.** [[fixtures-from-the-code-under-test-cannot-fail]]
- **Assert two independent ways** — a structural JPEG marker walk for APP11 **and** c2patool — and
  **do not derive the byte-scan expectation from the validator's output**; that is one check wearing
  two hats. [[a-self-referential-control-cannot-detect-a-broken-pipeline]]
- **Raw substring counts are not evidence:** `no_manifest.jpg` contains the bytes `c2pa` without
  carrying a manifest. [[mechanical-sweeps-need-a-mechanical-check]]

### Preserve the observed/inferred split

The spike is careful about what it saw versus what it reasoned, and **that distinction must survive
into the spec**:

- `meta set` — **OBSERVED** broken, Valid → Invalid.
- `meta clean` — **INFERRED.** The mechanism was observed (APP11 kept, EXIF rewritten); the
  Valid → Invalid transition was **not**, because no fixture had both a valid manifest and GPS.
  **Confirm before fixing.**
- `meta copy` — **INFERRED, never driven.** It only grafts EXIF+ICC so it cannot transplant a
  manifest onto a foreign image, but it rewrites the *destination's* EXIF/ICC while keeping the
  destination's APP11. **Drive it both ways** — signed donor, signed recipient.

## Dependencies

### Depends on
- `docs/research/c2pa-provenance-spike.md` — the evidence. **Committed alongside this stage**; it
  had been sitting untracked, which also means `just validate` never saw it (STAGE-042 chore).
- `DEC-003` (byte-scan, no parsing) · `DEC-030` (`meta copy` is JPEG-only).

### Enables
- The deferred C2PA work — the lint rule and `info` reporting — which becomes a real option once the
  lane is safe. None of it is in scope here.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
