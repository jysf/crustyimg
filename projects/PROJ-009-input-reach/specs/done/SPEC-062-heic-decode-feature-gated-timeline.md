# SPEC-062 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — HEIC decode behind an off-by-default `heic` feature (implements DEC-052); Failing
  Tests (default-build `.heic`→exit-4 CodecNotBuilt, `#[cfg(feature="heic")]` decode to dims,
  dimension-cap→LimitsExceeded, corrupt→typed error, is_heic-vs-avif brands, dir-source discovery,
  optimize→webp / convert→png under the feature) + full Implementation Context. Load-bearing probe done
  in design: brew libheif 1.23.1 + a real `.heic` (via `sips`) + libheif-rs 2.7.0 → **decoded to 64×48
  with correct pixels; libheif-rs/-sys are MIT so `just deny` stays green with NO exception (the LGPL is
  the SYSTEM C lib, invisible to deny — unlike ansi_colours); system-linked via pkg-config**. Approach →
  **DEC-056** (emit at build). Key wiring: a decode-side `ImageError::CodecNotBuilt`→exit-4 (mirror
  SinkError), ftyp-brand detection (AVIF dispatched first), a system-libheif CI job, and exclusion from
  every distributed artifact (DEC-052). Framing, 2026-07-08.
- [x] **build** — add `src/image/heic.rs` (`is_heic` ftyp scan + `#[cfg(feature="heic")]` libheif-rs
  decode → canonical `Image`, DEC-034-capped, stride-honoring), `ImageError::CodecNotBuilt`→exit-4,
  dispatch in `decode_with_limits` (after AVIF), `.heic`/`.heif` in `IMAGE_EXTENSIONS`,
  `heic = ["dep:libheif-rs"]`, system-libheif CI job, distribution excludes heic, LGPL attribution,
  fixture via sips + tests + fuzz target, DEC-056. Verify default(exit-4) + `--features heic`(decode) +
  lean + `just deny`(green, no exception) + clippy×3 + fmt; check MSRV (bindgen/libheif-sys floor).
  **DONE 2026-07-08 — PR #68.** All 8 acceptance criteria met; DEC-056 emitted. Gates green: 581 default
  / 588 `--features heic` / 581 lean tests, clippy×3, fmt, `just deny` (**no new exception** — libheif-rs
  + libheif-sys are MIT and DO enter the `all-features` graph). Probe corrected two design assumptions:
  **no bindgen** (pre-generated bindings → no libclang, **no MSRV move**, floor stays 1.90), and the
  version feature must be pinned **`v1_17`** (libheif-rs's default `latest`=v1_21 would demand a system
  libheif ≥1.21, more than `ubuntu-latest`'s apt 1.17.6) — which in turn puts `set_security_limits`
  (`#[cfg(v1_19)]`) out of reach; the DEC-034 handle-dim pre-check is the load-bearing bound. Est. cost
  ~260k tokens / ~$2.34 (labelled estimate — main-loop, not separately metered).
- [x] **verify** — ✅ APPROVED (fresh Opus session, run from scratch). Re-ran all three builds (default
  582, --features heic 588, lean 582, clippy×3, fmt, `just deny` green + `git diff main -- deny.toml`
  empty), re-audited all 8 `Err(_)` catch-alls (only lint needed the fix), proved cap-before-decode on
  the handle, and made a 67×45 HEIC to prove the stride-padding path (stride 208 vs row_bytes 201,
  unsheared). Confirmed the ubuntu heic job actually DECODES (with libheif-plugin-libde265), the plugin
  gotcha via the real failed CI run, distribution excludes heic, DEC-056 consistent with DEC-052. 3
  non-blocking ship items (DEC-056 affected_scope + stride-test follow-up + a 581→582 cosmetic). 2026-07-08.
- [x] **ship** — squash-merged PR #68 → main (b2f370a); appended verify + ship cost sessions + totals
  (490k, labelled estimates §4), ship reflection, marked cycle ship; archived to done/; STAGE-019 shipped;
  **PROJ-009 PROJECT-SHIPPED** (final stage — project reflection filled in brief.md). Ship items applied
  (DEC-056 affected_scope += cli/lint, 581→582); `fuzz/heic_decode` + stride-test + Windows/v1_19 carried
  as follow-ups in docs/roadmap.md. 2026-07-08.
