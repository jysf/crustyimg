# SPEC-124 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-124-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-18. 9 ACs, 5 settled design calls, 2 failing tests.
      Framed on the **maintainer's ruling to pin now and ride SPEC-121/122's wave** rather than
      pay a second lockfile migration later. Closes both riders SPEC-123 measured (DEC-094): the
      core-count variance behind STAGE-042's `[env]`/`diff` false positive, and the multi-tile
      compression penalty the shipped build pays for tiles that buy **no parallelism** — `ravif`
      is compiled without `threading`, so the encode is serial.
      ⚡ **N is deliberately NOT settled at design.** The prior is that N=1 may be strictly better
      (same speed, materially smaller files) — but it is stated as a **prior to measure**, not a
      conclusion, because SPEC-123 cost $60.33 largely for a design-time prediction asserted with
      confidence. Call 2 lists what must be driven, including the forward cost of N=1 if
      `image/rayon` is ever enabled.
      ⚠ **Blocked on SPEC-122 merging.** Same wave; the migration is keyed on `crate::version()`,
      so what makes it one migration is landing in the same **release**, not the same PR.
- [x] **build** — 2026-08-21. PR [#184](https://github.com/jysf/crustyimg/pull/184), branch
      `fix/spec-124-pin-the-avif-encoder-tile-count`, head `2e77269`. 9/9 ACs claimed, **DEC-096**
      written. Cost **$57.74** (full session; the cycle recorded $53.89 at its own snapshot, 479 of
      503 usage-bearing messages — re-derived by the orchestrator, corrected at ship).
      ⚠ **The dispatch was Sonnet, not the Opus the prompt asserted** — the cycle caught the
      mismatch itself from `.message.model` (503/503) and corrected the spec's `implementer`.
      **N = 1 was measured, not assumed** — four axes in DEC-096, including a reported
      counter-finding (a synthetic 24 MP worst case costs N=1 ~17% serial time) that the
      recommendation survives only via the realistic-content distinction.
      ⚠ **AC-9's CI claim covers `79d8615`, not the head** — the cost commit pushed afterwards
      triggered a fresh run the build never read, and that run went red on `pages / build + browser
      smoke`. **Resolved: a flake, proven not argued** — the same tree re-ran on the `c20c96b`
      merge commit and passed. CI at the head is **16 pass / 6 skipping / 0 fail**. The claim was
      still unearned when written; that is a method finding, not a code one.
      prompt: `prompts/SPEC-124-build.md` (2026-08-20). **Opus**, own worktree, branch
      `fix/spec-124-pin-the-avif-encoder-tile-count`. **DEC-096 reserved in the prompt**, not left
      to `next_id`. The spec's `implementer` was updated to Opus to match — SPEC-122's prompt said
      Sonnet while the dispatch used Opus, and the cycle had to flag the mismatch itself.
      **Unblocked 2026-08-20**: SPEC-121 (`9075bc3`) and SPEC-122 (`2bd74b0`) are both merged.
      ⚠ **Deadline-bound.** The shared lockfile migration only stays *one* migration if this lands
      in the same release as 121/122 — the key is a function of `crate::version()`, so the window
      closes at the next tag.
      Carries three lessons the earlier prompts in this wave lacked: **never poll CI** (SPEC-122's
      build spent ~$60 of $103.60 there, from a prompt that carried no CI instruction at all); **a
      green local matrix does not predict CI** (twelve local exit-0s against eight red CI legs when
      stable floated to 1.98); and **list every file the diff touches** (SPEC-122's Deviations
      claimed `src/operation` + `tests/` only and was wrong by two `scripts/` files, which left
      `affected_scope` blind).
- [x] **verify** — 2026-08-21, **Opus**, read-only at `c20c96b`, no commits, tree left clean.
      **⚠ PUNCH LIST — 5 items, none blocking the mechanism.** Cost **$15.27** (full session; the
      cycle recorded $13.74 at 129 of 136 messages **and flagged its own snapshot, asking for
      re-derivation at ship** — the first cycle in this project to anticipate the undercount rather
      than fall into it). **Verify at 26 % of build cost**, holding the pattern: every substantive
      defect this wave came from a verify pass, not a build.
      **Confirmed as claimed:** both encode arms are the only two `AvifEncoder` sites in `src/`; §2
      replicated byte-for-byte cross-checking to DEC-094 leg E; §3's clamp driven behaviourally and
      decoded through crustyimg's own `re_rav1d`, not just "did not panic"; the deviation away from
      the Python harness is correct against DEC-094's actual legs; `Cargo.toml`/`Cargo.lock`
      untouched; the lean CI leg really does cover the example (positive control run).
      **All 5 items were records overclaiming what was established**, applied by the orchestrator on
      `main` at `7b9b04d` — see the commit. The two that matter most: **§4b's dismissal of a real
      ~17 % serial-time regression is not established** (neither 24 MP fixture carried real 24 MP
      detail — 0.157 and 0.22 bpp against 1.86 bpp native), and **the SPEC-123 Python harness's
      positive control is now dead**, so re-running it reports a green whose control cannot fail.
      ⚡ **Verify drove the negative-control cross-product and the DEC had it wrong**: the spread is
      three-way not five-way, and the `quality`-side asymmetric revert leaves AC-2 **green** — only
      `sink`-side flips it. Coverage is still complete, but now because it was driven, not assumed.
      **Item 3 ruled acceptable-with-follow-up**: the in-test `cargo build` is the only lever that
      discriminates, but it costs ~+15–25 min CI per PR across 7 job instances, leaks a 1.3 GB dir
      per test process (13 dirs / 16 GB reclaimed from one session), and ships to crates.io because
      `exclude` does not cover `/tests`. Filed on STAGE-042.
      **Withdrew build follow-up #3 as a false finding** — the `Count:` line never undercounted.
      ⚖ **`cycle:` HELD at `verify` for maintainer re-approval, not advanced** — PR #184 merged
      (`0107a49`) while the punch list was being processed.
- [ ] **ship**
