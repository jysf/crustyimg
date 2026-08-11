# C2PA / Content Credentials — empirical spike

> **Spike, not a spec.** Time-boxed to ~1 hour, run 2026-08-10/11 against
> crustyimg 0.7.0. No production code was changed. No DEC opened.
>
> Every row below is labelled **OBSERVED** (a command was run, output captured)
> or **INFERRED** (reasoning from an observation). The commands are cited so any
> claim can be re-driven.

## Headline

The question was "dropped, or carried-and-broken?" The answer is **both**, and the
split does not follow the lane boundary the opportunity analysis assumed.

- **The pixel lane drops it.** `web`, `optimize`, `convert`, `auto-orient` re-encode
  and the manifest is gone — not one byte survives. **OBSERVED.**
- **`meta strip` also drops it** — cleanly, at the container level. **OBSERVED.**
- **`meta set` carries it and breaks it.** A file that validated `Valid` going in
  comes out `Invalid` with `assertion.dataHash.mismatch`, manifest fully intact.
  **OBSERVED.**

That third line is the finding. The analysis named "never emit a manifest that fails
validation" as the anti-goal. **crustyimg 0.7.0 already violates it.** Not
hypothetically, not if someone adds passthrough later — today, on a shipped verb, in
the release binary. `meta set --artist` is a manifest-forger.

It is not a design decision that produced this. `meta` is documented as the lane that
does not re-decode pixels, so it rewrites the container and preserves segments it
does not recognise. APP11 is a segment it does not recognise. It gets preserved by
default, and the EXIF rewrite next to it invalidates the hash it is signed over.

## What was tested

| | |
|---|---|
| crustyimg | **0.7.0**, release build from `7fffbc6` (`cargo build --release`) |
| c2patool | **0.27.9** (`brew install c2patool`) — independent validator |
| exiftool | **13.55** — used only to author a GPS test file |
| byte-scanner | `scan.py`, written for this spike; walks JPEG markers / PNG chunks / RIFF chunks / ISOBMFF boxes directly |

**A stale binary would have made this whole spike a lie.** The checkout had a
`0.6.0` binary sitting in `target/release`. It was rebuilt before any test ran, and
`--version` was re-read afterwards to confirm `0.7.0`.

### Fixtures — external origin, not ours

From `contentauth/c2pa-rs@main`, `sdk/tests/fixtures/` — the C2PA project's own test
corpus. **Deliberately not produced by the code under test**, and not produced by
`c2patool` either.

| File | SHA-256 (first 16) | Baseline |
|---|---|---|
| `CA.jpg` | `e71bff58fc576408` | **`Valid`** — primary subject |
| `adobe-20220124-E-clm-CAICAI.jpg` | `b3ff3f00c6660228` | manifest present, 10× APP11 |
| `no_manifest.jpg` | `9ac395ca04fc9d34` | **negative control** — `No claim found` |

The negative control matters: `c2patool` returns `Error: No claim found` on
`no_manifest.jpg` and a full manifest on `CA.jpg`, so the validator is discriminating,
not rubber-stamping.

### Baseline, established two ways

```bash
c2patool in/CA.jpg | jq '.validation_state'      # → "Valid"
python3 scan.py in/CA.jpg
```

```
== CA.jpg [jpeg] 166864 bytes
   APP11 len=64010 head=b'JP\x02\x11...jumb\x00\x00\x00\x1e'
   APP11 len=53259 head=b'JP\x02\x11...jumb\x80!\xaa('
```

Two APP11/JUMBF segments, **117 KB of a 167 KB file** — the manifest is 70% of this
image's bytes. The byte-scan was written against the JPEG spec, not against
`c2patool`'s output; the two agree independently.

**On `validation_state: "Valid"`:** the fixture also reports
`signingCredential.untrusted`, because it is signed with a test certificate that is
not on the C2PA trust list. That is a *signer trust* finding and is present in the
baseline. It is orthogonal to the *hash* findings this spike is about, and it is
constant across every row below — so it is excluded from the verdicts. The state
that moves is `assertion.dataHash.mismatch`, which appears only where noted.

## Findings — verb × output format

Every output inspected **twice, independently**: `python3 scan.py` (structural
container walk) and `c2patool` (validator). The byte-scan was not derived from the
validator's output. **The two never disagreed.**

| Verb | Output | Bytes | JUMBF in container | `validation_state` | Verdict |
|---|---|---|---|---|---|
| `web` | avif | 20 310 | **absent** | *no claim* | **Dropped** |
| `optimize` | jpeg | 49 560 | **absent** | *no claim* | **Dropped** |
| `convert --format webp` | webp | 241 050 | **absent** | *no claim* | **Dropped** |
| `convert --format png` | png | 273 521 | **absent** | *no claim* | **Dropped** |
| `convert --format jpeg` | jpeg | 49 560 | **absent** | *no claim* | **Dropped** |
| `auto-orient` | jpeg | 49 560 | **absent** | *no claim* | **Dropped** |
| `meta strip` | jpeg | 49 591 | **absent** | *no claim* | **Dropped** |
| `meta clean --gps` *(no GPS present)* | jpeg | 166 864 | **present, 2× APP11** | **`Valid`** | **Survives** — byte-identical no-op |
| **`meta set --artist`** | jpeg | 166 911 | **present, 2× APP11** | **`Invalid`** | **⚠ Carried-and-BROKEN** |

