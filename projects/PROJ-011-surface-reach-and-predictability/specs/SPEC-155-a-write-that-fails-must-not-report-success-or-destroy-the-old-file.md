---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-155
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  sessions:
    - cycle: frame
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 5158426
      tokens_breakdown:
        input: 90
        cache_creation: 121766
        cache_read: 4999745
        output: 36825
      estimated_usd: 4.18
      note: >
        Measured from the session JSONL, found by content ("SPEC-155 — FRAME prompt"), deduped by
        message.id (45 messages; max output_tokens per id), 0 subagent transcripts. Measured after the
        push, before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: design
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 23760498
      tokens_breakdown:
        input: 290
        cache_creation: 246555
        cache_read: 23402100
        output: 111553
      estimated_usd: 16.03
      note: >
        Measured from the session JSONL, found by content ("SPEC-155 — DESIGN prompt"), deduped by
        message.id (124 messages; max output_tokens per id), 0 subagent transcripts. Measured after the
        push, before this cost commit's own turns. Most of it is cache_read across ~15 full-suite
        mutation runs on a prototype.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: build
      date: 2026-11-22
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 18797882
      tokens_breakdown:
        input: 184
        cache_creation: 238403
        cache_read: 18481198
        output: 78097
      estimated_usd: 7.61
      note: >
        Measured from the session JSONL, found by content ("SPEC-155 — BUILD prompt"), deduped by
        message.id (92 messages; max output_tokens per id), 0 subagent transcripts. Measured after the
        last code push, before this cost commit's own turns. Most of it is cache_read across the
        15-row mutation run, the three-leg matrix and the CI waits.
        Rates (Sonnet): $3/MTok input, $3.75 cache_creation (x1.25), $0.30 cache_read
        (x0.10), $15 output.
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

### Maintainer rulings (2026-11-22)

All four recommendations accepted:
1. **Read-only directory, writable file, `-y`: refused**, exit 5, with a message naming the
   directory. No in-place fallback. A behaviour change, so it gets a CHANGELOG line.
2. **No `fsync`, no directory `fsync`.** The guarantee covers errors and killed processes, not
   power loss, and the new DEC says so.
3. **F1 is in scope: `--save-recipe` gains the overwrite guard.** An existing recipe without `-y`
   exits 5 and keeps its bytes. A behaviour change, so it gets a CHANGELOG "Changed" line.
4. **Full disk: the loud failure is accepted.** Staging needs room for the old and new file at once.

Also accepted as the frame recorded it: F2 (`write_lock` moves onto the helper) is in scope, and a
replaced file keeps its permission bits but not its owner, ACLs, xattrs (on macOS, Finder tags) or
hard links. That is documented under `-y`.

## Design

Design session 2026-11-22, Opus, on `main` at `9560c2c`. `main` and an unpushed prototype of this
design were each built `--release` in their own fresh `CARGO_TARGET_DIR` under the session
scratchpad (deleted at the end). Every number below was driven on one of those two binaries unless
it says *read* or *reasoned*.

### Design corrections to the frame

1. **The leading dot does not keep a staged file out of crustyimg's inputs; the `.tmp` suffix
   does.** `resolve_directory` (`src/source/mod.rs:345-391`) and `resolve_glob` (`:282-334`) do not
   skip dotfiles. Both drop any entry without an image extension (`has_image_extension`, `:135-144`;
   `IMAGE_EXTENSIONS`, `:125-133`), and `tmp` is not one. Driven on the prototype: a folder holding
   `a.png` and `.a.png.crustyimg-1-0.tmp` gives one output for `src` and for `src/*`. The suffix is
   load-bearing; FT-E20 pins it.
2. **`--watch` cannot loop on a staged file.** A staged file is made in its destination's own
   directory, and every target's `out` is in the watch set's `excluded` prefixes
   (`src/build/watch.rs:119-120`, checked at `src/cli/build.rs:1132`). Under `--watch` no lockfile
   is written (`src/cli/build.rs:933`). Driven on the prototype with `out = "src/out"` inside the
   watched `src/*.png` root, `--no-cache`: three builds for the initial run plus two `touch`es, no
   rebuild loop, no staged file left. `main` gives the same count.
3. **Windows' `rename` tries `MoveFileExW` first, not the POSIX rename.** Read in std's
   `library/std/src/sys/fs/windows.rs` at tags `1.90.0` (the MSRV leg) and `1.99.1` (current
   stable; CI uses `dtolnay/rust-toolchain@stable`, `.github/workflows/ci.yml:185`). The `rename`
   bodies are identical apart from one moved comment. It calls
   `MoveFileExW(old, new, MOVEFILE_REPLACE_EXISTING)`. Only when that fails with
   `ERROR_ACCESS_DENIED` does it open the *source* with `DELETE` access and retry with
   `SetFileInformationByHandle(FileRenameInfoEx)`, flags
   `FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS`. It does **not** set
   `FILE_RENAME_FLAG_IGNORE_READONLY_ATTRIBUTE`, so a read-only destination still fails; any other
   error returns `MoveFileExW`'s error.
4. **A rename replaces a read-only file on Unix; today's write refuses it.** The frame missed this.
   `-y` over a `0444` file in a writable directory: `main` exits 5 (`open(truncate)` gets
   `EACCES`); a bare temp-then-rename replaces the file and copies `0444` onto it. Design call: keep
   today's refusal. Before staging, the helper opens an existing destination for writing without
   truncating it (`OpenOptions::new().write(true).open(path)`), which fails exactly where today's
   open failed and changes nothing. Driven: the prototype exits 5, keeps the file, leaves no staged
   file. Windows' read-only attribute fails the same open (`ERROR_ACCESS_DENIED`), so Windows row (c)
   is refused before staging, not by the rename.
5. **The frame's M5 ("a refused write leaves no temp") cannot go red.** The RAII remover drops a
   staged file on every early return, so a guard that ran after staging would still leave nothing.
   Guard order is visible only where staging itself fails: a read-only directory holding the file,
   without `-y`, must say `already exists`, not the directory message. That is FT-E12. The mutation
   row for guards (M5 below) drops the guard instead.
6. **`--save-recipe`'s guard runs before the edit, not at the recipe write.** Otherwise a refused
   recipe would still write the image output and then exit 5. Driven on the prototype: an existing
   `r.toml`, no `-y`, `-o e.png` exits 5 and `e.png` is not created.
7. **The write-failed message changes.** Today a sink write failure prints `could not write output`
   with no path and no cause (`src/sink/mod.rs:212`). A failed file write now names both (Messages,
   below). `Sink::Stdout` keeps today's text.
8. **The stage backlog's "flush and sync" is superseded** by ruling 2 (no `fsync`).
9. **A batch where every write fails exits 6, not 0.** Driven: `convert in --out-dir od -y` over two
   small inputs under `ulimit -f 1` (XFSZ ignored) exits **0** on `main` with both outputs cut to
   512 B. The prototype exits 6, prints one `error: <input>: could not write output …` line per input
   and `error: 2 of 2 inputs failed`, and keeps both old files.

### Things the frame asked design to check (driven)

- **The 8 KiB line.** `BufWriter::new` takes `DEFAULT_BUF_SIZE`, which is
  `if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 }` (`library/alloc/src/io/mod.rs:44`,
  `1.99.1`). Driven on `main` with `meta strip <png> -o old.png -y` under `ulimit -f 1`, XFSZ
  ignored, over inputs made by `resize --max N`: 7,759 B and 7,909 B exit **0** with a 512 B file;
  8,229 B, 8,484 B and 8,723 B exit 5 with a 512 B file. The tests use 1,328 B (a 40×40 PNG) and
  83,916 B (the 512×512 fixture), far from the line on both sides, because an encoder update could
  move a near-line size across it.
- **Parallel writes.** `build` fans each target's inputs over rayon (`src/cli/build.rs:804`, pool at
  `:825`), and `optimize` does too (`src/cli/optimize.rs:256`, `:281`). The staged name's `<n>` comes
  from one process-wide `AtomicU64`, so two threads never pick the same name. Driven on the
  prototype: four targets over the same 120 inputs into one `dist`, `-j 8 --no-cache`, three runs:
  480 outputs each, exit 0, no staged file, no `AlreadyExists`.
- **Writing over the input.** Every input is read whole with `std::fs::read`, which closes its handle
  before returning: `Image::load` (`src/image/mod.rs:274`), the metadata lane (`src/cli/ops.rs:739`),
  `meta copy` (`:1031-1032`), the build cache key (`src/cli/build.rs:436`). The one `File::open` on an
  input, the 64-byte format sniff (`src/cli/ops.rs:799-801`), drops the handle at the end of that
  statement. So no handle crustyimg holds is open at the rename. Driven: `convert self.png -o
  self.png -y` and `meta copy --from a.jpg --to b.jpg -y` (in place) exit 0 on the prototype. Windows
  row (d) covers the input-is-the-output case on CI.
- **The read-only-directory refusal.** The call that fails is `create_new` on the staged path
  (`PermissionDenied`, `EACCES`). The helper returns it as `PublishError::CreateTemp`, the sink maps it
  to `SinkError::DirNotWritable`, and `CliError::Sink(_) => 5` (`src/cli/mod.rs:867`) gives exit 5.
  Driven: `main` exits **0** and rewrites `ro/old.png` in place; the prototype exits 5 with the
  message below and keeps the file. As root the directory is writable, so the row asserts its
  precondition and fails if it cannot make a read-only directory (the
  `tests/messages_tell_the_truth.rs:362` pattern). GitHub's hosted Ubuntu and macOS runners run as
  the non-root `runner` user, so the row runs there.
