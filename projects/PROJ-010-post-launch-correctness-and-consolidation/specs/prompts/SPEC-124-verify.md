# SPEC-124 — VERIFY prompt

Cycle: **verify**. New session, **read-only**, detached worktree. You did not build this.

**What it claims:** every AVIF encode arm — `sink::encode_to_bytes_with` **and**
`quality::encode_candidate_bytes_with` — now calls `.with_num_threads(Some(AVIF_TILE_THREADS))`
with `AVIF_TILE_THREADS = 1`, so `ravif` never reaches
`threads.unwrap_or_else(current_num_threads)`'s fallback and the AV1 tile count stops being the
machine's core count (DEC-094). **N = 1 was measured, not assumed** — four axes, in DEC-096.

**PR #184**, branch `fix/spec-124-pin-the-avif-encoder-tile-count`, head **`2e77269`**. **DEC-096**
is new (the id was reserved in the build prompt, not minted by `next_id`). Cycle is at `verify`.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec124-verify 2e77269
```

**Make no commits.** Emit your `## Cost readout` and verdict in the return message — that is the
deliverable (AGENTS §13).

## Read in order

1. **The spec** — 9 ACs, 5 design calls (Call 2 deliberately unsettled), 2 failing tests, and
   `## Build Completion` with **three deviations and five follow-ups**.
2. **DEC-096** — the whole measurement. **DEC-094** — the record it acts on (⚠ read the
   *corrected* leg F: it bounds a band ~13–20, it does not identify a point).
3. `src/sink/mod.rs`, `src/quality/mod.rs`, `tests/avif_tile_pin.rs`,
   `examples/spec124_tile_count_probe.rs`, and STAGE-042's backlog.

---

## Already settled by the orchestrator — do NOT re-derive these

**Three things are checked. Spending on them is waste.**

1. **The build's own cost is re-derived and correct-as-of-its-snapshot.** `$53.89` prices exactly
   at Sonnet anchors over the first **479** usage-bearing messages; the session ran to **503**, so
   the true full-session figure is **$57.74** (162,602,599 tokens). The orchestrator applies the
   corrected figure at ship. **Measure your own cost; do not audit the build's.**
2. **`implementer: claude-sonnet-5` is correct.** All 503 messages report `claude-sonnet-5`. The
   build prompt's "You are Opus" was wrong and the build was right to correct the field.
3. **SPEC-122's linear-light CHANGELOG entry is genuinely missing** (build follow-up #2 — confirmed:
   no `linear`/`gamma`/`sRGB` string anywhere in `CHANGELOG.md`). **Not this spec's diff. The
   orchestrator owns it before the tag.** Do not fix it, do not count it against SPEC-124.

---

## Six specific things

### 1 — ⚠ The build's AC-9 CI claim covers a SHA that is no longer the head

AC-9 says "all 16 real checks pass." That was read at run `32441039059` on commit **`79d8615`**. The
build then pushed **`2e77269`** (the cost-session commit) and **never re-read CI**. The fresh run on
the true head has a **failing `pages / build + browser smoke` leg**.

**Orchestrator ruling on the failure itself: infrastructure flake, not this diff.** Evidence, so you
can check the reasoning rather than repeat the work: `2e77269` is **docs-only** (spec markdown, no
`src/`, no `tests/`); the log shows the wasm build, `demo-assemble`, and the local server all
**succeeding**, and the failure is `demo-smoke: headless Chrome never came up (no
DevToolsActivePort)`; the same job passed on this same branch 23 minutes earlier and on `main` at
23:03.

**Your job is the part the build skipped: read CI at `2e77269` individually, and confirm the legs
that were still pending went green** — the 3-OS `build/test/clippy/fmt` matrix, `webp-lossy`, and
both `heic` legs. **Do not `--watch`-poll them**; take one snapshot. If `pages` is still the only
red, say so and treat it as non-blocking; if anything else is red, that is a punch-list item.

### 2 — ⚡ AC-1's test is GREEN through a full revert, by design. Drive the control that isn't.

DEC-096 states it openly: `both_encode_paths_set_the_thread_count` asserts **lockstep**, so removing
the pin from **both** arms leaves it green — a symmetric absence is still lockstep. The build says
`avif_output_is_identical_across_ambient_core_counts` is what covers that case, and that a full
revert turns it **red with a five-way hash spread**.

