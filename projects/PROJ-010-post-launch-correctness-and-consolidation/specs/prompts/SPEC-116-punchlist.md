# SPEC-116 — PUNCH LIST prompt

Cycle: **build** (punch-list return). Verify returned **⚠ PUNCH LIST** on PR
[#171](https://github.com/jysf/crustyimg/pull/171). **Two of its three items are already done
on `main` by the orchestrator.** You have exactly one, and it is a rename plus two comment edits.

**Do not re-verify the spec.** The fix is correct and was independently confirmed: byte output is
identical to `main`, the negative control was re-run in three stages with changed binary hashes,
the matrix was re-baselined at +6 on all three legs, and the extra AC-7 test was proven non-vacuous
by mutation. **None of that is in question. Do not redo it.**

## Your one item

`tests/build.rs` — the test currently named `build_output_bytes_unchanged_for_a_clean_input`
(around `:1222`) **is correctly implemented and wrongly labelled.**

It asserts `build`'s bytes == `apply --recipe web`'s bytes, **both on this branch**. Its name says
"unchanged", which means *versus before*, and its doc comment opens *"AC-6: this spec adds a
diagnostic only — it must not perturb encoding."* Neither claim is what the body checks.

**Verify ruled: keep the invariant, fix the label.** The same-branch cross-verb pin is the better
standing check — a committed golden would go red on every `ravif`/`image` bump for reasons
unrelated to this spec, and the repo has no such golden and should not acquire one here.

So, three edits:

1. **Rename** it to state the real claim — e.g. `build_and_apply_agree_on_bytes_for_a_clean_input`.
2. **Rewrite the doc comment** to describe the same-branch cross-verb invariant, and say plainly
   that the cross-version property was driven once out of band rather than pinned by this test.
3. **Record the cross-version evidence in `## Build Completion`**, the same treatment AC-8's
   negative control already gets. Verify's driven result, to transcribe:

   | | |
   |---|---|
   | `main` binary (`7ac9f27`) | `sha256 a82bb937…` |
   | branch binary (`7c6ff59`) | `sha256 5ed4a7c3…` |
   | input | `bench/corpus/photo_forest_cc0.jpg` |
   | target | `recipe = "web"`, default `{stem}.{ext}` → Decide plan |
   | `main` output | `clean.avif` `sha256 1c5ed3f1…` |
   | branch output | `clean.avif` `sha256 1c5ed3f1…` |

   **Byte-identical, with differing binary hashes as the positive control** that two genuinely
   different builds were compared. AC-6 as written holds; it is simply evidenced rather than pinned.

That is the whole change: a rename, a comment, and a Build Completion paragraph. **No source
changes. No new tests.**

## Already done — do NOT redo these

Verify's items 2 and 3 were bookkeeping on `main`, and the orchestrator applied them:

- **STAGE-042 now carries both bullets** — the `encode_one` / `Preserve` / `Pinned` silence (filed
  `7c49340`, before verify ran, which is why verify's grep against the PR tip missed it) and the
  cache-hit swallow verify found (F1).
- **DEC-085's `affected_scope` now includes `src/cli/build.rs` and `tests/build.rs`**, so
  `decisions-audit --changed` will surface it on a build-only PR.

Neither belongs on your branch. If you find yourself editing `decisions/` or `stages/`, stop.

## When you finish, in this order

1. Amend `## Build Completion` with the deviation and the cross-version table above.
2. **Do not add another cost session entry** — this is a continuation of the same build cycle, not
   a new one. If the punch-list work is material (it should be well under an hour), add its tokens
   to the existing build entry and say so in the note. Do not double-count.
3. **Leave `cycle:` at `verify`.** Do not run `advance-cycle`. Verify re-reads and advances.
4. Push to the same branch. **Do not merge. Do not bump the version.**

## Guardrails

- **Own git worktree**, on `feat/spec-116-build-threads-truncated-jpeg-warning`. `main` has moved
  several times today — **do not work in the primary checkout**, and check
  `git branch --show-current` before committing.
- **Do not rebase the branch on `main`.** The PR is green; a rebase re-runs 16 CI jobs to change
  nothing. If `main` has moved under it, that is fine — the diff does not touch anything that moved.
- `git commit -s` (DCO). macOS has no `timeout(1)`.
- **Budget: this is minutes, not hours.** If it is taking more than ~30 minutes you have
  misread the scope — re-read "Your one item" and stop.

## For the record — one correction to carry forward

Verify found that the **`decisions-audit --changed` warning in the verify prompt is stale**. The
script now detects a clean tree, says so, and falls back to `origin/main...HEAD`; both forms
produced identical output. Passing the base ref is still the reliable habit, but "that green cannot
go red" is no longer true. Future prompts should not repeat the stronger claim.
