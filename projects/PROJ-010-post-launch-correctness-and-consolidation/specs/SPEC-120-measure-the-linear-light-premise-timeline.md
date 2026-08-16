# SPEC-120 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-120-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-15. 8 ACs, 5 settled design calls, no failing tests (a
      measurement spike). Design found that SSIMULACRA2 cannot score a downscale
      against its source (`report.rs:329`), which reshaped the experiment: it
      needs an independently-generated reference at the target dimensions.
- [x] **build** — 2026-08-16, Opus. PR #175, `src/` and `tests/` untouched.
      $8.69 / 11,365,430 tokens / 22 min. **Verdict: premise holds, spec the fix.**
      AC-2 fired: −88.07% physical error registered as a **163.85-point** SSIMULACRA2 swing, so
      the realistic rows (70.45 on `graphic_large`, 84.45 on the photo) are readable rather than
      an uninterpretable null. DEC-092 written.
      **The alpha half of the premise is REFUTED** — `fast_image_resize` 6.0.0's
      `ResizeOptions::default()` sets `mul_div_alpha: true` and `new()` is `Default::default()`,
      so `Resize::apply` has always premultiplied. The fix spec is one premise, not two.
- [ ] **verify** — prompt: `prompts/SPEC-120-verify.md`. Opus, new session, own worktree.
      Four open items flagged at handoff, the first load-bearing: the prototype scores ~100 partly
      because it shares the reference's algorithm, so the verdict must rest on **today's** score
      against an independent reference, not on the delta's size.
- [ ] **ship**
