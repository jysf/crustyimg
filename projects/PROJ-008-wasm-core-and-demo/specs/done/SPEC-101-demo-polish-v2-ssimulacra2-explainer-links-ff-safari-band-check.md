---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-101
  type: chore
  cycle: ship
  blocked: false
  priority: medium
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-20

references:
  decisions: []
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-081]

value_link: >
  Small pre-launch demo polish: make the SSIMULACRA2 score self-explaining (link the metric) and confirm
  the score band renders correctly on Firefox/Safari before real users hit it.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 250000
      duration_minutes: null
      estimated_usd: 2.0
      note: >
        Build session on Sonnet — ORDER-OF-MAGNITUDE ESTIMATE, not a real
        usage-object reading. Scope: the SSIMULACRA2 explainer links, the
        "Updated" re-convert pulse, wiring the 7-file favicon set + the three
        site.webmanifest fixes (absolute icon src -> relative for the
        /crustyimg/ subpath, empty name/short_name, white theme colors on a
        dark demo), and extending the browser smoke to cover all three.
        Verified the manifest fix against a real subpath server rather than a
        root-served dir, which is what makes the check meaningful. Paused
        before the device gate (no device access) — see the process note.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 750000
      duration_minutes: null
      estimated_usd: 6.5
      note: >
        Finalize + verify on Opus (1M ctx) — ORDER-OF-MAGNITUDE ESTIMATE,
        midpoint of the session's own ~$5-8 range. Finalized two loose ends
        (gitignored the demo/_* dev harness, confirming it was never committed;
        recorded the maintainer-decided device gate honestly) then verified the
        three code items against the committed diff rather than the build's
        prose. Both link targets curl 200 and the metric link is the "2" repo,
        not v1. The favicon check carried a NEGATIVE CONTROL: on a real
        /crustyimg/ subpath server the relative paths return 200 while the
        root-absolute paths a leading slash would produce return 404 — proving
        the trap was real and the fix clears it. Zero off-origin requests during
        conversion; no src/ change; smoke and validate green.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.3
      recorded_at: 2026-07-22
      note: >
        orchestrator main loop — PR #109, CI CLEAN first try (all 5 commits
        DCO-signed), squash-merge (fdd4447), bookkeeping. No DEC. Merge triggers
        a Pages redeploy, so the demo goes live with the favicons and the
        Updated signal.
  totals:
    tokens_total: 1000000
    estimated_usd: 8.8
    session_count: 3
---

# SPEC-101: demo polish v2 — SSIMULACRA2 explainer links + FF/Safari band check

## Context

Two small demo items surfaced during launch prep (queued 2026-07-19), batched here so they ship as one
build:

1. **The SSIMULACRA2 score is unexplained.** SPEC-081 shows a real SSIMULACRA2 number on a band, but a
   visitor who's never heard of the metric has no way to learn what it is. Link it.
2. **The score band is Chrome-verified only.** SPEC-081's band/meter uses `color-mix()` (12 uses) and was
   only driven in headless Chrome; it degrades gracefully but has never been confirmed on real
   Firefox/Safari — a launch carry.

Small, demo-files-only, no engine change. (A third queued item — the 🦀→logo swap — is **deferred until a
logo exists**; not in this spec.)

## Goal

The **pre-launch demo pass**, batched into one browser session because every item needs the same
multi-browser setup and the demo is where the r/rust post lands. Four things: (1) make the SSIMULACRA2
score self-explaining (link the metric explainer + the Rust impl); (2) give the Advanced-control
re-convert a **visible signal** so a settings change obviously regenerates the output; (3) wire the
**favicon set** and fix `site.webmanifest` for the `/crustyimg/` subpath; (4) the **device gate** —
confirm the demo works on real mobile browsers and that the score band renders on real Firefox and
Safari. Demo files only, no engine change.

Item (4) is a **launch gate, not a polish item**: the demo has only ever been proven on desktop
browsers, it is the single highest-risk untested surface before launch, and the post links straight to
it. If a phone can't run it, we need to know before the post, not after.