- **`--save-recipe`'s surfaces.** `/usr/bin/grep -rn "save_recipe" src/cli/` → one flag, on `edit`
  only (`src/cli/mod.rs:511-513`, dispatched at `:1210`, `:1229`). `-y` is global
  (`src/cli/mod.rs:106-108`, `global = true`), so `edit` takes it. No design question.
- **The mode rules.** std's `OpenOptions::new()` starts at `mode: 0o666`
  (`library/std/src/sys/fs/unix.rs:1123`, `1.99.1`), masked by the umask at `open(2)`. Driven on both
  binaries: a new file is `0644` under umask 022 and `0600` under umask 077; `-y` over `0600` and
  `0640` keeps the mode. On Windows `fs::Permissions` is the file's attribute word and
  `set_permissions` is `SetFileAttributesW` with it, so the old file's attributes (say, hidden) carry
  over (*read*). The read-only attribute never does, because a read-only destination is refused
  first (correction 4). No `cfg` is needed: `metadata().permissions()` and `fs::set_permissions` are
  portable.
- **`sync_all` on Apple is `F_FULLFSYNC`.** Read: `fsync` calls `os_fsync`, which is
  `libc::fcntl(fd, libc::F_FULLFSYNC)` under `cfg(target_vendor = "apple")`
  (`library/std/src/sys/fs/unix.rs:1264-1271`, `1.99.1`). Moot under ruling 2.
- **wasm32.** `src/sink` is in the wasm build. The helper uses only `std::fs`, which compiles there
  and returns `Unsupported` at run time. `std::process::id()` **panics** on `wasm32-unknown-unknown`,
  so `process_id()` returns 0 under `cfg(target_arch = "wasm32")`. Nothing on wasm writes a file
  (`src/wasm.rs` returns bytes), so the helper is unreachable there. See the wasm line under
  Verification for the command.
- **The full disk** (ruling 4) is accepted as a loud failure; the frame's driven rows stand.


## Inputs

- **Files to read:** `src/sink/mod.rs` (`:480-690`, `:868-888`), `src/cli/ops.rs` (`run_edit`
  `:1681-1764`), `src/cli/build.rs` (`write_lock` `:673-689`), `src/source/mod.rs` (`:125-144`,
  `:282-391`), `src/build/watch.rs` (`:104-147`), `tests/sink.rs`, `tests/messages_tell_the_truth.rs:350-370`
  (the precondition pattern).
- **External APIs:** none.
- **Related code paths:** `src/sink/`, `src/cli/`.

## Outputs

- **Files created:** `src/sink/publish.rs` (the helper and its seam), `tests/atomic_write.rs`,
  `decisions/DEC-<next>-a-file-write-is-all-or-nothing.md`.
- **Files modified:** `src/sink/mod.rs`, `src/cli/ops.rs`, `src/cli/build.rs`, `src/cli/mod.rs`,
  `SECURITY.md`, `docs/cli-reference.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`,
  `CHANGELOG.md`.
- **New exports:** none public; `pub(crate)` `publish_file`, `publish_file_io`, `guard_overwrite`.
- **Database changes:** none.

## Acceptance Criteria

Each AC names its tests (FT ids below). "Old bytes kept" means the destination reads back exactly as
before; "no staged file" means the directory holds no name containing `.crustyimg-`.

- [ ] **AC-1 A failed write under 8 KiB no longer exits 0.** A 1,328 B PNG output whose write fails
  with `EFBIG` (`ulimit -f 1`, SIGXFSZ ignored) exits 5 on each of the four sink arms: pixel lane
  (`convert`) and byte lane (`meta strip`), each to `-o` (`Sink::File`) and `--out-dir`
  (`Sink::Dir`). — FT-E1, FT-E2, FT-E3, FT-E4.
- [ ] **AC-2 A failed write keeps the old file, at any size.** On each row of AC-1, and the same four
  rows with the 83,916 B output, the old destination's bytes are kept and no staged file is left. —
  FT-E1…FT-E8.
- [ ] **AC-3 Every step of the helper keeps the old file and cleans up.** A failure injected at
  create, the first write, the second write (1,024 B already staged), flush, set-permissions and
  rename each returns `Err`, keeps the old bytes (or leaves no file, for a new path; set-permissions
  runs only for an existing file) and leaves no staged file. — FT-U1…FT-U7.
- [ ] **AC-4 A killed process keeps the old file.** Under `ulimit -f 1` with SIGXFSZ at its default,
  the process dies by signal 25, the old bytes are kept, and at most one staged file is left. — FT-E9.
- [ ] **AC-5 The message names the file and the cause.** A failed file write prints
  `error: could not write output <path>: <io error>` and exits 5. — FT-E1…FT-E8 (assert
  `could not write output` and `File too large`), FT-E15, FT-E22.
- [ ] **AC-6 A directory that can't take a new file is refused by name.** `-y` over a writable file
  in a `0555` directory exits 5 with
  `error: could not write output ro/old.png: cannot create a new file in ro: Permission denied (os error 13)`,
  keeps the old bytes, leaves no staged file. — FT-E11.
- [ ] **AC-7 The guards run before staging.** The same directory without `-y` prints
  `error: output file already exists (use --yes to overwrite): ro/old.png` (exit 5), not the directory
  message. — FT-E12.
- [ ] **AC-8 `-y` keeps the permission bits.** `-y` over a `0600` and a `0640` file leaves the new
  content at the old mode (two modes, each non-default, so a skipped copy shows). — FT-U11 (helper),
  FT-E13 (CLI).
- [ ] **AC-9 A new file follows the umask.** A new output is `0644` under umask 022 and `0600` under
  umask 077. — FT-E10.
- [ ] **AC-10 A destination this user can't write is refused, as today.** `-y` over a `0444` file
  exits 5 with `error: could not write output old.png: Permission denied (os error 13)`, keeps the
  bytes, leaves no staged file. On Windows, the read-only attribute: exit 5, bytes kept. — FT-U12,
  FT-E15, FT-W3.
- [ ] **AC-11 `--save-recipe` refuses an existing recipe without `-y`.** Exit 5,
  `error: output file already exists (use --yes to overwrite): r.toml`, the recipe's bytes kept, and
  the image output (`-o e.png`) not created. — FT-E16.
- [ ] **AC-12 `--save-recipe -y` replaces it.** Exit 0, the file now starts `version = "1"`, no staged
  file. — FT-E17.
- [ ] **AC-13 A failed `--save-recipe` write keeps the old recipe.** Under `ulimit -f 0`, SIGXFSZ
  ignored, image to stdout: exit 5, `could not write output r.toml: File too large`, `PRECIOUS` kept,
  no staged file. — FT-E22.
- [ ] **AC-14 A failed lockfile write keeps the old lockfile.** `build --no-cache` in a `0555` project
  directory (outputs into a writable `dist/`) exits 5 with
  `error: could not write build lockfile crustyimg.build.lock: Permission denied (os error 13)`, the
  old lockfile's bytes kept, no staged file. — FT-E21.
- [ ] **AC-15 Parallel writes into one directory don't collide.** Four targets over the same 60
  inputs into one `dist`, `-j 8 --no-cache`: exit 0, 240 outputs, no staged file in `dist` or the
  project. Staged names taken from eight threads are all different. — FT-E14, FT-U10.
- [ ] **AC-16 A batch reports each failed write and keeps each old file.** One `0444` destination
  among two `--out-dir` outputs: exit 6, `od/a.png: Permission denied` on stderr, `a.png` kept,
  `b.png` written. Every write in a two-input batch failing (`ulimit -f 1`): exit 6, both old files
  kept. — FT-E19, FT-E23.
- [ ] **AC-17 A staged file is never read as an input.** A folder with `a.png` and
  `.a.png.crustyimg-1-0.tmp` gives one output for the folder and for `folder/*`. — FT-E20.
- [ ] **AC-18 A long name still writes.** A 244-byte file name (240 `a`s + `.png`) writes. — FT-U9.
- [ ] **AC-19 Output bytes don't change.** The file a sink writes equals the bytes the same command
  sends to stdout, for `convert --format png` and `meta strip`. A successful helper write leaves
  exactly the payload. — FT-E18, FT-U8.
- [ ] **AC-20 Windows (CI only).** (a) `-y` over an existing file replaces it, no staged file;
  (b) a destination held open with `share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)` (no
  `FILE_SHARE_DELETE`) exits 5, keeps the bytes, leaves no staged file; (c) the read-only attribute:
  AC-10; (d) `-y` with the input as the output exits 0 and leaves no staged file. — FT-W1…FT-W4.
- [ ] **AC-21 The docs, CHANGELOG and DEC say it, verbatim.** Each sentence in Implementation Context
  → Docs is in place and driven on its listed paths (Build Completion quotes each), the two CHANGELOG
  sections hold the three lines, and `decisions/DEC-<next>-a-file-write-is-all-or-nothing.md` holds
  the DEC text with its block-list `affected_scope`. No user-facing text names a spec, stage or DEC
  id. — read by verify; FT-E11, FT-E13, FT-E16 drive the sentences.
- [ ] **AC-22 wasm32 still builds.** `just wasm-check` and `just wasm-test` pass. — the wasm CI leg.

## Failing Tests

