# SPEC-126 — RE-APPROVE prompt

Cycle: **re-approve** (not one of the five — an extra gate, because a punch list was applied by
the person who received it). New session, **read-only**. You did not write any of this.

**What you are reviewing:** commit **`77f1050`** on `fix/spec-126-apply-and-build-agree` — the
orchestrator's application of verify's 7-item punch list. **Base is `f8deb55`**, the commit verify
reviewed. `git diff f8deb55..77f1050` is the entire scope: 4 files, +157/−42.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec126-reapprove fix/spec-126-apply-and-build-agree
```

## ⚠ Why this cycle exists, and what that means for your posture

Verify returned ⚠ PUNCH LIST. The **orchestrator applied all 7 items itself** rather than
dispatching a punch-list cycle, then wrote the commit message describing what it had done, then
told the maintainer it was done. **Nobody has independently read any of it.**

So the failure mode here is not "did the fix work" — verify settled that. It is **an author
grading their own homework**, and specifically:

- Records that now *claim* a control was driven, written by the person who claims to have driven it.
- Prose added to a **published contract document** (`docs/api-contract.md`) that no test checks.
- A test edit that could have dropped an assertion while still going green.

**Attack the claims, not the code.** Where a claim is about something that was driven, drive it.

## Already settled — do NOT re-open

1. **SPEC-126's fix itself is approved.** Verify confirmed the code at `f8deb55`: the two Calls,
   the call-graph containment, AC-1 through AC-7, and a 39-file / 9-path blast-radius sweep. Do not
   re-verify the fix. Your scope is the delta `f8deb55..77f1050`.
2. **CI is green at the true head `77f105086c13586b482dea447a1abc493ffc0bd9`** — 16 SUCCESS,
   6 release-only skips, snapshot taken directly at that SHA. `front-matter validation` and
   `cost-capture audit` both pass, which is what gates the front-matter and cost-block edits below.
   **Never poll CI.**
3. **The verify cost figure ($15.64 / 21.6 min / 22,698,850 tokens, Opus) is checked** — priced per
   component, exact to the cent, components sum to `tokens_total`. Measure your own; don't audit it.

---

## The seven items, and what to test about each

### 1 — ⚡ `docs/api-contract.md` (+15). This is the one that ships to users.

Prose was added to the `apply` entry claiming: the format rule is `--format` > `-o` ext >
preserve-source at every arity; a literal `--name-template` extension does **not** pin; and three
single-input `-o` invocations moved **exit 4 → exit 0**.

**Every one of those is a testable statement about a shipped binary. Test them, don't read them.**
Build the branch and drive all three `-o` cases (`-o -`, an extensionless path, an unrecognised
extension) plus the `--name-template` claim. If any sentence overstates, it is worse than the
silence it replaced — a doc nobody can check is how this repo got here.

Then two judgement calls the orchestrator made and you should confirm or overturn:

- It asserts the old exit 4 was **outside** this document's own enumeration of code 4 (whose three
  cases are unrecognisable bytes / decoder not built / encoder not built). Read the table at
  `docs/api-contract.md` and rule whether that reading is fair, or whether it is a convenient one.
- It documents the exit-code change **only in the `apply` entry**, not in the exit-code table
  itself. Is that the right home?

### 2 — The two deviations now named in `## Build Completion`

"Deviations: None" became two entries. Check they describe what actually happened rather than what
is easiest to defend — particularly the `--name-template` one, which asserts `main` wrote **PNG
bytes into a `.jpg` file**. That is a strong claim about mislabelled output on a shipped verb.
**Reproduce it at `9b4fb80`.** If it is true it is arguably the most user-visible thing in this
spec, and it is currently buried in a deviation note.

### 3 — Front-matter and the corrected reflection

`references.decisions` gained DEC-015 and DEC-087; `references.constraints` gained
`ergonomic-defaults`. Confirm all three genuinely apply (read them — `ergonomic-defaults` in
particular is a claim that this fix removes required boilerplate). The build reflection's Q2 was
**rewritten** to admit "both were named" was false. Check the rewrite is an honest correction and
not a retroactive improvement of the answer — the original said what it said.

### 4 — DEC-098's AC-6 boundary paragraph

