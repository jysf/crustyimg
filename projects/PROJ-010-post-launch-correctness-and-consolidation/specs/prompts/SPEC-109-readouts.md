# SPEC-109 — cycle readouts (maintainer review)

Prompts: `SPEC-109-{build,verify}.md`. Build ran interactively (main-loop);
verify ran interactively against `spec-109-classifier-evidence-integrity` @ `2006cc4`.

---

## verify — Opus (`spec-109-classifier-evidence-integrity` @ `2006cc4`, 2026-07-26)

**Prompt:** `prompts/SPEC-109-verify.md` · **Cost:** 21,152,459 tok measured (see the spec's
`cost.sessions`)

> **VERDICT: the spec delivered.** The one thing that decides this cycle holds: the calibration
> guard is red at 5.5 and red at 3.2, green at 4.0, and the pre-SPEC-109 tree is green at 5.5.
> Every acceptance criterion verified independently; four findings below, none blocking.

### The gate — re-run from scratch, both trees

`cargo test --release --lib analysis`, editing `PHOTO_ENTROPY_STRONG` in place, `Compiling
crustyimg` confirmed on every run (1 each). Exit codes read directly, never through a pipe.

| tree | threshold | exit | result | failing tests |
|---|---|---|---|---|
| **pre-SPEC-109** (`main`'s `src/analysis/mod.rs`) | 4.0 | 0 | 52 passed, 0 failed | — |
| **pre-SPEC-109** | **5.5** | **0** | **52 passed, 0 failed — GREEN** | — (the premise, reproduced) |
| pre-SPEC-109 | 3.2 | 101 | 51 passed, 1 failed | `wide_flat_manycolour_with_edges_is_ui_screenshot` |
| pre-SPEC-109, control | 7.0 | 101 | 49 passed, **3** failed | `real_exif_stripped_colour_photo_*`, `real_grayscale_photo_*`, `calibration_gap_holds_for_committed_fixtures` |
| **branch** | 4.0 (shipped) | 0 | 54 passed, 0 failed | — |
| **branch** | **5.5** | **101** | **52 passed, 2 failed** | `calibration_gap_matches_the_documented_gap`, `boundary_specimens_measure_their_recorded_values` |
| **branch** | **3.2** | **101** | **51 passed, 3 failed** | + `wide_flat_manycolour_with_edges_is_ui_screenshot` |
| branch, control | 7.0 | 101 | 50 passed, **4** failed | + `real_exif_stripped_colour_photo_*`, `real_grayscale_photo_*` |

The 7.0 control fires on both trees, so "green at 4.0" is a result and not a build that never
recompiled. Verbatim message at 5.5:

> `threshold 5.5 must fall in the calibration window (3.6414278, 4.5176096] (width 0.87618184 bits, cap 1.2)`

### The width cap, driven rather than read

Dropping a specimen from the roster re-widens the window past the 1.20-bit cap and the guard
says so — both sides, with the numbers I derived independently in Python:

| mutation | window | width | result |
|---|---|---|---|
| drop `FX_DITHER_32` from the graphic roster | (3.0259476, 4.5176096] | 1.491662 | **RED** — "A boundary specimen is missing from the roster." |
| drop `FX_PHOTO_FLOOR` from the photo roster | (3.6414278, 6.074273] | 2.4328454 | **RED** — same message |

(Deleting a fixture *file* instead fails at `include_bytes!`, i.e. it fails to compile. Also red,
by a different mechanism.)

### Are the specimens genuinely independent? — read, not trusted

`scripts/seed-classify-specimens.py` is stdlib-only (`zlib`/`struct`/`math`), decodes PNG itself
(filters 0–4, colour types 0/2/6), and never imports or invokes crustyimg. Its luma is
`(77R + 150G + 29B) >> 8`, a separate implementation of the same definition as
`src/analysis/mod.rs:248`. `--check` exits 0 against the committed bytes.

Its docstring advertises a positive control over the four pre-existing fixtures. **`main()` never
runs it** — `--check` only regenerates the two new specimens. I ran the control by hand; it
passes on all four, to better than 5e-4:

| fixture | script's implementation | claimed | engine (`web --json`) |
|---|---|---|---|
| `grayscale_photo_leica.png` | 6.0743 | 6.0743 | 6.07 |
| `grayscale_photo_canon.png` | 6.8300 | 6.8300 | 6.83 |
| `color_photo_fuji.png` | 6.3737 | 6.3737 | 6.37 |
| `dithered_graphic.png` | 3.0259 | 3.0259 | 3.03 |
| `photo_entropy_floor.png` | **4.5176** | 4.5176 | 4.52 |
| `dither_32color.png` | **3.6414** | 3.6414 | 3.64 |

Two limits worth stating plainly. **(a)** The independence is from the *implementation*, not the
*definition* — both compute the same documented luma/entropy, so a wrong definition would agree
with itself. **(b)** `photo_entropy_floor.png` is a tonal curve applied to
`grayscale_photo_leica.png`, which is already in the photo roster. The photo side of the window
is therefore pinned by a documented proxy for DEC-047's 4.58 real-photo floor, not by one of the
48 real crops that floor came from (they are not in the repo). The spec asked for exactly this,
so it is a scope limit, not a defect.

### Attacks, one by one

| # | attack | verdict |
|---|---|---|
| 1 | specimens independent? | **verified** — see above |
| 2 | width cap does what it claims? | **verified** — driven both sides |
| 3 | AC-6: is `optimize.rs:1059` reached? | **verified by instrumentation** |
| 4 | AC-7: does the no-EXIF test take the no-EXIF path? | **verified** |
| 5 | AC-8: un-gated into a tautology? | **no** — made it fail twice on the lean leg |
| 6 | AC-9: generator or comment? | **comment, legitimately** — no fixture nudging |
| 7 | zero behaviour change? | **verified** |
| 8 | the 32-colour deviation | **confirmed, not refuted** |

**#3 — the SPEC-084 branch is genuinely reached.** I put `panic!()` inside the `if !has_lossy`
block. Under `spec_084_metadata_forced_fallback_is_reached` the child process panicked at
`src/cli/optimize.rs:1059:17` — the branch is hit. Under
`optimize_detailed_icc_source_ships_lossy_disposition` it did **not** panic, so the new test is
the only end-to-end route to that branch, exactly as AC-6 claims. (Watch out for a false signal
here: the marker string appears in *both* logs, because `panic!` makes the following code
unreachable and rustc echoes the source line in an `unreachable_code` warning. The runtime
`panicked at` line is the real signal.) Then, separately, mis-conditioning the call site to
`if false && !has_lossy` turns the test red on the disposition assertion, with the report showing
a lossless-only shortlist — `[webp lossless 28,652 B, png lossless 35,445 B]` for a 15,291 B
source. So the assertion depends on that call site, not on a promising name.

**#4 — AC-7 takes the path it names.** Both fixtures report `"has_exif":false` from `info --json`
(the test asserts this itself). Raising `PHOTO_ENTROPY_STRONG` to 7.0 drops the photo case to
`"class":"graphic-logo"` and the test goes red, so the `photograph` verdict is genuinely produced
by rule 3.5 on the no-EXIF path rather than incidentally.

**#5 — AC-8 is not a tautology, but its reach moved sideways.** Both `#[cfg(feature = "avif")]`
gates are gone and `audit_bench` runs 6 tests on the lean leg (was 5). Two falsifications, both
on `--no-default-features`:

- Remove `"ssim"` from the golden key set → **RED**, actual keys include `ssim` for all three
  verbs. So the lean leg really does produce a scored winner; the assertion is live there.
- Inject a real per-verb fork — an additive top-level key gated on `has_alpha`, which `web` and
  `apply` report `true` and `optimize` reports `false` for the *same* file → **RED** with
  `web keys diverge from golden`, while the `optimize` assertion one line earlier passed. That is
  precisely the fork class the test exists to catch, caught on the leg the gate used to silence.

The cost: the source was swapped from a `Lossy`-bucket photo to a `LosslessFlat` graphic, so **no
leg now covers cross-verb schema consistency for a `Lossy`-bucket source**. The lean leg gained
coverage it had none of; the default leg lost coverage it had. The build states this and files
the follow-up, so it is disclosed rather than hidden — but the trade is worth the maintainer's
eye, and the existing fork still ships undetected.

I also independently reproduced the build's unpredicted finding: for the same file,
`optimize` reports `has_alpha: false` while `web` and `apply` report `true`.

**#6 — AC-9 fixed the comment and asserted the measured values.** No nudging: the asserted
entropies are unchanged from the spec's own measurements. I transcribed both generators'
arithmetic into Python and measured with a separate implementation of the luma / Shannon /
forward-difference definitions:

| fixture | occupied bins | entropy | flat_ratio | edge_ratio |
|---|---|---|---|---|
| `ui_screenshot_fixture` | **25** (asserted 25) | **3.3964** (asserted 3.3964) | 0.7611 | 0.1206 |
| `ambiguous_square_fixture` | **14** (asserted 14) | **3.1905** (asserted 3.1905) | **0.6105** | 0.1172 |

`0.6105 > FLAT_GRAPHIC_RATIO 0.60` and `0.1172 ≥ GRAPHIC_EDGE_MAX 0.08` — the corrected comment
is true. Note the *screenshot* fixture is in the same position (flat_ratio 0.7611 > 0.60, held
out of the flat-graphic gate only by edge_ratio); only the ambiguous one asserts it.

**#7 — zero behaviour change, confirmed mechanically.** `src/analysis/decide.rs` is not in the
diff at all. Every one of the 11 hunks in `src/analysis/mod.rs` starts at line ≥ 881, and
`#[cfg(test)] mod tests {` opens at line 633–634 — so the whole `+202 / −27` lands inside the
test module. No production path changed.

**#8 — the 32-colour deviation is arithmetically correct.** I re-ran the recipe at both level
counts over all three committed photographs:

| source | source H | 16-level dither | 32-level dither |
|---|---|---|---|
| `grayscale_photo_leica.png` | 6.0743 | **2.4559** | 3.2666 |
| `grayscale_photo_canon.png` | 6.8300 | **2.8781** | 3.8396 |
| `color_photo_fuji.png` | 6.3737 | **2.8042** | **3.6414** ← the committed specimen |

The prompt's own prediction — "16 levels of a 6.07–6.83-bit source should land ≈2.46–2.88" — is
exactly right (2.4559 … 2.8781). All three are **below** the 3.0259 dither already committed, so
a 16-level specimen would not have moved the lower bound and 3.2 would still pass. The deviation
is justified. The "histogram-equalise first → 3.94" figure also reproduces (Fuji 3.9452, Leica
3.9420), confirming the 0.06-bit-margin reasoning for rejecting it. DEC-047 records the
substitution in place and leaves the unreproducible 16-colour 3.43 row standing with the
arithmetic beside it, rather than silently swapping the specimen it cites.

### DEC-047's corrections, re-measured

| claim | re-measured | verdict |
|---|---|---|
| 128×128 centre crop of the Leica frame → entropy **5.15**, `icon`, lossless | Python 5.1453; engine `"class":"icon"`, entropy 5.15, shortlist `[webp lossless, png lossless]` | ✅ exact |
| …parenthetical: 128×128 **downscale** → **6.02**, same verdict | engine reports **6.03** (`web --max 128`), `icon`, lossless-only | ⚠ 0.01 off; verdict unchanged |
| `dithered_graphic.png` = 3.03 native but **7.08 at `--max 256`**, classifies `photograph`, offered a lossy candidate, SSIMULACRA2 **81.8** | entropy 7.08, `"class":"photograph"`, `[avif lossy (winner), png lossless]`, `"ssim":81.8` | ✅ exact |

Every checkable claim in `tests/fixtures/classify/RECIPES.md` checks out: the seven-row entropy
table (all seven classes and entropies reproduced by the engine), the window `(3.6414, 4.5176]`
at 0.88 bits, the pre-specimen window `(3.03, 6.07]` at 3.04 bits, `--check` agreeing with the
committed bytes, `photo_entropy_floor.png`'s `flat_ratio` of **1.00**, the 0.36-bit margin, and
the 16-level arithmetic above.

### AC-11 — clean full matrix, re-run

Isolated `git worktree` at `2006cc4`, `CARGO_TARGET_DIR` `rm -rf`'d first (201 crates compiled
from nothing on the first leg). Exit codes captured directly, no pipes.