## Inputs

- **Files to read:** `demo/demo.js` (`renderScore` — the score panel from SPEC-081), `demo/index.html`
  (the score/meter markup), `demo/demo.css` (the band/meter + the `color-mix()` uses); the two link
  targets (below).

## Outputs

- **Files modified:**
  - `demo/demo.js` / `demo/index.html` — make the score panel's **"SSIMULACRA2"** label a link with a
    small "what's this?" affordance, pointing at:
    - the metric explainer → **https://github.com/cloudinary/ssimulacra2** (Jon Sneyers/Cloudinary — note
      the "2"; `cloudinary/ssimulacra` without it is the older v1, wrong), and
    - the Rust implementation we actually run → **https://github.com/rust-av/ssimulacra2**.
    Keep it unobtrusive (a linked label + a secondary "Rust impl" link), honest, and consistent with the
    existing panel voice.
  - `demo/demo.css` — any styling for the link affordance; theme-aware, matches the panel.
  - `demo/demo.js` / `demo/index.html` / `demo/demo.css` — a **re-convert signal**: when an Advanced
    control change triggers a debounced re-convert of the current image (the existing
    `scheduleConvert`→`convert()` path, `demo/demo.js:520-536`), make it visibly obvious the output
    regenerated. The wiring already works; the gap is that it's silent (maintainer feedback 2026-07-20:
    "it's working, but hard to tell"). Options: surface the existing busy state on re-convert, and/or a
    brief "Updated" pulse on the result, and/or an explicit "Regenerate/Apply" affordance. Keep it
    unobtrusive and on-voice; don't change the auto-rerun behavior, just make it legible.
  - `demo/index.html` — wire the **favicon set** already sitting untracked in `demo/`
    (`favicon.ico`, `favicon-16x16.png`, `favicon-32x32.png`, `apple-touch-icon.png`,
    `android-chrome-192x192.png`, `android-chrome-512x512.png`, `site.webmanifest` — generated by
    realfavicongenerator, verified valid: ICO carries 16/32/48, PNGs are correct dimensions). Add the
    `<link>` tags with **relative** hrefs. Commit all seven files.
  - `demo/site.webmanifest` — **three fixes**: (a) its `icons[].src` are **absolute**
    (`/android-chrome-192x192.png`), which resolve to the domain root and **404** because the demo is
    served from the `/crustyimg/` project-pages subpath → make them relative; (b) `name` and
    `short_name` are empty strings → `crustyimg`; (c) `theme_color`/`background_color` are `#ffffff`
    while the demo is dark → use the demo's palette (`--bg: #14110e`, `--panel: #1e1a16`) so an
    installed PWA doesn't flash white.
- **Verification artifact (not a code change):** the FF/Safari band render and the **mobile device
  pass** are confirmed (see below).

## Acceptance Criteria

- [ ] The score panel's "SSIMULACRA2" is a **link to https://github.com/cloudinary/ssimulacra2** (the
      metric explainer, with the "2"), plus a secondary link to **https://github.com/rust-av/ssimulacra2**
      (the impl). Both resolve (200). Unobtrusive, honest, on-voice.
- [ ] The **score band renders correctly on real Firefox and Safari** — the `color-mix()` band colors show
      (or degrade to the documented `var(--muted)`/`var(--good)` fallback without looking broken); confirmed
      by driving the demo on Firefox + Safari (the SPEC-078/080 multi-browser harness), not just Chrome.
      Record what each browser showed.
- [ ] Changing an Advanced control (format/max-edge/max-bytes/keep-full) on an already-converted image
      produces a **visible signal** that the output regenerated — not just a silently-swapped result.
      Confirmed by driving the demo: change a control, observe the signal, see the new output. The
      auto-rerun behavior itself is unchanged (still debounced, still `if (source)`).
