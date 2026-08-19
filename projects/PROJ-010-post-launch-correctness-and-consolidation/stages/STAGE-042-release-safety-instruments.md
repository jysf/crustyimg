---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-042                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-10
shipped_at: null

value_contribution:
  advances: >
    PROJ-010 fixed five defects. Four of them escaped the same way — they sat in a cell of a
    matrix nobody had enumerated, so every existing test was green and every one of them was
    found by a human driving the binary. This stage builds the instruments that would have
    caught them mechanically, and the signal that would have stopped the fixes sitting
    unreleased for two weeks afterwards.
  delivers:
    - "A conformance matrix derived from the code's own lists, so coverage extends when the product does"
    - "A release-lag signal, so fixed-but-unreleased cannot go unnoticed again"
    - "Guards that actually run: a wasm CI leg, and the two RELEASING.md steps this cut earned"
  explicitly_does_not:
    - "Chase the defects themselves — all five are fixed and shipped in 0.7.0"
    - "Gate STAGE-041. This protects the NEXT release, not the launch."
    - "Adopt STAGE-036's unsourced candidate list. Different stage, different standard of evidence."
---

# STAGE-042: release-safety instruments

## What This Stage Is

The stage that answers *"how do we make sure we don't release bugs like that again?"* with
instruments rather than intentions.

PROJ-010's five defects were not five unrelated mistakes. **Four of them escaped by the same
route**: a shipped asset crossed with a shipped entry point, in a cell nobody had enumerated.

| defect | the unenumerated cell |
|---|---|
| SPEC-111 — `build` cannot run a bundled recipe | bundled recipe × `build` target |
| SPEC-112 — `wasm::transform` cannot either | bundled recipe × the wasm entry point |
| SPEC-110 — seven verbs returned sideways images | orientation-bearing input × pixel-lane verb |
| SPEC-107 — a truncated JPEG succeeded silently | hostile input × decode path |

The fifth (SPEC-108's 18.5× blow-up) was a *parameter interaction* — verb × `--max` × content
class — which is a different instrument and one the repo now has, in SPEC-109's boundary
specimens. SPEC-107 likewise closed its own class with a committed hostile corpus. **The two
classes still uninstrumented are the first two rows**, and they are the two that shipped total
failures on documented paths.

## Why Now

- **The gap is measurable, and it is worse than it looks.** `bundled::names()` — the canonical
  list of what crustyimg ships — is called in **exactly one place in the entire repo**
  (`src/cli/common.rs:171`, building an error message). **No test iterates it.** All six test
  files that touch bundled recipes hardcode `"web"`, `"gallery"`, `"product"` as string literals.
  A fourth bundled recipe would land with zero coverage, silently, on every surface.
- **The same shape holds for verbs.** The verb loops that exist
  (`tests/hostile_inputs.rs:390,423`, `tests/cli.rs:223`) are hand-written subsets of 3–4 verbs.
  SPEC-110 fixed seven broken verbs; nothing stops verb fifteen from being added without
  orientation baking, which is precisely how those seven got there.
- **A guard that does not run is not a guard.** SPEC-112's verify found that **no CI leg runs
  `just wasm-test`** — `ci.yml` has no wasm32 step, and `pages.yml`'s browser smoke drives only
  the demo's markerless path. All 37 wasm tests execute on a maintainer's machine only.
- **And the multiplier was separate from all of it.** Every PROJ-010 fix sat on `main` for two
  weeks while `brew install` served the broken build. That is not a testing failure — it is a
  missing signal, and it is what turned four bugs into two weeks of shipping them.

## Success Criteria

- **A new bundled recipe cannot be added without being exercised on every entry point that
  accepts one** — because the test iterates `bundled::names()` rather than a literal list.
- **A new pixel-lane verb cannot be added without orientation coverage** — and the verb list the
  test iterates is asserted **exhaustive** against the source of truth, or the guard is just
  another subset that rots. [[a-guards-advertised-reach-is-a-claim]]
- Both are proven by **negative control**: adding a fake fourth recipe, and a fake verb, each
  turns the matrix RED without editing the test.
- `just wasm-test` runs in CI on every PR.
- `just status` surfaces release lag in a line, with a threshold that is a recorded decision
  rather than a guess.
- `RELEASING.md` gains the two steps this cut earned.

## Scope

### In scope
- The shipped-surface conformance matrix (recipes × entry points; verbs × orientation).
- The release-lag signal.
- A wasm32 CI leg.
- The two `RELEASING.md` additions.

### Explicitly out of scope
- **Re-fixing any of the five defects.** All shipped in 0.7.0.
- **STAGE-036's candidate list.** Those need provenance first; these have it.
- Any change to what the product does. Every item here is a guard, not a behaviour.
- A general coverage push. The matrix is targeted at a demonstrated escape route, not at a number.

## Spec Backlog

- [ ] (not yet written) — [S] **A `build` cache HIT swallows the truncated-JPEG warning.**
  `build_one` returns at `src/cli/build.rs:415` on `cache.lookup`, before the format-plan match
  at `:424`, so SPEC-116's emit is never reached on a hit: run 1 warns, run 2 is silent, and
  `--no-cache` warns every time. `apply` has no cache and warns always, so the two verbs
  disagree on the second run in a project. **Severity is genuinely limited** — `.crustyimg/` is
  gitignored so CI is always cold, a new truncated input changes the key and warns on first
  encounter, and DEC-085's wording is "every verb that **decodes** it" (a cache hit does not
  decode), so this is arguably consistent with the decision's letter. Found and driven by
  SPEC-116's verify, 2026-08-15.