Now states what the original corpus did **not** cover, and folds in verify's 39-file / 9-path
extension. Check the numbers match verify's actual readout and that the boundary statement is
complete. `affected_scope` gained `docs/api-contract.md`.

⚠ **DEC-098 uses the block-list form, so the audit can read it. Confirm that.** A live tooling
defect means an **inline-array** `affected_scope` yields zero globs silently
(`scripts/decisions-audit.sh:77-91`; DEC-015 and DEC-043 are both invisible this way, filed on
PROJ-013 STAGE-047). If DEC-098 were written inline it would be invisible too.

### 5 — ⚡ The only code change: the test split

`apply_honours_format_at_every_arity` → `apply_honours_format_at_single_input` +
`apply_honours_format_at_multi_input`. `tests/apply_batch.rs` is +59/−35.

**A split is exactly the shape that silently drops an assertion.** The orchestrator's evidence is a
test count moving 940 → 941. **That is weak evidence** — it counts functions, not assertions.
Diff the two versions properly and confirm all four original blocks survive with every assertion
intact, then satisfy yourself the two new tests still cover **two target formats each**, which is
AC-1's anti-coincidence requirement.

Then re-drive the claim that justifies the split at all: **revert Call 2 alone** (hardcode
`format_override` back to `None` in `run_apply`'s multi-input branch) and confirm that in **one
run** multi-input goes RED while single-input stays GREEN. The orchestrator reports exactly that.
Verify it independently — this is the whole point of item 5, and it is a claim about a control.

### 6 — The file-list parenthetical

Corrected from claiming DEC-098 was untracked and outside `git diff --name-only main...HEAD`. Check
the correction is right, and that the list is all six files.

### 7 — The two backlog filings, on `main` (not this branch)

- `PROJ-013 / STAGE-047` — the `decisions-audit.sh` inline-array defect.
- `PROJ-011 / STAGE-050` — the `--name-template` pin question.

⚠ **Known and NOT hidden: `just backlog --all` returns ZERO for STAGE-047**, because PROJ-013 is a
different project and still `proposed`. The item is in the right topical home but is currently
invisible to tooling — the same class of problem it describes. **Do not treat that as newly
discovered; rule on whether STAGE-047 is still the right home given it.**

Also check the STAGE-047 entry's own claims: that DEC-015 **and DEC-043** both yield zero globs,
that DEC-087/DEC-006 yield 8 and 2 (the positive control), and that `decisions/_template.md:33`
teaches the unreadable form. Those were measured by the orchestrator; measure them again.

---

## Also check

- **Nothing else moved.** `git diff f8deb55..77f1050 -- src/` must be **empty**. If any `src/` file
  changed, that is a scope violation and the fix verify approved is no longer the fix on the branch.
- **The version was not bumped**, no tag, no release, and `cycle:` is still `verify` — held for
  this re-approval, not advanced.
- **The verify cost session** parses and sits alongside the build's; `cost.totals` stays at ship.
- Gates on your own clean checkout: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`. Expect **941** passing.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge. Do not bump the version.
- **Budget ~80 exchanges.** This is a delta review of 4 files, not a re-verify.
- macOS has no `timeout(1)`. A piped command reports the **pipe's** exit code — redirect and read
  `$?`. `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal — redirect
  stdout, don't "fix" it. Use `/usr/bin/grep` [[rtk-can-silently-corrupt-grep-counts]].
- ⚠ **Another session recently worked in the shared checkout and its `git commit -a` swept
  uncommitted files that were not its own.** Work in your own worktree, and check
  `git branch --show-current` before anything.

## When you finish

1. **No commits.** 2. Emit `## Cost readout` (per component, at the anchors `.message.model`
reports — DEC-083). 3. Verdict:

- **✅ APPROVED** — then say plainly that the next step is **merge #187 and run SPEC-126's ship
  cycle, with NO TAG**: the release batches with STAGE-050 as one lockfile migration.
- **⚠ PUNCH LIST / ❌ REJECTED** — say what must change. A finding here is cheap; nothing has
  shipped, and the whole reason this cycle exists is that the previous one was self-graded.

**Lead your verdict with item 1** — the api-contract prose is the only part of this commit that
reaches users, and it is the only part no test covers.
