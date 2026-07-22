# SPEC-083 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-083-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-20; **reframed + sharpened 2026-07-20 for the 0.5.0-live
  reality** (build now, crustyimg side = shipped 0.5.0). BENCHMARKS.md = honest, EQUAL-QUALITY,
  reproducible cross-tool comparison vs sharp/`@squoosh/cli`/ImageMagick on size+speed, off a real
  `--corpus`. **Pinned the concrete reference corpus** (`~/PSeven/experiments/crustimg_redo_plus/_incoming0`,
  8 photos 0.7–47 MP / 5 cameras — the STAGE-030 set; committed CC0 corpus is all <2048px, SPEC-088 carry).
  **Pinned the matched-quality method: score EVERY tool's output with ONE scorer — `crustyimg diff`
  (SSIMULACRA2) vs the same original — show the quality column; iso-quality-band OR honest size-vs-quality
  scatter, methodology fixed BEFORE numbers are read.** Expects a **DEC** (methodology/scorer/tool-set/
  corpus provenance). Judgment-bound, not mechanical → **recommend OPUS build** (credibility stakes), Opus
  verify. Extends the SPEC-088 `just bench` discipline to competitors; report losses honestly; tell the
  q85-AVIF "high" (~80) story straight. Complexity M (leans L — installs 3 competitors + a cross-tool
  harness). **Build-ready; awaiting dispatch decision (Opus build).**
- [x] build — Opus, 2026-07-20 on `spec-083-honest-benchmarks`. Chose **(a) iso-quality** at an
  SSIMULACRA2 ~82 band (probe confirmed the band is hittable for every tool). Deliverables:
  `scripts/bench-compare.py` (cross-tool harness — one scorer = `crustyimg diff`, own-reference
  encode-fidelity scoring, fixed per-tool grids picked-nearest-band, competitors labelled not dropped),
  `BENCHMARKS.md` (methodology + machine + pinned versions + exact commands + per-bucket AND per-photo
  tables + honest caveats + reproduce section), `DEC-080` (methodology), `just bench-compare`, README
  link. **Honest headline: crustyimg is NEITHER smallest (sharp wins size 5/8) NOR fastest (sharp/IM
  3–8× faster on wall-clock, being multi-threaded); per-core it's a wash vs single-thread libvips.**
  Value framing = zero-dep pure-Rust binary + measured quality + RAW + wasm. Reproducibility proven
  (run1≡run2 on all deterministic fields, wall-times ≤~2%). Key finds: the distributed 0.5.0 binary has
  **no AVIF** (needs `--features avif`) — the central honesty pivot; brief's "5 cameras/Sony" corrected
  from EXIF (6 models/4 brands, no Sony); no IM cell for the 47MP Leica — magick won't write the lossless
  PNG *reference* this method scores against (bad iCCP), though its AVIF encode of that file is fine, so
  it's a limit of the harness (first written as an IM failure; corrected in the prose pass).
  `just validate` green, no `src/` change. **Handed to verify (Opus) — NOT merged.** See Build Completion.
- [x] verify — Opus, 2026-07-20 on `spec-083-honest-benchmarks` @ 0ded21e. **⚠ PUNCH LIST — back to build.**
  What held: every published cell re-derived from `run1.json` matches exactly (per-photo, per-bucket
  medians, per-core table, smallest-AVIF tally 5/2/1); determinism confirmed 47/47 deterministic fields
  identical run1≡run2; `just validate` green; the `--features avif` pivot is real (`dist-workspace.toml`
  builds default features); RAW extension list correct; harness runs end-to-end and does label "NOT RUN".
  **Blocking (1):** `@squoosh/cli` is invoked with BOTH `width` and `height` set, which squashes aspect —
  its outputs are distorted 2048×2048 on **6 of the 8 photos** (verified: 6016×4016 → 2048×2048 while every
  other tool gives 2048×1367). Own-reference scoring masks it (still scores ~82), so the quality column
  provides no protection, and the doc's "same pipeline for every tool" is false for squoosh. Corrected
  aspect-preserving re-run of DSC_9952 gives 21 KB @ cq18, not the published 26 KB @ cq14.
  **Also:** the per-core table's DSCN3478 row is not iso-quality (sharp picked q78 multi-thread vs q70 at
  `VIPS_CONCURRENCY=1` — libaom output shifts with thread count), and it is the closest row the "faster on
  4 of 8" tally rests on; `sharp`/`cwebp` mis-size PORTRAIT sources (long edge 3068, not 2048) which breaks
  the "run it on your own corpus" promise; DEC-080's calibration is wrong — `crustyimg web` is byte-identical
  to `convert -q 80` (md5-verified), not `-q 85`, and lands 73.5–79.0 not 79–82, so the stated rationale for
  the 82 band centre is unfounded; "3–8× faster" understates ImageMagick (up to 14.1×); "none of the
  competitors ship a perceptual quality readout" is false (`magick compare -metric DSSIM/SSIM`,
  `cwebp -print_ssim`). Minor: prose "79–83.5" vs the table's own 83.6; the documented squoosh command omits
  `"method":"lanczos3"`; front-matter `cycle:` was never advanced past `design`. See the verify report.
