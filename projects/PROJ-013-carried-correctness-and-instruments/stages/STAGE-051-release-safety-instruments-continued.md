---
stage:
  id: STAGE-051
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-013
repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value_contribution:
  advances: >
    the instruments that catch a bad release before users do.
  delivers: []
  explicitly_does_not:
    - "Re-open the shipped stage it continues — that stage closed in place with its specs"
---

# STAGE-051: Release-Safety Instruments, Continued

## What This Stage Is

⚠ **A continuation, not a re-home.** The stage this carries — `STAGE-042-release-safety-instruments` —
**shipped in PROJ-010 and closes there**, because a stage with shipped specs closes in place rather
than moving. Only its **open** items came here.

**16 items carried, unchanged**, on 2026-08-23. Their evidence, measurements and rulings
came with them; nothing was summarised away.

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

- [ ] (not yet written) — [S] ⚡ **`display_sink_refuses_non_tty` passes in CI and FAILS in the
  maintainer's terminal — so the release gate suite is unrunnable as RELEASING.md tells you to run
  it.** Hit live during the 0.7.1 cut, 2026-08-22.

  `tests/sink.rs:437` asserts the Display sink always refuses, on the premise stated in its own
  comment: *"Under `cargo test` stdout is piped (non-tty)."* **That premise is false in an
  interactive terminal.** libtest captures what a test *prints*, but it does not reopen the
  process's fd 1, so `std::io::stdout().is_terminal()` (`src/sink/mod.rs:536`) is `true`, the sink
  **accepts**, and `.unwrap_err()` panics on `Ok(())`.

  **Driven, across two operators on the same commit and binary:**

  | stdout | result |
  |---|---|
  | a real TTY (maintainer, `cargo test`) | **FAILED** — `called Result::unwrap_err() on an Ok value` |
  | redirected to a file (CI, and `cargo test > log 2>&1`) | **passed** — 1 passed, exit 0 |

  ⚠ **This is worse than a test that simply fails.** It is green on every CI leg, so nothing
  upstream catches it, and it goes red exactly when a human follows the documented release
  procedure — which teaches the reader to distrust a red gate. **Pre-existing; not from this wave.**

  **Fix: make the test's condition true by construction rather than by assumption** — force a
  non-tty (or assert conditionally on `is_terminal()`, so the test is meaningful in *both*
  environments instead of accidentally correct in one). ⚠ **Do not "fix" it by deleting the
  assertion**: the `NotATty` guard is real behaviour worth pinning.
  📌 **Workaround until then:** run the gate suite with stdout redirected —
  `cargo test > /tmp/gate.log 2>&1; echo $?` — and redirect rather than pipe, since a pipe reports
  the pipe's exit code [[a-piped-command-reports-the-pipes-exit-code]].

- [ ] (chore) — ⚡ **`RELEASING.md`'s checklist is in an order that cannot be followed.** Hit live
  during the 0.7.1 cut, 2026-08-22. **Step 3 is `cargo publish --dry-run`; step 5 is "commit the
  release".** `cargo publish` refuses a dirty working tree —
  `error: 3 files in the working directory contain changes that were not yet committed into git:
  CHANGELOG.md, Cargo.lock, Cargo.toml` — which are exactly the three files steps 1, 2 and the
  `just release` shortcut just modified. **The dry-run cannot run at step 3, ever.**
  **Fix: swap steps 3 and 5.** Committing first is safe — the commit is not the irreversible act,
  the tag push is, and a failed dry-run or gate is answered by amending the commit. ⚠ **Do not
  "fix" it by adding `--allow-dirty`**: that verifies a tree different from the one being tagged,
  which is the opposite of what the step is for.
  📌 This checklist has been followed for every release to date, so either every previous cut
  silently deviated from it or the dry-run was skipped. Worth one look at which.

- [ ] (chore) — **Two `RELEASING.md` steps, both earned by the 0.7.0 cut.** (a) Diff the CHANGELOG
  against the specs merged since the previous tag — 0.7.0's `[Unreleased]` section was written in
  advance and had **no entry for SPEC-112**, so the release would have shipped its headline fix
  silently had the roll not caught it. (b) Run `just wasm-test`, which no CI leg does. Complexity
  **S**.

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

- [ ] (not yet written) — [S] **A multi-candidate search on a >8-bit source now prints one warning
  per attempted candidate, so the user cannot tell which downgrade actually shipped.** Disclosed in
  DEC-097 and correctly deferred (the real fix is a `solve_candidate` restructure, not a one-liner),
  but **verify measured two warning lines on a 16-bit photo through `web`**, so it is more visible
  than the record's "pre-existing" framing suggests. ⚠ Note what is NOT wrong here: at `f35e28a` the
  only warning printed on that same input was for **JPEG — the candidate that LOST** — while the
  AVIF downgrade that actually shipped stayed silent. The widened set is strictly better; it just
  now needs to say which line describes the file on disk.

- [ ] (not yet written) — [S] **The `/v1` JSON contract's "gated additive key" rule is precedent,
  never written down.** `crustyimg.optimize.explain/v1` already carries three keys added under this
  discipline — `larger_than_source` (DEC-075), `ssim` (SPEC-086), `timing` (SPEC-088) — and
  `ssim_source_depth` (SPEC-125) now makes four. Verify confirmed the pattern holds and that this
  repo treats a gated additive key as `/v1`-compatible, **but a downstream consumer reading
  `docs/api-contract.md` cannot learn that**: nothing states which kinds of change are compatible
  with a `/v1` pin. One paragraph. This is the repo's debt, not any one spec's.

