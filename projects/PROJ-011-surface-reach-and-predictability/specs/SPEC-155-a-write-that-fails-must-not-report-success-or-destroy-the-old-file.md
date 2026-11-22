---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-155
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-11-22

references:
  decisions:
    - DEC-035
    - DEC-044
    - DEC-059
    - DEC-061
  constraints: []
  related_specs:
    - SPEC-154
    - SPEC-157
    - SPEC-158

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: "STAGE-063 success criterion: a write that fails exits non-zero and leaves the previous file intact"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-155: A write that fails must not report success or destroy the old file

## Context

STAGE-063's entry "A write that fails must not report success or destroy the old file (review P1)",
from `docs/reviews/2026-11-10-external-code-review.md`. Every image crustyimg writes to a file goes
through four arms in `src/sink/mod.rs`: `Sink::File` and `Sink::Dir`, in `write` (pixel lane) and
`write_bytes` (container lane: `meta`, `optimize`'s pass-through, `build` outputs). Each arm runs the
symlink and overwrite guards, then opens the destination with `truncate(true)`, wraps it in a
`BufWriter` (8 KiB), calls `write_all` and returns `Ok(())`. Nothing calls `flush()`. An output under
8 KiB stays in the buffer, the buffer is flushed when it is dropped, and that flush's error is thrown
away. A larger output writes straight through and its error propagates, but the old file was
truncated before the first byte, so a failure destroys it. That includes `-y` over the input itself
and `meta` writing in place. `--save-recipe` (`src/cli/ops.rs`, `fs::write`) truncates the same way.

## Goal

A file write either replaces the destination completely or leaves it exactly as it was, and any
failure exits non-zero. Nothing crustyimg writes reports success over a short file, and a failed
`-y` never costs the user the file they already had.

## Frame

**Verdict: GO at [M].** Build starts after SPEC-154 has merged (STAGE-063 lands SPEC-154 first and
alone). Frame session 2026-11-22, Opus, on `main` at `ff3c3fc`, release build in a fresh
`CARGO_TARGET_DIR` (deleted after).

### Premise check (driven)

Probe: a 512×512 PNG fixture (`tests/fixtures/classify/photo_entropy_floor.png`, 83,916 B; PNG out
198,662 B, "big") and a 40×40 PNG made from it (1,328 B, "small"; under the 8 KiB buffer). The old
destination holds a 29-byte marker. Each command ran in a subshell under `ulimit -f 1` (512 B, zsh
counts 512-byte blocks), once with `trap '' XFSZ` (ignored) and once with the default action.

SIGXFSZ ignored (the kernel returns `EFBIG`):
- `convert small.png --format png -o old.png -y` → **rc=0**, empty stderr, `old.png` → 512 B
  starting `89504e470d0a1a0a`. **Silent success over a truncated PNG** (the review's case).
- `convert small.png --format png --out-dir od -y` (`Sink::Dir`) → **rc=0**, `od/small.png` → 512 B.
  Silent.
- `meta strip small.png -o old.png -y` (byte sink, `Sink::File`) → **rc=0**, `old.png` → 512 B.
  Silent.
- `convert big.png … -o old.png -y` → rc=5 `error: could not write output`, `old.png` → 512 B.
  Loud, **old file destroyed**.
- `convert big.png … --out-dir od -y` → rc=5, `od/big.png` → 512 B. Loud, destroyed.
- `meta strip big.png -o old.png -y` and `--out-dir od -y` → rc=5, 512 B left. Loud, destroyed.

SIGXFSZ at its default: all seven rows → **rc=153** (killed by SIGXFSZ, 128+25), empty stderr,
destination → 512 B. Loud (the shell sees a signal), old file destroyed. Temp-then-rename fixes the
destroyed file here too; it cannot stop the kill, and leaves a temp file behind (see Q3).

Full disk (a 2 MB HFS+ image mounted in the scratchpad, filled with `dd`):
- `convert big.png … -o mnt/old.png -y` over an 11-byte file → rc=5, `old.png` → **0 B**. Loud,
  destroyed.
- `convert small.png … -o mnt/old.png -y` → rc=0, 1,328 B, correct: the truncate freed the old
  file's block and the small output fit in it. Fine today; under temp-then-rename this case needs
  room for both files and would fail loudly instead.
- `meta strip small.png -o mnt/new2.png` (new file, 4 KiB free after one small write) → rc=5, no
  file left. Fine.
I could not make the full disk return the silent case (the `ulimit` probe covers it).

Read-only directory (`chmod 555`, the file inside is writable):
- `convert small.png … -o ro/old.png -y` → **rc=0**, written in place, correct. **Temp-then-rename
  breaks this** (no new entry can be made in the directory). Ruling needed (maintainer Q1).
- `convert small.png … -o ro/new.png` → rc=5 `could not write output`. Fine.

stdout:
- `convert big.png --format png | head -c 10` → rc=5 `error: could not write output`.
- `convert small.png --format png | true` → rc=5. Loud on both sizes. Fine.

`--save-recipe`:
- `edit small.png --invert -o e2.png --save-recipe r2.toml`, with `r2.toml` already holding
  `PRECIOUS`, no `-y` → **rc=0, `r2.toml` replaced.** No overwrite guard on this writer (finding F1,
  below).
- Same with `-o -` under `ulimit -f 0`, XFSZ ignored → rc=5 `Error: could not write output`,
  `r3.toml` → **0 B**. Loud, old recipe destroyed. (`fs::write` has no `BufWriter`, so there is no
  silent case here.)

Classification:
- **Silent success** (exit 0, short file): every sink arm on an output under 8 KiB (both lanes,
  File and Dir), when the failure comes back as an error rather than a signal (`EFBIG` with XFSZ
  ignored, `ENOSPC`, `EIO`, `EDQUOT` on a network filesystem whose write is deferred to close).
- **Loud, old file destroyed:** every sink arm on an output of 8 KiB or more; every arm under the
  default SIGXFSZ; `--save-recipe`; the full-disk `-y`.
- **Fine:** stdout (rc=5); new files in an unwritable directory (rc=5); the lockfile and the cache
  (below).

### Q1: premise
Driven above. The review's case is real, but only for outputs under 8 KiB with XFSZ ignored: web
thumbnails, icons, small `meta` rewrites. For large outputs the error already propagates, and the
bug is that the old file is gone.

### Q2: every writer
Sweep (scope: production code in `src/`, every `*.rs`; test bodies filtered out by reading each hit):
`/usr/bin/grep -rn -E 'fs::write|File::create|OpenOptions|\.persist\(|fs::rename|fs::copy|NamedTempFile|tempfile|create_new' src '--include=*.rs'`
and, for callers, `/usr/bin/grep -rn -E '\.write_bytes\(|sink\.write\(|\.write\(&|Sink::(File|Dir) *\{' src '--include=*.rs'`.
- `Sink::File` in `write` (`src/sink/mod.rs:501`): **bug**, both halves.
- `Sink::Dir` in `write` (`:535`): **bug**, both halves.
- `Sink::File` in `write_bytes` (`:645`): **bug**, both halves.
- `Sink::Dir` in `write_bytes` (`:665`): **bug**, both halves.
- Every image writer reaches one of those four: `src/cli/ops.rs` (`run_pixel_op` :55/:672, the
  `meta` lane :702/:707/:887/:937, `meta copy` in place :1085), `src/cli/optimize.rs` (:191,
  :1599, :2019), `src/cli/common.rs:159` (`build` outputs, `Sink::Dir`, `Overwrite::Allow`), and
  `responsive` through per-width `Sink::File` (DEC-044's note). One fix in the sink covers them all.
- `--save-recipe` (`src/cli/ops.rs:1761`, `fs::write`): errors propagate; **old file destroyed**; no
  overwrite guard (F1).
- Build lockfile (`src/cli/build.rs:683` `write_lock`): already writes `{path}.tmp{pid}` then
  `rename`, and removes the temp if the rename fails. Errors propagate and the old lockfile
  survives. No `sync_all`. Two gaps: the temp is opened with `fs::write`, which follows a symlink
  planted at the predictable `{path}.tmp{pid}`; and a failed `fs::write` leaves its partial temp
  behind (only the rename failure cleans up). Not the reported bug; folding it costs one call-site
  swap onto the new helper (recommended, below).
- Build cache (`src/build/cache.rs:420` `store`, `:469` `write_entry`): `File::create` plus
  `write_all` with no `BufWriter`, then `rename`, and the temp is removed on any error. Every entry
  is hash-checked on read (`parse_entry`). **No bug**, leave it.
- `lint`: there is no `--fix`; `lint` only prints commands (`src/lint/mod.rs` `fix_command`).
  Nothing to fix.
- The wasm `transform` writes no files.

### Q3: the fix's shape
Stage beside the destination, write, flush, then rename over it. Evaluated:
- **Location and name:** the temp goes in the destination's own directory (`path.parent()`, or `.`
  for a bare name), because `rename` is atomic only within one filesystem. Name it
  `.<file_name>.crustyimg-<pid>-<n>.tmp`. The leading dot keeps it out of `ls` and out of
  crustyimg's own directory and glob inputs. Check at design whether `source::resolve_directory`
  skips dotfiles, and test it if it doesn't. Open it with `OpenOptions::create_new(true)`
  (`O_CREAT|O_EXCL`), which refuses an existing path and will not follow a symlink planted at the
  temp name. Retry with the next `n` on `AlreadyExists`.
- **Permissions:** today a new file is `0666 & ~umask` (usually 0644) and `-y` keeps the old file's
  inode, so its mode, owner, ACLs, xattrs and hard links all survive. After the change:
  - a new file must still be `0666 & ~umask`. This rules out `tempfile`'s default of `0o600`
    (`tempfile-3.27.0/src/lib.rs:407`), which would make every web output unreadable to a web
    server. `std`'s `OpenOptions` defaults to 0o666 masked by umask.
  - a replaced file: copy the old file's permission bits onto the temp before the rename
    (`fs::set_permissions` from `symlink_metadata`).
  - Owner (only root can keep it), ACLs, xattrs and hard links don't survive a rename. Accept that
    and document it: `-y` makes a new file at that name.
- **Cleanup:** one RAII guard owns the temp path and removes it on drop unless the rename
  succeeded. That covers every `?` (create, write, flush, permissions, rename). A killed process
  (SIGXFSZ, SIGKILL, power loss) leaves `.<name>.crustyimg-<pid>-<n>.tmp`; the destination is
  untouched. Removing stale temps is out of scope; the leading dot and the fixed infix make them
  easy to find.
- **Guards first:** `reject_symlink_destination` and `guard_overwrite` keep running before anything
  is created, unchanged, so DEC-035 holds and a refused write leaves no temp. `rename(2)` does not
  follow a symlink at the destination; it replaces the link itself. So a symlink planted between the
  guard and the rename gets swapped for a regular file, and the file it pointed at is never written.
  That is stronger than today's `open(truncate)`, which follows the link. A symlinked *parent*
  directory is resolved by both the old and the new code; that is SPEC-157's and DEC-061's ground.
- **flush and fsync:** drop the `BufWriter`. The bytes are already one `Vec<u8>`, so a single
  `write_all` on the `File` returns every error, and an explicit `flush()` is then a no-op kept for
  clarity. `sync_all` protects only against power loss, not against the failures this spec is
  about. Measured here (Python, 200 KB file, temp then rename, 200 files, macOS APFS): no sync
  0.19 ms per file, `fsync` 0.26 ms, `F_FULLFSYNC` 4.97 ms. Rust's `File::sync_all` on Apple
  platforms calls `F_FULLFSYNC`. That is from memory of std's source: it isn't installed here and I
  didn't read it, so design confirms it. At about 5 ms per file, a 22,000-image `build` (SPEC-158's size) pays about
  110 s of serialized sync. Directory `fsync` is needed only to make the rename itself survive power
  loss. **Recommend:** no `sync_all` and no directory `fsync`, and the DEC says the guarantee covers
  errors and killed processes, not power loss. Maintainer Q2.
- **A crate?** `tempfile` 3.27.0 is in `Cargo.lock` as a **dev-dependency**
  (`Cargo.toml:324`, native-only). It is also in the default *normal* tree transitively,
  through `viuer` (`cargo tree -e normal -i tempfile`), but not in the lean
  `--no-default-features` tree. Its licence is `MIT OR Apache-2.0`, allowed by
  `guidance/constraints.yaml:128`. Its `persist` uses `renameat`/`rename` on Unix and `MoveFileExW`
  with `MOVEFILE_REPLACE_EXISTING` on Windows. Against it: `src/sink` also compiles for wasm32, so a
  direct dependency must be target-gated; its default 0o600 mode must be overridden every time; and
  its API hides the steps a fault-injection seam needs to fail one by one. **Recommend hand-rolling
  about 40 lines on `std`** (`create_new`, `write_all`, `set_permissions`, `fs::rename`, an RAII
  remover), with no new dependency. Design rules on it.

### Q4: Windows
`std::fs::rename` on Windows replaces an existing destination; std's docs name `MoveFileExW` with
`MOVEFILE_REPLACE_EXISTING`. Newer std first tries a POSIX-semantics `SetFileInformationByHandle`
rename; design confirms this on the CI toolchain. A rename fails with `ERROR_ACCESS_DENIED` or a
sharing violation when:
- the destination is open in another process without `FILE_SHARE_DELETE` (an image viewer, an
  antivirus scan);
- the destination has the read-only attribute;
- the temp itself is still open.
The cache already closes its file before the rename for that last reason
(`src/build/cache.rs:466-467`). Today's `open(truncate)` fails on the first two as well, so there
is no regression, but now the old file must survive. The Windows leg must cover:
- (a) `-y` over an existing file: replaced, no temp left.
- (b) the destination held open by a handle with `share_mode` that lacks `FILE_SHARE_DELETE`
  (`std::os::windows::fs::OpenOptionsExt`): error, exit 5, old bytes intact, no temp. Rust's own
  `File::open` shares delete, so the test must set `share_mode`.
- (c) a read-only destination: refused, old bytes intact, no temp.
`just check` doesn't run Windows. CI's `windows-latest` leg does, and that leg is the evidence for
(a) to (c).

### Q5: stdout and the display sink
Out of scope. Driven: a closed pipe exits 5 with `could not write output` on both sizes, because
`out.write_all` propagates `EPIPE`; Rust ignores SIGPIPE. A stream has no old file to protect and
cannot be staged. A failure that only shows at process exit, when std flushes stdout, is possible in
principle, but I could not drive one (`/dev/full` does not exist on macOS). `Sink::Display` writes
to a terminal through `viuer` and maps its error. Neither has the bug.

### Q6: how to test a failing write
The seam is a private, crate-internal function in `src/sink`, for example
`publish_atomic(path, bytes, fs: &dyn StageFs)`. `StageFs` is a small trait with `create_new`,
`write_all`, `flush`, `set_permissions` and `rename`, implemented once for the real filesystem, and
the four sink arms call it through a `RealFs` value. `Sink` stays non-generic and its public
signature is unchanged. A `#[cfg(test)]` `FailingFs` wraps `RealFs` and returns
`io::Error::other` on its Nth `write_all`, on `flush`, or on `rename`. It must also call write
chunk by chunk, so "the second write fails" is a real state with partial bytes in the temp. Unit
tests in `src/sink/mod.rs` assert, for each injection point:
- `Err` is returned;
- the destination's old bytes are unchanged (or no file exists, for a new path);
- the directory holds no `.crustyimg-` temp.
End to end on CI, `tests/` (Unix, the Linux and macOS legs):
- a `sh -c 'trap "" XFSZ; ulimit -f 1; exec crustyimg convert small.png … -o old.png -y'` row
  asserting exit 5, old bytes intact, no temp;
- the same with XFSZ at its default (killed, old bytes intact, one stale temp is allowed);
- a read-only directory row (skip when `geteuid()==0`, since root ignores modes);
- a `--save-recipe` row.
Windows: (a) to (c) from Q4.

### Q7: mutation rows
The red set is read off one full `cargo test --no-fail-fast` of every target (AGENTS §15). A mutant
that doesn't compile stops `cargo test` before any integration target runs, so that row runs the
targets separately.
- M1, the error propagates: in `publish_atomic`, swallow the write or flush error
  (`let _ = f.write_all(bytes);`). Expected red: the injected-write and injected-flush unit tests,
  and the `ulimit` E2E row (exit 0).
- M2, the old file survives: write straight to `path` with `truncate(true)` instead of the temp,
  keeping the `?`. Expected red: every "old bytes unchanged" assertion, while the error assertions
  stay green. That split is the point of this row.
- M3, no temp left behind: delete the RAII remover's `Drop` body (or `mem::forget` the guard).
  Expected red: the "no `.crustyimg-` temp" assertions on the injected-write, injected-rename and
  `ulimit` rows.
- M4, the mode is preserved: skip the `set_permissions` copy. Expected red: a Unix row that chmods
  the old file to 0600 (or 0640) and asserts the mode after `-y`. A row needs a non-default mode,
  or the mutant stays green.
- M5, guards first: move `guard_overwrite` after the temp is created. Expected red: a row asserting
  that a refused write (no `-y`) leaves no temp.
- M6, `--save-recipe`'s guard (if F1 is folded): drop its `guard_overwrite`. Expected red: the
  "existing recipe without `-y` → exit 5, old bytes intact" row.

### Q8: the DECs
DEC-035 (symlinked destination refused before opening) still holds: its check runs first, and the
rename hardens it (Q3). It needs no change. Design may add a "see also" line. DEC-044 (out-dir
auto-create) is unaffected; the temp goes inside the directory it created. DEC-059 already records
that the lockfile is written atomically (`decisions/DEC-059-*.md:132`); the new DEC generalises
that.

**Recommend a new DEC: "A file write is all or nothing."** Content:
- Every file crustyimg writes for the user is staged in a new file in the destination's directory,
  created exclusively, and renamed over the destination only after every byte is written. Today
  that covers image outputs on every verb, `build` outputs, `meta` in place, `--save-recipe` and
  the lockfile.
- A failed write exits non-zero and leaves the destination's old bytes untouched, or leaves no file
  if there was none. No temp survives an error that the process returns; one a killed process
  leaves is a dot-named `.crustyimg-` file.
- The symlink and overwrite guards (DEC-035) run before staging. `-y` replaces the file at that
  name: the permission bits carry over, while owner, ACLs, xattrs and hard links do not.
- The guarantee covers errors and killed processes. It does not cover power loss (no `fsync`), if
  the maintainer rules that way (Q2).
- A destination whose directory can't take a new entry fails with exit 5 and says why (Q1 ruling).
- stdout is a stream: its errors exit non-zero, and there is nothing to stage.
- `affected_scope` (block-list form, HANDOFF trap 7): `src/sink/mod.rs`, `src/cli/ops.rs`,
  `src/cli/build.rs`, `SECURITY.md`, `docs/cli-reference.md`.

### Q9: fold-ins
I read every open STAGE-063 entry: SPEC-154, 156, 157 and 158; the `build_watch` diagnosis; the
watermark strict mode; SPEC-148's optional findings; `optimize` to stdout; finding A; SPEC-150's
leftovers; `docs/recipes.md:148`; SPEC-150's optional findings. **None fold.** SPEC-157 and SPEC-158
touch `src/cli/build.rs` and `src/cli/common.rs`. SPEC-157 decides the destination's identity, and
this spec decides how bytes reach it. Expect a rebase between them, not a merge of scope.

Found by this frame, and **recommended in scope** (both use the same helper and the size stays M):
- **F1: `--save-recipe` overwrites an existing file without `-y`** (driven: rc=0, `PRECIOUS`
  replaced). It also destroys the old recipe on a failed write (rc=5, 0 B). `SECURITY.md:87`
  promises "no overwrite without `--yes`" in the row that names `--save-recipe`. Route it through
  the helper with `guard_overwrite`. That is a behaviour change, so it is maintainer Q3.
- **F2: `write_lock`'s predictable temp follows a symlink and leaks on a failed write**
  (by reading; not driven). Move it onto the helper.

### Q10: docs
- `SECURITY.md:41-44` (threat row: symlinked destination refused even with `--yes`, "does not
  overwrite an existing file without `--yes`"). After: unchanged, plus one sentence: "A write that
  fails leaves the existing file as it was; output is staged beside the destination and renamed into
  place."
- `SECURITY.md:87` (row 3: "… (image output AND `--save-recipe`); no overwrite without `--yes`").
  After: true once F1 lands. Add "writes are staged and renamed (all or nothing)".
- `docs/cli-reference.md:55` (exit 5: "Output write refused (exists without `--yes`; path
  traversal; symlinked destination)."). After: "Output write refused or failed (exists without
  `--yes`; path traversal; symlinked destination; the write itself failed, in which case the
  existing file is left as it was)." Exit 5 is already what a failed write returns (driven).
- `docs/cli-reference.md:39`, `src/cli/mod.rs:106` (`-y`: "Assume 'yes' to overwrite prompts
  (non-interactive)"). No change to the flag text. The fact that `-y` makes a new file (mode kept,
  hard links and owner not) goes in the `-y` row's long text or a note under the exit-code table.
  Design picks which.
- `README.md:292-294` (`-y` and exit 5 one-liners): no change needed.

### Q11: size
**[M], confirmed.** The code is small: one helper of about 40 lines plus four sink arms,
`--save-recipe` and `write_lock` on it. The verification is not:
- three guarantees (the error propagates, the old file survives, no temp) × two lanes × File and
  Dir;
- permissions and the umask;
- the read-only-directory behaviour;
- three OS legs, where Windows' rename semantics are tested only on CI;
- a fault-injection seam;
- six mutation rows.
That is past S. It is not L: there is one mechanism and one helper, and no format or user-facing
surface changes beyond F1.

### Scope
In:
- one staged-write helper in `src/sink` with a test-only fault seam;
- all four sink arms;
- `--save-recipe` (F1, guard plus helper);
- `write_lock` (F2);
- the docs in Q10;
- the tests in Q6, on Linux, macOS and Windows;
- the new DEC.

Out (and where each lives):
- stdout and display (Q5; no bug);
- the build cache (no bug);
- `fsync` and power-loss durability (Q2; recorded as a non-guarantee in the DEC; revisit only on
  demand);
- removing stale temps after a kill (filed nowhere; the dot name makes them findable);
- symlinked parent directories and destination identity (SPEC-157, DEC-061);
- the lockfile size contract (SPEC-158).

### Rough build file list
- `src/sink/mod.rs`: helper, seam, four arms, unit tests.
- `src/cli/ops.rs`: `--save-recipe`.
- `src/cli/build.rs`: `write_lock`.
- `tests/atomic_write.rs` (new): the Unix `ulimit`, read-only directory, mode and `--save-recipe`
  rows, plus the Windows rows.
- `SECURITY.md`, `docs/cli-reference.md`, and possibly `src/cli/mod.rs` (the `-y` doc comment).
- `CHANGELOG.md` (build cycle, `### Fixed`).
- `decisions/DEC-<next>-a-file-write-is-all-or-nothing.md` (design).

### Questions for the maintainer
1. **A read-only directory with a writable file in it, `-y`:** today it writes in place (rc 0).
   Staging can't. Recommend: fail with exit 5 and a message naming the unwritable directory, with no
   in-place fallback. A fallback would bring back the destroy-on-failure path this spec removes.
2. **`fsync`:** recommend none. The guarantee covers errors and kills, not power loss.
   `F_FULLFSYNC` costs about 5 ms per file on macOS (measured), about 110 s on a 22,000-image build.
3. **F1, `--save-recipe` gains the overwrite guard** (an existing recipe without `-y` → exit 5).
   Recommend yes: `SECURITY.md:87` already promises it.
4. **A full disk with `-y` over an existing small file** worked by reusing the freed block (driven).
   Staging needs room for both files. Recommend accepting the loud failure.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

- stdout and the display sink: a closed pipe already exits 5; a stream has no old file to keep.
- The build cache (`src/build/cache.rs`): already temp→rename with errors propagated and hash-checked reads.
- `fsync` / power-loss durability, pending maintainer Q2 (recommended: not guaranteed, said so in the DEC).
- Removing stale `.crustyimg-` temps a killed process leaves.
- Symlinked parent directories and destination identity: SPEC-157 and DEC-061.
- The lockfile size contract: SPEC-158.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