All nine exited `0`. **OBSERVED**, via:

```bash
crustyimg web in/CA.jpg -o out/web.avif -y      # …and the other eight
python3 scan.py out/*.avif out/*.jpg out/*.webp out/*.png
c2patool out/metaset.jpg | jq '.validation_state'
```

### The broken case, in full

```
--- out/metaset.jpg  [c2patool exit=0] validation_state=Invalid
      FAIL signingCredential.untrusted: signing certificate untrusted
      FAIL assertion.dataHash.mismatch: asset hash error, name: jumbf manifest,
           error: hash verification( Hashes do not match )
```

`meta set --artist "Spike Test"` added a 45-byte `APP1/Exif` segment (+47 bytes
total), kept both APP11 segments byte-for-byte, and thereby produced an image
carrying a complete, well-formed, cryptographically signed manifest **that no longer
matches its own pixels.** To a validator that is indistinguishable from tampering.

`meta clean --gps` uses the same lane. On `CA.jpg` it was a pure no-op — no GPS to
remove, output byte-identical to input (`cmp` reports identical), manifest still
`Valid`. That is a real result but a narrow one: it survives because *nothing
happened*, not because the lane is safe.

To test `clean` where it actually acts, GPS was written into a copy with exiftool.
**That file's baseline was already `Invalid`** — exiftool's own EXIF write breaks the
hash too, which is itself worth knowing. So this test **cannot** show a Valid→Invalid
transition for `clean`. What it *does* show, **OBSERVED**:

```
== gps_clean.jpg [jpeg] 166952 bytes
   APP11 len=64010 …        ← both manifest segments retained
   APP11 len=53259 …
   APP1/Exif len=86         ← EXIF rewritten, GPS gone
```

`meta clean --gps` retains the manifest while rewriting the bytes beside it — the
same mechanism that broke `meta set`. **INFERRED** (strongly, same lane + the
`meta set` observation): `meta clean --gps` on a file with both a valid manifest and
real GPS produces a broken manifest. Not directly observed, because no fixture exists
with both, and manufacturing one with an independent tool destroys the thing being
measured.

### Does anything survive a format change?

**No. OBSERVED.** JPEG → AVIF, WebP, and PNG all produced zero JUMBF bytes — the
scanner found `'jumb'=0 'c2pa'=0` and the AVIF's top-level box walk is
`ftyp / meta / mdat` with no `jumb` box. Nothing survives, and there is no partial
carriage to reason about.

### Does the user get any signal?

**No — and it is worse than silence. OBSERVED.**

```bash
crustyimg info in/CA.jpg
```
```
file size:  166864 bytes
icc:        no
exif:       no        ← on a file that is 70% Content Credentials
```

- `info` reports **`exif: no`**, which a user reads as *this file has no metadata*.
  117 KB of signed provenance goes unmentioned.
- `info --json` has no field for it — grepping the entire JSON for
  `c2pa|jumbf|provenance|credential|app11` returns **nothing**.
- `lint in/CA.jpg` → `1 scanned · 0 error · 0 warn · 0 info`.
- `web` and `meta strip` wrote **0 bytes to both stdout and stderr**. Confirmed with
  `wc -c` on the captured streams.

There is no warning, no JSON field, no lint finding, and no exit code. The one verb
that *does* volunteer metadata information actively implies there is none.

## Answers to the open questions

**Where does WebP store the manifest?** A RIFF chunk with FourCC **`C2PA`**.
**OBSERVED** in the SDK source — `sdk/src/asset_handlers/riff_io.rs:59`:

```rust
const C2PA_CHUNK_ID: ChunkId = ChunkId {
    value: [0x43, 0x32, 0x50, 0x41],
}; // C2PA
```

WebP is handled by `riff_io.rs`, not a dedicated handler. This closes the
`(unverified)` row in the analysis's container table.

**What does the `c2pa` SDK pull in?** **381 locked packages, 256 unique crates**
(`cargo tree --edges normal`), probed in a throwaway crate in the scratchpad —
**crustyimg's `Cargo.toml` was not touched.** That is a large surface for a crate
that would otherwise be one dependency.