- [x] build (fix) — Opus, 2026-07-20 on `spec-083-honest-benchmarks`. Cleared the whole punch list with a
  **full re-run**, not a doc patch. Harness: squoosh `--resize` now constrains ONE axis (its
  `resizeWithAspect` stretches when given both); sharp gets the full `resize E E --fit inside` box and
  cwebp pins the long axis, so **portrait sources downscale correctly** (both really produced 2048×3068
  before — confirmed); new **dimension guard** measures every reference and every grid output against the
  source long edge + aspect and exits 3 on a violation; `--self-test` (8 shapes, no corpus/tools) and
  `--q-from` (hold quality fixed across conditions) added. **Negative control:** re-injected the old
  squoosh call and watched the guard fail the run end-to-end, reproducing the exact poisoned published
  cell. Re-ran run1/run2 at an IDENTICAL `--runs 3` config + run3 per-core via `--q-from`; dimension check
  PASSED on all three; determinism 141/141, wall-time drift median 1.6% / max 19.6% (was overclaimed as
  "≤~2%"). **Tally flipped: sharp 4 / IM 2 / squoosh 2** (aspect-correct squoosh is much smaller and now
  wins two photos); crustyimg's worst case vs the smallest widened 1.5×→1.7×; headline unchanged (neither
  smallest nor fastest; per core still faster on 4 of 8, now at margins of 12–45% with scores shown).
  DEC-080 calibration corrected: `web` is byte-identical to `convert -q 80` (md5-verified), lands median
  **75.2** not 79–82, so the 82 band is re-justified as a deliberate tune-up *away* from crustyimg's home
  setting. Also re-measured the resampler claim — 91–94 on three photos but **~82 for sharp on one**, so
  own-reference scoring is load-bearing, not a formality. Prose fixed: 3–9×/4–14×, quality-readout claim
  narrowed to SSIMULACRA2, 79.0–83.6, exact commands match the harness, README "about one to five seconds".
  Every published cell mechanically cross-checked against the fresh JSON (47+18+8, all match).
  `just validate` green (225 blocks), no `src/` change. **Handed back to re-verify — NOT merged.**