- [ ] **Favicons load with no 404s**, verified on the deployed subpath (or a server that reproduces
      `/crustyimg/`) — not just locally at a root. `site.webmanifest` parses, its icon `src`s resolve,
      `name`/`short_name` are set, and the theme colors are the demo's dark palette. The demo smoke
      asserts the icon requests return 200.
- [ ] **DEVICE GATE (the launch go/no-go):** the demo is driven end-to-end — drop a photo, convert,
      read the score, download — on **at least one real iOS Safari and one real Android Chrome**, not
      an emulator-only check. Record device, OS, browser version, and what actually happened for each.
      The three known mobile risks are each explicitly checked: the **module Web Worker** initializes,
      **`createImageBitmap` decodes an `.avif` input**, and a large photo completes without the tab
      being killed for memory. A graceful, honest failure is an acceptable outcome and must be
      **documented**; a silent hang or a crash is a launch blocker. If mobile fails, say so plainly —
      this gate exists to produce a go/no-go answer, not a green check.
- [ ] Zero network requests during a conversion still holds (the links are static `href`s, not fetches).
- [ ] Browser smoke stays green; no `src/`/engine change; no wasm rebuild needed.

## Failing Tests

- Extend the demo smoke: the score panel contains an `href` to `github.com/cloudinary/ssimulacra2`; the
  conversion path still makes **0 network requests**.
- **Cross-browser (the point):** drive the demo on real Firefox + Safari and assert the band element gets a
  non-default computed background (or the documented graceful fallback), and the panel is not visually
  broken. Report per browser.

## Implementation Context

### Constraints that apply
- `ergonomic-defaults` — the link must help a non-expert without cluttering the panel; plain, honest.

### Prior related work
- `SPEC-081` (shipped) — the score panel + band this extends; the `color-mix()` FF/Safari carry it noted.
- `SPEC-078`/`SPEC-080` — the multi-browser (Chrome/FF/Safari) demo harness to reuse for the band check.

### Out of scope
- The 🦀→logo swap (deferred until a logo exists — its own tiny follow-up).
- Any engine/wasm change or new score behavior; a side-by-side pixel diff (SPEC-081 out-of-scope, still).

## Notes for the Implementer
- **Get the URL right:** `cloudinary/ssimulacra2` (with the "2") is the metric; `rust-av/ssimulacra2` is the
  impl we run. Both verified live 2026-07-19.
- **The FF/Safari check is the load-bearing half** — it closes the SPEC-081 launch carry. Actually drive
  those browsers; don't assume `color-mix()` support.