- [ ] (not yet written) — [S] **`build`'s `Preserve` and `Pinned` arms never warn on a truncated
  JPEG.** They route through `encode_one` (`src/cli/common.rs`), which has no truncation check at
  all — so a truncated JPEG stays silent on those two arms, on `main` and after SPEC-116. Found
  and correctly reported (not fixed) by SPEC-116's build; AC-7 put it out of scope. Same class
  SPEC-116 closed for the Decide arm. Confirm it reproduces before speccing.

- [ ] **SPEC-118** (framed 2026-08-15) — **The shipped-surface conformance matrix.** Iterate
  `recipe::bundled::names()` across every entry point that accepts a recipe — `apply --recipe`,
  a `build` manifest target, and `wasm::transform` — and assert each runs and produces valid
  output for the requested format. Three recipes × three entry points is nine assertions today,
  and it extends by itself. **This single test would have caught SPEC-111 and SPEC-112 before
  either shipped.**
  The verb half needs one extra piece to be real: a `PIXEL_LANE_VERBS` list that a test asserts
  is **exhaustive** against `Commands` (`src/cli/mod.rs:229`), so adding a verb without
  classifying it fails the build rather than silently skipping coverage.
  Note the wasm leg only runs under `just wasm-test`, so this spec is coupled to the CI chore
  below — a matrix nobody runs is worth nothing. Complexity **M**.

- [ ] (not yet framed) — **The release-lag signal.** `just status` (and/or a CI job) reports when
  `main` has drifted from the last tag in a way users can feel: commits touching `src/**` since
  the tag, and how long a non-empty `[Unreleased]` has been sitting. Wants a **recorded
  threshold** — "N src commits or M days" — as a small DEC rather than a magic number, and it
  should be advisory, not a blocking gate (a red CI leg for "you haven't released lately" is the
  kind of alarm people learn to ignore). Complexity **S**.

- [ ] (chore) — **A wasm32 CI leg** running `just wasm-check` + `just wasm-test`. Currently filed
  as STAGE-038 item #8 and **moved here**, because it is the same thesis as this stage rather
  than housekeeping: a guard that does not run. The runner needs the `wasm32-unknown-unknown`
  target and `wasm-bindgen-test-runner` — [[probe-load-bearing-crates-at-design]] applies to the
  test *runner* for a new target. Complexity **S–M**.