| leg | exit | suites | passed | failed | `Compiling`/`Checking crustyimg` |
|---|---|---|---|---|---|
| `cargo test --no-default-features` | 0 | 32 | **776** | 0 | 1 |
| `cargo test` | 0 | 32 | **796** | 0 | 1 |
| `cargo test --features webp-lossy` | 0 | 32 | **803** | 0 | 1 |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | 0 | | | | 1 |
| `cargo clippy --all-targets -- -D warnings` | 0 | | | | 1 |
| `cargo clippy --all-targets --features webp-lossy -- -D warnings` | 0 | | | | 1 |
| `cargo fmt --check` | 0 | | | | (no output) |

**The loose end is resolved: the `webp-lossy` leg is 803 passed / 0 failed across 32 suites**,
`tests/cli.rs` 132/132, `tests/audit_bench.rs` 6/6. The earlier "0" was a log-capture artifact,
as suspected — there is no failure there. No warnings on any clippy leg.

All seven tests the spec listed under **Failing Tests** exist, carry `#[test]`, and are defined
exactly once; both renamed predecessors are gone (0 occurrences each).

### Scope

**`projects/_templates/prompts/cost-snippet.md` is no longer in this PR.** Commit `2006cc4`
("move the cost-snippet change out of this PR") reverted it; the file on this branch is
byte-identical to `main`, and the change now lives on the local branch
`chore/cost-measurement-methodology` (`0f21cc2`). **`one-spec-per-pr` is satisfied — not
blocking.** The verify prompt was written at `8db9010`, one commit before that fix.

