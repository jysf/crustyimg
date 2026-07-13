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
- [ ] **build** — through a PR. Finalize package.json + target + name; packaging recipe through
  `just wasm-build` (size-profiled, DEC-066); `tests/npm_smoke.mjs` + `just wasm-npm-smoke`
  (pack→install→init→run); DEC-067. Native unaffected. Commit with `-s`.
- [ ] **verify** — fresh adversarial session: re-drive pack→install→run in a clean temp dir,
  confirm no native addon / no postinstall, packaged `.wasm` is the ~1.33 MB profiled one, native
  + deny unaffected. Confirm NO publish happened.
- [ ] **ship** — squash-merge, bookkeeping on main, cost totals, reflection, memory + brag.