Written during **design**, before build. Every test below except the four Windows rows ran on the
prototype (`atomic_write`: 23 passed; the 12 unit tests passed in the 64-target baseline). Red on
`main` means the old code restored (Trap 14 row in the mutation table).

- **`src/sink/publish.rs`, `#[cfg(test)] mod tests`** (all OS unless marked). Each injection test runs
  twice where the frame says so: over an existing `OLD` file, and onto a new path.
  - FT-U1 `a_failed_create_keeps_the_old_file` — `FailAt::Create`: `Err`, old bytes kept; new path
    absent; no staged file.
  - FT-U2 `a_failed_first_write_keeps_the_old_file` — `FailAt::Write(1)`: as U1.
  - FT-U3 `a_failed_second_write_keeps_the_old_file` — `FailAt::Write(2)`: as U1.
  - FT-U4 `a_failed_flush_keeps_the_old_file` — `FailAt::Flush`: as U1.
  - FT-U5 `a_failed_set_permissions_keeps_the_old_file` — `FailAt::SetPermissions`, existing file
    only: `Err`, old bytes kept, no staged file.
  - FT-U6 `a_failed_rename_keeps_the_old_file` — `FailAt::Rename`: as U1.
  - FT-U7 `a_second_write_failure_leaves_partial_bytes_before_cleanup` — drives `FailingFs` directly:
    the first `write` returns 1,024, the second errs, and the staged file holds 1,024 B. Proves U3's
    state is real.
  - FT-U8 `a_successful_write_replaces_the_file_and_leaves_no_staged_file` — `FailingFs` that never
    fails (chunked writes): the file equals the 5,000-byte payload, no staged file.
  - FT-U9 `a_long_file_name_still_writes` — 244-byte name, `RealFs`: written.
  - FT-U10 `staged_names_differ_across_threads` — eight `std::thread::scope` threads each call
    `next_staged_name("a.png")`; all eight differ.
  - FT-U11 `a_replaced_file_keeps_its_mode` (unix) — `0600` and `0640`, `publish` over each, mode
    kept.
  - FT-U12 `a_read_only_destination_is_refused_and_kept` (unix) — `0444`: asserts the precondition
    (opening it for write fails, else "running as root?"), then `Err`, old bytes, no staged file.