Three files are touched that the spec's **Outputs** section does not list: `tests/common/mod.rs`
(modified — the `jpeg_with_icc` split and `flat_graphic_png`, both required by AC-6/AC-8),
`scripts/seed-classify-specimens.py` and `tests/fixtures/classify/RECIPES.md` (added — both
required by "Seed it independently"). All three are in scope for the work; only the bookkeeping
is short.

### Findings

1. **`rtk` dropped a commit.** `git log --oneline main..HEAD` through `rtk` reported **three**
   commits; the branch has **four**. The missing one is the newest — `2006cc4`, the commit that
   resolves the scope question this prompt asks about. Had I not cross-checked with
   `/usr/bin/git`, I would have reported a `one-spec-per-pr` violation that had already been
   fixed. The existing note covers grep counts; it covers `git log` too.
2. **The build's "27 `cfg(feature)` attributes remain under `tests/`" does not reproduce.** Under
   six different readings I get 19 (`#[cfg(feature`), 20, 26 (the broadest cfg-with-feature
   reading), 33, 41, 41 — never 27, and the scope of the count was not stated. The load-bearing
   half is exactly right: `tests/audit_bench.rs` holds **0** real gates, its single `cfg(feature`
   match being the phrase inside the new doc comment. Positive control: 360 `#[test]` attributes
   found by the same walker.
