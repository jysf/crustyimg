# PROJ-007 threat-model / attack-surface review (SPEC-068)

*STAGE-024 LEAD. A systematic adversarial pass over the **new untrusted-input
surface** PROJ-007 added — the build manifest, recipe files, the `.crustyimg/`
cache store, the committed lockfile, and the tree `--watch` walks — mirroring
SPEC-037's shape for this wave. Every verdict below was **earned by driving a
hand-authored hostile file (or the real release binary), not by reading the
code**. A dismissed suspect ("here's the attack, here's why it held") is a
first-class result.*

- **Binary under attack:** `target/release/crustyimg 0.4.0`, default features
  (`display`, `watch`).
- **Posture checked against:** `guidance/constraints.yaml → untrusted-input-hardening`;
  DEC-034/DEC-035 (decode caps + sink traversal refusal); DEC-025 (exit codes).
- **Threat model:** in CI (the "reviewed like code" story) all five files arrive
  from a pull request the maintainer did **not** write. They are untrusted input,
  not just local config.
- **Verdict summary:** 3 of 5 surfaces held with **no defect**; **2 fixes applied**
  — recipe top-level `deny_unknown_fields` (found in review), and the **out-directory
  write-escape clamp** (Surface 1, SPEC-068 — a real ship-blocker the *verify* pass
  found: a hostile `out = "../.."` wrote re-encoded bytes outside the project tree at
  exit 0; now rejected at exit 2 before any write); 1 confirmed correctness wart
  pinned + filed (cache off-by-53); the exit-code item **resized** as predicted.
  Decisions + accepted risks recorded in **DEC-061**.

---

## Surface 1 — Manifest (`crustyimg.build.toml`)

- **Entry point.** `run_build`/`load_manifest` (`src/cli/mod.rs:1222`, on-disk cap
  `:1228`) → `BuildManifest::from_toml` (`src/build/mod.rs:203`).
- **Guards in place.** Size cap before parse (`:205`); `#[serde(deny_unknown_fields)]`
  on `BuildManifest` (`:188`) and `Target` (`:166`); version gate (`:217`);
  target-count cap (`:224`); per-target `validate` (`:243`) rejecting empty fields,
  a `-` (stdin) source, **and an `out` that escapes the build tree** (SPEC-068 clamp,
  below); global injectivity `find_output_collision` (`:300`, SPEC-065). Loader path
  handling uses no `.to_str()`/index.
- **Hostile inputs driven (real binary).**

  | Attack | File | Result |
  |---|---|---|
  | unknown top-level key | `version = 1` + `bogus = 42` | **exit 2**, `unknown field 'bogus'` |
  | unknown target key | `[[target]] … evil = 1` | **exit 2**, names `evil` |
  | oversize (64 KiB + 1, all `#`) | comment-only, would parse | **exit 2**, `too large (65537 … max 65536)` — rejected *before* parse |
  | duplicate output (2 sources → 1 name) | `source = ["a/logo.png","b/logo.png"]` | **exit 2**, `output collision … "dist/logo.{ext}"` |
  | stdin source | `source = "-"` | **exit 2**, `` `source` may not be `-` `` |
  | output-name traversal | `name = "../../../pwned.png"` | **exit 6**, `output path escapes the target directory`; no file written outside `out` (`safe_join`) |
  | **out-DIRECTORY traversal (write-escape)** | `out = "../ESCAPE/planted"` | **exit 2**, `` `out` escapes the build tree via `..` ``; **no file written outside the tree** — the SPEC-068 clamp, below |
  | **out-directory absolute** | `out = "/abs"` (or `C:\…`) | **exit 2**, `` `out` must be within the build tree, not an absolute path `` |

