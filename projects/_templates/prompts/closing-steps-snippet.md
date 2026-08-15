# Closing-steps snippet for cycle prompts (paste into build / verify prompts)

> Companion to `cost-snippet.md`, and it exists for the same reason. AGENTS.md
> §15 lists what a cycle owes when it finishes; every hand-authored prompt then
> restates that list in its own `## When you finish` section — and every one of
> them so far dropped the same item.
>
> **Measured, 2026-08-14:** across SPEC-110 … SPEC-115, *not one* build prompt
> mentioned `just advance-cycle`. Their reading lists all say
> *"`/AGENTS.md` — §4 cost, §6 commands, §12 testing, §13 git/PR"* — **§15 is
> not among them** — and each prompt's own closing checklist reads as complete
> without it. A builder told to follow the prompt exactly has no reason to open
> §15, and no reason to suspect the checklist in front of it is short one step.
>
> **What the omission did and did not cause.** SPEC-113 and SPEC-115 both reached
> ship having never been marked `build` or `verify` — they jumped `design → ship`
> in one late commit, and SPEC-114 has never moved off `design`. But SPEC-112,
> three specs earlier, advanced correctly with a prompt that also never mentioned
> `advance-cycle`: its builder read AGENTS §15 and complied anyway. So the gap is
> a standing weakness, not the cause of the recent drift (that traces to
> overlapping sessions with no single owner per spec — see SPEC-113's ship
> reflection). Closing it removes the last excuse; it will not by itself keep the
> field honest, which is why `archive-spec` now warns at ship as well.
>
> Paste the matching block verbatim into the prompt's `## When you finish`
> section so the next prompt cannot re-drop it.

---

**For a BUILD prompt:**

```
When you finish, in this order:

1. Fill in the spec's `## Build Completion`, including its reflection questions.
2. Append a build cost session entry to `cost.sessions` (see cost-snippet.md).
3. Create any `DEC-*` the build earned, with `affected_scope` set to the path
   globs it governs — that is what lets `decisions-audit --changed` surface it
   later.
4. Run `just advance-cycle SPEC-NNN verify`, and CONFIRM it moved: the command
   prints the file it wrote, and `git diff` on the spec should show the
   `cycle:` line change. It reports success even when it changes nothing.
5. Open the PR. Do not merge it.
```

**For a VERIFY prompt:**

```
When you finish, in this order:

1. Append a verify cost session entry to `cost.sessions` (see cost-snippet.md).
2. Run `just advance-cycle SPEC-NNN ship`, and CONFIRM it moved (see above).
3. Give the verdict: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.
```

---

## Why step 4 says "confirm it moved"

`advance-cycle` resolves the spec through `find_spec`, which until 2026-08-14
also matched `prompts/SPEC-NNN-<cycle>.md` — a cycle prompt shares the spec's
filename prefix. It would "update" a file with no `task:` block and still print
its success hint. That specific bug is fixed, but the shape is worth guarding
against generally: **a tool printing success is not evidence a file changed.**
One `git diff` costs nothing and closes the loop.