License scan over the full locked tree: **no copyleft blocker.** 192 `MIT OR
Apache-2.0`, 51 `MIT`, 36 `Apache-2.0 OR MIT`, 29 `MIT/Apache-2.0`, 18 `Unicode-3.0`,
and small BSD/Zlib/ISC tails. The only hit on an AGPL/GPL/SSPL/BUSL grep is `r-efi`
(`MIT OR Apache-2.0 OR LGPL-2.1-or-later`) — a disjunctive `OR`, so MIT is
selectable, and it is a UEFI-target crate that does not build on our platforms.
**Nothing here trips the no-AGPL rule.** Note this is a license scan, not a
`cargo deny` run — advisories and bans were not evaluated.

**Does it compile to wasm?** Declared, not verified. `sdk/Cargo.toml` carries
`wasm-bindgen`, `serde-wasm-bindgen`, `getrandom/wasm_js`, and distinct
`cfg(target_arch = "wasm32")` / `cfg(target_os = "wasi")` dependency blocks. But
`default = ["openssl", "default_http"]` — **INFERRED**: a wasm build would need
`--no-default-features` and a non-openssl crypto path. Not compiled. Do not treat
in-browser verification as costed.

## What could not be determined

- **Whether `meta clean --gps` breaks a genuinely valid manifest.** Mechanism
  observed; the Valid→Invalid transition was not, for the fixture reason above.
  Resolvable by signing a GPS-bearing image with `c2patool` and a test cert — ~20
  minutes, not done inside the box.
- **Signed PNG / WebP / AVIF *inputs*.** The corpus's `sample1.png`, `sample1.webp`,
  `sample1.avif` all return `No claim found` — they are plain samples. So the
  PNG `caBX` and RIFF `C2PA` **read** paths were never exercised against crustyimg;
  those containers were only tested as *outputs*. **The pixel-lane result is not at
  risk** (nothing survives re-encode regardless of source container), but a
  signed-PNG-through-`meta set` case is untested and could differ.
- **`cargo deny`** was not run — licenses were scanned, advisories and bans were not.
- **Whether stripping is *correct*** for `meta strip`. It is defensible. It is also
  undocumented and unannounced, which is the actual complaint.

## Recommendation — on detect-and-warn only

*(Scoping the derive/re-sign feature is out of scope for this note and is not
attempted.)*

**Detect is small. Warn is small. But the spike found a defect that outranks both.**

**Detection is genuinely cheap.** Locating a manifest needs no parsing and no
`c2pa` dependency: JPEG is an APP11 marker walk, PNG a `caBX` chunk, WebP a `C2PA`
RIFF chunk, AVIF/HEIF a `jumb` box. The scanner in this spike does all four in ~120
lines of Python with no dependencies, and it agreed with Adobe's validator on every
file tested. That is a direct fit for DEC-003's byte-scan-not-parse rule, and it
would not add a single crate. **S, and the estimate is now evidence-backed rather
than assumed.**

**But the priority ordering in the analysis is wrong, and the spike is why.**
Stage 1/2 was argued as "a correctness bug, same class as SPEC-110." That
undersells it. SPEC-110 dropped orientation — data loss, recoverable. `meta set`
emits a signed provenance claim that does not match its own pixels. Data loss is
one thing; **emitting a cryptographic assertion that reads as tampering is
another.** A newsroom running `meta set --artist` across an archive would silently
convert "no credentials" into "credentials that fail validation," which is the one
outcome the analysis explicitly named as worse than stripping.

So the work splits differently than staged:

1. **Fix `meta set` / `meta clean` first.** When a container-lane op rewrites bytes
   covered by a C2PA hash, it must not keep the manifest. Drop it and say so. This
   is a bug fix, it is smaller than detection, and it stops the bleeding. It needs
   only the JPEG APP11 scan, not all four containers.
2. **Then detect and report** — `info` should stop saying `exif: no` on a file that
   is 70% manifest. That single line is the most misleading output found.
3. **Then the lint rule and the pixel-lane warning**, which is the analysis's Stage
   1/2 as written and remains correct.

Nothing here requires the `c2pa` crate, a certificate, or a position on conformance.
The 256-crate dependency question belongs to Stage 3 and does not gate any of the
above.

**One caution on scope creep.** "Warn on the pixel lane" is not quite free: `web`
and `optimize` currently print *nothing at all* on success, so a warning means
deciding what these verbs' stderr contract is. That is a small design question, not
a small code question, and it is the part most likely to be underestimated.

---

## Reproducing

Fixtures, outputs, and `scan.py` are in this session's scratchpad, which is not
durable. To re-drive from scratch:

```bash
brew install c2patool
curl -O https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/CA.jpg
cargo build --release && ./target/release/crustyimg --version   # must read 0.7.0
c2patool CA.jpg | jq '.validation_state'                        # → "Valid"
./target/release/crustyimg meta set CA.jpg --artist "x" -o broken.jpg -y
c2patool broken.jpg | jq '.validation_state'                    # → "Invalid"
```

The last two lines are the whole finding.