**That is the single load-bearing control in this spec and you must drive it yourself**, not read
the claim. One revert per independent condition (AGENTS §15): sink-only → red, quality-only → red,
**both → red on AC-2's test**. The behavioural flip is the evidence, not a hash.

⚠ And check AC-1's test can go red *for the reason it exists*: it compares `sink::encode_to_bytes_with`'s
emitted length against `choice.score` from the byte-budget search. **Confirm `choice.score` is
actually the probe's byte length at `choice.quality`** — if it is anything else, the assertion is
green for the wrong reason.

### 3 — AC-2's test shells out to `cargo build` from inside a `#[test]`

`probe_binary()` builds a whole `--features image/rayon` binary in a `OnceLock`, in a PID-named
`CARGO_TARGET_DIR`. The reasoning for needing a probe at all is sound and I am not asking you to
overturn it (see item 4). **The cost and reach are what need a ruling:**

- `avif` is a **default** feature, so this test runs in the default matrix on **all three OSes**,
  every PR. What does it add to CI wall clock? Did it work on **Windows**?
- It makes `cargo test` require `cargo` on `PATH` and a **writable source tree with the full
  workspace**. A downstream consumer or distro packager running `cargo test` from the published
  crates.io tarball is the case to think about — crustyimg is a **published library**.
- Nested cargo: is there a lock-contention or `CARGO_TARGET_DIR` interaction with the outer test
  run? ([[concurrent-differently-featured-builds-corrupt-a-shared-target-dir]] is why it isolates
  the dir — confirm the isolation is complete, including on a `-j`-parallel test run where two
  tests could enter `get_or_init` from different processes.)
- Is the PID-named temp dir ever **cleaned up**?

**Rule it: acceptable as shipped, acceptable with a follow-up filed, or a punch-list item.**

### 4 — Deviation #2 reinterpreted an explicit guardrail. Check the reasoning, not the outcome.

The build prompt said *"`scripts/spec123_avif_thread_determinism.py` is committed and reproduces.
AC-2 reuses it. Do not write a second harness."* The build wrote a Rust test instead, and argued:
the Python harness's legs vary `RAYON_NUM_THREADS`/`--jobs` against the **shipped** (non-`threading`)
binary, which DEC-094 already proved is an **inert lever** — so extending only that harness yields a
test that is green whether or not the pin exists, and AC-5 would have nothing to flip red.

**If that argument is correct, the build was right to deviate and this is the good kind of
deviation — flagged, not silent.** Verify it against DEC-094's actual legs. Then check the
consequence nobody addressed: **is `scripts/spec123_avif_thread_determinism.py` now stale or
misleading** — does it still describe what it can and cannot discriminate?

### 5 — The N = 1 recommendation rests on one distinction. Check that distinction.

N **was** measured (§1–§4), and §4b reports a **counter-finding rather than burying it**: a
synthetic 24 MP maximally-busy gradient costs N=1 a repeatable **~17% serial-time regression**
(28.9/29.4/29.8s vs 24.2/24.3/24.5s) for +0.76% bytes. The recommendation survives only because a
**realistic** 24 MP proxy shows the effect vanishing (21.85/22.08 vs 21.78/21.87), and the argument
is *"crustyimg's corpus is real photographs, not synthetic noise."*

**That single sentence is what makes N=1 the answer instead of a trade.** Check it:

- Is §1's "flat within noise" honest? The deltas are ~0.005s on ~0.51s. Best-of-5 on how many runs,
  and is the sweep's spread smaller than its between-cell differences?
