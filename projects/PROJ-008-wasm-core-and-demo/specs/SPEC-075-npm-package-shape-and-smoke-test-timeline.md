# SPEC-075 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time probe of `just wasm-build`'s `pkg/`: wasm-pack emits a valid package.json + typed
  `.d.ts`; the npm names `crustyimg` / `crustyimg-wasm` / `@jysf/crustyimg` are ALL free (404); and
  the `--target web` package needs explicit `init()`/`initSync`. Spec = settle identity/target/
  versioning (DEC-067) + `npm pack` → fresh-install → run `info`/`transform` client-side smoke test.
  NO live publish (SPEC-076, gated).
- [x] **build** (2026-07-12, PR #84, ~130k tok / ~$1.20 est) — DEC-067 settled: **`crustyimg-wasm`**,
  `--target web` (one artifact), version in lockstep with the crate, publish gated. `just wasm-npm-pkg`
  (finalize, **depends on `wasm-build`** so the DEC-066 size profile can't be bypassed) +
  `just wasm-npm-smoke` (npm pack → fresh install → import the bare specifier → `initSync` → `info` +
  `transform` to png/jpeg/webp/avif, output decoded back). No native addon, no lifecycle script, zero
  transitive deps. Size guard mutation-tested against a real bare build (1,503,817 B vs the profiled
  1,394,313 B — it trips). Native gates + deny + validate green; `src/` untouched. No publish.
- [ ] **verify** — fresh adversarial session: re-drive pack→install→run in a clean temp dir,
  confirm no native addon / no postinstall, packaged `.wasm` is the ~1.33 MB profiled one, native
  + deny unaffected. Confirm NO publish happened.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