- [x] (chore, done 2026-08-15) — **`npm publish` is the one link the build chain does not cover, so the unguarded
  path is shorter than the guarded one.** Raised by the maintainer 2026-08-10, immediately before
  publishing 0.7.0.

  The repo already makes unsafe paths *unreachable by chaining*, and says so explicitly:
  `wasm-npm-pkg: wasm-build` is documented as depending on the profiled build **"ON PURPOSE — the
  packaging step must never be reachable without going through the size profile (DEC-066), or the
  package silently ships a stock-profile .wasm, +109 KB on the wire"**, and `demo-build` likewise
  **"REFUSES a .wasm that did not come through the profiled build."**

  The chain is `wasm-build → wasm-npm-pkg → wasm-npm-smoke` — **and then it stops.** There is no
  publish recipe (`grep publish justfile` finds only comments). So the actual publish is
  `cd pkg && npm publish`, which runs **no** build, **no** size profile and **no** smoke test, and
  is *easier to type* than the safe route. `pkg/` is gitignored, so nothing ties the artifact to the
  current checkout: switching branches leaves it untouched, and the version guard in
  `wasm-npm-finalize.mjs` (which dies if `pkg.version != Cargo.toml version`) only runs at **build**
  time, never at publish time. A stale or wrong-branch artifact publishes silently, and npm
  publishes are effectively irreversible.

  **Fix: `wasm-npm-publish: wasm-npm-smoke`** — one more link, exactly the argument DEC-066 already
  makes one step earlier. This does **not** weaken SPEC-076's maintainer gate; the gate is *"a human
  decides to publish"*, not *"it must be typed as raw npm"*, and chaining strictly increases what
  runs before bytes leave the machine. npm's OTP prompt works fine inside a recipe. Worth also
  printing the resolved name@version and the git commit before the final step, so the maintainer
  confirms against something rather than nothing. Queued item #11. Complexity **S**.

- [ ] (chore) — **Two `RELEASING.md` steps, both earned by the 0.7.0 cut.** (a) Diff the CHANGELOG
  against the specs merged since the previous tag — 0.7.0's `[Unreleased]` section was written in
  advance and had **no entry for SPEC-112**, so the release would have shipped its headline fix
  silently had the roll not caught it. (b) Run `just wasm-test`, which no CI leg does. Complexity
  **S**.