- The realistic 24 MP proxy is `photo_forest_cc0.jpg` **upscaled by crustyimg's own `resize
  --exact`** — and SPEC-122 just changed what `resize` does. Is an upscale of an 800×532 source a
  legitimate stand-in for a real 24 MP photograph, or does the upsampling itself smooth away exactly
  the high-frequency content that drove the synthetic result? **This is the weakest link in the
  chain; DEC-096's own "Revisit if" flags it. Say whether it holds.**
- §2 claims N=1 is the smallest output **at every N and content class with no exception**. Confirm
  the table supports the universal quantifier [[documentation-has-no-green]].
- §3's structural-safety claim rests on `rav1e`'s `MAX_TILE_WIDTH`/`MAX_TILE_AREA` clamping tiles
  **up** regardless of the request. That is a claim about a dependency's internals
  [[a-grep-of-src-cannot-see-a-dependencys-default]] — was it driven behaviourally (the 6000×4000
  encode) or only read?

### 6 — Call 5: an item was CLOSED. Confirm the closure, and that the remainder is readable.

STAGE-042's `[env]`-cannot-express-"same-machine" item is marked `[x]` and a narrower remainder
re-filed as `- [ ]`. **Run `just backlog` and read the new item back** — this failed three times last
session [[a-document-is-not-a-backlog-unless-tooling-reads-it]]. Then check the closure is honest:
SPEC-124 closes the **one measured** route to a same-arch differently-cored false positive. It does
**not** make output machine-independent in general — no other codec was measured. The stage text
says this; confirm it says it clearly enough that a future reader cannot over-read it.

---

## Also check

- **AC-6's migration** was driven manually with a temporary uncommitted version bump (SPEC-121's
  AC-8 precedent). Confirm `Cargo.toml` is genuinely **unchanged** in this diff and no stray bump
  survived.
- **Both arms actually in lockstep at the source level** — `sink`'s const is `pub`, `quality`'s is
  private, and **nothing links them at compile time** beyond a doc comment saying "MUST equal".
  (`pub` + `#[cfg(feature = "avif")]` matches `AVIF_SPEED`/`AVIF_DEFAULT_QUALITY` exactly, so the
  visibility itself is convention, not drift.) Is the behavioural test enough, or does this want a
  static assertion?
- **`examples/spec124_tile_count_probe.rs`** compiles under `--no-default-features --all-targets`
  (it has the `gen_avif_fixture.rs`-style no-op arm — confirm clippy's lean leg actually covers
  examples).
- **DEC-096's `affected_scope`** lists `src/sink/mod.rs`, `src/quality/mod.rs`, `src/build/lock.rs`.
  The diff does **not** touch `lock.rs`. Is including it right (the consequence lands there) or
  does it make `decisions-audit --changed` noisy?
- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref**, or a
  clean checkout reports "No changed files in scope" and exits 0 on a green that cannot go red.
- **Every file the diff touches is listed** in Build Completion (9 files — this is the check
  SPEC-122 failed by two).
- **Build follow-up #3** — STAGE-042's `Count:` line undercounts by one **pre-existing**. Confirm
  it predates this diff (`git show main:...`) rather than counting it against the build.
- **Build follow-up #5** — `gh pr checks --watch`'s summary line reported `451 pass / 0 fail / 223
  pending` and exited 0 while the direct snapshot disagreed. Worth confirming; it changes what every
  future prompt should say about reading CI.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge; do not bump the version.
- **⚡ NEVER POLL CI.** One snapshot, or `gh pr checks 184 --watch --interval 30` backgrounded and
  **left alone** — do not re-read a running watcher's output. Measured: ~$60 of SPEC-122's $103.60
  build went on polling. Take your cost reading **once, at the end**.
- **A green local matrix does not predict CI.** Your toolchain is the one installed; CI resolves
  `stable`.
- **Budget ~200 exchanges.** The build ran 503 messages against a ~150 budget.
- macOS has no `timeout(1)`. A piped command reports the **pipe's** exit code — redirect and read
  `$?`. zsh does **not** word-split unquoted parameters — use `while IFS= read -r`, and write
  `"${B}:path"`, never `$B:path`.
- **`just wasm-check` fails on this machine** (`rust-lld: Library not loaded: @rpath/libLLVM.dylib`)
  and reproduces on a clean `main` — pre-existing local toolchain gap, not this spec's. Don't chase.

## When you finish

1. **No commits.** 2. Emit `## Cost readout` (`cost-snippet.md`; price at the anchors
`.message.model` actually reports — this session's model, not the one this prompt names).
3. Verdict — ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED, with **item 2's negative-control result** and
**item 3's ruling on the in-test cargo build** stated explicitly.
