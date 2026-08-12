# SPEC-114 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `meta set` takes a file whose Content Credentials validate `Valid` and emits
one a validator reports `Invalid` / `assertion.dataHash.mismatch`, manifest fully intact. It keeps a
signed manifest while rewriting the bytes it is signed over. **Drop the manifest and warn.**

**This is a bug fix, NOT the C2PA feature work.** The scope fence is the most important part of this
prompt, and it is there because the obvious failure mode is this quietly becoming the detection
feature.

## Read in order

1. **The spike** — `docs/research/c2pa-provenance-spike.md`, **in full, first.** Method, fixtures,
   and the observed/inferred split.
2. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-114-meta-lane-never-emits-a-broken-manifest.md`.
   **9 acceptance criteria, 6 pre-written failing tests, one negative control.**
3. **The code** — `src/metadata/mod.rs`: `sniff` `:65`, `strip_all` `:90`, `write_exif_block` `:143`,
   `clean_gps` `:177`, `set_tags` `:211`, `copy_metadata` `:248`, and `PNG_METADATA_CHUNKS`.
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR. **`DEC-003`** (byte-scan, no
   parsing) and **`DEC-030`** (`meta copy` is JPEG-only).

## What is measured vs what is claimed — do not flatten this

| path | status |
|---|---|
| `meta set` | **OBSERVED** Valid → Invalid |
| `meta clean` | **INFERRED** — mechanism seen, transition NOT. **Confirm before fixing.** |
| `meta copy` | **INFERRED, never run.** Drive both ways: signed donor, signed recipient. |
| `meta strip` (**PNG**) | **UNTESTED.** `caBX` is not in `PNG_METADATA_CHUNKS`, so PNG may be the mirror image of JPEG — strip removes `eXIf` and **keeps** `caBX`. Determine it. |

The author marked the middle two INFERRED **deliberately, so you re-drive them rather than
inheriting a read.** `meta copy` was only read, never run.

## The design call is SETTLED

**Drop the manifest, warn loudly on stderr naming what was removed, exit 0.** It matches the repo's
safe-default-with-notice posture, and **there is no valid "keep" option to opt into** — a retained
manifest is broken by construction.

Hard-error-with-opt-out was considered and rejected: no *valid* workflow is lost by dropping, since
those invocations currently produce broken output. **Do not implement both.** If driving convinces
you hard-error is right, that is a finding **and needs a DEC** — not a second code path.

## Scope — the fence, and why it is this long

**The named failure mode: this quietly becoming the C2PA detection feature.** Every item below is
individually reasonable and looks cheap *once the APP11 byte-scan exists*. That is exactly why it is
written down.

**OUT:** the `provenance/credentials-*` lint rule · C2PA reporting in `info`/`info --json` (yes,
`info` prints `exif: no` on a file that is 70% manifest — real, misleading, not this change) ·
warnings on the pixel-lane verbs · **the `c2pa` crate or any new dependency** · signing,
certificates, ingredients, re-signing.

> **Tripwires. If any trips, you have left the fix — stop and report:**
> 1. Adding a dependency.
> 2. Editing a pixel-lane verb.
> 3. Reporting what you detect anywhere beyond the stderr warning this spec authorises.

## Verification — the validator must be independent of our code

```sh
brew install c2patool     # 0.27.9 in the spike
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/CA.jpg
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/no_manifest.jpg
```

`no_manifest.jpg` → "No claim found" is your **negative control**, proving the validator
discriminates rather than rubber-stamps.

**Traps, all of which the spike hit:**

- **`CA.jpg` reports `signingCredential.untrusted` in the BASELINE** — a test cert. Constant across
  every output, **not the defect**. The verdict that moves is `assertion.dataHash.mismatch`.
  **Never assert on `validation_state` alone.**
- **`CA.jpg` has no EXIF**, so `clean_gps` early-returns at `:180` and is a byte-identical no-op.
  **A green result there means the code did nothing.**
- **exiftool cannot author AC-3's fixture** — writing GPS with exiftool breaks the hash itself, so
  the baseline is already Invalid. **Sign a GPS-bearing image with c2patool + a test cert.**
- **Never generate a signed fixture with crustyimg.** A fixture from the code under test cannot
  fail.
- **Assert two independent ways** — a structural JPEG marker walk for APP11 **and** c2patool — and
  **do not derive the byte-scan expectation from the validator's output**; that is one check wearing
  two hats.
- **Raw substring counts are not evidence**: `no_manifest.jpg` contains the bytes `c2pa` without
  carrying a manifest.

**At least one new test must FAIL on `HEAD`.**

## Full matrix

Fresh per-leg `CARGO_TARGET_DIR`, **sequentially**, **through `rtk proxy` from the first leg** (it
deletes the `Compiling crustyimg` line and mangles binary through `cat`; use `/bin/cat` for binary).
Reference on `main`: **lean 821 / default 841 / webp-lossy 847**. **A piped command reports the
pipe's exit code** — redirect and read `$?`. Run AC-8's negative control and prove the revert reached
the **built artifact**. **Then read the CI legs individually.**

## Repo guardrails

`git commit -s` (DCO). Never `git reset --hard`. **Own git worktree — two other sessions are live in
this repo** (docs/recipes and demo look-and-feel), so do not work in the primary checkout and do not
assume `target/` is yours. macOS has no `timeout(1)`. **Do not merge the PR. Do not bump the
version.**

## When you finish

Fill in `## Build Completion`, including the two questions it asks specifically: **what AC-3 and
AC-4 actually showed** for the inferred paths, and **what AC-6 showed for PNG**. "PNG is unaffected"
is a claim that needs evidence.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. **Identify your transcript by something only
your session emitted** — a probe symbol, your agent id — **never by "the newest `.jsonl` in the
directory."** SPEC-112's build did the latter, read the parent orchestrator's session, and reported
the wrong model *and* wrong volume while confidently flagging a mismatch that did not exist. Price
per component at the anchors `.message.model` actually reports. Close with the `## Cost readout`
block, verbatim, last.

**Report what you could not do as clearly as what you did.**
