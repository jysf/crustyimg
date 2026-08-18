# SPEC-123 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-123-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-16. 8 ACs, 4 design calls, no failing tests (a measurement).
      Framed because **two roadmap items are gated on it** — encoder threading and
      `par_iter run_pixel_op` — and because three shipped things (`build --frozen`, the
      lockfile's `hash`, the cache key) already assume an answer nobody has measured.
      ⚠ Thread count is **not** a component of the cache key and **not** in the lockfile's
      list of things output stability is qualified against.
- [x] **build** — 2026-08-17, Opus, PR #179, $46.17 / 215 min / 318 messages.
      Prompt: `prompts/SPEC-123-build.md`. DEC-094 was reserved in the prompt rather than left
      to `next_id` — no collision.
      **Verdict: Call 3's THIRD branch — the encoder ignores the thread setting.** `ravif` is
      compiled without its `threading` feature (reachable only via `image/rayon`, which
      `avif = ["image/avif"]` does not enable), so the encode is **serial** and the tile count is
      `available_parallelism()`. 18/18 cells identical; `--jobs` and `RAYON_NUM_THREADS` reach the
      batch pool on some verbs and the **encoder on none**. DEC-094. No `src/` change (AC-7).
      ⚠ **Two riders outrank the null:** AVIF output varies with the machine's **core count**,
      which is in neither the cache key nor the lockfile's `[env]`/caveat list — so `diff` can call
      a differently-cored machine a regression; and the shipped build takes the worst cell,
      **+1.5% / +47.9%** bytes vs a 1-tile encode at **5.7× / 4.4×** the wall clock of the same
      tiles in parallel. That **splits STAGE-042's pin item**: `image/rayon` is the performance
      lever, `with_num_threads(Some(N))` the determinism lever.
      ⚠ **Design predicted the opposite verdict, twice**, by quoting `image`'s doc comment without
      checking the feature set. Both errors are corrected in place on STAGE-042.
- [x] **verify** — 2026-08-17, Opus, **⚠ PUNCH LIST**, $14.16 / 14 min / 135 messages.
      Read-only as instructed — no commits, worktree left clean and detached at `7b3b130`.
      **The verdict stands.** Verify re-derived the mechanism by a stronger method than the DEC
      used — `image` declares `[dependencies.ravif] default-features = false`, `ravif?/threading`
      appears at exactly one place in `image`'s manifest (inside `rayon = [...]`), no dependent
      enables it so feature unification cannot turn it on — and rebuilt all three binaries from
      scratch, reproducing every hash in every leg bit-for-bit.
      **Four items, applied to the branch by the orchestrator (`1d522f8`), `cycle:` HELD at
      `verify` for re-approval, not advanced** (SPEC-116 precedent):
      **P1** leg F's *"lands on the 14-tile point and nowhere else"* is **false** — `rav1e`
      quantizes to a legal tile grid, so the graphic matches at every N ≥ 12 (including the 16 the
      DEC said did not match) and the photo at N = 13–20. Leg F **bounds** a band, it does not
      identify a point. ⚡ Rider: core-count sensitivity is **quantized** (14 and 16 agree, 8 and
      14 do not), so raw core count is the wrong `[env]` key.
      **P2** the `cargo tree` check is non-discriminating — it returns identical values on a tree
      where threading is provably ON. Replaced with the build fingerprint.
      **P3** Validation said 37 hashes; the spec's 43 is right.
      **P4** the AC-7 deferral was filed in `docs/backlog.md`, which **no command reads** — now a
      STAGE-042 checkbox, plus `src/build/lock.rs` in DEC-094's `affected_scope`.
      **⚖ AC-6 — does not fire.** Thread axis not falsified; shipped language accurate. All three
      sweep numbers reproduced (316/91 raw, 30 narrowed, RELEASING.md 0 against a positive
      control); all 30 read, none falsified. ⚠ But the sharper defect is `lock.rs:124-129`, not
      the `:32-37` the build filed: `[env]` claims to distinguish *"this same machine"* while
      `env.target` is only `{ARCH}-{OS}`, and `:459-466` marks a same-`env` hash change as drift
      **unconditionally**. A live false positive in shipped code — unreachable in this repo's CI
      (no committed `*.build.lock`, no workflow runs `--check`/`--frozen`), user-facing only.
      **⚖ AC-7 — the build read it correctly**, and it did **not** block a correction AC-6 wanted:
      the right fix was never a comment edit but a maintainer call on whether raw core count
      belongs in `[env]` at all — and P1 says it does not. The SPEC-117 pattern (AGENTS §15).
      ⚠ **The trade was only sound if the deferral was tracked, and it was not** — P4 fixes that.
      Also verified: CI green at the true head `7b3b130` (16 legs read individually, 9–16.5 min on
      the OS legs, `scripts/` inside the changes filter so no docs-only short-circuit); the
      in-process control is signal not noise (`web` 0.99→1.01→1.17 while sibling `optimize` stays
      flat at 0.99 in the same run); leg E enabled the variable structurally, by fingerprint.
      💰 **Cost to apply at ship:** 17,012,782 tokens (in 270 / out 140,172 / cache-write 385,276 /
      cache-read 16,487,064), 14 min, **$14.16**, `claude-opus-5`, Opus anchors, measured over 135
      assistant messages. Transcribed here per AGENTS §13 — `cost.sessions` gets it on `main`
      after the PR merges.
- [ ] **re-verify / ship** — punch list landed on the branch; `cycle:` still `verify`. Needs the
      maintainer's re-approval of `1d522f8` before advancing.
- [ ] **ship**
