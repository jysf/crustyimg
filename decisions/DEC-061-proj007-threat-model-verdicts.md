---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-061
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-10
supersedes: null
superseded_by: null

affected_scope:
  - src/recipe/mod.rs
  - src/build/cache.rs
  - src/build/watch.rs
  - src/build/mod.rs
  - src/build/lock.rs
  - tests/build.rs
  - tests/build_lock.rs
  - docs/research/proj-007-threat-model.md

tags:
  - security
  - threat-model
  - untrusted-input-hardening
  - build
  - recipe
  - cache
  - lockfile
  - watch
  - accepted-risk
---

# DEC-061: PROJ-007 threat-model verdicts + accepted risks

## Decision

The SPEC-068 adversarial pass over PROJ-007's five new untrusted-input surfaces —
manifest, recipe, `.crustyimg/` cache store, committed lockfile, `--watch` tree —
plus the `.to_str()` and exit-code cross-cutting seams, drove **hostile files against
the release binary** for each. The posture verdict: the wave's hardening holds, with
**two fixes** — recipe top-level `deny_unknown_fields`, and (found in the *verify*
pass, a ship-blocker) the **out-directory write-escape clamp**. One correctness wart is
pinned and filed; three risks are **explicitly accepted** with the rationale below (an
accepted risk is a decision, not an omission). The full evidence table is
`docs/research/proj-007-threat-model.md`.

### Verdicts (one per surface)

1. **Manifest — FIXED (out-directory write-escape) + SAFE otherwise.** Unknown key /
   oversize / duplicate-output / stdin source / name-traversal all resolve to a typed
   exit 2 (or a clamped exit 6 on the *name*-traversal attempt); nothing panics.
   **The verify pass found a real defect the original review missed:** `Target::validate`
   checked `out` only for empty, and `safe_join` clamps the per-input *name* while
   canonicalizing the `out` dir **as-declared** — so `out = "../ESCAPE/planted"` wrote
   re-encoded bytes **outside the project tree at exit 0** (reproduced on the real
   binary, reachable via `build --check`). Now clamped: `Target::validate` rejects an
   `out` that escapes the build tree (`..` or absolute) at the prepare/validate phase,
   exit 2, before any write (see the clamp decision below).
2. **Recipe — TIGHTENED (top level) + ACCEPTED (step level).** Added
   `#[serde(deny_unknown_fields)]` to `Recipe`. The surface-map premise that flatten
   blocked this was imprecise: the flatten is on `RecipeStep` only; `Recipe` has no
   flattened field. The top-level gap was the dangerous one — a typo'd key
   (`steps`/`stpe`) parsed to a zero-step recipe that copies its input unchanged.
3. **Cache store — SAFE + one correctness wart pinned/filed.** Verify-on-read,
   symlink refusal, and the `ext_len` bound all held against hand-corrupted bytes on
   the real binary. The store-vs-read **off-by-53** (payload-bounded store vs
   frame-bounded read) is confirmed, pinned by a boundary test, and filed — it is a
   silent-miss/wasted-slot **correctness** wart, never a panic or a wrong byte, so it
   does not force re-opening DEC-058.
4. **Lockfile — SAFE, no change.** SPEC-066's fix holds: non-hex `key` **and** `hash`,
   unknown field, bad version, and oversize are each a typed exit 2 produced *before
   any write*, with no panic. Regression test extended to both digest fields + a
   fail-before-write assertion.
5. **`--watch` — ACCEPTED + DOCUMENTED (not clamped).** Roots are un-clamped; an
   escaping `source = "../.."` watches out of tree (confirmed live). See accepted risk
   #2.

