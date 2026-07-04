# SPEC-043 build prompt — supply-chain advisory response (deny.toml ignores)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-043 in the `crustyimg` repo
(cwd is the repo root). This is a **tiny, config-only supply-chain repair**: add three
documented `deny.toml` advisory `ignore` entries so `cargo deny check advisories` is
green again. **No code, no dependency change.** Open a PR and STOP. Follow this prompt
exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-043-supply-chain-advisory-response-quickxml-ttfparser.md`
   — the whole spec: `## Acceptance Criteria`, `## Failing Tests`, `## Notes` (it has the
   suggested `deny.toml` block).
2. `decisions/DEC-042-accept-quickxml-ttfparser-advisories.md` — authoritative rationale
   + revisit triggers to encode in the comments.
3. `deny.toml` — the `[advisories].ignore` list; the existing `RUSTSEC-2024-0436`
   (`paste`) entry is the exact style to mirror.

## What to build
1. **Confirm the failure first:** `cargo deny check advisories` → it FAILS on
   RUSTSEC-2026-0194, -0195 (quick-xml) and -0192 (ttf-parser).
2. **Edit `deny.toml`** — append three entries to `[advisories].ignore`, with preceding
   comments (dep chain + reachability + revisit trigger), matching the `paste` entry's
   style and referencing DEC-042. Use the block in the spec's `## Notes for the
   Implementer` as the template (adapt wording from DEC-042):
   - `RUSTSEC-2026-0194` + `RUSTSEC-2026-0195` — quick-xml via little_exif (EXIF-only
     path, not reached; no upgrade — little_exif pins `^0.37`).
   - `RUSTSEC-2026-0192` — ttf-parser unmaintained via ab_glyph (font input bundled/
     `--font`, not untrusted; no fix).
3. **Do NOT** touch `yanked = "deny"`, the `[bans]`/`[sources]`/`[licenses]` sections, or
   change the advisories check from `deny` to `warn`. Only add the three ignore entries.

## Hard rules
- **Config only** — `deny.toml` is the ONLY non-doc file changed. No `src/`, no
  `Cargo.toml`, no dependency upgrade/replacement (none is available).
- Add ONLY these three advisory IDs. Do not ignore anything else.
- DEC-042 is already authored — do NOT create a new DEC.

## Gates (all must pass)
```
cargo deny check advisories bans sources licenses    # MUST now report `advisories ok` (exit 0)
cargo fmt --all -- --check                            # unaffected (no src)
cargo build --quiet                                   # sanity (no code changed)
git grep -c 'RUSTSEC-2026-019' deny.toml              # == 3
grep -c 'warn' deny.toml                              # advisories check still `deny`, not downgraded
git diff --stat                                       # only deny.toml (+ spec/DEC docs)
```
(You do NOT need to run the full clippy/test suite — no code changed — but `cargo build`
should still succeed and `cargo deny` is the real gate here.)

## Git / PR
- Branch `fix/spec-043-advisory-ignores` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md`.
- PR title: `fix(SPEC-043): document quick-xml + ttf-parser advisory ignores (green the supply-chain gate)`.
- PR body per AGENTS.md §13: Decisions (DEC-042, DEC-037); Constraints
  (untrusted-input-hardening, one-spec-per-pr); New decisions (none — DEC-042
  pre-authored). State plainly: repairs the ambient supply-chain red on `main`; three
  documented, revisit-tracked ignores; no code/dep change; unblocks SPEC-042 + v0.1.0.
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session entry (agent `claude-sonnet-4-6`, numerics null).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-07-03
  notes: "deny.toml: 3 documented advisory ignores (RUSTSEC-2026-0194/-0195 quick-xml via little_exif; -0192 ttf-parser via ab_glyph), each with reason + revisit trigger per DEC-042. cargo deny advisories now green; no code/dep change; advisories check still `deny`."
```

## When done
`just advance-cycle SPEC-043 verify` (if it mis-globs, set `cycle: verify` in the spec
frontmatter by hand), open the PR with `gh`, and **stop** — the orchestrator pauses for
the maintainer before merge.
