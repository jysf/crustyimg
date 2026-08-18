# SPEC-123 — VERIFY prompt

Cycle: **verify**. New session, clean checkout of the branch under review. You did not build this.

**What it claims:** AVIF output is invariant to every thread setting crustyimg exposes — because
`ravif` is compiled **without its `threading` feature**, so the encode is serial and the tile count
comes from `std::thread::available_parallelism()`. Call 3's **third** branch: *the encoder ignores
the thread setting*. Not "deterministic".

**PR #179**, branch `chore/spec-123-avif-byte-determinism`, head `7b3b130`. 6 commits, no `src/`
change, DEC-094, harness at `scripts/spec123_avif_thread_determinism.py`.

## ⚠ The surrounding documents contain orchestrator predictions that were WRONG. Twice.

Design predicted **non-deterministic by construction** from `ravif`'s
`tiles = threads.min((w*h)/min_tile_size²)`, and told the build to expect AC-6's correction sweep
to fire. It did not. Design also wrote *"the encoder already takes every core; a pin can only match
or reduce that"* — measured `cpu/wall ≈ 0.99`, i.e. **serial**.

Both errors have the same root: **a dep's documented default is a claim about a FEATURE SET.**
`image`'s doc comment (*"all threads in the default rayon thread pool"*) is true of `image` **with
`rayon` on**, and `avif = ["image/avif"]` does not enable it.

**So do not treat the spec's `## Inputs`, the build prompt, or STAGE-042's item as authority.** The
build corrected all three in place. Your job includes checking whether the *corrections* are right.

## Read in order

1. **The spec** — `.../specs/SPEC-123-avif-byte-determinism-across-thread-counts.md`, all 8 ACs
   and its `## Build Completion`.
2. **DEC-094** — `decisions/DEC-094-avif-thread-settings-never-reach-the-encoder-core-count-does.md`.
3. **The harness** — `scripts/spec123_avif_thread_determinism.py`.
4. **`docs/backlog.md`**'s determinism entry, and **STAGE-042**'s two amended items.
5. **`/AGENTS.md`** §4 cost, §12 testing, §13 git/PR, **§15** — including the three measured
   verify rules and *"an acceptance criterion may not transfer."*

## Verify the mechanism before anything downstream

Everything rests on one claim: **`ravif/threading` is off in the shipped build.** If that is wrong,
the verdict, both riders and the STAGE-042 split are all wrong together.

Re-derive it yourself, from the feature graph, not from the write-up:

- `image` 0.25.10 `Cargo.toml` — `avif = ["dep:ravif", "dep:rgb"]` against
  `rayon = ["dep:rayon", "ravif?/threading", "exr?/rayon"]`. Confirm `ravif?/threading` is reachable
  **only** through `image/rayon`.
- `ravif` 0.13.0 `Cargo.toml` — `default = ["asm", "threading"]`, so confirm how `image` declares
  the dep such that the default does not apply.
- Then confirm it **behaviourally**: `cpu/wall ≈ 1.0` on a shipped-config encode is the observable.

⚠ **`rayon` IS in the dependency graph** — `av-scenechange → rav1e → ravif` pulls it for rav1e's
own paths. Its mere presence proves nothing about `ravif/threading`. Do not stop at
`cargo tree -i rayon`.

## The controls — the build claims three. Check that each can fail.

1. **Positive control (leg E)** — a `--features image/rayon` probe that moves bytes (3 distinct
   hashes/input), clock (0.530 → 0.093 s) and cpu/wall (1.00 → 7.09).
   **Prove leg E actually built with threading on**, rather than being labelled so. A positive
   control that did not enable the variable is the same failure this spec exists to catch, one
   level up [[a-control-you-never-verified-applied-is-not-a-control]].
2. **In-process control** — `web`'s auto path at cpu/wall 1.17 with 14 threads vs 1.00 with one,
   over three runs. This is the sharp one: it shows the env var **reached the program but not the
   encoder**. Check the margin is real and not noise at n=3.
3. **Cross-check** — shipped bytes land exactly on the probe's **14-tile** point, on both inputs,
   **and no other count**. This is a positive identification of the tile count, so it is the
   strongest evidence in the package. **Confirm "no other count" was actually tested across the
   range**, not asserted from two points.

## Per-AC, with the two that need a ruling flagged

- **AC-1** — ≥3 thread counts, hashes reported not verdicts. 18 cells claimed.
- **AC-2** — the control fires. See above; this is the load-bearing criterion.
- **AC-3** — driven through the shipped binary on `convert --format avif`, `web`, `optimize`.
- **AC-4** — run-to-run stability at a fixed count. **10 repeats/verb claimed** — confirm.
  Design elevated this from "the cheap adjacent extra": it decides whether a pin would suffice.