- **`tests/atomic_write.rs`** (new). Helpers: `small_png` (`resize <fixture> --max 40`, 1,328 B),
  `big_png` (copy of `tests/fixtures/classify/photo_entropy_floor.png`, 83,916 B), `staged_left(dir)`,
  `limited(dir, ignore_xfsz, args)` = `sh -c "[trap '' XFSZ; ]ulimit -f 1; exec <BIN> <args>"`.
  `ulimit -f` counts 512-byte blocks under `sh` (dash on Ubuntu, bash-as-sh on macOS); both test
  outputs exceed 1,024 B, so either unit fails them.
  - FT-E1…FT-E8 (unix) `efbig_{small,big}_{pixel,bytes}_{file,dir}` — `convert <in> --format png`
    (pixel) or `meta strip <in>` (bytes), to `-o old.png` or `--out-dir od` (where `od/<name>` holds
    `OLD`), `-y`, XFSZ ignored. **Assert in this order** (trap 13): exit 5; old bytes kept; no staged
    file in the project or `od`; stderr contains `could not write output` and `File too large`. (The
    prototype asserted the message first, and on `main` FT-E5…E8 went red on the message, not on the
    destroyed file.) Red on `main`: E1-E4 by exit code
    (0), E5-E8 by old bytes (512 B left).
  - FT-E9 (unix) `sigxfsz_kill_keeps_the_old_file` — big, SIGXFSZ default: `status.signal() ==
    Some(25)`, old bytes kept, at most one staged file. Red on `main` by old bytes.
  - FT-E10 (unix) `a_new_file_follows_the_umask` — `sh -c "umask 022; exec …"` → `0644`; `077` →
    `0600`. Green on `main` (a pin: a helper that chmods new files, or uses a `0600` temp, reds it).
  - FT-E11 (unix) `a_read_only_directory_is_refused_by_name` — precondition asserted
    (`File::create(ro/probe)` fails); exit 5; stderr contains
    `error: could not write output ro/old.png: cannot create a new file in ro: Permission denied`;
    old bytes; no staged file. Red on `main` (exit 0, rewritten in place).
  - FT-E12 (unix) `a_read_only_directory_without_yes_says_already_exists` — no `-y`: exit 5,
    `output file already exists (use --yes to overwrite): ro/old.png`. Green on `main` (a pin).
  - FT-E13 (unix) `yes_keeps_a_non_default_mode` — `0600`, `0640`: exit 0, content changed, mode kept.
    Green on `main` (same inode).
  - FT-E14 `a_parallel_build_into_one_directory_leaves_no_staged_file` (pins completeness and
    cleanup, not name uniqueness; see M7b) — manifest `version = 1`, four
    `[[target]]`s, `source = "src/*.png"` (60 copies of the small PNG), one recipe
    (`version = "1"`, `[[step]] op = "auto-orient"`), `out = "dist"`, `name = "{stem}_<t>.{ext}"`;
    `build -j 8 --no-cache`: exit 0, 240 entries in `dist`, no staged file in `dist` or the project.
  - FT-E15 (unix) `yes_over_a_read_only_file_is_refused` — `0444`, precondition asserted; exit 5,
    `error: could not write output old.png: Permission denied`, old bytes, no staged file. Message red
    on `main` (`could not write output` alone).
  - FT-E16 `save_recipe_without_yes_refuses_an_existing_recipe` — `edit small.png --invert -o e.png
    --save-recipe r.toml` with `r.toml` = `PRECIOUS`: exit 5,
    `error: output file already exists (use --yes to overwrite): r.toml`, `PRECIOUS` kept, `e.png`
    absent. Red on `main` (exit 0, replaced).
  - FT-E17 `save_recipe_with_yes_replaces_an_existing_recipe` — same with `-y`: exit 0, starts
    `version = "1"`, no staged file.
  - FT-E18 `a_file_output_holds_the_same_bytes_as_stdout` — for `convert <small> --format png` and
    `meta strip <fixture>`: `-o f.png` bytes equal `-o -` stdout bytes (and stdout > 1,000 B).
  - FT-E19 (unix) `a_batch_with_one_refused_destination_keeps_it_and_writes_the_rest` — `in/a.png`,
    `in/b.png`, `od/a.png` = `OLD` at `0444`, `convert in --format png --out-dir od -y`: exit 6, stderr
    contains `od/a.png: Permission denied`, `a.png` kept, `b.png` written, no staged file.
  - FT-E20 `a_staged_file_in_an_input_folder_is_not_an_input` — `src/a.png` and
    `src/.a.png.crustyimg-1-0.tmp` (a PNG): `convert src` and `convert 'src/*'` each write one output.
  - FT-E21 (unix) `a_failed_lockfile_write_keeps_the_old_lockfile` — project dir with `in/`, writable
    `dist/`, `recipes/`, manifest, `crustyimg.build.lock` = `OLD LOCK`; project dir `0555`;
    `build --no-cache`: exit 5,
    `error: could not write build lockfile crustyimg.build.lock: Permission denied`, `OLD LOCK` kept,
    no staged file. Restore `0755` before asserting.
  - FT-E22 (unix) `save_recipe_failed_write_keeps_the_old_recipe` — `sh -c "trap '' XFSZ; ulimit -f 0;
    exec <BIN> edit small.png --invert -o - -y --save-recipe r.toml >/dev/null"`, stderr **captured
    by the test through a pipe** (a stderr file would be capped by the same limit and `eprintln!`
    would panic, exit 101): exit 5, then `PRECIOUS` kept, then no staged file, then
    `could not write output r.toml: File too large` (that order, trap 13). Red on `main` (0 B left).
  - FT-E23 (unix) `a_batch_where_every_write_fails_exits_6_and_keeps_every_file` — `in/a.png`,
    `in/b.png` (small), `od/a.png`, `od/b.png` = `OLD`; `limited(…, true, "convert in --format png
    --out-dir od -y")`: exit 6, both `OLD` kept, no staged file. Red on `main` (exit 0, 512 B each).
  - FT-W1 (windows) `windows_yes_replaces_an_existing_file` — exit 0, content changed, no staged file.
  - FT-W2 (windows) `windows_a_destination_held_open_without_share_delete_is_kept` — hold
    `OpenOptions::new().read(true).share_mode(0x1 | 0x2)` (`std::os::windows::fs::OpenOptionsExt`;
    Rust's own `File::open` shares delete, so the test must set it) on `old.png`, then `convert … -o
    old.png -y`: exit 5, stderr contains `could not write output old.png:` (don't assert the OS text),
    `OLD` kept (read through the held handle or after dropping it), no staged file.
  - FT-W3 (windows) `windows_a_read_only_destination_is_refused_and_kept` — `set_readonly(true)`:
    exit 5, `OLD` kept, no staged file; clear the attribute before the tempdir drops.
  - FT-W4 (windows) `windows_yes_over_the_input_itself_replaces_it` — `convert a.png --format png -o
    a.png -y`: exit 0, the file decodes, no staged file.

### Baseline the build records before changing code (trap 13)

Run `cargo test --test atomic_write --no-fail-fast` with the new test file on unchanged `main` code
(publish.rs absent, so only the E rows compile; the U rows arrive with the helper). Quote each red
row's failure message. Expected red on macOS (driven, see the Trap 14 row): FT-E1…FT-E9, FT-E11, FT-E15, FT-E16, FT-E19, FT-E22, FT-E23 (15 rows; FT-E1…E4 and FT-E23 by exit code 0, FT-E5…E9 and FT-E22 by old bytes once the assertions are ordered as below, FT-E11 by exit code 0, FT-E15 and FT-E19 by the message without a cause, FT-E16 by exit code 0). A row red
for another reason (a precondition panic, a fixture missing) is not a valid baseline: fix it first.

### Mutation table (trap 12): apply each alone, record every test red

All rows **driven** on the prototype (macOS, 2026-11-22), each alone on a full
`cargo test --no-fail-fast` of every target (64 targets; baseline 0 red, 0 warnings), red set read off
the run, not reasoned. FT ids map to the test names in Failing Tests. "Green" means every other test
in the 64 targets stayed green. Windows rows can't be driven here; the build reads the Windows leg's
red set on its PR.

- **M1, a write error is swallowed.** `src/sink/publish.rs`, `publish`'s write loop:
  `Err(e) => return Err(PublishError::Write(e)),` → `Err(_) => break,`. Red: FT-U2, FT-U3,
  FT-E1…FT-E8, FT-E22. Green: FT-U4 (flush), FT-E18 (no failure in that row). *Driven.*
- **M1b, a flush error is swallowed.** `fs.flush(&mut file).map_err(PublishError::Write)?;` →
  `let _ = fs.flush(&mut file);`. Red: FT-U4 only (a real `File::flush` cannot fail, so no CLI row can
  see it). *Driven.*
- **M2, write in place.** `src/sink/mod.rs`, `publish_file`'s body → `OpenOptions::new().write(true)
  .create(true).truncate(true).open(path)` then `write_all`, both errors mapped to `WriteFailed`
  (keeping the `?`). Red: FT-E1…FT-E9, FT-E11, FT-E22. Green: every FT-U (they call `publish`
  directly), FT-E21 (the lockfile uses `publish_file_io`), and the error/message halves of every red
  row. That split is the point. *Driven.*
- **M3, the remover does nothing.** `StagedFile::drop`: delete `let _ = fs::remove_file(&self.path);`.
  Red: FT-U2…FT-U6, FT-E1…FT-E8, FT-E22. Green: FT-U1 and FT-E21 (staging fails before a file
  exists). *Driven.*
- **M4, the mode isn't copied.** `if let Some(perm) = old_perm {` →
  `if let Some(perm) = old_perm.filter(|_| false) {`. Red: FT-U5 (no set-permissions call, so no
  `Err`), FT-U11, FT-E13. *Driven.*
- **M5, the pixel-lane `Sink::File` overwrite guard is dropped.** `src/sink/mod.rs` `write`,
  `Sink::File`: replace `guard_overwrite(path, overwrite)?;` (and its comment) with
  `let _ = overwrite;`. Red: `tests/sink.rs::overwrite_guard_forbids_then_allows` (existing), FT-E12.
  The frame's M5 (guard after staging → a temp is left) cannot go red under the RAII remover (design
  correction 5); FT-E12 pins the order. *Driven.*
- **M6, `--save-recipe`'s guard is dropped.** `src/cli/ops.rs` `run_edit` pre-check:
  `crate::sink::guard_overwrite(path, overwrite)?;` → `let _ = overwrite;`. Red: FT-E16 only.
  *Driven.*
- **M7, the staged name's `<n>` is fixed.** `next_staged_name`:
  `staged_name(file_name, NEXT.fetch_add(1, Ordering::Relaxed))` → `staged_name(file_name, 0)`. Red:
  FT-U10 only. **FT-E14 stays green**: the `AlreadyExists` retry absorbs the collision. *Driven.*
- **M7b, a fixed `<n>` and no retry.** M7 plus `&& tries < 16` → `&& tries < 0`. Red: FT-U10 only.
  FT-E14 stays green even here (240 outputs, 8 threads, three runs on the CLI probe plus this one): a
  staged file lives for microseconds, so a collision is too rare to drive. **FT-U10 is the uniqueness
  pin; FT-E14 pins "every output written, nothing left behind", not uniqueness.** *Driven.*
- **M8, the remover is disarmed before the rename.** Move `staged.armed = false;` above
  `fs.rename(…)?`. Red: FT-U6 only. (Windows FT-W2 is the CLI row that would see it; *reasoned*.)
  *Driven.*
- **M9, a read-only directory falls back to writing in place.** In the create loop, before the
  catch-all: `Err(source) if source.kind() == io::ErrorKind::PermissionDenied => { return
  fs::write(path, bytes).map_err(PublishError::Write); }`. Red: FT-E11, FT-E21. *Driven.*
- **M10, the write-probe on an existing destination is dropped.** Delete the
  `OpenOptions::new().write(true).open(path).map_err(PublishError::Write)?;` statement. Red: FT-U12,
  FT-E15, FT-E19. *Driven.*
- **M11, a short write reported as full** (the bytes pin, trap 13). `RealFs::write`: `file.write(buf)`
  → `file.write(&buf[..buf.len().min(4096)]).map(|_| buf.len())`. Red: 46 tests across the suite,
  including **FT-E18 by content** (`assertion left == right failed: ["meta", "strip", …]`: the
  83,916 B case; the 1,328 B `convert` case fits under 4,096 B, which is why FT-E18 carries both) and
  FT-E23 (exit 0). *Driven.*
- **Trap 14a, `main`'s `src/sink/mod.rs` restored verbatim, alone.** Does not compile (5 errors:
  `cannot find function publish_file_io in module crate::sink`, `… publish_file …`,
  `function guard_overwrite is private`), so every target is red at compile. *Driven.*
- **Trap 14b, `main`'s `src/sink/mod.rs`, `src/cli/ops.rs` and `src/cli/build.rs` restored**
  (`publish.rs` then isn't compiled, so no FT-U runs). Red, 15: FT-E1…FT-E9, FT-E11, FT-E15, FT-E16,
  FT-E19, FT-E22, FT-E23. Green on `main` (pins of behaviour that must not change): FT-E10, FT-E12,
  FT-E13, FT-E14, FT-E17, FT-E18, FT-E20, FT-E21. FT-E21 is green on `main` because `main`'s
  `write_lock` already keeps the old lockfile; F2's fix (an exclusive, unpredictable staged name that
  never follows a symlink) rests on `create_new`'s documented semantics and FT-U1, *reasoned*.
  *Driven.*

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### The code shape

- **`src/sink/publish.rs` (new, about 160 lines plus tests).** The helper and its seam, in its own file
  because `src/sink/mod.rs` is 1,449 lines. All `pub(crate)`:
  - `enum PublishError { CreateTemp { dir: PathBuf, source: io::Error }, Write(io::Error) }`.
    `CreateTemp` is any failure to create the staged file; `Write` is every later step.
  - `trait StageFs { fn create_new(&self, &Path) -> io::Result<File>; fn write(&self, &mut File,
    &[u8]) -> io::Result<usize>; fn flush(&self, &mut File) -> io::Result<()>; fn set_permissions(&self,
    &Path, Permissions) -> io::Result<()>; fn rename(&self, &Path, &Path) -> io::Result<()>; }`.
  - `struct RealFs;` implements it over `std`: `create_new` is
    `OpenOptions::new().write(true).create_new(true).open(path)`; `write` is `File::write` (one call;
    the helper loops); `flush`, `fs::set_permissions`, `fs::rename`.
  - `struct StagedFile { path: PathBuf, armed: bool }` whose `Drop` runs
    `let _ = fs::remove_file(&self.path)` while `armed`. The one remover.
  - `static NEXT: AtomicU64`, `fn process_id() -> u32` (`0` on wasm32), `const NAME_KEEP: usize = 100`,
    `fn staged_name(file_name: &str, n: u64) -> String` →
    `.{first NAME_KEEP bytes of file_name, cut back to a char boundary}.crustyimg-{pid}-{n}.tmp`, and
    `fn next_staged_name(file_name: &str) -> String` (takes `n` from `NEXT`). The cut keeps a
    240-byte name plus the suffix under the 255-byte name limit.
  - `fn publish(path: &Path, bytes: &[u8], fs: &dyn StageFs) -> Result<(), PublishError>`, in this
    order:
    1. `dir` = `path.parent()`, or `.` when it is empty.
    2. If `fs::symlink_metadata(path)` succeeds: open `path` for writing with no truncate and drop it
       (a failure is `Write`), then keep `meta.permissions()`. Otherwise no permissions to copy.
    3. Loop: `fs.create_new(dir.join(next_staged_name(name)))`. On `AlreadyExists` retry, at most 16
       times; any other error, or the 17th, returns `CreateTemp { dir, source }`. On success wrap the
       path in an armed `StagedFile`.
    4. Write: loop `fs.write` until every byte is out. `Ok(0)` is `WriteZero`; `Interrupted` retries;
       any other error returns `Write`.
    5. `fs.flush`, then `drop(file)`. The file is closed before the rename because Windows refuses to
       rename an open file.
    6. If permissions were kept: `fs.set_permissions(staged, perm)`.
    7. `fs.rename(staged, path)`, then disarm the remover. The remover is disarmed **after** the rename
       returns `Ok`, never before.
    No `BufWriter`, no `sync_all`, no directory `fsync`.
  - `#[cfg(test)] mod tests`: `enum FailAt { Create, Write(usize), Flush, SetPermissions, Rename }`
    and `struct FailingFs { at: FailAt, writes: Cell<usize> }`, which delegates to `RealFs` and fails
    with `io::Error::other("injected failure")` at its step. Its `write` passes at most
    `CHUNK = 1024` bytes to `RealFs.write` and counts calls (1-based), so `FailAt::Write(2)` fails
    with 1,024 bytes already in the staged file. The unit tests use a 5,000-byte payload, so a full
    write takes five calls.
- **`src/sink/mod.rs`.**
  - `:39-40`: drop `std::fs::OpenOptions` and `BufWriter` from the imports; add `mod publish;`.
  - `:210-213`: `SinkError::Io` stays, text unchanged, doc now "An I/O error while writing to stdout."
    Add after it `WriteFailed { path: String, source: io::Error }` and
    `DirNotWritable { path: String, dir: String, source: io::Error }` (texts under Messages).
  - The four arms: `write` `Sink::File` (`:499-509`), `write` `Sink::Dir` (`:533-543`), `write_bytes`
    `Sink::File` (`:645-652`), `write_bytes` `Sink::Dir` (`:665-672`). Each keeps its guards where they
    are and replaces `OpenOptions … truncate(true) … BufWriter … write_all … Ok(())` with
    `publish_file(<path>, <bytes>)`. Fix the comment at `:497` ("never truncate then error") and `:499`
    ("then write to a BufWriter<File>").
  - `:881`: `guard_overwrite` becomes `pub(crate)`.
  - New, after `guard_overwrite`: `pub(crate) fn publish_file(path: &Path, bytes: &[u8]) ->
    Result<(), SinkError>` (calls `publish::publish(path, bytes, &publish::RealFs)` and maps
    `CreateTemp` → `DirNotWritable`, `Write` → `WriteFailed`, each with `path.display()`), and
    `pub(crate) fn publish_file_io(path: &Path, bytes: &[u8]) -> io::Result<()>` (the same call,
    returning the bare `source`) for the lockfile. Gate `publish_file_io` with
    `#[cfg(not(target_arch = "wasm32"))]`: only the CLI calls it, and `just wasm-check` on the
    prototype (rc=0) warned "function `publish_file_io` is never used" (PR #240 makes wasm clippy
    `-D warnings`).
  - Module doc `:13-16` and `:22-26`: the overwrite guard runs before anything is created, and a file
    output is staged beside the destination and renamed over it. No spec or DEC ids.
- **`src/cli/ops.rs`.**
  - `run_edit`, right after step 1 (`:1687-1688`): when `save_recipe` is `Some(path)`, run
    `reject_symlink_destination(path)?` and `guard_overwrite(path, overwrite)?`, with `overwrite` from
    `global.yes` as `run_copy_metadata` builds it (`:1076-1080`). Comment: the recipe's destination is
    guarded before anything is written, so a refused recipe leaves the image output unwritten too.
  - `:1758-1761`: drop the late symlink check (moved up) and replace `std::fs::write(path, toml)…`
    with `crate::sink::publish_file(Path::new(path), toml.as_bytes())?`.
  - `:1675-1680` doc: an existing recipe without `--yes` is refused before the run; a write failure
    is `SinkError::WriteFailed` or `DirNotWritable` (exit 5).
  - `:22`: `SinkError` becomes unused in this file; drop it from the `use` (clippy `-D warnings`).
- **`src/cli/build.rs`.** `write_lock` (`:673-689`): body becomes
  `crate::sink::publish_file_io(Path::new(path), contents.as_bytes()).map_err(write_err)`. The doc
  comment drops "the same temp→rename discipline the cache store uses" for "staged beside it and
  renamed into place, like every file output". `CliError::LockWrite`'s text is unchanged.
- **`src/cli/mod.rs:511`** (`--save-recipe` help): "Write the op chain as a TOML recipe to FILE after
  a successful edit. An existing FILE needs --yes."
- **`tests/atomic_write.rs` (new):** the end-to-end rows, `#[cfg(unix)]` and `#[cfg(windows)]`
  modules as marked.

### Messages (verbatim, each with its exit code and its test)

- A file write that fails, any sink arm, `--save-recipe`, a read-only destination:
  `error: could not write output <path>: <io error>`, **exit 5**. `<path>` is the `-o` path as given,
  or, for `--out-dir`, the joined path `safe_join` returns (canonical directory plus name). Driven:
  `error: could not write output old.png: File too large (os error 27)`;
  `error: could not write output old.png: Permission denied (os error 13)` (0444, `-y`);
  `error: could not write output r.toml: File too large (os error 27)`. — FT-E1…E8, FT-E15, FT-E17.
- A directory that won't take the staged file:
  `error: could not write output <path>: cannot create a new file in <dir>: <io error>`, **exit 5**.
  `<dir>` is `path.parent()` as given, or `.`. Driven:
  `error: could not write output ro/old.png: cannot create a new file in ro: Permission denied (os error 13)`.
  — FT-E11.
- `--save-recipe` over an existing file without `-y`: the image lane's own text,
  `SinkError::AlreadyExists` (`src/sink/mod.rs:219-221`):
  `error: output file already exists (use --yes to overwrite): r.toml`, **exit 5**. — FT-E16.
- The lockfile: `CliError::LockWrite`, unchanged (`src/cli/mod.rs:727`):
  `error: could not write build lockfile crustyimg.build.lock: Permission denied (os error 13)`,
  **exit 5**. — FT-E21.
- In a batch, each failed input prints `error: <input>: could not write output <path>: <cause>`, then
  `error: N of M inputs failed`, **exit 6** (the batch path, unchanged). — FT-E19.
- stdout: `error: could not write output`, exit 5, unchanged (no test change; out of scope).

No message names a spec, stage or DEC id.

### What doesn't change: output bytes

The change is only in how bytes reach the disk; no encoder is touched. Today's pins of output bytes
go through `encode_to_bytes` or stdout (`tests/recipe_corpus_output.rs` hashes encoded bytes;
`tests/avif_tile_pin.rs`), not through a file sink, so none of them would see a write path that
changed bytes. New pin **FT-E18**: the file a sink writes is byte-identical to what the same command
sends to stdout (which does not use the helper), for the pixel lane (`convert … --format png`) and the
byte lane (`meta strip`). M11 turns it red by content (a hash-style `left == right` on the 83,916 B
case), not by exit code (see the table).

### The new DEC (final text; the build creates the file)

The build creates `decisions/DEC-<next>-a-file-write-is-all-or-nothing.md` from
`decisions/_template.md`, taking the next free id at build time (other STAGE-063 designs may have
claimed ids since). `affected_scope` in **block-list form** (HANDOFF trap 7):
`src/sink/mod.rs`, `src/sink/publish.rs`, `src/cli/ops.rs`, `src/cli/build.rs`, `SECURITY.md`,
`docs/cli-reference.md`, `docs/api-contract.md`, `tests/atomic_write.rs`. Body:

> **Decision: a file write is all or nothing.**
>
> - Every file crustyimg writes for the user is staged in a new file in the destination's own
>   directory, created exclusively (`create_new`, which refuses an existing path and never follows a
>   symlink), and renamed over the destination only after every byte is written. That covers image
>   outputs on every verb, `build` outputs, `meta` writing in place, `edit --save-recipe` and the
>   build lockfile. One helper does it, `src/sink/publish.rs`.
> - A failed write exits non-zero and leaves the destination's old bytes untouched, or leaves no file
>   if there was none. No staged file survives an error the process returns. A killed process can
>   leave one, named `.<name>.crustyimg-<pid>-<n>.tmp`; its `.tmp` extension keeps it out of
>   crustyimg's folder and glob inputs.
> - The symlink and overwrite guards (DEC-035) run before anything is created, on every output
>   except the build lockfile, which has no symlink guard: a symlinked lockfile is replaced by a
>   regular file, and nothing is read from or written through the link. An existing destination this
>   user cannot open for writing is refused, as it was before writes were staged, so `-y` never
>   replaces a read-only file.
> - `-y` replaces the file at that name with a new file. Its permission bits carry over. Its owner
>   (unless crustyimg runs as root), ACLs, extended attributes (on macOS, Finder tags) and hard links
>   do not.
> - A destination whose directory can't take a new file fails with exit 5 and names the directory.
>   There is no in-place fallback: it would bring back the write that destroys the old file.
> - Replacing a file needs room for the old and the new file at once. On a full disk that fails
>   loudly and keeps the old file.
> - The guarantee covers errors and killed processes. It does not cover power loss: nothing calls
>   `fsync` on the file or its directory. (`sync_all` is `F_FULLFSYNC` on macOS, about 5 ms a file.)
> - stdout is a stream: its errors exit non-zero, and there is nothing to stage. The build cache
>   keeps its own staged write (`src/build/cache.rs`), whose entries are hash-checked on read.
>
> See also: DEC-035 (symlinked destinations), DEC-044 (the out-dir is created first; the staged file
> goes inside it), DEC-059 (the lockfile, already written by temp and rename).

**DEC-059's pointer (ship-time text, not an edit now):** DEC-059's "Written atomically (temp →
`rename`), so a concurrent reader never sees a half-file" (`decisions/DEC-059-*.md:132`) stays true.
At ship, append to that bullet: "The staged write is DEC-<next>'s helper."

### Docs (re-checked `file:line`, exact text)

Every sentence below states behaviour. The build drives each on the exit paths listed with it and
quotes the result in Build Completion; verify drives them again.

1. `SECURITY.md:41-44`, threat 3. After "and the tool does not overwrite an existing file without
   `--yes`." insert: "A write that fails leaves the existing file as it was: output is staged beside
   the destination and renamed into place, and a destination crustyimg could not write before is
   still refused." Paths: one input `-o` (FT-E1, FT-E15), `--out-dir` batch (FT-E2), a batch with
   one failing input (FT-E19), `build` (FT-E21).
2. `SECURITY.md:87`, row 3. Replace "no overwrite without `--yes`" with "no overwrite without `--yes`
   (image output AND `--save-recipe`); writes are staged and renamed, all or nothing". Paths: `-o`
   (FT-E1), `--save-recipe` without and with `-y` (FT-E16, FT-E17).
3. `docs/cli-reference.md:55`, exit 5. New text: "Output write refused or failed (exists without
   `--yes`; path traversal; symlinked destination; a read-only destination; a directory that can't
   take a new file; the write itself failed, in which case the existing file is left as it was)."
   Paths: `-o` (FT-E1, FT-E15), `--out-dir` (FT-E2, FT-E11), `--save-recipe` (FT-E16).
4. `docs/cli-reference.md`, a note right under the exit-code table (after `:58`), not in the `-y`
   row: "**Replacing a file.** crustyimg writes each output to a new file beside the destination and
   renames it over the old one only when every byte is written, so a failed write keeps the old file.
   With `-y` the result is a new file at that name: it keeps the old file's permissions, but not its
   owner, ACLs, extended attributes (Finder tags on macOS) or hard links. The directory must allow
   new files: in a read-only directory `-y` exits `5`." Paths: `-y` mode kept (FT-E13), read-only
   directory (FT-E11), failed write (FT-E1).
5. `docs/cli-reference.md:332-337`, `--save-recipe`. After "A failed run writes no recipe." add:
   "An existing FILE is replaced only with `-y`; without it `edit` exits `5` before writing anything."
   Paths: FT-E16, FT-E17.
6. `docs/api-contract.md:60`, exit 5. New text: "Output write failed / refused (exists without
   `--yes`; name/path traversal; a symlinked destination, refused even with `--yes` — SPEC-034 /
   DEC-035; a failed write leaves the existing file as it was)." (The SPEC/DEC ids already in this
   row stay; this is a contract doc, not user-facing help.) Paths: FT-E1, FT-E2.