- [ ] (not yet written) — [S] **`tests/avif_tile_pin.rs` builds a whole cargo binary inside a
  `#[test]`, and the costs are measured.** Verify ruled it **acceptable as shipped** — it is the only
  lever that discriminates a pinned build from an unpinned one (DEC-094 leg E), isolation is complete,
  and it passes on all three OSes. But three costs were measured and should be paid down deliberately:
  **(a) CI compute.** `avif` is a default feature, so the probe build runs in **7 job instances per
  PR**. Against a pre-SPEC-124 run on `main` (`2bd74b02`): windows 18m→20m, ubuntu 12m→14m50s, macos
  10m→14m, avif 13m→15m9s — roughly **+15–25 min of CI compute per PR**, on a repo whose changed-paths
  gate exists precisely to stop paying ~15 min for nothing.
  **(b) Nothing ever removes the probe dir.** It is **1.3 GB**, PID-named, and leaked once per test
  process. One verify session left **13 dirs / 16 GB** in `$TMPDIR`, reclaimed by the orchestrator
  2026-08-21 — so this is measured, not hypothetical. CI runners are ephemeral; a developer running
  `cargo test` is not. `tempfile::TempDir` or one `remove_dir_all` fixes it.
  **(c) `tests/` ships to crates.io.** `exclude` covers `/decisions /docs /projects /reports
  /guidance /feedback /scripts /.github /.claude` but **not `/tests`**, and `cargo package --list`
  confirms `tests/avif_tile_pin.rs` in the tarball. A downstream consumer or distro packager running
  `cargo test` therefore triggers a nested `cargo build --features image/rayon`, needing `cargo` on
  `PATH`, registry access, and a writable tree. crustyimg is a **published library**. Adding `/tests`
  to `exclude` is one line, but it is a packaging-policy call, not a mechanical fix.

- [ ] (not yet written) — [S] **Nothing with real 24 MP detail has ever been measured through the
  AVIF encoder, and one open finding depends on it.** SPEC-124's DEC-096 §4b measured a real,
  reproducible **~17 % serial-time regression at N=1** on a 6000×4000 fixture, then dismissed it as
  "a property of adversarial content." **Verify showed that dismissal is not established**: the
  "synthetic worst case" `(x%256, y%256, (x+y)%256)` is a deterministic sawtooth encoding at **0.157
  bpp**, and the "realistic" control is `photo_forest_cc0.jpg` upscaled 7.5× from 800×532, encoding
  at **0.22 bpp** against **1.86 bpp** for the same photograph at native size. Both legs are smooth
  images; neither carries 24 MP of real detail. **The cheap fix is a corpus one** — add a large
  real photograph to `bench/corpus/` and re-run §1 and §4b against it directly. Until then, N=1's
  large-input timing is an open question, not a closed caveat. ⚠ It does **not** threaten the pin:
  §2's compression win and §3's structural-safety argument are independent of it, and rav1e's own
  tiling clamp keeps N=1 bitstream-legal at any size.

- [ ] (not yet written) — [S] **`lock.rs:124-129`'s "same machine" prose is still wrong on its own
  terms, independent of any known mechanism.** The remainder of the item above: `env.target` is
  `"{ARCH}-{OS}"`, which cannot establish "this is the same machine" as a general claim even with
  SPEC-124's AVIF tile-count mechanism closed — it is an overclaim in the doc comment, not (as far as
  this repo has measured) a live false-positive path today. ⚡ **The fix is NOT "add core count to
  `[env]`".** DEC-094's leg-F rider measured that core-count sensitivity is **quantized** — a 14-core
  and a 16-core host emit identical bytes; 8-core and 14-core do not — so a raw core count would churn
  `[env]` between machines whose output agrees. Key on the resulting tile grid, or on nothing, or just
  correct the prose to what `[env]` actually establishes (arch/OS, not machine identity). **That is
  the maintainer call.**
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

- [ ] (not yet written) — [S] ⚡ **`image` 0.25.10's ICO encoder writes a file its own ICO decoder
  cannot read back — independent of bit depth, surfaced while deriving SPEC-125's depth-downgrade
  set (Call 1's behavioural measurement, not the spec's candidate list).** Measured for 8-bit RGB
  (no alpha), 8-bit RGBA, 16-bit RGB, and 16-bit RGBA: three of the four fail
  `image::load_from_memory_with_format(_, ImageFormat::Ico)` with `Format error decoding Ico: The
  PNG is not in RGBA format!`; only 8-bit RGBA round-trips. So `convert --format ico` succeeds
  (exit 0) and writes a file that **the very next `crustyimg info` on that same file cannot open**
  — for a plain opaque 8-bit RGB source with no alpha and no >8-bit depth anywhere in play, not
  only the >8-bit case SPEC-125 is about. `image`'s ICO encoder embeds a PNG sub-image at the
  source's own colour type (consistent with DEC-095's preserve policy — not itself wrong), but
  `image`'s own ICO decoder hard-requires that embedded PNG to be exactly `Rgba8`. Framing this as
  a depth-downgrade warning would misattribute it, so SPEC-125 deliberately does NOT warn for ICO
  (see DEC-097) — this item is the real defect, filed rather than fixed (a correct fix — forcing
  RGBA8 before ICO encode — would change output bytes for every non-alpha ICO source, out of scope
  for a reporting-only spec). `tests/sink.rs::ico_round_trip_defect_is_orthogonal_to_depth` pins the
  measurement; goes red if `image` ever fixes this upstream. Needs a maintainer ruling on whether
  to warn, fix (byte-changing), or accept as a known `image` limitation.

**Count:** 0 closed / **15 pending** — re-derived by grep 2026-08-23.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