- **DEFECT FOUND IN VERIFY → FIXED HERE (the out-directory write-escape).** The
  original review recorded this surface as SAFE on the strength of the *name*-clamp
  (`safe_join`, the exit-6 row). That was **wrong for the `out` dir itself**:
  `Target::validate` checked `out` only for empty, and `safe_join` clamps the
  per-input *name* within the `out` dir while canonicalizing that dir **as-declared**.
  So a manifest `out = "../ESCAPE/planted"` drove `crustyimg build` to `create_dir_all`
  the escaping directory and write re-encoded image bytes **outside the project tree,
  at exit 0** — reproduced on the real binary, and reachable via `build --check`, the
  review's named "safe CI surface". This falsified the "never writes outside `out`"
  claim here and accepted-risk #3 in DEC-061.
  - **Fix (SPEC-068):** `Target::validate` (`src/build/mod.rs`) now rejects an `out`
    that escapes the build tree — **lexically** (the out dir may not exist yet, and
    canonicalize would follow symlinks): it reuses the watcher's `lexical_clean`
    (`src/build/watch.rs`) and rejects a cleaned `out` whose first component is `..`
    (`ParentDir`) or absolute (`RootDir`/`Prefix`, covering `/abs`, `C:\…`, `\srv`).
    **Containment base = the build root**, i.e. the process working directory that
    `source`/`recipe` resolve against (DEC-057); for a relative `out` this lexical
    check is equivalent to "resolve against the root, assert `starts_with` the root".
    Caught at the **prepare/validate phase** — before `Cache::open`, before any write —
    a typed `BuildError::InvalidTarget` → **exit 2**, mirroring the SPEC-065 injectivity
    precedent (a hostile manifest never starts a build). No new `CliError` variant, so
    `code()` stays compiler-exhaustive and `exit_code_mapping_is_total` still holds.
  - **Pinned by** `build_rejects_out_directory_escape` (`tests/build.rs`, drives the
    real binary with a hostile FILE: relative `..` escape + absolute escape → exit 2,
    nothing written outside the tree; a contained `out = "dist"` / `out = "build/thumbs"`
    still builds) and unit tests `out_escape_is_typed_invalid_target` /
    `accepts_contained_out_directories` (`src/build/mod.rs`).
- **Verdict: FIXED for lexical `../`/absolute escapes — a traversal `out` is rejected
  at exit 2 before any write; nothing lands outside the tree via `..` or an absolute
  path.** The remaining hostile manifests are each a typed exit-2 (or a clamped exit-6
  on the *name*-traversal attempt); nothing panics. **One residual is NOT closed: a
  symlinked *out dir* (below) still escapes — accepted (higher exploit bar) and filed.**