7. `skills/crustyimg/SKILL.md:54-55`. After "only when overwriting is what the user wants." add: "The
   same holds for `edit --save-recipe FILE`." `skills/crustyimg/SKILL.md:335`, exit 5 row: "Output
   write refused or failed (exists, symlink, path traversal, read-only directory; a failed write keeps
   the old file)" | "Add `-y` if overwriting is intended; write to a directory you can create files
   in". Paths: FT-E16, FT-E11.
8. `--help`: `src/cli/mod.rs:511` as in the code shape. Path: FT-E16.
9. Unchanged, checked: `README.md:292-295` ("`5` output refused") stays a one-line summary;
   `docs/cli-reference.md:39` and `src/cli/mod.rs:106` (`-y`: "Assume "yes" to overwrite prompts
   (non-interactive).") stay as they are, because the note in 4 carries the detail.

### CHANGELOG (verbatim, under `## [Unreleased]`)

`### Fixed`:
- **A failed write no longer leaves a short or empty file.** Every file crustyimg writes (image
  outputs on every verb, `build` outputs, `meta` writing in place, `edit --save-recipe` and the build
  lockfile) now goes to a new file beside the destination and is renamed over it only when every
  byte is written. A write that fails (a full disk, a file-size limit) exits 5, names the file and
  the cause, and leaves the file that was there as it was. 0.8.2 and earlier exited 0 over a
  truncated output under 8 KiB, and destroyed the old file on a larger one. `-y` now makes a new
  file: it keeps the old file's permissions, but not its owner, ACLs, extended attributes (Finder
  tags on macOS) or hard links, and replacing a file needs room for both at once.