- [x] re-verify — Opus, 2026-07-21 on `spec-083-honest-benchmarks` @ a0954cc. **⚠ PUNCH LIST (3) — numbers
  CLEAN, prose is not.** Ran an INDEPENDENT three-pass benchmark and re-derived everything from it: all 157
  deterministic cells match the doc exactly (per-photo, bucket, per-core), tally **sharp 4 / IM 2 / squoosh 2**
  and per-core **4 of 8** recomputed, determinism 141/141 (fresh drift median 1.1% / max 11.1% — the doc's
  1.6% / 19.6% is the more conservative report of its own runs). Against the fix cycle's own JSONs all 230
  cells match INCLUDING wall-times → no hand-edited numbers. **The guard is real:** `--self-test` 8/8, and
  two end-to-end negative controls — the shipped squoosh both-axes bug AND an independent `sharp --fit fill`
  distortion — both exit 3 with the row flagged, while the matching unpatched runs exit 0; the poisoned row
  still scored 81.2 in-band, which is the whole argument for the guard. No false positive on an EXIF
  Orientation=6 source (`web` transposes; six-tool portrait run exits 0). Old portrait args reproduce
  2048×3068 for sharp and cwebp; documented args give 1367×2048. 12 sampled encodes reproduce byte- and
  score-exact **from the doc's own command block**; squoosh outputs are 2048×1367 and DSC_9952 is 21,464 B at
  81.85 (matched quality, not distortion). `web` == `convert --format avif -q 80` md5-identical on **all 8**
  photos (never `-q 85`), median 75.17. sharp's 24 MP resampler outlier reproduced: 81.8/81.9/80.8 vs the
  others' 91.6–94.2. **Findings (all doc prose, no re-measurement needed):** (1) **"ImageMagick refused the
  47 MP Leica outright" is false** — magick encodes that file to AVIF fine (rc 0, 2048×1367, 116,462 B), and
  to JPEG/TIFF/PPM; only the PNG write fails, and that PNG is the harness's *scoring reference*, not the
  benchmarked pipeline (the source's sRGB/Linotronic ICC trips magick's PNG writer; `-strip` fixes it). The
  doc's one competitor-robustness dig is unearned and trivially disproved by any reader. (2) "cwebp is larger
  than every AVIF tool here" (twice) is contradicted by the doc's OWN table — cwebp beats ImageMagick on
  DSC_2011 (166 vs 167 KB) and DSC_9952 (65 vs 105 KB); the true claim is 1.22–3.03× the *smallest* AVIF.
  (3) resampler range "92–94" vs DEC-080's "91–94"; measured 90.9–94.5. Nits: "`web` and `optimize` report
  [the score] as part of the encode" — `optimize` is score-free without `--verify` (per cli-reference) and
  `web -o FILE` prints nothing; "it's the slowest" holds on all bucket medians and 7 of 8 photos (squoosh is
  slower on the 47 MP). `just validate` green (225), `--self-test` green, no `src/` change, cycle `verify`.
- [x] build (prose) — Opus, 2026-07-21 on `spec-083-honest-benchmarks`. **Wording pass only — nothing
  re-measured, no number changed, no table cell moved** (diff is prose in BENCHMARKS.md, DEC-080, the spec,
  this timeline, and one harness docstring line). (1) The ImageMagick caveat was false and any reader with
  magick could disprove it: reproduced here that `magick L1024678.JPG -resize '2048x2048>' -quality 70
  out.avif` succeeds and only the **PNG reference** write fails (the source's ICC profile trips magick's PNG
  writer; `-strip` fixes it) — restated in all three places as a limit of OUR scoring method, with the
  "less tolerant of odd inputs" dig and the "the others read it without complaint" contrast deleted.
  (2) "cwebp is larger than every AVIF tool here" (twice) → the true, table-checkable claim: ~1.2×–3.0× the
  *smallest* AVIF on every photo + the largest median in all three buckets, and the doc now says outright
  that cwebp beats ImageMagick's AVIF on two 24 MP photos. (3) One resampler range everywhere: **90.9–94.5**
  (was 92–94 in the doc, 91–94 in DEC-080). Nits: score-readout narrowed to what actually prints (verified:
  `web -o FILE` prints nothing, `--out-dir` prints `· ssim`; `optimize` needs `--verify`); "it's the
  slowest" → all three bucket medians and 7 of 8 photos; the harness's two-process claim is now disclosed
  in BENCHMARKS.md and the docstring cites only that. `just validate` green, `--self-test` green, no `src/`
  change. **Handed back for a short prose-only re-verify — NOT merged.**