- **Residual risk — reads vs. writes (now distinct).**
  - **WRITES: lexically clamped.** `out` may not escape the build tree via `..` or an
    absolute path (this fix). A symlinked *out dir* is the one un-caught residual (below).
  - **READS: still follow the declared manifest (accepted).** A manifest may declare
    **out-of-tree sources** (`source = "../../**/*.png"`): the live `--watch` run below
    resolved exactly such a glob and matched files two directories up. This is the
    **declared-build trust model** — a build reads the paths it is told to, like a
    `Makefile` — and reading an out-of-tree file only ever produces an *in-tree* output
    (now that `out` is clamped) or a decode error, never an escape. Accepted (DEC-061
    accepted-risk #3, reads); the watch-specific amplification is Surface 5.
  - **Symlink residual — NOT caught (accepted + filed).** A pre-existing symlink
    *inside* the `out` path (`out = "linkdir/x"`, `linkdir → <out-of-tree dir>`) passes
    the lexical containment check **and is not stopped at write time either.**
    `safe_join` canonicalizes the out dir — which *follows* the symlink to its
    out-of-tree target — and then only checks the expanded *name* stays within that
    already-escaped dir; the DEC-035 guard (`reject_symlink_destination`) fires only on
    a symlink *destination file*, not a regular file inside a symlinked dir. **Driven in
    re-verify: `out = "linkdir/x"` wrote bytes to the symlink target outside the tree at
    exit 0** — the "second layer" does not contain this case. **Accepted residual:** the
    exploit needs manifest control **plus** a symlink committed *inside* the repo pointing
    out — a reviewable artifact under the "reviewed like code" model, and a materially
    higher bar than the `..` escape (which needed nothing pre-planted). Closing it
    (require the *canonicalized* out dir to stay within the canonicalized build root)
    would reject intentionally symlinked output dirs (e.g. `dist → ramdisk`) — filed as
    backlog item #10, not done here.

---

## Surface 2 — Recipe files

- **Entry point.** `run_apply` (`src/cli/mod.rs:1020`, on-disk cap `:1022`) →
  `Recipe::from_toml` (`src/recipe/mod.rs:222`).
- **Guards in place.** Size cap before parse (`:224`); version gate (`:235`);
  step-count cap (`:244`); unknown **op name** is a hard `UnknownOperation`
  (`:266`), never silently skipped.
- **Suspect (from the surface map).** `Recipe` and `RecipeStep` carried **no**
  `deny_unknown_fields`, unlike the manifest + lockfile — so unknown keys were
  silently tolerated. The map attributed this to `#[serde(flatten)] params`.
- **Hostile inputs driven (real binary, `apply --recipe … img.png -o o.png`).**

  | Attack | Result *(before fix)* |
  |---|---|
  | unknown **top-level** key (`bogus = 42`) | **exit 0**, output produced — silently tolerated |
  | unknown **step** param on `resize` (`EVIL = "x"`) | **exit 0**, resize ran, param ignored |
  | unknown **step** param on paramless `auto-orient` | **exit 0**, tolerated |
  | malformed TOML | exit 1, `could not parse recipe TOML` |
  | unsupported version (`"2"`) | exit 1, `unsupported recipe version '2'` |

- **Finding — the suspect's premise was imprecise.** The `#[serde(flatten)]` is on
  **`RecipeStep`** only (`OperationParams` is a `BTreeMap<String, toml::Value>`, so
  it absorbs any extra step key). **`Recipe` itself has no flatten** — its fields are
  `version` / `name` / `description` / `step` — so `deny_unknown_fields` *is*
  applicable at the top level. The top-level gap was the more dangerous one: a typo'd
  key (`steps` plural, `stpe`, `verison`) parsed to a **zero-step recipe that copies
  its input unchanged** — a silent wrong output on a committed file the maintainer
  didn't write.
- **Verdict: TIGHTENED HERE (top level) + ACCEPTED (step level).**
  - **Fix:** `#[serde(deny_unknown_fields)]` added to `Recipe` (`src/recipe/mod.rs`).
    A top-level unknown key is now `exit 2`-class `RecipeError::Parse`. No existing
    recipe fixture uses extra top-level keys (grep-confirmed), and round-trip is
    unaffected (`to_toml` emits only known keys). Pinned by
    `from_toml_rejects_unknown_top_level_key` (hand-authored hostile strings incl.
    the `steps`/`stpe` footgun).
  - **Accepted risk:** an unknown **step** param stays tolerated — inert (never a
    path, panic, or wrong output), and a strict per-step check needs each op to
    publish its accepted param names through the registry. Pinned as *accepted* by
    `from_toml_tolerates_unknown_step_param_by_design`, so a future strict-params
    spec flips it deliberately. Filed as a new backlog item.
- **Residual risk.** Recipe parse errors map to exit **1** (generic runtime), not
  exit 2 like the manifest/lockfile. This is pre-existing shipped behavior
  (`CliError::Recipe(_) => 1`, documented) and out of this spec's scope; noted for
  consistency, not fixed.

---

## Surface 3 — `.crustyimg/` cache store

- **Entry point.** `Cache::lookup` → `read_entry`/`parse_entry`
  (`src/build/cache.rs:388`/`:431`); write via `store_bounded`/`write_entry`
  (`:354`/`:415`). Frame: `MAGIC(12) | ext_len(1) | ext | payload_len(8) | hash(32)
  | payload` — a **53-byte + ext** header.
- **Guards in place.** `symlink_metadata` + non-regular-file refusal (`:392`);
  bounded read `take(max+1)` (`:402`); checked frame splits (`split_at_checked`,
  no panic); verify-on-read re-hash (`:453`); hex-only entry paths (`:323`), so no
  caller string reaches a path component (no traversal); atomic stage-and-rename.
- **Hostile inputs driven (real binary — a real cached build, then hand-corrupt an
  entry; each must be a MISS = rebuild, never a panic).**

  | Corruption | Result |
  |---|---|
  | bad magic | **miss → rebuilt**, exit 0 |
  | truncated to 1 byte | **miss → rebuilt** |
  | empty file | **miss → rebuilt** |
  | flipped last payload byte (verify-on-read) | **miss → rebuilt** — the altered byte is caught |
  | `ext_len` byte set to 200 (`> MAX_EXT_LEN`) | **miss → rebuilt** |
  | entry replaced by a **symlink** to a valid out-of-tree entry | **miss → rebuilt**; the link is **not** followed, and the atomic store overwrites it with a real file |

- **Suspect — the store-vs-read off-by-53.** `store_bounded` bounds the **payload**
  (`bytes.len() > max`, `:361`); `read_entry` bounds the **whole frame**
  (`53 + ext + payload`, `:393`/`:402`). So a payload in the top `53 + ext` band of
  `CACHE_ENTRY_MAX_BYTES` is **stored but unreadable** — a permanent silent miss.
  The existing `oversize_entry_is_a_miss` (10-byte payload) never reaches the band.
  - **Confirmed** by a new boundary unit test
    `near_cap_payload_round_trips_or_is_a_clean_miss` (exercised against a small
    `max` like the shipped oversize test, not a 256 MiB write): a near-cap payload is
    stored, then read-refused as a **clean miss with no panic**; a sub-band payload
    round-trips.
- **Verdict: SAFE (never a panic, never a served wrong byte) + one CORRECTNESS wart
  PINNED & FILED.** The off-by-53 is a wasted cache slot + always-rebuild, not a
  safety hole — it does not force re-opening DEC-058. The one-line fix (bound the
  **frame** in `store_bounded`) is its own STAGE-024 backlog spec; the test pins the
  current safe behavior so the boundary can't drift.
- **Residual risk.** None for safety. The wart's only cost is a permanently
  uncacheable near-256-MiB output (rebuilt every run).

---

## Surface 4 — Committed lockfile (`crustyimg.build.lock`)

- **Entry point.** `run_build --check` (`src/cli/mod.rs:1507`) →
  `BuildLock::from_toml` (`src/build/lock.rs:221`). **This is the surface that
  already bit us (SPEC-066's `short()` panic on a non-hex digest).**
- **Guards in place.** Size cap before parse (`:223`); `deny_unknown_fields`
  (`:173`); version gate (`:234`); **hex validation of every `key`/`hash`**
  (`:248-257`) at the boundary; `short()` slices with `get(..n)` (`:391`, boundary-
  safe); `Display` quotes paths literally, never `{:?}` (no Windows double-escape);
  keys/hashes/paths compared as opaque strings (nothing reaches a filesystem path).
- **Hostile inputs driven (real binary, `build --check`; MD5 before/after proves
  no write).**

  | Attack | Result | Lockfile written? |
  |---|---|---|
  | non-hex `key` = `"a€€€"` (the SPEC-066 panic vector) | **exit 2**, `key is not a hex digest` | **no** |
  | non-hex `hash` = `"not-hex!"` | **exit 2**, `hash is not a hex digest` | **no** |
  | unknown field (`bogus = 1`) | **exit 2**, names `bogus` | **no** |
  | bad version (`999`) | **exit 2**, `unsupported … version 999` | **no** |
  | oversize (4 MiB + 1) | **exit 2**, `too large` | **no** |
  | empty `key` (`""`) | **exit 2**, `key is not a hex digest` | **no** |

- **Verdict: SAFE — no change. SPEC-066's fix holds under hostile bytes.** Every
  hostile lockfile is a typed exit 2, produced in the prepare phase **before any
  output or lockfile write**; no panic anywhere (`panicked` absent from stderr).
  Regression coverage extended: `non_hex_digest_in_lock_is_exit_2_not_a_panic` now
  drives **both `key` and `hash`** through the real binary and asserts fail-before-write.
- **Residual risk.** None found.

---

## Surface 5 — `--watch` tree

- **Entry point.** `watch_roots` (`src/build/watch.rs:98`) →
  `is_excluded`/`normalize_abs` (`:144`/`:227`) → notify `register_roots`
  (`src/cli/mod.rs:1864`), `watch_impl` (`:1881`). `watch` is a **default feature**;
  `--watch` blocks until Ctrl-C.
- **Guards in place.** `is_excluded` normalizes both sides and matches by path
  **components** (`dist` ≠ `distortion`), so the build's own writes to
  `out`/`.crustyimg`/lockfile don't self-trigger; an unwatchable root is a warning,
  not a failure; the manifest/recipe dirs are watched *shallow* to avoid the sibling
  cache tree overflowing inotify.
- **Suspect — un-clamped watch roots.** `source_root`/`lexical_clean` keep `..`
  above a relative root (no containment clamp), so a manifest `source = "../.."`
  makes `--watch` register a recursive OS watch **outside** the project tree.
- **Hostile input driven (real binary, backgrounded + `SIGINT`).** A project at
  `…/watch/proj/` with `source = "../../**/*.png"`:
  - the initial build **resolved sources two directories up**, matching
    `../../attack/a/logo.png` and `../../attack/b/logo.png` (leftover fixtures from
    Surface 1) — concretely proving the out-of-tree reach;
  - the initial build failed with the injectivity `output collision` (exit-caught,
    **no panic**), and — by design — the **watcher still started** (`watching for
    changes (Ctrl-C to stop)…`): an initial-build failure is recoverable under a
    watch loop;
  - `SIGINT` (default handler, no `ctrlc` dep) terminated cleanly.
  - Pinned by `watch_root_escaping_source_follows_the_manifest_documented`
    (`source_root("../..") == ".."`; `watch_roots` on an escaping source yields a
    `../..` recursive root).
- **Verdict: ACCEPTED + DOCUMENTED (not clamped) — READS/WATCHES only.** `--watch`
  is a **local, interactive dev loop** (DEC-060), *not* the CI surface — CI runs
  `build --check`, which never registers a watcher. Its **watch roots** follow the
  declared manifest, the same *read* reach `build` itself has. This is strictly a
  read/watch question: the **write** side is now clamped independently — a target's
  `out` cannot escape the build tree (Surface 1, SPEC-068), on top of `safe_join`'s
  per-name clamp. The only effect of an out-of-tree watch root is extra inotify
  descriptors + rebuild triggers from unrelated file activity — a **resource/nuisance
  cost, never code-exec or exfil**.
  Clamping was rejected: it would break legitimate monorepo layouts (a source in
  `../shared/assets`) and *under*-watch silently (miss real edits). Recorded as an
  accepted risk in DEC-061 and pinned so it can't drift.
- **Residual risk / notify symlink question.** `normalize_abs` deliberately does not
  canonicalize, so it won't *match* a symlink out of a watched root; whether the OS
  backend (FSEvents on macOS, inotify on Linux) *follows* a symlinked directory out
  of a recursive root is platform-dependent. Even if it does, the sole consequence is
  another rebuild trigger — the rebuild only ever reads the **declared** sources, so
  there is no escalation. A new **low-severity** backlog item proposes a *warning*
  (not a clamp) when a watched root escapes the manifest dir.

---

## Cross-cutting A — `.to_str()` stem / ext / path seams

- **Map.** Silent `.unwrap_or("")` (unless noted): source stem
  (`src/source/mod.rs:54` → `""`), source ext (`:117`, `None` → not-an-image, a safe
  gate), **sink ext (`src/sink/mod.rs:186` — ERRORS, good)**, sink `{name}` (`:273`,
  falls back to `{stem}.{ext}`), sink `{parent}` (`:281` → `""`), cache-key
  `input_ext` (`src/cli/mod.rs:1351` → `""`). No non-test `.to_str().unwrap()` on a
  path exists.
- **Analysis — is any silent `""` reachable to a *wrong output* the injectivity
  check doesn't catch?** The build's collision key (`output_collision_key`,
  `src/cli/mod.rs:1288`) is composed from `expand_template(template, input.stem(),
  EXT_SENTINEL, path)` — i.e. **it uses the same silent stem**. So two inputs with
  non-`to_str`-able stems both expand to `dist/.png` → the *same* collision key →
  caught as an `OutputCollision` (exit 2), not a silently-overwritten output. A
  **single** non-UTF-8-stem input writes a surprising-but-deterministic `.png` (a
  hidden file on Unix), path-safe via `safe_join`, never an escape.
- **Verdict: SAFE for this spec's threshold — the full sweep is a separate item.**
  The silent seams degrade to a *deterministic, path-safe, collision-detected*
  fallback, not a silent wrong output that corrupts or escapes. Converting them to
  typed errors is a UX/correctness improvement (a clear "filename isn't
  representable" beats a `.png`), but it is the **unusual-filename hardening sweep**
  already on the backlog, not a security defect to fix inline. No seam met the
  "clearly reachable to a silent wrong output" bar for an inline fix.
- **Residual risk.** A single non-UTF-8-stem (or empty-stem) input produces a
  `.png`/`.{ext}` output name. Low severity; folded into the filed sweep.

---

## Cross-cutting B — Exit-code totality

- **Map.** `CliError` (`src/cli/mod.rs:493`); `code()` (`:632`) is a match with **no
  `_` wildcard** — it is **compiler-exhaustive**, so adding a variant breaks the
  build. `exit_code_mapping_is_total` (`:4737`) is a hand-listed **value**-assertion
  test. Confirmed against `docs/api-contract.md` (codes 0–7): every `code()` arm
  matches the contract.
- **Finding — the item RESIZES exactly as predicted.** There is **no totality hole**
  (the compiler guarantees arm coverage). The real gap is only **missing value
  assertions** in the hand list: `exit_code_mapping_is_total` asserts
  `Metadata::UnsupportedFormat` (→ 4) but **not** `Metadata::Container` (→ 1) or
  `Metadata::Exif` (→ 1); it asserts `Watch::Watcher` but not the sibling `Watch::Watch`
  (both hit the same `Watch(_) => 1` arm); several `Sink(_) => 5` variants ride the
  catch-all unasserted. (The two prior misses, `Cache` and `Metadata`, were likewise
  missing *value assertions*, not missing arms.)
- **Verdict: RESIZED — confirmed, filed, not over-built.** The audit is "assert every
  variant's documented value + keep the hand list complete", a small
  test-completeness patch — **its own backlog spec** (in this spec's out-of-scope
  list), not a totality fix. No safety issue: a mis-mapped exit code cannot ship,
  because the compiler-exhaustive `code()` already forces an explicit arm per variant.

---

## Fuzz targets (context for the backlog)

`fuzz/fuzz_targets/{avif_decode,heic_decode,svg_decode}.rs` → `Image::from_bytes`;
`raw_preview.rs` → `raw_preview(data)`; all assert "never panic". **Running** them
needs `cargo +nightly fuzz`, absent from the build/verify env, so this LEAD does not
run them — it ranks the run **highest-severity** and hands off the recipe:
`cargo +nightly fuzz run <target> -- -max_total_time=<budget>` per target, triage any
crash, record the run. This is the untrusted-**binary**-decode surface (PROJ-009's
codecs), the one high-severity unknown this review can't close from here.

---

## Reprioritized STAGE-024 backlog

*Each item: **confirmed / resized / dismissed** + severity + one-line why. This is
the LEAD's output that makes the rest of STAGE-024 targeted.*

| # | Item | Disposition | Severity | Why |
|---|---|---|---|---|
| 1 | **Run the decoder fuzz gate** (AVIF/SVG/RAW/HEIC) + fix findings | **Confirmed — frame next** | **High** | The one surface this review can't close: untrusted-binary decode, never fuzzed. Recipe handed off above; needs nightly/cargo-fuzz. |
| 2 | **Cache off-by-53** read-bound fix + regression test | **Confirmed (correctness, not safety)** | **Low–Med** | Store bounds payload, read bounds frame → near-cap payload stored-but-unreadable. Pinned by `near_cap_payload_…`; one-line frame-bound fix in `store_bounded`. |
| 3 | **Pre-decode format sniff** (closes SPEC-065 `{ext}` false positives + SPEC-066 literal-`{ext}` residual) | **Confirmed** | **Med** | Still real; the injectivity check + lockfile catch the residual today, but only after the prepare phase. DEC-059 threat-model item. |
| 4 | **Cache-key / determinism-envelope completeness** (build profile) | **Confirmed** | **Med** | Debug-vs-release is output-affecting yet unkeyed (not in DEC-058's seven). Add to the key or bump `CACHE_SCHEMA_VERSION`. |
| 5 | **Unusual-filename hardening** (non-UTF-8 / empty stem → typed error) | **Confirmed — resized to UX/correctness** | **Low** | Cross-cutting A: silent `""` seams degrade to a deterministic, collision-detected, path-safe fallback — not a security hole. Worth a typed error, not urgent. |
| 6 | **Exit-code totality audit** + `is_total` value-assertion completeness | **Resized** | **Low** | Cross-cutting B: `code()` is already compiler-exhaustive; the gap is missing *value* assertions (`Metadata::Container`/`Exif`, `Watch::Watch`), a test-completeness patch. |

**New items this review adds:**

| # | Item | Severity | Origin |
|---|---|---|---|
| 7 | **Strict per-step recipe params** (reject unknown `[[step]]` keys via registry-published param names) | Low | Surface 2 accepted risk — top level is now `deny_unknown_fields`; step level stays tolerant by design. |
| 8 | **`--watch` root-containment *warning*** (warn, don't clamp, when a watched **read/watch** root escapes the manifest dir — reads only) | Low | Surface 5 accepted risk — preserves monorepo layouts while surfacing an out-of-tree watch. |
| 9 | **out-directory containment** (reject a target `out` that escapes the build tree) | **DONE (lexical)** | ~~High~~ — **fixed here (SPEC-068 punch-list).** Verify found the write-escape; clamped at `Target::validate` (exit 2, prepare-phase), pinned by `build_rejects_out_directory_escape`. Catches `..`/absolute; the symlinked-out-dir residual is **not** caught (→ item #10). |
| 10 | **Canonicalize-contain the `out` dir** (require the *canonicalized* out dir to stay within the canonicalized build root) | Low–Med | Re-verify: a committed in-tree symlink (`out = "linkdir/x"`, `linkdir →` out-of-tree) escapes the lexical clamp and the write-time `safe_join`. Accepted residual (needs a committed symlink + manifest control); closing it rejects intentionally symlinked output dirs (`dist → ramdisk`) — a real tradeoff, hence its own spec. |

**Carried from SPEC-067 verify (unchanged by this review):**
`--watch` as a global clap flag is a silent no-op on non-build subcommands (reject or
document); orphaned-output prune on source removal (a future `--clean`).

---

## What was fixed here vs. what was recorded

- **Fixed inline (with hostile-file regression tests):**
  - recipe top-level `deny_unknown_fields` (`src/recipe/mod.rs`) — surfaced by the
    original review.
  - **out-directory write-escape clamp (`src/build/mod.rs`, SPEC-068 punch-list)** —
    surfaced by the *verify* pass as a ship-blocker: `Target::validate` now rejects an
    `out` that escapes the build tree (`..` / absolute) at exit 2, before any write.
    Pinned by `build_rejects_out_directory_escape` + two unit tests.
- **Pinned + filed (no inline fix — its own spec):** cache off-by-53 (boundary test
  added, fix deferred).
- **Accepted risks (DEC-061):** recipe step-param tolerance; `--watch` un-clamped
  **read/watch** roots; manifest out-of-tree declared **sources** (reads); the
  `.to_str()→""` seams (folded into the unusual-filename sweep). *(Out-of-tree
  **writes** are no longer an accepted risk — they are clamped, above.)*
- **Held with no defect (dismissed suspects):** manifest hardening; cache
  verify-on-read / symlink refusal / `ext_len` bound; lockfile SPEC-066 fix
  (both `key` and `hash`); exit-code arm coverage (compiler-exhaustive).