- **AC-5** — verdict stated as exactly one of Call 3's three outcomes. It is branch 3.
- **AC-6 — ⚖ NEEDS YOUR RULING.** It reads *"if non-deterministic: every shipped 'reproducible'
  claim located and corrected."* The thread axis is **not** falsified, so on a literal reading AC-6
  does not fire. But rider (a) says **output varies with the machine's core count**, which is a
  cross-machine non-determinism the lockfile's caveat list does not cover. The build ran a sweep
  anyway (**30 hits**, self-corrected up from 26 counted by eye) and filed rather than corrected.
  **Rule explicitly: is the shipped language now false, misleading, or accurate?** Cite the grep and
  its scope — an unstated scope is an unverifiable claim [[mechanical-sweeps-need-a-mechanical-check]].
- **AC-7 — ⚖ NEEDS YOUR RULING, and it collides with AC-6.** AC-7 says `git diff` against `main`
  shows **no** `src/` change (confirmed: empty). But the correction rider (a) wants is to
  `src/build/lock.rs:32-37`'s **doc comment** — non-functional, yet still a `src/` diff. The build
  read AC-7 literally and filed it. **Was that right?** AGENTS §15 warns that an AC may not transfer
  to the surface it lands on; say whether AC-7 as written blocked a correction AC-6's spirit wanted,
  and whether the follow-up is filed somewhere tooling reads.
- **AC-8** — reproducible from the committed harness. **Re-run it** and confirm the numbers land in
  the same place. Note the harness is timing-sensitive; a loaded machine is a confound, not a
  finding.

## Also check

- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref.** A bare
  `--changed` on a clean checkout reports "no changed files" and exits 0; that green cannot go red.
- **DEC-094** — `affected_scope` correct for a documentation-only finding, confidence honest
  (AGENTS §17), and it does not conflate encode determinism with DEC-077's *decode* thread pin.
- **CI** — 16 legs reported green at **`69c0500`**, but head is **`7b3b130`** (a front-matter cost
  note). Confirm the head is green, and **read the legs individually** — a green summary is not a
  matrix [[a-green-gate-on-one-os-is-not-the-required-matrix]].
- **The two corrections to design's claims** — that the encode is serial, and that the pin item
  splits into `image/rayon` (performance) and `with_num_threads(Some(N))` (determinism). Both are
  now written into STAGE-042. Confirm they are right, since two documents already carried the
  wrong version.
- **Build reflection** answered honestly; `cost.sessions` carries design + build.

## Guardrails

- **Your worktree — detached, at the PR head, read-only.** Two other sessions have been live in
  this repo, so do not use the primary checkout:

  ```
  git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec123-verify 7b3b130
  ```

  **Detached is deliberate:** it makes it impossible to accidentally commit onto the branch you are
  reviewing. Confirm you are at `7b3b130` (`git rev-parse HEAD`) — that is the PR head, one commit
  past the `69c0500` the build reported CI against.
- **Do not merge the PR. Do not bump the version. Do not fix what you find** — a punch list is an
  output, not an edit.
- **Budget ~150 exchanges.** The build ran 318 and cost $46.17 against an S-sized $8.69 precedent.
- macOS has no `timeout(1)`. `git commit -s`. A piped command reports the pipe's exit code.

## When you finish, in this order

1. **MAKE NO COMMITS.** Verify is read-only here. AGENTS §13 puts verify/ship bookkeeping on
   `main` **after** the PR merges, and you are detached — anything you write to disk goes nowhere.
   **This has stranded a verify cost block twice** (SPEC-119, SPEC-120), because the shared
   closing-steps template used to tell verify to do exactly that. It no longer does.
2. **Emit your `## Cost readout` block** as the last thing you write. The orchestrator lands it in
   `cost.sessions` on `main` at ship and runs `just advance-cycle SPEC-123 ship` there.
3. **Give the verdict** — ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED. Itemize any punch list in the
   return message; **do not apply it.**

**Your return message is the deliverable.** Rulings on AC-6 and AC-7 belong in it explicitly —
they are the two things the orchestrator cannot decide without you.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by content — **never
by "the newest `.jsonl`."** Price per component at the anchors `.message.model` actually reports
(expected **Opus**: $5/$25 per MTok; cache_creation ×1.25, cache_read ×0.10 of input).

⚠ **Take the reading AFTER CI has settled, and prefer one long wait to many short polls.** Measured
on this spec's build: $5.80 — 13% of its total — went on watching a CI matrix it had already
triggered, and the "almost done" reading at 242 messages came in **29% low**.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