3. **DEC-047's revised "≤3.64 counting dithers-of-photos" is the specimen's value, not a
   ceiling.** The same 32-grey-level Floyd–Steinberg recipe applied to the repo's own
   `grayscale_photo_canon.png` measures **3.8396** — still under 4.0, but it cuts the stated
   margin from 0.36 bits to **0.16**. The sentence reads as a bound over the class; it is a
   bound over the one specimen chosen. (Choosing Fuji over Canon was the right call — Canon
   would have tightened the window to 0.678 bits at the price of a 0.16-bit fixture — but the
   prose should say which it is.)
4. **DEC-047's "6.02" parenthetical re-measures at 6.03** on this build via `web --max 128`.
   Same verdict (`icon` → lossless), so the correction's substance is unaffected.

Two smaller notes: the seeding script's positive control is prose, not executable — `main()`
measures only the two new recipes, so nothing re-runs the four-fixture cross-check; and `--check`
compares **exact bytes**, which means it will report `MISMATCH` on any zlib version change even
though the pixels are identical. Neither affects the Rust tests, which decode the committed PNGs.

### What I did NOT check

- **CI on any OS but this one.** Everything here ran on macOS/darwin locally. The GitHub Actions
  legs were not run and their per-OS results are unverified.
- **The build session's own cost figures** (65,339,132 tok / $43.21) against its transcript.
- **SPEC-105's 4.58 floor from 48 real photo crops** — those sources are not in the repo, so the
  number DEC-047 calibrates against remains unreproduced here. Only the committed proxy is
  pinned.
- **DEC-047 outside the two corrections and the new specimen table** — the rest of the record was
  not re-derived.
- **The provenance claims for the four pre-existing fixtures** ("real frames the maintainer
  owns", "a real 8-colour error-diffusion dither"). Unfalsifiable from inside the repo.
- **`--profile docs` coverage** (out of scope; listed as a follow-up) and the **wasm target /
  demo**, untouched by this branch.
- **The `AC-4` shortlist composition on every leg** beyond what the test asserts — I confirmed the
  test passes on all three legs and that no lossless candidate appears, but did not enumerate the
  shortlists directly.

### One trap I walked into, recorded because it nearly cost a finding

Mid-cycle I re-measured the whole `RECIPES.md` table with `./target/debug/crustyimg` and got
**`graphic-logo` for all seven fixtures** — apparently contradicting the table. The binary was
stale from the `PHOTO_ENTROPY_STRONG = 7.0` mutation two steps earlier: restoring the source does
not rebuild the binary, and nothing in the invocation says so. Rebuilding reproduced the table
exactly. Every engine-measured number in this readout is from a binary rebuilt from the pristine
tree. Separately, `git checkout main -- <path>` stages the older file; `git checkout <path>`
afterwards then restores *from the index*, silently leaving `main`'s version in the working tree.
Caught with `git status --porcelain`; fixed with `git restore --staged --worktree`. The tree is
clean at `2006cc4`.