- [x] **SPEC-123** (**shipped 2026-08-18**, PR #179, $60.33) — [S] **is AVIF output byte-deterministic across thread
  counts?** `build --frozen`, the lockfile's `hash` and the cache key all assume it; upstream
  gives no guarantee and rav1e has a filed nondeterminism bug. **Thread count is not a cache-key
  component and not in the lockfile's list of qualifiers.** Gates two roadmap items.

- [ ] (not yet written) — [S] **`run_pixel_op` is a serial for-loop, so six batch verbs do not
  parallelize at all.** `src/cli/ops.rs:421` (`for input in &all`). `build` and `apply --recipe`
  fan out over rayon; `convert`, `resize`, `thumbnail`, `auto-orient`, `edit` and `watermark` do
  not. **Filed as SPEC-091 follow-up #3 on 2026-07-18 and never done** — and it lived only in
  STAGE-030's backlog, which is `shipped`, so no command has surfaced it since.
  **It has a measured payoff:** SPEC-091 pinned AVIF decode to one thread (DEC-077) to escape the
  `re_rav1d` DisjointMut race, which cost a **~3.8× single-decode regression**; moving this loop
  to file-level rayon (DEC-006) reclaims that *without reopening the race*, because the pin stays.
  **It is also decision drift, not only perf.** `ops.rs:227`'s comment cites DEC-006 as the reason
  it is sequential — but DEC-006 says *"Batch work parallelizes with **rayon** data parallelism
  across input files (landed for `apply` in STAGE-005)"*. The parenthetical records where it landed
  first; it is not a limit. So the comment inverts its own citation and six verbs sit outside the
  decision. `decisions-audit` cannot catch this: it compares scope globs, not claims.
  **And it is user-visible: `--jobs` is silently ignored by all six.** `global.jobs` is read in
  exactly two places (`build.rs:661`, `optimize.rs:177`), so `crustyimg resize *.jpg -j 8` accepts
  the flag, warns nothing, and runs serially. The docs also disagree: `cli-reference.md:34` scopes
  it to *"`apply` batch"*, while **`docs/api-contract.md:33` promises "Parallel workers for batch"
  unscoped** — a contract line that is false for six verbs. Whatever the fix, the two docs must
  end up agreeing with the code and with each other.
  Perf, drift, and a false contract line — non-blocking. **Sequence after SPEC-123** — confirm file-level parallelism cannot
  perturb per-file output bytes before shipping it.
  ✅ **Gate cleared — SPEC-123 / DEC-094 (2026-08-17).** It cannot, *today*: the AVIF encoder never
  consults a rayon pool (`ravif` is built without `threading`), so pool size is invisible to it —
  `--jobs` 1/4/14 and `RAYON_NUM_THREADS` 1/4/14 all produced identical output hashes. ⚠ **The
  clearance is conditional on that feature staying off.** If the encoder-pin item above enables
  `image/rayon` for the encode speed-up, a scoped `--jobs` pool becomes an encoder parameter and
  this gate closes again — so pin `with_num_threads(Some(N))` in the same change.

- [x] **SPEC-124** (design 2026-08-18) — [S] **Pin the AVIF encoder's thread count so output does
  not depend on the ambient rayon pool.** ✅ **Maintainer ruled 2026-08-18: pin, and ride
  SPEC-121/122's wave** so users pay one lockfile migration rather than two. Blocked on SPEC-122
  merging; must ship before the next tag. `image/rayon` (the 5.7×/4.4× perf lever) stays a
  separate, later decision — and this spec is what makes it safe to take.
  `src/sink/mod.rs:679` constructs `AvifEncoder::new_with_speed_quality(..)` and **never calls
  `with_num_threads`**, so the encoder takes `image` 0.25.10's documented default — *"all threads
  in the default `rayon` thread pool"* (`codecs/avif/encoder.rs:89-91`).
  **That is not merely a scheduling detail.** `ravif` 0.13.0 `av1encoder.rs:651-655` computes
  `tiles = threads.min((w*h) / min_tile_size²)`, so the ambient count sets the **AV1 tile count**,
  and tile boundaries reset entropy-coding contexts — a different tile count is a different
  bitstream by construction, before rav1e's nondeterminism bug (#2781) is reached.
  ✅ **MEASURED — SPEC-123 / DEC-094 (2026-08-17). Two claims below were wrong; they are corrected
  in place rather than deleted, because the shape of the error is the useful part.**
  **The concrete exposure — corrected.** It is *not* the thread setting. `ravif` is compiled
  **without its `threading` feature** (reachable only through `image`'s `rayon` feature, which
  `avif = ["image/avif"]` does not enable), so it substitutes its own `rayoff` shim: the encode is
  **serial**, and `current_num_threads` is `std::thread::available_parallelism()` — the OS core
  count. So `crustyimg build -j 2` and `build -j 8` write **identical** bytes (measured: `--jobs`
  1/4/14 invariant; `RAYON_NUM_THREADS` 1/4/14 invariant across 18 cells), and `RAYON_NUM_THREADS`
  reaches every verb's **batch pool** but reaches the **encoder** on none of them. The real exposure
  is one step over: the pool size is neither a cache-key component (DEC-058) nor part of the
  lockfile's `[env]` — **and neither is the core count**, which is the thing the encoder actually
  reads. `src/build/lock.rs` treats an output-hash change under the **same** `env.target` as *a real
  regression*, so the tool would flag a **differently-cored machine** as drift.
  **Why a pin is attractive rather than just retracting the claim:** both terms of that `min` are
  machine-independent once `threads` is fixed, so `with_num_threads(Some(N))` makes tiling
  machine-independent. Unlike DEC-077's decode pin to one thread (a measured ~3.8× cost), N is a
  free parameter, so the encode *could* stay multi-threaded — but **the pin is not a performance
  win, and should not be sold as one.**
  ⚠ **"Today the encoder already takes every core" was wrong — measured `cpu/wall ≈ 0.99` on every
  shipped leg. The encode is serial, at core-count tiles: crustyimg pays the full multi-tile
  compression penalty and collects none of the parallelism.** That splits this item in two, and the
  halves are separable: **`image/rayon` is the performance lever** (measured 5.7× on the photo,
  4.4× on the graphic, at byte-identical output on a 14-core host, since the tile count does not
  move), **`with_num_threads(Some(N))` is the determinism lever** (and it changes every byte).
  Scope them as two decisions, not one.
  ⚠ **The real second axis is quality-per-byte, and it runs the other way.** A still image is one
  frame, so rav1e's parallelism here is tile-level: `threads` and `tiles` are the same knob
  (`:653-654`; `cfg.with_threads` at `:690` is only set when `threads` is `Some`). Tiles are coded
  independently, so more of them costs compression efficiency — ravif's own comment concedes it:
  *"AV1 needs all the CPU power you can give it, except when it'd create inefficiently tiny
  tiles."* **You cannot buy encode parallelism without spending quality-per-byte**, and today
  crustyimg picks maximum parallelism implicitly by taking the default.
  **Which means output quality-per-byte today varies with the machine's core count** — a 4-core
  laptop and a 32-core CI box plausibly differ, on a tool whose thesis is quality-per-byte.
  ✅ **CONFIRMED, and it is not small.** Driving the tile count directly (a `--features image/rayon`
  probe, DEC-094 leg E) against a 1-tile encode of the same input: **+1,497 B / +1.5 %** on
  `photo_forest_cc0.jpg`, **+412 B / +47.9 %** on `graphic_large.png` at 14 tiles. The proportional
  cost is far worse on small/graphic content — 14 tiles over 512×512 is exactly ravif's
  "inefficiently tiny tiles". The shipped binary sits at that end of the trade today.
  ✅ **Call 4 answered — a pin would be *sufficient*, not merely narrowing.** Run-to-run at a fixed
  count was stable over 10 repeats × 3 verbs (30 runs, 1 distinct hash each), so there is no
  residual nondeterminism underneath the tiling for a pin to leave behind. rav1e #2781 did not fire
  in any run here.
  ⚠ **It changes every AVIF output byte**, so if it goes ahead it should ride STAGE-046's
  byte-changing wave (SPEC-121/122) rather than land alone. **SPEC-123 has now reported — this is
  scopeable, against DEC-094's numbers.**

**Count:** 1 framed (SPEC-118) / 7 pending / 1 chore done

> **Nine framework-tooling items moved to STAGE-047 on 2026-08-16.** They were about `just`,
> `next_id`, the stage template, prompt budgets and squash-merge behaviour — the harness that
> builds crustyimg, not crustyimg or its release. Keeping them here diluted the item that
> actually blocks a maintainer ruling.

**Count:** 2 framed (SPEC-118, SPEC-123) / 6 pending / 1 chore done

> **The two `IMAGE_EXTENSIONS` items moved to STAGE-046 on 2026-08-16.** They are a defect —
> the tool silently processes less than the user handed it and reports success — not an
> instrument for catching one. They landed here only because they were filed while looking at
> instruments.

> **One follow-on candidate filed 2026-08-17**, from a design-time read of the encoder call site
> while writing SPEC-123's build prompt: crustyimg never pins the AVIF encoder's thread count, and
> that count sets AV1 tiling. Filed rather than acted on — it is gated on SPEC-123's verdict and
> wants a maintainer ruling on whether to pin or to retract the reproducibility language.


- [ ] (not yet written) — [S] **`[env]` cannot express "same machine", so `diff` reports a
  differently-cored host as a real regression.** SPEC-123's AC-7 deferral, filed here at verify
  because the build filed it in `docs/backlog.md`, **which no command reads** — `just backlog`
  reads this section.
  **The wrong text is not the one the build filed.** It filed the caveat list at
  `src/build/lock.rs:32-37`. The directly false statement is `:124-129` — *"`[env]` exists so
  `diff` can tell 'the encoder produced different bytes on this same machine'"* — when `env.target`
  is `{ARCH}-{OS}`, which **cannot** establish same-machine. Combined with `HashChangedSameEnv ⇒
  drift = true` **unconditionally** (`:459-466`, not even `strict`-gated), a same-arch host with a
  different core count is reported as a **real regression**. That is a live false positive in
  shipped code, not incomplete prose.
  **Severity, measured at verify:** not reachable in this repo's own CI — no committed
  `*.build.lock` exists and no workflow runs `build --check`/`--frozen`. It is a **user-facing**
  exposure only. Filed-not-fixed is the right call.
  ⚡ **The fix is NOT "add core count to `[env]`".** DEC-094's leg-F rider measured that core-count
  sensitivity is **quantized** — a 14-core and a 16-core host emit identical bytes; 8-core and
  14-core do not — so a raw core count would churn `[env]` between machines whose output agrees.
  Key on the resulting tile grid, or on nothing. **That is the maintainer call**, and it is why
  AC-7's literal "no `src/` diff" was the right reading: the correct fix was never a one-line
  comment edit.
  **Two shipped claims are true only while `ravif/threading` is off, and must be named conditional
  when this lands:** `README.md:258` (*"the round trip is byte-stable"*, printed beside a `-j 8`
  example) and `docs/USAGE.md:135` (*"`apply` replays it byte-identically across the directory, in
  parallel"*). Both break the day `image/rayon` is enabled — so this item and the encoder-pin item
  above move together.

- [ ] (not yet written) — [S] **The cache-key-changes-on-release safety net only fires on an
  actual version bump; a same-version code fix is invisible to it.** SPEC-121's AC-8 drive
  (2026-08-18): `cache_key_for` includes `crate::version()` (DEC-058), and Call 4's premise —
  "old and new renders cannot collide in the cache" — is TRUE **only when the version changes**.
  **Driven both ways on a real target:** built with `main`'s pre-fix binary, committed the
  lockfile, then ran the SPEC-121 branch binary (same `0.7.0`, unbumped, per its own "do not bump
  the version" guardrail) — `build --check` reported **"lockfile is up to date," exit 0**, and a
  plain `build` served the stale, pre-fix bytes from cache (`0 cached` became `1 cached, 0
  rebuilt`; `dist/photo.webp` stayed `rgba8`, not the branch's correct `rgb8`) with **zero
  warning**. Rebuilding the identical branch source at `0.7.1` instead: key changed
  (`b16f3ef6…` → `31d6ea01…`), `--check` failed **exit 7** with an explicit drift message, a plain
  `build` regenerated, and the on-disk output flipped to the correct `rgb8`. All four of AC-8's
  checks hold — conditionally.
  **So this wave's migration story is sound only if a version bump actually lands with it.**
  SPEC-124's stage note above ("must ship before the next tag") already assumes this; this item
  makes the reason concrete and measured rather than assumed. Between a tag and the next one, any
  number of behavior-changing specs can merge to `main` at an unchanged version — SPEC-121/122/124
  among them — so a user who builds from `main` mid-wave and freezes a lockfile gets a **silent**
  stale-cache hazard the moment the *next* same-version fix lands, not merely a hypothetical one.
  **Not a `src/` fix** — Call 4 explicitly forbids inventing cache-key machinery in SPEC-121; this
  is a process/release-discipline finding (bump before tagging, or thread a git-describe-style
  component into the key), and the choice is the maintainer's, matching the pattern set by the
  `[env]` same-machine item above.

- [ ] (not yet written) — [S] **Lossless WebP silently halves a >8-bit source, on the DEFAULT
  path, and reports `ssim 100.0` while doing it.** SPEC-121's Call 3 settled its warning scope as
  JPEG + lossy WebP only; `image`'s own *lossless* WebP encoder — no feature flag, always built —
  has no 16-bit mode either, so it takes the same "automatically convert the image to some color
  type supported by the encoder" path with no diagnostic.
  **Driven on the branch binary (2026-08-18), 32×32 16-bit RGB PNG:** `convert --format webp`
  prints `png → webp · 4791 → 686 B (86% smaller) · ssim 100.0` and the output round-trips as
  **8-bit** RGB. `web` reaches it too — `optimize`'s smallest-candidate search picks WebP for that
  fixture. The SSIM figure is computed on 8-bit renderings, so it cannot see the loss it is
  reporting on; **the honest-size line reads as reassurance for the one thing that did go wrong.**
  Same class as the JPEG/lossy-WebP gap SPEC-121 closed, and the same fix shape (one line at the
  sink) — but widening Call 3's scope to every 8-bit-only format (lossless WebP, GIF, BMP, ICO) is
  a design call SPEC-121 did not reopen.
  **Filed here rather than in `docs/backlog.md`** because no command reads that file
  ([[a-document-is-not-a-backlog-unless-tooling-reads-it]]) — `just backlog` reads this section.
  SPEC-121 recorded it only in DEC-095's Consequences prose, and a test comment cited a
  `docs/backlog.md` entry that was never written; both are corrected on that branch.

**Count:** 2 framed (SPEC-118, SPEC-124) / **1 shipped (SPEC-123)** / 9 pending / 1 chore done

## Design Notes

- **Derive the matrix from the code, never from a literal list.** This is the whole idea. A
  hand-written list of three recipes is exactly what the six existing test files already have,
  and it is why the gap existed. The test must fail when the *product* grows, not when someone
  remembers to update the test.
- **An exhaustiveness assertion is what separates this from another subset.** For recipes,
  `bundled::names()` gives it for free. For verbs it must be built: match on `Commands`
  non-exhaustively-forbidden, or assert list length against the enum. Without it the verb matrix
  is `tests/cli.rs:223` again — three verbs, hand-picked, green forever.
- **The repo already does this for documentation, and it works.** `tests/docs_ops.rs`,
  `tests/adoption_glue.rs` and `tests/demo_copy.rs` police doc claims mechanically, and
  `docs_ops.rs`'s own header states the principle better than this note can: prose that reads
  like a caveat while standing in for a check nobody runs. **This stage points that same
  instinct at surfaces instead of sentences.**
- **What this stage would NOT have caught, stated up front so it is not oversold:** SPEC-108's
  classifier blow-up (a parameter interaction — covered by SPEC-109's boundary specimens) and
  SPEC-107's truncated JPEG (hostile input — covered by its committed corpus). Two of five. The
  honest claim is that the repo would then have an instrument for four of the five escape routes
  it has actually experienced, not that it becomes defect-proof.
  [[a-guards-advertised-reach-is-a-claim]]
- **Sequenced after STAGE-041 by maintainer decision.** The launch is the time-sensitive item;
  this protects the next release, and nothing in it gates the current one.

## Dependencies

### Depends on
- STAGE-040 (shipped) — the defects and their escape routes are the evidence this stage is built
  from, and they are only fully documented after that close-out.

### Enables
- Every subsequent release. And, specifically, the confidence to cut one without a human
  re-driving the flagship paths by hand — which is what 0.7.0 required.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