Cross-cutting: the `.to_str()→""` seams degrade to a deterministic, collision-detected,
path-safe fallback (accepted risk #4); the exit-code item **resizes** — `code()` is
compiler-exhaustive, so the only gap is missing *value* assertions (a filed
test-completeness patch, not a totality hole).

### Fix decisions

- **Recipe top-level `deny_unknown_fields`** (`src/recipe/mod.rs`) — see verdict #2.
- **Out-directory containment clamp** (`src/build/mod.rs`, SPEC-068 punch-list). A
  build must not write outside its declared tree.
  - **Where:** `Target::validate`, the manifest prepare/validate phase — fails
    **before** any filesystem write and before `Cache::open`, mirroring the SPEC-065
    injectivity precedent (a hostile manifest never starts a build).
  - **How:** a **lexical** containment check — the out dir may not exist yet, and
    canonicalize would follow symlinks. Reuses the watcher's `lexical_clean`
    (`src/build/watch.rs`, now `pub(crate)`): reject a cleaned `out` whose first
    component is `..` (`ParentDir`) or absolute (`RootDir`/`Prefix` — covers `/abs`,
    `C:\…`, `\server`). Component-based, so it is correct for both `/` and `\`.
  - **Containment base:** the **build root** = the process working directory that
    `source`/`recipe` resolve against under DEC-057. For a relative `out`, the lexical
    check is equivalent to "resolve against the root, assert `starts_with` the root".
    Asserted by `accepts_contained_out_directories` (`out = "dist"`, `"build/thumbs"`,
    `"./dist"`, `"a/../b"`, `"."` all pass) and `out_escape_is_typed_invalid_target`.
  - **Error + exit code:** typed `BuildError::InvalidTarget` → `CliError::Build(_)` →
    **exit 2**, naming the escaping `out` (Display, no `{:?}` — SPEC-065 Windows
    double-escape lesson). No new `CliError` variant, so `code()` stays
    compiler-exhaustive and `exit_code_mapping_is_total` still holds.
  - **Regression:** `build_rejects_out_directory_escape` (`tests/build.rs`) drives the
    real binary with a hostile FILE (relative `..` + absolute escape → exit 2, nothing
    written outside the tree; contained `out` still builds). Confirmed to fail without
    the clamp and pass with it.
  - **Residual (symlink) — un-caught, accepted, filed.** A pre-existing symlink
    *inside* the `out` path (`out = "linkdir/x"`, `linkdir →` out-of-tree) passes the
    lexical check **and is not stopped at write time either.** `safe_join` canonicalizes
    the out dir — following the symlink to its out-of-tree target — then checks only that
    the expanded *name* stays within that already-escaped dir; `reject_symlink_destination`
    fires only on a symlink destination *file*, not a regular file inside a symlinked dir.
    Re-verify drove `out = "linkdir/x"` and bytes landed **outside the tree at exit 0**;
    the "second layer" does not contain this case. **Accepted:** the exploit needs
    manifest control PLUS a symlink committed into the repo pointing out (a reviewable
    artifact under "reviewed like code"; a higher bar than the `..` escape). Closing it
    (require the *canonicalized* out dir to stay within the canonicalized build root)
    would reject intentionally symlinked output dirs (`dist → ramdisk`) — filed as
    backlog item #10, not done here.

### Accepted risks (each a decision, with rationale)

1. **Recipe step params stay tolerant.** An unknown `[[step]]` key is absorbed by
   `RecipeStep`'s `#[serde(flatten)] params` (a `BTreeMap`) and ignored. It is inert —
   never a path, a panic, or a wrong output. Strict per-step validation needs each
   operation to publish its accepted param names through the registry; that is a
   feature, filed as a new backlog item (#7), not a hardening fix. Pinned by
   `from_toml_tolerates_unknown_step_param_by_design`.
2. **`--watch` roots follow the declared manifest (un-clamped).** `--watch` is a
   local, interactive dev loop (DEC-060), not the CI surface (`build --check` is, and
   it registers no watcher). An out-of-tree root's only effect is extra inotify
   descriptors + rebuild triggers from unrelated activity — a resource/nuisance cost,
   never code-exec or exfil; outputs remain clamped by `safe_join`. Clamping would
   break legitimate monorepo layouts (source in `../shared/assets`) and under-watch
   silently. Pinned by `watch_root_escaping_source_follows_the_manifest_documented`;
   a *warning* (not a clamp) is filed as backlog item #8.
3. **A manifest may declare out-of-tree source READS (not writes).** The declared-build
   trust model — a build *reads* the paths it is told to, like a `Makefile`
   (`source = "../shared/**/*.png"`). Reading an out-of-tree file only ever yields an
   **in-tree** output (now that `out` is clamped, above) or a decode error. **Split
   from the original risk #3:** out-of-tree *writes* are **no longer accepted** — they
   are clamped (see the out-directory containment fix decision). The `out` write side
   is bounded by that lexical clamp **plus** `safe_join`'s per-name clamp (rejects
   empty / absolute / `..` / separator in the expanded name), both verified by attack.
   **One accepted exception:** a symlinked *out dir* still escapes both layers — see the
   symlink residual above; accepted (higher exploit bar: a committed in-tree symlink +
   manifest control) and filed as backlog item #10.
4. **The `.to_str()→""` stem/ext seams stay silent (for now).** They degrade to a
   deterministic, path-safe, injectivity-detected fallback, not a silent wrong output.
   A typed "filename not representable" error is a UX/correctness win folded into the
   filed unusual-filename hardening sweep (#5), not a security fix.

## Context

STAGE-024 is PROJ-007's closing hardening sweep; SPEC-068 is its LEAD — the SPEC-037
threat-model precedent applied to this wave's surface. The wave's recurring lesson is
that green exit-code tests miss hostile-serialized-input and committed-file defects
(three shipped-green defects proved it, incl. the SPEC-066 `short()` panic). So this
review drove hand-authored malformed `.toml`/`.lock`/cache-entry **bytes** against the
real binary rather than constructing Rust structs.

## Consequences

- The recipe layer now matches the manifest + lockfile `deny_unknown_fields`
  discipline at the top level; a typo'd recipe key is a typed error, not a silent
  no-op build.
- A build can no longer be made to write outside its declared tree: a target `out`
  that escapes via `..` or an absolute path is a typed exit-2 rejection at the
  prepare/validate phase, before any filesystem write. The out-of-tree *write* is no
  longer an accepted risk; out-of-tree *reads* still follow the declared manifest.
- The cache off-by-53 boundary is pinned; a future one-line fix can flip the test from
  clean-miss to round-trip without hunting for the boundary.
- The `--watch` root behavior is pinned as accepted, so a later change to `source_root`
  can't silently start (or stop) watching out of tree.
- STAGE-024's remaining backlog is reprioritized (see the note): fuzz-run is highest
  severity; the off-by-53, format sniff, build-profile key, unusual-filename sweep, and
  exit-code value-assertion patch are each their own spec; two new low-severity items
  (strict step params, watch-root warning) were added.

## Alternatives considered

- **Clamp `--watch` roots to under the manifest dir.** Rejected: breaks monorepo
  layouts and silently under-watches (accepted risk #2).
- **Fold in the cache off-by-53 fix here.** Rejected: it is a correctness wart on a
  shipped DEC-058 module, not a safety defect forcing a re-open; keeping it its own
  spec keeps the LEAD focused (per the spec's boundaries).
- **A `deny_unknown_fields`-equivalent for recipe step params.** Deferred: needs
  registry-published per-op param names — a feature, filed as backlog #7.
- **Convert the `.to_str()→""` seams to typed errors now.** Deferred to the
  unusual-filename sweep: no seam is reachable to a silent wrong output the injectivity
  check doesn't already catch.
