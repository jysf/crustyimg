# SPEC-124 — BUILD prompt

Cycle: **build**. The design calls are settled except one, which is deliberately left to you to
measure. You are **Opus** — the spec's `implementer` was updated to match this prompt, so price at
Opus anchors.

**One-line summary:** crustyimg never calls `with_num_threads`, so `ravif` derives the AV1 tile
count from `available_parallelism()`. Output therefore varies with the machine's core count, and the
shipped build pays a multi-tile compression penalty for tiles that buy **no parallelism at all** —
the encode is serial because `ravif` is compiled without its `threading` feature.

## Read in order

1. **The spec** — `.../specs/SPEC-124-pin-the-avif-encoder-tile-count.md`. **9 ACs, 5 design calls,
   2 failing tests.**
2. **DEC-094** — SPEC-123's measurement. ⚠ **Read the corrected version**: leg F was amended at
   verify and **bounds a band (~13–20), it does not identify a point**. Its Validation section was
   also corrected — the `cargo tree -e features` check is **not evidence**; the **build fingerprint**
   is (`features: []` shipped vs `["threading"]` probe).
3. **STAGE-042's two items** — the pin item (this spec) and the `[env]`-cannot-express-"same
   machine" item, which Call 5 says you may close.
4. `/AGENTS.md` §4, §12, §13, §15.

## ⚡ Call 2 — N is NOT settled. Measure it, then choose.

**The prior, and it is a prior, not a conclusion:** the encode is serial today, so tiles cost
compression and buy nothing — **N = 1 may be strictly better, same speed and materially smaller
files.**

**That is exactly the shape of reasoning that cost SPEC-123 $60.33.** Its design asserted a
mechanism from reading source and was wrong at a layer below where anyone looked. So drive all
three:

- **Wall clock, serial 1-tile vs serial N-tile.** Nobody has measured whether tile count affects
  *serial* encode time. If 1 tile is slower, the trade is real and N is a judgement call.
- **Compression at 1 tile**, re-measured on your branch — do not quote DEC-094's +1.5% / +47.9%.
- **The forward cost of N = 1.** If `image/rayon` is ever enabled, N = 1 means a single-threaded
  encode forever; a larger N preserves future parallelism at a known compression cost.

**Recommend a value, justify it from your numbers, and record the reasoning in the DEC.**

## The failure a partial fix introduces — AC-1

There are **two** encode sites and DEC-019/DEC-068 require them to stay in lockstep:

- `src/sink/mod.rs:717`
- `src/quality/mod.rs:411`

Pin one and not the other and the byte-budget search probes stop matching the bytes actually
written. `tests/avif_tile_pin.rs`'s `both_encode_paths_set_the_thread_count` exists for exactly
this — it asserts the **lockstep**, not just the pin.

## Call 3 — the quantization rider binds the test

`rav1e` quantizes a requested tile count to a legal grid, so a **range** of requests collapses to one
layout (DEC-094: graphic matched at every N ≥ 12, photo at 13–20). **A test asserting "N=14 and N=16
differ" would assert something false.** Pin the **observable** — output byte-identical across
differing ambient core counts — not the requested number.

## Call 5 — you may close a filed item; check rather than assume

Pinning removes the core-count variance, which is the mechanism behind STAGE-042's
`[env]`-cannot-express-"same machine" item. **Confirm that and close it, or state precisely what
remains.** ⚠ `lock.rs:124-129`'s prose is wrong on its own terms even once the variance is gone —
say so if it is still wrong.

## Reuse, do not rebuild

`scripts/spec123_avif_thread_determinism.py` is committed and reproduces. **AC-2 reuses it.** If you
need to extend it, extend it — do not write a second harness.

## Guardrails

- **Own git worktree:**

  ```
  git -C ~/PSeven/experiments/crustimg_redo_plus/crustyimg worktree add \
    ../crustyimg-spec124 -b fix/spec-124-pin-the-avif-encoder-tile-count main
  ```

- **Your DEC is DEC-096. The ID is reserved — do not run `next_id`**, which scans only the working
  tree and has already produced one collision here (SPEC-119 and SPEC-120 both minted DEC-092).
- **⚡ NEVER POLL CI, and do not re-read a backgrounded watcher's output while it runs.**
  `gh pr checks <PR> --watch --interval 30`, then leave it alone. **Measured: SPEC-122's build spent
  ~$60 of $103.60 on CI polling** because backgrounded watchers were repeatedly checked. Take the
  cost reading **once, after CI settles.**
- ⚠ **A green local matrix does not predict CI.** SPEC-122's punch list ran twelve checks and twelve
  exit-0s locally while CI failed all eight compile legs — stable floated to 1.98 and added a lint.
  Your local matrix runs the toolchain installed; CI resolves `stable`.
- **Budget ~150 exchanges.** Checkpoint and report past that.
- **Push a WIP as soon as it compiles**, before the matrix.
- macOS has no `timeout(1)`. `git commit -s`. A piped command reports the pipe's exit code —
  redirect and read `$?`. **Do not merge. Do not bump the version.**

## When you finish

1. Fill in `## Build Completion` including the reflection questions, and **list every file the diff
   touches** — SPEC-122's Deviations claimed "`src/operation` and `tests/` only" and was wrong by two
   `scripts/` files, which left `affected_scope` blind.
2. Append a build cost session (`cost-snippet.md`); price at the anchors `.message.model` reports.
3. Write **DEC-096** with `affected_scope` covering **both** encode sites.
4. `just advance-cycle SPEC-124 verify`, and **confirm it moved** with `git diff`.
5. Open the PR. **Do not merge it.**

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