- [x] re-verify (prose) — Opus, 2026-07-21 on `spec-083-honest-benchmarks` @ 2413557. **All three prose fixes
  PASS; a NEW substantive finding surfaced outside the prose scope — ⚠ back to build.** Prose scope, all
  independently re-derived: no number moved (44/44 table lines byte-identical by md5; the only numeric-token
  deltas in BENCHMARKS.md are 92/94 → 90.9/94.5 plus the prose counts 7-of-8, 47 MP, 24 MP; DEC-080 only
  91/94 → 90.9/94.5; the harness only drops a `DEC-080` citation). F1 reproduced first-hand — `magick
  L1024678.JPG -resize '2048x2048>' -quality 70 out.avif` → rc 0, valid `ftypavif` 2048×1367 (sips + magick
  agree), `... ref.png` → rc 1 "Incorrect data in iCCP" leaving a 33-byte IHDR stub, `-strip` fixes it;
  `ImageMagick.ref_pipeline()` does write `ref.png` and `bench_tool` bails at the reference step before the
  encode grid, so the gap really is this method's. Zero residue of the deleted claims in BENCHMARKS.md.
  F2 checked against the run JSON: cwebp is 1.215–3.034× the smallest AVIF (raw bytes) and under IM on
  exactly DSC_2011 + DSC_9952. F3 consistent in all three docs. All 39 size cells re-matched `run1.json`.
  **FINDING (F4, pre-existing, not introduced here): the `_crustyimg web (default)_` rows do not measure
  `web`'s default.** `crustyimg web IN -o out.avif` pins the format via the `-o` extension, which skips the
  auto-decision and falls back to `convert`'s `AVIF_DEFAULT_QUALITY` = 80; the real default path
  (`web IN`, or `--out-dir`) uses `FAST_LOSSY_QUALITY` = 85 — the constant whose own doc-comment says
  `convert`'s default is what "the fast path never uses". Proven byte-identically on 4 photos: pinned ==
  `convert -q 80`, default == `convert -q 85`, same 2048×1367 dims. The harness's `CrustyimgWeb.enc_pipeline`
  uses `-o web.avif`, so every `web` row is the pinned q80 path: DSC_9952 28603 B @ 78.95 (doc) vs 36791 B
  @ 81.6 (actual default); DSC_2011 70524 vs 98599; DSCF1154 61753 vs 83275; L1024678 46698 vs 63680 —
  29–40% larger, ~4–5 SSIMULACRA2 points higher. This undercuts the doc's "web lands at **75** median, a real
  notch below the 82 band" narrative and the "`web`'s preset is byte-for-byte `convert --format avif -q 80`"
  line (that equality is the *symptom* of the pin, not web's preset), and it makes the freshly-corrected
  score bullet point readers at `--out-dir` — the form that does NOT produce the benchmarked bytes. Note
  DEC-080's original calibration (`-q 85`, 79–82) was right and was "corrected" against the pinned
  measurement. Two verify passes missed it because both graded the doc against the harness's own run JSONs —
  a control that shares the defect with the artifact ([[a-self-referential-control-cannot-detect-a-broken-pipeline]]).
  Gates: `just validate` green (225), `--self-test` green (8/8), no `src/` change, cycle `verify`, demo
  favicons untracked and untouched. **NOT merged.**
- [x] build (fix #3) — Opus, 2026-07-21 on `spec-083-honest-benchmarks`. Cleared **F4**: the harness's
  `CrustyimgWeb.enc_pipeline` now runs `web IN --max E --out-dir D` instead of `-o web.avif`, so the
  `web` rows measure `web`'s actual default. **Blast radius established BEFORE any change, then confirmed
  rather than assumed:** the iso-quality grid sets quality *explicitly* (`convert --format avif -q Q`), so
  `unwrap_or(AVIF_DEFAULT_QUALITY)` never fires and there is no auto-decision to skip — and a full grid
  re-run reproduced **all 8 published rows exactly** (bytes + score). So the tally (sharp 4 / IM 2 /
  squoosh 2), the per-core table and every competitor row STAND; only the three `web` bucket rows moved.
  Byte-proved the mechanism by hand first (md5: `web -o FILE` == `convert -q 80` = 28603 B @ 78.95;
  `web --out-dir` == `convert -q 85` = 36791 B @ 81.64). **New structural control — the operating-point
  guard**, sibling to the dimension guard: a row claiming a tool's fixed default must prove it two ways,
  exit 3 on either — STATIC (the timed command carries no format-pinning `-o`/`--format`) and OBSERVED
  (`web --json`, the engine's own decision report, shows the claimed quality + format). `--self-test` 8 →
  18 cases; guard proven by an end-to-end negative control (re-injected the pinned call into a copy →
  reproduced the published wrong number 29 KB @ 79.0, flagged the row, **exit 3**; fixed harness exit 0).
  Re-measured web rows (`--runs 3`, both guards PASSED, all 8 at q=85/AVIF): small `81.6 · 203 KB · 86.1%
  · 1090 ms`, medium `80.2 · 182 KB · 88.8% · 4685 ms`, large `80.2 · 64 KB · 99.3% · 2791 ms`; corpus
  median **80.8** (75.4–83.1), median 97% smaller. **DEC-080's calibration reverted to the truth** with the
  double-correction trail recorded (original `-q 85` was right; the "correction" to `-q 80` / 75.2 measured
  the pinned path) and the ~7-point "we handicapped crustyimg" narrative **withdrawn** — the band is ~1
  point above web's default, and on **4 of 8 photos the grid picks web's own q85**, making the tuned row
  and the default row the same encode byte-for-byte. Deleted the "byte-for-byte `convert --format avif
  -q 80`" line; relabelled the exact-commands comment; ImageMagick lead → "often the least size-efficient"
  (the doc's own table bolds IM smallest on IMG_3855 + DSCN3478). Fresh numbers corroborated three
  independent ways: DEC-069's pre-existing q85 table (DSC_0163 81.6 vs 81.59 measured), `scripts/bench.py`
  (a different harness, always `--out-dir`) byte-identical on all 8, and the 4 byte-identical grid/web rows.
  **⚠ Out of scope, flagged not fixed:** `README.md:39`'s "median 98% smaller" measures **97%** today via
  its own cited harness — pre-existing from SPEC-082, NOT a pin artifact (`bench.py` always used
  `--out-dir`); maintainer call. Gates: `just validate` green (225), `--self-test` green (18/18), no `src/`
  change, demo favicons untracked and untouched. **NOT merged.**
- [x] build (fix #4) — Opus, 2026-07-21 on `spec-083-honest-benchmarks`. Cleared re-verify #3's **F5 + F6**;
  that re-verify read the *guards* rather than the numbers, re-drove every published cell by hand and found
  them clean, so **no number moved this pass**. **F5 — the operating-point guard did not cover what three
  documents said it covered** (`scripts/bench-compare.py`, `DEC-080`, this spec's fourth-pass entry).
  `pinning_arg()` matched whole tokens, so clap's attached spellings
  (`--format=avif`, `--output=x.avif`) walked past the STATIC half; and the OBSERVED half could not
  compensate, because `observe_operating_point()` issues its **own separate** `web --out-dir --json` probe —
  it describes the engine's default, not the row's encode. The disproof was already written three lines
  under the claim ("only the static one caught it"). Reproduced first: `--format=avif` injected into the
  shipped harness → **exit 0**, zero violations, publishing 202,492 B @ 75.84 labelled q=85 (observed).
  **Primary fix — the observation must be ABOUT the row:** `check_operating_point` now asserts
  `observed["bytes"] == out_bytes`. The encoder is deterministic on a fixed source + bound, so agreeing
  bytes mean the probe and the measured encode took the same path — spelling-independent, and it needs no
  list of flags to enumerate. Secondary: `pinning_arg()` reads attached spellings (fails earlier, names the
  flag). **Coverage proven, not asserted** — same injection, three variants: byte tie ALONE (`pinning_arg`
  deliberately left whole-token) → **exit 3** ("shipped 281,617 B, row publishes 202,492"); both fixes →
  **exit 3**; both fixes with no injection over the full 8-photo corpus → **exit 0**, no false positive.
  `--self-test` 18 → **24** (both attached spellings, a byte mismatch, a report with no byte count at all →
  fails closed, plus positive controls). Five surfaces RESTATED, three of which had CARRIED the overclaim:
  `scripts/bench-compare.py` (docstring + guard comment), `DEC-080`, and the spec's fourth-pass entry. The
  other two never made the claim — `BENCHMARKS.md`'s reader-facing sentence described only the observed
  half, and the `justfile` recipe comment described only the dimension guard, never mentioning the second
  one — and were brought up to the corrected wording alongside them.
  **F6 — `README.md:39` 98% → 97%** (maintainer's call), re-confirmed here from the control run's own
  JSON: `82.1/86.1/93.2/95.4/98.7/99.3/99.6/99.7`, median
  **97.05%**; `BENCHMARKS.md:264` already said 97%, so the branch was shipping two numbers for one corpus +
  command. Pre-existing from SPEC-082, corrected opportunistically. **No published number moved:** the
  control run reproduces all three `web` bucket rows exactly (small `81.6 · 203 KB · 86.1%`, medium
  `80.2 · 182 KB · 88.8%`, large `80.2 · 64 KB · 99.3%`) with `observed["bytes"] == out_bytes` on all 8
  photos. Gates: `just validate` green, `--self-test` green (24/24), no `src/` change, demo favicons
  untracked and untouched. **NOT merged.**
- [x] build (fix #5) — Opus, 2026-07-21 on `spec-083-honest-benchmarks`. Cleared re-verify #4's **F7 + F8 +
  F9**, all three about how the last pass DESCRIBED itself: guards and numbers were confirmed, so this pass
  is **prose only** — no code, no number, no benchmark run, tables byte-identical to `23b1206`.
  **F7 — the coverage claim cited the wrong row.** "That middle row is the point" pointed at the *both
  fixes* variant, where the static half also fires; the row that isolates the byte tie is the **first** one,
  where `pinning_arg` was deliberately blinded. Repointed — and re-anchored on a better exhibit than either:
  re-verify #4's **v10**, an attached-short `-o<path>` (`-oout.avif`) that `pinning_arg()` returns `None`
  for (no `=` in the token, so it never matches `-o`; re-confirmed here by importing the shipped module and
  calling it), caught by the byte tie on the **shipped** harness with nothing blinded. The claim demonstrated
  rather than staged. **F8 — "five documents said it covered" was itself an unchecked count.** Five surfaces
  were RESTATED; **three** had CARRIED the claim (`scripts/bench-compare.py`, `DEC-080`, the spec's
  fourth-pass entry) — checked at `cf99eb3`, not recalled: `BENCHMARKS.md` described only the observed half
  and the `justfile` never mentioned the operating-point guard at all. The entry above said "five" in one
  bullet and disproved itself two lines later; both halves now state the split, agreeing with the spec's own
  F5 header. **F9 — `BENCHMARKS.md` advertised a reach the static half lacks:** "checked for anything that
  would pin the format" → "checked for a format-pinning `-o`/`--format`", so the generality rests on the
  byte tie, where v10 shows it lives. **One volunteered beyond the punch list:** the `justfile` said "no
  format-pinning **flag**" — the same overreach one word wide — bounded to `-o`/`--format`; a repo-wide grep
  for both phrasings and for "five surfaces"/"five documents" finds nothing else. Gates: `just validate`
  green, `--self-test` green (24/24), `git diff -- src/` empty, every `|` table line in `BENCHMARKS.md` +
  `README.md` byte-identical to the prior commit, demo favicons untracked and untouched. **NOT merged.**
- [x] ship — squash-merged PR #108 (**0465c67**) 2026-07-21, CI CLEAN. `BENCHMARKS.md` +
  `scripts/bench-compare.py`/`just bench-compare` + `DEC-080` + a README link now on `main`;
  README's stale "98% smaller" corrected to the measured 97%. **HONEST HEADLINE SHIPPED: crustyimg
  is neither the smallest nor the fastest** (sharp 4 / IM 2 / squoosh 2 on size, crustyimg 0 but
  within ~1.7× on every photo; 3–9× and 4–14× slower on the clock) — **and per core it's a wash
  (faster on 4 of 8), so the gap is threading, not the encoder.** `web`'s default lands 80.8 median
  and shrinks these photos ~97%. Two exit-3 guards (dimension + operating-point) + `--self-test`
  24/24 keep it honest. **Cost ~$40.5 / 12 sessions** (5 build passes, 4 verify passes + a
  reconstructed re-verify entry that had been missing from the ledger). Ship mechanics: one verify
  commit lacked `-s` → `git rebase --signoff main` (content byte-identical); one cargo-deny failure
  was a Docker Hub image-pull timeout, not a policy violation — cleared by re-running the job
  (same SHA had passed cargo-deny in a duplicate run). **Five real defects caught across the
  cycles** — squoosh benchmarked squashed on 6 of 8 photos, portrait mis-sizing, per-core rows that
  weren't iso-quality, `web` rows measuring the wrong operating point, and a guard advertising reach
  it lacked. Four were invisible to number-checking; driving the CLI by hand out-of-band found them.
  Lessons banked: [[a-guards-advertised-reach-is-a-claim]], [[documentation-has-no-green]],
  [[a-number-from-an-unproven-path-is-not-a-measurement]],
  [[a-self-referential-control-cannot-detect-a-broken-pipeline]].
