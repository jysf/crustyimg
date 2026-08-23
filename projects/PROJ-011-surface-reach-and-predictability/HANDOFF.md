# PROJ-011 — orchestration handoff

**Written 2026-08-23.** First handoff for this project. PROJ-010's handoff is superseded for
day-to-day work but keep it for its measured evidence.

> **This file deliberately does NOT restate repo state.** `just status`, `just backlog`,
> `just roadmap` and `just specs-by-stage` all report correctly — **trust them over any summary,
> including this one.** What follows is only what the tooling cannot show.

---

## Read first

`/AGENTS.md`, then `just status` and `just backlog`. Then this file.

**You orchestrate; you do not build.** Build and verify go to separate CLI sessions via a persisted
prompt in `specs/prompts/`, pushed to `main` **before** the branch is cut.

---

## Where this stands

**v0.7.1 shipped 2026-08-22** and is verified live on all three channels — GitHub release (15
assets), crates.io `max_version 0.7.1`, Homebrew formula at `0.7.1`. Thirteen user-visible fixes on
**one** lockfile migration.

**PROJ-011 is active. SPEC-126 is designed and its build prompt is on `main`, ready to dispatch.**

### The one sequencing rule that matters

⛔ **Everything in PROJ-011 is byte-changing on a shipped verb, so the whole project carries ONE
lockfile migration and ships as ONE release.** The key is a function of `crate::version()`, so what
makes it one migration is landing in the same *release*, not the same PR.

**Do not cut a release for SPEC-126 alone.** STAGE-049 → STAGE-050 → then tag.

---

## Open, waiting on the maintainer

- **The watermark coverage threshold** — a number nobody has measured. Reference points driven
  2026-08-23: 24 px → 47 % coverage, 64 px → 16 %, 800 px → 0.32 %. ~25 % separates them.
  **Validate against real output; do not adopt that number from a note.** Needed by STAGE-050.
- **`mp4-atom` DEC** — blocks the animated-AVIF fork (PROJ-012) from being specced at all.
- **The ICO round-trip ruling** — warn / fix / accept. A real fix changes bytes. PROJ-010.
- **The `-o`-extension pin ruling** — warn / re-trigger / document-and-keep. PROJ-010.
- **STAGE-041's real status.** The handoff has said since **2026-08-16** that three of its four
  items are substantially done outside the repo. **A week unreported.** `just backlog` reports four
  open items that may be one. **Do not re-plan STAGE-041 against what the repo says.**
- **The jysf.org project page** — a fourth public surface, in no release checklist, now a version
  behind. A refresh prompt exists (written 2026-08-23, in the session scratchpad, not committed).

---

## Traps this session paid for

1. **⚡ RELEASING.md had never been executed as written. Running it end to end broke three times.**
   Its `cargo publish --dry-run` is step 3 and "commit the release" is step 5 — **cargo refuses a
   dirty tree**, and the three files it names are the ones steps 1–2 just modified, so the dry-run
   **cannot run** at step 3. Then `cargo test` failed on a test that is **green in CI and red in a
   terminal**. Then every push reported **"Bypassed rule violations"**, because branch protection
   requires PRs while AGENTS §13 says design/docs commit to `main` directly.
   **A checklist that has only ever been read is not a checklist.** All three are filed.
2. **⚡ NEVER PUSH DURING SOMEONE ELSE'S RELEASE.** The maintainer ran `git commit -am` while I was
   committing a stage-file edit; `-a` swept my file into their release commit, and **my subsequent
   `git push` sent their release commit to `main` before the dry-run and gates had run.** Nothing
   irreversible, but it removed their option to amend freely. **If a release is in flight, commit
   with an explicit pathspec and do not push at all.**
3. **`str.index()` on a repeated anchor ate ten backlog items.** A slice from `s.index("**Count:**
   1 framed…")` matched an *earlier* Count line and deleted everything between. Caught only by the
   tally check afterwards. **Use anchors verified unique with `s.count(x)==1`, and re-derive the
   `- [ ]` / `- [x]` tally after every stage edit.**
4. **PNG filtered bytes are not pixels.** A first measurement of what `watermark` drew reported
   "100 % of pixels changed" on every image, because adaptive row filters make the raw IDAT bytes
   differ everywhere. **Undo the filters before comparing**, or the measurement is noise — and it
   hid the real finding, which was a 0.00 % silent no-op.
5. **A cycle's cost block ALWAYS under-reports, structurally** — a cycle cannot count the messages
   that write its own cost block. Four cycles this wave, all under. Re-derive at ship from the
   transcript, **identified by content, not recency** — a naive search matched the orchestrator's
   own session as well as the build's.
6. **Three external review batches, and all three led with an inferred flagship.** Batch 1 asked
   for three fuzz targets that already exist; batch 2's headline optimizer concern did not
   reproduce; batch 3's top recommendation (unify `sink` duplication) describes a refactor **that
   is already the architecture**. ⚠ **In all three, the genuinely valuable items were the quiet
   ones.** Treat any recommendation containing *"likely"*, *"inferred"* or *"assumed"* as a
   hypothesis to drive, never a finding to schedule.
7. **I did the same thing.** I told the maintainer to rule on "SPEC-118 vs the conformance matrix"
   as duplicates — **from the title, without reading the spec.** They are different matrices on
   different axes and neither would catch the other's defect. **Read the artifact before asking for
   a decision about it.**
8. **The same findings keep resurfacing because nowhere machine-readable holds them.** Four of the
   eight CLI-surface findings filed this session match notes from an **earlier** audit marked "5
   unfiled findings". `docs/backlog.md` is read by **no command**, and 358 lines of measured
   research sat there invisible to `just backlog` for days.

## What is working, and worth keeping

- **Drive it before you file it.** Every finding filed this session carries a reproduction that was
  re-run, and two of eight turned out **sharper** than reported. The one design call in SPEC-126
  was settled by measuring all six sibling verbs rather than by argument.
- **Verify remains the cheapest and most valuable cycle — five waves running.** 26 % and 28 % of
  build cost this wave, and it produced **every** finding: 8 punch-list items, none of them a code
  defect, all records claiming more than had been measured.
- **Reserve DEC ids in the prompt.** `next_id` scans only the working tree, so a record on an
  unmerged branch is invisible. **Highest is DEC-097; DEC-098 is reserved for SPEC-126.**
- **Budget prompts in exchanges (~150), never minutes.** Four consecutive cycles blew it without
  the checkpoint firing.
- **File findings where `just backlog` reads** — a stage's `## Spec Backlog`, as `- [ ]`. Then run
  `just backlog` and **read it back.**