`### Changed`:
- **`-y` into a directory that can't take new files is refused.** It exits 5 with
  `could not write output <file>: cannot create a new file in <dir>: …`. 0.8.2 and earlier rewrote
  the file in place, which is the write that could destroy it.
- **`edit --save-recipe` needs `-y` to replace an existing recipe.** Without it `edit` exits 5 with
  `output file already exists (use --yes to overwrite): <file>` before writing anything. 0.8.2 and
  earlier replaced the recipe silently.

### Size

**[M] holds.** Derivation: code in 5 files (`src/sink/publish.rs` new, `src/sink/mod.rs`,
`src/cli/ops.rs`, `src/cli/build.rs`, `src/cli/mod.rs`). The prototype's `git diff --stat`: 5 files,
865 insertions, 46 deletions; `publish.rs` 375 lines of which 167 before `#[cfg(test)]`
(`/usr/bin/grep -n` → line 168); call sites `src/sink/mod.rs` 85, `src/cli/ops.rs` 15,
`src/cli/build.rs` 8 changed lines; `tests/atomic_write.rs` 428 lines (without the Windows rows).
Tests: 12 unit + 23 end-to-end + 4 Windows = 39 (counted by FT id: U1-U12, E1-E23, W1-W4). Docs in 5 files plus the CHANGELOG and the DEC. One mechanism and one helper, so not L; the
test matrix (3 guarantees × 2 lanes × 2 sinks × 2 sizes, plus modes, directories, recipes, the lock,
3 OSes) is past S.

### Overlap with open PRs and pushed branches (from their file lists, 2026-11-22)

Read with `gh pr list --state open --json number,title,files` and `git branch -r`. There is **no
pushed `feat/spec-15[4-8]*` branch**; the remote `feat/spec-150…153` and `feat/spec-159…` branches
belong to shipped specs. Files this spec touches that another open PR also touches:
- `src/cli/ops.rs` — **PR #240** (`ci: run clippy on the wasm32 library build`). Its hunks: the
  `use super::common::{…}` block (`@@ -25,6 +25,14 @@`), and `run_metadata_lane` (`:799`, `:868`) and
  `run_copy_metadata` (`:984`). This spec changes the `use crate::sink::{…}` line (`:22`, drops
  `SinkError`) and `run_edit` (`:1681-1764`). The `:22` line sits three lines above #240's first hunk:
  expect a trivial conflict on rebase; keep both sides.
- `CHANGELOG.md` — **PR #237** (heifer). It adds lines at the top of `## [Unreleased]`
  (`@@ -7,6 +7,21 @@`). This spec adds under the existing `### Changed` and `### Fixed`. Keep both.
- `src/cli/mod.rs` — **PR #226** (`docs(agents)`). This spec changes one doc comment (`:511`). If it
  conflicts, keep both.
- `src/cli/build.rs` — no open PR touches it today. **SPEC-157** (destination identity, the
  lockfile's paths) and **SPEC-158** (lockfile size) will; their designs name their functions. This
  spec changes **only `write_lock`** (`:673-689`), its body and doc comment, and nothing in
  `lock_output_path`, `lock_record`, `prepare_target`, `check_output_injective` or `run_build`. A
  conflict inside `write_lock`: keep this spec's body (the helper call) and the other side's
  path/contents logic around it.
- `src/sink/mod.rs`, `src/sink/publish.rs`, `tests/atomic_write.rs`, `SECURITY.md`,
  `docs/cli-reference.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`: no open PR.
On a rebase conflict in the STAGE-063 file, keep both sides and recount its `**Count:**` line with a
fresh `/usr/bin/grep -c`. On a conflict in any other file not named above, stop and report. Never
edit `projects/PROJ-017-geometry-as-recipe-steps/`.

### Verification the build runs

- `just check` with stdout redirected to a file (an interactive terminal fails
  `display_sink_refuses_non_tty`); read `$?`, not a pipe's.
- The full mutation table below, each row alone, on a full `cargo test --no-fail-fast` of every
  target, red set read off the run. A mutant that doesn't compile stops `cargo test`; run that row's
  targets separately and say so.
- `just wasm-check` and `just wasm-test` (AGENTS §15 build rule; `src/sink` is in the wasm build).
- `cargo clippy --all-targets -- -D warnings` on the default and the lean
  (`--no-default-features`) builds (the verify-includes-lean rule).
- **CI legs to read on the PR:** the three-OS `cargo test` matrix (`.github/workflows/ci.yml:177-201`),
  the msrv 1.90.0 leg (`:360`), the lean leg, the wasm legs. **Windows' leg is the only evidence for
  FT-W1…W4**: read its red set on the PR, quote the four rows' results, and if W2's exit code or
  message differs from the reasoned one below, stop and report rather than loosen the assertion.
  Don't poll CI: start one background `gh pr checks <n> --watch` and wait for its notification.
- **Checkpoints:** push the branch after (1) the helper and its unit tests, (2) the four sink arms,
  (3) `--save-recipe` and `write_lock`, (4) the end-to-end file, (5) docs, CHANGELOG and DEC.

### What the Windows rows will see (reasoned; CI only)

- W1: `MoveFileExW(…, MOVEFILE_REPLACE_EXISTING)` succeeds; exit 0.
- W2: crustyimg's write-probe open succeeds (the holder shares write). `MoveFileExW` fails (the
  destination is open without `FILE_SHARE_DELETE`; Windows reports this as `ERROR_ACCESS_DENIED`).
  std retries with `FileRenameInfoEx` + POSIX semantics, which also fails on a target open without
  share-delete, and returns the first error: exit 5,
  `could not write output old.png: Access is denied. (os error 5)`. The remover deletes the staged
  file (it is closed, so delete works). If CI shows a sharing violation (`os error 32`) instead, the
  test still passes, because it asserts only the prefix.