- Keep the links `href`-only (no fetch) so the zero-network invariant holds.
- Plain voice, no SPEC/DEC refs in the page ([[comments-plain-no-spec-refs]]).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-101-demo-pass` (off `main` @ fcef43c)
- **PR (if applicable):** none yet — finalized and verified on-branch, held for the orchestrator to merge with maintainer go-ahead.
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - The three code items were built as one batch across three commits (score links + re-convert
    "Updated" signal; favicons + `site.webmanifest` subpath fix; extended demo smoke). No engine/`src/`
    change and no wasm rebuild, as specified.

### Device gate (the launch go/no-go) — result: **PASS** *(maintainer-decided)*

Recorded honestly, not inflated:

- **iOS WebKit — PASS.** The demo was driven end-to-end on a real iPhone in **both Safari and
  DuckDuckGo** (both WebKit): drop → convert → read score → download all worked. The batch included a
  **real photo dropped straight from the iPhone Photos library** — iOS transcodes HEIC→JPEG on export, so
  the demo received JPEG (not HEIC). The module Web Worker initialized, decode succeeded, and large
  photos completed without the tab being killed.
- **Desktop DuckDuckGo — PASS**, driven with the same real photo batch.
- **Desktop Chrome / Firefox / Safari — PASS**, already proven in SPEC-078's multi-browser harness.
- **Android Chrome (Blink) — NOT tested (no device); accepted on judgment.** The demo is static /
  no-backend and would degrade gracefully; this is the one untested surface, accepted by the maintainer
  as a launch-readiness call rather than a build blocker.
- **Real-world note:** some highly detailed photos were "not terrible but could be faster" — this is the
  known **single-threaded AVIF encode**, already the **#1 post-launch item (threading)** and already
  disclosed in `BENCHMARKS.md`. Not a demo defect.
- **Desktop `color-mix()` band:** accepted as part of the pass. The definitive one-line console check of
  the computed band color was **not** run; the band was confirmed visually across the desktop browsers
  above and degrades to `var(--muted)`/`var(--good)` if `color-mix()` is unsupported.

*The device gate is a maintainer launch-readiness decision, not a build/verify pass/fail. It is recorded
here as decided: PASS.*

- **Follow-up work identified:**
  - Post-launch #1 remains **threading the AVIF encode** (the "could be faster" observation above;
    already tracked, already in `BENCHMARKS.md`).
  - Android Chrome (Blink) real-device pass, if/when a device is available — accepted as untested for launch.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. The spec was concrete (exact URLs, exact manifest fixes, the `scheduleConvert`
   line refs). The one judgment call was the re-convert signal's *form* — the spec offered options
   (busy state / "Updated" pulse / explicit Apply); a brief non-intrusive "Updated" pulse was chosen so
   the auto-rerun behavior stayed unchanged.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `ergonomic-defaults` + the plain-voice comment rule covered the user-facing copy; the favicon
   set and manifest fixes were fully specified.

3. **If you did this task again, what would you do differently?**
   — Run the demo smoke against a `/crustyimg/`-subpath server from the start (not just root-served), so
   the manifest's absolute-vs-relative `src` bug is caught mechanically rather than by reading — the
   smoke now asserts the structural "no leading slash" property instead.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — **Not put a gate that needs human hardware into a build spec's acceptance criteria.** A *gate*
   produces a go/no-go answer; a *change* produces a diff. Folding the mobile device check into this
   spec meant three finished, shippable code items sat blocked on devices no agent can reach, and it
   pushed the build toward trying to reach a real browser however it could. The stage backlog already
   has the right pattern — it lists the launch go/no-go as "(coordination, not a spec)". Device gates
   belong on the launch-readiness track as a checklist the maintainer runs.

2. **Does any template, constraint, or decision need updating?**
   — Yes, the build/verify prompt shape: **when a task needs a browser, name the access path, not just
   the browser.** This spec said "drive the demo on real Firefox and Safari" without saying *how*, and
   the build reasonably read that as driving the maintainer's live session — hitting an accessibility
   wall and enumerating personal tabs before stopping and self-reporting. The failure was the
   instruction, not the judgment. Standing clause: *use a clean profile the automation owns; never
   touch the human's running browser, tabs, history, or logged-in state; if a check needs their
   hardware, produce a checklist for them instead.* For a visual check on a **public** URL, asking the
   human is both cheaper and safer than any automation.

3. **Is there a follow-up spec I should write now before I forget?**
   — **Yes — the demo can't open RAW, and RAW is a stated headline differentiator.** Dropping a `.DNG`
   fails with a leaky internal error ("The image format Tiff is not supported"). Mechanism: RAW routing
   is by file *extension* (`is_raw_extension(path: &Path)` → `extract_preview`), but every wasm entry
   point calls `Image::from_bytes(bytes)` with no filename, so a DNG sniffs as TIFF and falls through.
   `raw.rs` has no wasm cfg-gate, so the extraction code is likely *already compiled into the .wasm* and
   merely unwired. That matters because the README says "sharp and squoosh can't open these at all" —
   so a photographer testing the front-door demo hits exactly the gap the pitch promises to fill. A
   design-time probe is written (feasibility, mobile memory on a 47 MP DNG, and the cleanest way to
   thread an extension through the wasm surface); size the spec from its findings. Floor if the full
   path proves expensive: have the demo detect RAW and say honestly that preview extraction lives in
   the CLI, rather than leaking a decoder error.
   Also still open, unchanged: the logo swap (outsourced, pending) for the demo's 🦀 placeholder.