- W3: the write-probe open fails with `ERROR_ACCESS_DENIED` before anything is staged: exit 5, no
  staged file was ever made.
- W4: crustyimg holds no handle on the input at the rename (every input read is `std::fs::read`), so
  `MoveFileExW` replaces it: exit 0.

### Decisions that apply

- `DEC-035` — symlinked destinations are refused even with `--yes`; the guard still runs first and
  the rename (which replaces a link rather than following it) hardens it.
- `DEC-044` — `--out-dir` is created before the write; the staged file goes inside it.
- `DEC-059` — the lockfile is already written by temp and rename; it now uses the shared helper. Its
  ship-time pointer is above.
- `DEC-061` — symlinked parent directories and destination identity stay SPEC-157's ground.
- `DEC-<next>` (new, created by the build) — a file write is all or nothing.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — no new dependency: the helper is hand-rolled on `std`
  (`tempfile` stays a dev-dependency).
- `untrusted-input-hardening` — `create_new` never follows a symlink at the staged name, which fixes
  `write_lock`'s predictable `{path}.tmp{pid}` (F2).
- `no-unwrap-on-recoverable-paths` — every step returns its error; the remover ignores only its own
  `remove_file` result.
- `clippy-fmt-clean`, `test-before-implementation`, `every-public-fn-tested`.

### Prior related work

- SPEC-005 (the sink), SPEC-034/037 (symlink guard, DEC-035), SPEC-063/067 (build lockfile and
  `--watch`, DEC-059/060), PATCH-001 (out-dir auto-create, DEC-044).
- The frame of this spec (`88089d4`) and the maintainer's rulings above.

### Out of scope (for this spec specifically)

- stdout and the display sink: a closed pipe already exits 5; a stream has no old file to keep.
- The build cache (`src/build/cache.rs`): already temp→rename with errors propagated and hash-checked
  reads.
- `fsync` and power-loss durability (ruling 2; a non-guarantee in the DEC).
- Removing stale `.crustyimg-` staged files a killed process leaves.
- Symlinked parent directories and destination identity: SPEC-157 and DEC-061.
- The lockfile size contract: SPEC-158.
- Two `optimize` inputs mapping to one destination in a batch (the last rename wins, whole; no torn
  file). Not new, not this spec's.

## Notes for the Implementer

- Read `src/sink/mod.rs:480-690` and `src/cli/ops.rs:1681-1764` before writing; the four arms are
  near-identical and each keeps its own guards.
- The prototype this design drove kept `publish.rs` free of `cfg(unix)` except in tests. Don't reach
  for `OpenOptionsExt::mode`: std's default `0o666` already is the rule.
- Test preconditions that need a non-root user **assert** the precondition (fail with "running as
  root?"), never print a skip (HANDOFF trap 11).
- FT-E22 must capture stderr through a pipe (`Command::output()` does). A redirect to a file inside
  the `sh -c` would be capped by `ulimit -f 0` and turn the row into a panic (exit 101), driven at
  design.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.
- No version bump.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-155-atomic-writes`
- **PR (if applicable):** #268 (draft; verify approves, the orchestrator marks it ready)
- **All acceptance criteria met?** yes: AC-1 to AC-19, AC-21 and AC-22 driven locally. AC-20 (Windows) rests on the Windows CI leg: W1–W4 all passed on the Windows leg: `windows_yes_replaces_an_existing_file ... ok`, `windows_a_destination_held_open_without_share_delete_is_kept ... ok`, `windows_a_read_only_destination_is_refused_and_kept ... ok`, `windows_yes_over_the_input_itself_replaces_it ... ok`. W2 got exit 5 and the `could not write output old.png:` prefix with the old bytes kept, as reasoned; the OS text was not asserted.
- **New decisions emitted:**
  - `DEC-116` — A file write is all or nothing (next free id at build time: `main` ended at DEC-115)
- **Files this diff touches** (13, from `git diff --name-only origin/main...HEAD`, including this spec file):
  - `src/sink/publish.rs` (new) — `StageFs` seam, `RealFs`, `StagedFile` guard, `publish`, staged names, 12 unit tests (FT-U1…U12)
  - `src/sink/mod.rs` — four arms call `publish_file`; `WriteFailed`/`DirNotWritable`; `guard_overwrite` is `pub(crate)`; `publish_file`, `publish_file_io` (non-wasm); module docs
  - `src/cli/ops.rs` — `run_edit` guards `--save-recipe` before the run, writes it with `publish_file`; `SinkError` import dropped
  - `src/cli/build.rs` — `write_lock` only: body and doc comment
  - `src/cli/mod.rs` — `--save-recipe` help gains "An existing FILE needs --yes."
  - `tests/atomic_write.rs` (new) — FT-E1…E23 and FT-W1…W4
  - `SECURITY.md`, `docs/cli-reference.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md` — the doc sentences
  - `CHANGELOG.md` — one Fixed and two Changed entries
  - `decisions/DEC-116-a-file-write-is-all-or-nothing.md` (new)
  - this spec file — Build Completion and cost
  - `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION`, the lockfile format: no diff (`git diff --stat origin/main...HEAD -- Cargo.toml Cargo.lock` prints nothing)
- **Deviations from spec:**
  - The orchestrator's correction: wherever a doc sentence names the failed-write exit code for a case a batch can reach, it reads "exits 5 (6 for a batch, after the other inputs are written)": the CHANGELOG Fixed line, the CHANGELOG `-y`-into-a-read-only-directory line, and the `docs/cli-reference.md` "Replacing a file" note. The spec's text said "exits 5" in all three. Driven on both paths (below).
  - The unit-test module is `#[cfg(test)]` plus a separate `#[cfg(not(target_arch = "wasm32"))]`, not `all(test, …)`: clippy's test exemption (`clippy.toml`) matches only a bare `cfg(test)`, and the `all()` form failed clippy `-D warnings` with six `unwrap_used` errors on every leg. Fixed in 3802b14.
  - The Windows leg's first run failed clippy on two dead-code warnings in `tests/atomic_write.rs` (above); `single_quote` and `limited` are now `cfg(unix)`.
  - M9's quoted edit calls `fs::write`; inside `publish` the parameter `fs` shadows the module, so the edit applied is `std::fs::write(path, bytes)`.
- **Follow-up work identified:**
  - The mutation table's M1/M2/M3 "Red" lists omit FT-E23 (below); M11's 46 is 48 here.

### Build-phase evidence

**RED baseline on `main`'s code** (test file only, `cargo test --test atomic_write --no-fail-fast`, stdout to a file, `$?` read: 101; 8 passed, 15 failed). The first run was invalid: six rows panicked in `small_png` (`could not write output`, because `in/` didn't exist), so the helper creates parent directories. Baseline after that fix, each red row's failure line:
  - E1–E4 `efbig_small_{pixel,bytes}_{file,dir}`: `assertion left == right failed: stderr: … left: Some(0) right: Some(5)`
  - E5–E8 `efbig_big_*`: `bytes kept: …/old.png holds 1024 bytes, expected 29 bytes` (and `…/od/in.png` for the dir rows)
  - E9 `sigxfsz_kill_keeps_the_old_file`: `bytes kept: …/old.png holds 1024 bytes, expected 29 bytes`
  - E11 `a_read_only_directory_is_refused_by_name`: `left: Some(0) right: Some(5)`
  - E15 `yes_over_a_read_only_file_is_refused`: `stderr: error: could not write output` (no path, no cause)
  - E16 `save_recipe_without_yes_refuses_an_existing_recipe`: `left: Some(0) right: Some(5)`
  - E19 `a_batch_with_one_refused_destination…`: `stderr: error: …/in/a.png: could not write output` / `error: 1 of 2 inputs failed`
  - E22 `save_recipe_failed_write_keeps_the_old_recipe`: `bytes kept: …/r.toml holds 0 bytes, expected 8 bytes`
  - E23 `a_batch_where_every_write_fails…`: `left: Some(0) right: Some(6)`
  - Green on `main` (pins): E10, E12, E13, E14, E17, E18, E20, E21. Exactly the spec's 15 red rows, each for its stated reason.

**After the build:** `cargo test --test atomic_write`: 23 passed. `cargo test --lib publish`: 13 passed (12 FT-U plus one existing). Full `cargo test --no-fail-fast`, stdout redirected, `$?` = 0: 64 targets, all `ok`.

**Acceptance criteria, one line each**
  - AC-1 met (E1–E4). AC-2 met (E1–E8). AC-3 met (U1–U7). AC-4 met (E9). AC-5 met (E1–E8, E15, E22). AC-6 met (E11). AC-7 met (E12). AC-8 met (U11, E13). AC-9 met (E10). AC-10 met (U12, E15; W3 on CI). AC-11 met (E16). AC-12 met (E17). AC-13 met (E22). AC-14 met (E21). AC-15 met (E14, U10). AC-16 met (E19, E23). AC-17 met (E20). AC-18 met (U9). AC-19 met (E18, U8). AC-20 W1–W4 all passed on the Windows leg: `windows_yes_replaces_an_existing_file ... ok`, `windows_a_destination_held_open_without_share_delete_is_kept ... ok`, `windows_a_read_only_destination_is_refused_and_kept ... ok`, `windows_yes_over_the_input_itself_replaces_it ... ok`. W2 got exit 5 and the `could not write output old.png:` prefix with the old bytes kept, as reasoned; the OS text was not asserted.. AC-21 met (below). AC-22 met: `just wasm-check` rc 0 (one warning, `MANIFEST_DROPPED_WARNING`, which is not from this diff), `just wasm-test` rc 0, 56 passed.

**Each behaviour sentence, driven on every exit path it names** (debug binary built from the pushed code; paths shortened):
  1. SECURITY threat 3 ("a write that fails leaves the existing file as it was…"):
     - one input `-o`: `convert big.png --format png -o old.png -y` under `ulimit -f 1`, XFSZ ignored → `exit=5`, `error: could not write output old.png: File too large (os error 27)`, `old.png` still `OLDOLD`, no staged file.
     - `--out-dir`, one input: `exit=5`, `error: could not write output od/small.png: File too large (os error 27)`, `od/small.png` still `OLDOLD`, `od` holds only `small.png`.
     - batch with one failing input (`od2/a.png` 0444): `exit=6`, `error: in/a.png: could not write output od2/a.png: Permission denied (os error 13)`, `error: 1 of 2 inputs failed`, `a.png` still `OLDOLD`, `b.png` written.
     - batch where every write fails (`ulimit -f 1`): `exit=6`, one `error: in/X.png: could not write output od2/X.png: File too large (os error 27)` per input, `error: 2 of 2 inputs failed`, both `OLDOLD`.
     - `build` with a read-only project directory: `exit=5`, `error: could not write build lockfile crustyimg.build.lock: Permission denied (os error 13)`, lockfile still `OLD LOCK`.
     - "a destination crustyimg could not write before is still refused": `-y` over a 0444 file → `exit=5`, `error: could not write output ro.png: Permission denied (os error 13)`.
  2. SECURITY row 3 (`--save-recipe`, "staged and renamed"): `-o`: covered above. `--save-recipe` without `-y` → `exit=5`, `error: output file already exists (use --yes to overwrite): r.toml`, `r.toml` still `PRECIOUS`, `e.png` not created. With `-y` → `exit=0`, `r.toml` starts `version = "1"`.
  3. cli-reference exit 5 row: `-o` and `--out-dir` as above; read-only directory `-y` → `exit=5`, `error: could not write output rd/old.png: cannot create a new file in rd: Permission denied (os error 13)`; `--save-recipe` as above.
  4. cli-reference "Replacing a file" note: mode kept: `-y` over a 0640 file → `640` afterwards (E13 also 0600); read-only directory, one input → exit 5 (above); read-only directory, batch of two → `exit=6`, `error: in/a.png: could not write output rd/a.png: cannot create a new file in rd: Permission denied (os error 13)` (and b.png), `error: 2 of 2 inputs failed`. That is the case the orchestrator's correction covers; a failed write: E1 above.
  5. cli-reference `--save-recipe`: without `-y` exit 5 before writing anything; with `-y` exit 0 (above).
  6. api-contract exit 5: `-o`, `--out-dir` (above).
  7. SKILL.md: `--save-recipe` (above), read-only directory (above).
  8. `--help`: `edit --help` prints `Write the op chain as a TOML recipe to FILE after a successful edit. An existing FILE needs --yes.`

**Mutations**, each alone in a scratch worktree of the pushed head with its own `CARGO_TARGET_DIR`, `git diff` non-empty, `cargo clean -p crustyimg` before each, full `cargo test --no-fail-fast`, red set read off the run (the per-row diffs and outputs are in the session scratchpad). Edits applied are the spec's quoted edits except where noted.
  - M1 (`Err(_) => break,`): red 12 = U2, U3, E1–E8, E22, **E23**. Table: U2, U3, E1–E8, E22. **E23 is a cell the table misses** (every write swallowed → exit 0). U4 and E18 green, as the table says.
  - M1b (`let _ = fs.flush(…)`): red 1 = U4. Matches.
  - M2 (in-place write in `publish_file`): red 12 = E1–E9, E11, E22, **E23**. Table: E1–E9, E11, E22. Every FT-U, E21 and the message halves green, as the table says. E23 is missing from the table.
  - M3 (remover's `remove_file` deleted): red 15 = U2–U6, E1–E8, E22, **E23**. Table: U2–U6, E1–E8, E22. U1 and E21 green. E23 missing from the table.
  - M4 (`old_perm.filter(|_| false)`): red 3 = U5, U11, E13. Matches.
  - M5 (guard dropped in the pixel-lane `Sink::File`): red 2 = `tests/sink.rs::overwrite_guard_forbids_then_allows`, E12. Matches.
  - M6 (`--save-recipe` guard dropped): red 1 = E16. Matches.
  - M7 (fixed `<n>`): red 1 = U10; E14 green. Matches.
  - M7b (fixed `<n>`, `tries < 0`): red 1 = U10 only; E14 still green over this run. **The by-design claim holds:** U10 is the only row that sees a collision.
  - M8 (disarm before the rename): red 1 = U6. Matches.
  - M9 (in-place fallback on `PermissionDenied`, with `std::fs::write`): red 2 = E11, E21. Matches.
  - M10 (write-probe deleted): red 3 = U12, E15, E19. Matches.
  - M11 (`file.write(&buf[..buf.len().min(4096)]).map(|_| buf.len())`): red **48**, table says 46. The run includes E18 (`file and stdout bytes differ`, by content) and E23 (exit 0). The two extra are my unit rows **U9** and **U11** (`a_long_file_name_still_writes`, `a_replaced_file_keeps_its_mode`): they publish through `RealFs` with the 5,000 B payload, which is over 4,096 B. U8 stays green because `FailingFs` hands `RealFs` at most 1,024 B a call.
  - Trap 14a (`main`'s `src/sink/mod.rs` alone): does not compile, 3 distinct errors reported for the lib and again for the lib test: `cannot find function publish_file_io in module crate::sink`, `cannot find function publish_file …`, `function guard_overwrite is private`. Every target red at compile. The table says 5 errors; it counts the same three plus the repeats.
  - Trap 14b (`main`'s `mod.rs`, `ops.rs`, `build.rs`): red 15 = E1–E9, E11, E15, E16, E19, E22, E23. Matches exactly; E10, E12, E13, E14, E17, E18, E20, E21 green.
  - Restored and rebuilt clean: the scratch worktree at the pushed head, `cargo test --no-fail-fast`, `$?` = 0.

**Matrix** (each leg in a fresh `CARGO_TARGET_DIR`, each deleted after): `fmt --check`, `clippy --all-targets -D warnings` and `cargo test --no-fail-fast`, default, `--no-default-features`, and `--features webp-lossy`. The first pass failed clippy on all three legs (the test-module attribute, deviation 2); tests passed on all three with rc 0 on that pass, 64 targets on default. After the fix, clippy and `fmt --check` were re-run on all three legs in fresh directories: rc 0 each. The three test legs were not re-run for the attribute change, which does not alter what compiles in a test build; `cargo test --lib publish` is 13 passed on the final head. `just wasm-check` rc 0, `just wasm-test` rc 0.

**CI legs** (PR #268): head 47003ae, every required leg green: build/test/clippy/fmt on ubuntu, macos and windows; lean; msrv 1.90.0; avif; webp-lossy; heic (ubuntu, macos); wasm round-trip (Node); ddphotos parity; browser smoke; cargo-deny; DCO; front-matter; cost-capture audit. The first run (head 3802b14) had one red leg: Windows, at clippy `-D warnings` with two dead-code warnings (`single_quote`, `limited`, which only the Unix rows call) while its tests passed W1–W4. Fixed in 47003ae by gating both to `cfg(unix)`; I could not run a Windows clippy locally (no Windows target installed), so CI was the check, and it is green. The last `gh pr checks --watch` exited with nothing pending.

**Overlap with open PRs, after the last rebase onto `origin/main`:** #240 touches `src/cli/ops.rs` (its `use super::common` block and the metadata lane); no conflict arose on rebase. #237 touches `CHANGELOG.md` (new lines at the top of Unreleased): no conflict. #226 touches `src/cli/mod.rs`: no conflict. New since the first check: **SPEC-156 now has draft PR #269** (`feat/spec-156-strip-keeps-app14`): it touches `CHANGELOG.md`, `docs/api-contract.md` and `docs/cli-reference.md`, which this PR also edits (different hunks; expect a trivial conflict in whichever merges second), plus `src/metadata/mod.rs` and tests. It takes no decision id (`decisions/` on its branch ends at DEC-115, so DEC-116 is still free). No SPEC-157 build branch or PR exists.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Little. Two small things: the FT-E fixture helper must create parent directories (six rows panicked on a missing `in/` before they reached the code under test), and the spec's M1/M2/M3 "Red" cells omit FT-E23, so a reader checking the table sees an unlisted red.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The clippy test exemption: the lint config exempts tests only through a bare `#[cfg(test)]`, and the spec's code shape didn't say how to gate a test module that must also skip wasm32.

3. **If you did this task again, what would you do differently?**
   — Run clippy `-D warnings` on the first commit with the unit tests, not at the matrix. And never use `pgrep -f` for a wait loop: it matches its own shell, so each of my waiters waited on itself (I killed them; no data was affected).
   — No design call turned out wrong.

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
