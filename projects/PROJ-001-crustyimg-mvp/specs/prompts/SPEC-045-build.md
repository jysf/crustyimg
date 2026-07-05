# SPEC-045 — BUILD cycle prompt (Sonnet, prescriptive)

You are the **build** implementer for **SPEC-045**: replace **`little_exif`** with a small
**in-house binary TIFF-IFD reader+writer** in `src/metadata/`, removing `quick-xml`
(`RUSTSEC-2026-0194`/`-0195`) + `brotli` from the tree and deleting those two `deny.toml`
ignores. **Behavior-preserving** — `set` / `clean --gps` keep their exact public API and
observable behavior. **Hardening is a first-class requirement** (the EXIF block is
attacker-influenced): the parser must be bounds-checked and **never panic** on malformed
input.

## Where to work

- **Worktree (do ALL work here):**
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg-wt-spec045`
- **Branch (already checked out there):** `feat/spec-045-exif-writer` (based on `main`).
- Verify `git -C <worktree> rev-parse --abbrev-ref HEAD` prints `feat/spec-045-exif-writer`
  before starting. Do NOT touch the main checkout.

## Read first (in the worktree)

1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-045-in-house-tiff-ifd-exif-writer-to-drop-little-exif.md`
   — the spec. Its **## Failing Tests**, **## Acceptance Criteria**, and **## Implementation
   Context** are your contract.
2. `decisions/DEC-046-in-house-tiff-ifd-exif-writer.md` — the decision, the writer design,
   the `paste` residual (do NOT try to remove `-2024-0436`).
3. `src/metadata/mod.rs` — the module. Replace only the `little_exif` users: `set_tags`
   (~199–224) and `clean_gps` (~141–177). `strip_all` and `copy_metadata` stay. The
   `#[cfg(test)]` helpers seed EXIF via `little_exif` and MUST be re-based (crate is gone).

## What to change

1. **New `src/metadata/tiff.rs`** (a crate-internal submodule; `mod tiff;` in `mod.rs`):
   a bounded, panic-free TIFF-IFD parser + a normalizing little-endian serializer. The
   probe below is the **validated skeleton** — extend it (bounds checks, IFD1/thumbnail,
   the edit ops). Model:
   ```rust
   struct Entry { tag: u16, ty: u16, count: u32, value: Vec<u8>, sub: Option<Ifd> }
   struct Ifd   { entries: Vec<Entry>, next: Option<Box<Ifd>> } // next = IFD1 (thumbnail)
   ```
   **Parser** — header (`II`/`MM` + magic 42 + IFD0 offset). Each 12-byte entry:
   `(tag, ty, count)`, `vlen = type_size(ty) * count`, value inline (`vlen ≤ 4`) or at the
   4-byte offset. Recurse the pointer tags **ExifIFD 0x8769 / GPS 0x8825 / Interop 0xA005**
   into `sub` (**cap depth ≤ 8** to kill cycles). Follow IFD0's next-IFD link to IFD1;
   treat IFD1 `JPEGInterchangeFormat 0x0201` (offset) + `0x0202` (length) as a relocatable
   thumbnail blob. **Every index/offset/length read must be bounds-checked** → return
   `Err` (map to `MetadataError::Exif`) on any OOB/overflow; NEVER panic. `type_size`:
   1/2/6/7→1, 3/8→2, 4/9/11→4, 5/10/12→8, else→treat as error or 1 (guard `vlen`).
   **Serializer** — emit normalized LE: header, IFD0 directory, then sub-IFDs + out-of-line
   values appended, patching each entry's 4-byte offset slot; keep offsets even (pad odd
   blobs); emit IFD1 + relocate its thumbnail blob and fix `0x0201`.

   **Probe-validated skeleton (round-trips a real JPEG byte-identical; extend it):**
   ```rust
   const EXIF_PTR: u16 = 0x8769; const GPS_PTR: u16 = 0x8825; const INTEROP_PTR: u16 = 0xA005;
   fn tsz(ty: u16) -> usize { match ty {1|2|6|7=>1,3|8=>2,4|9|11=>4,5|10|12=>8,_=>1} }
   // parse: read u16/u32 by byte order; for each entry resolve value (inline if vlen<=4 else @offset);
   //        if tag in {EXIF_PTR,GPS_PTR,INTEROP_PTR} recurse parse_ifd at its LONG offset.
   // serialize (LE): out = [b'I',b'I',42,0, 8,0,0,0]; then put_ifd(&mut out, &ifd0):
   //   write entry count (u16), reserve n*12 dir bytes + 4-byte next-offset, then for each entry
   //   fill tag/ty/count; if sub -> append via put_ifd and patch the 4-byte offset slot;
   //   else if value.len()<=4 -> inline; else append value (pad to even) and patch offset slot.
   ```
   (The full working probe lives in the design session; the shape above is what passed.)

2. **`src/metadata/mod.rs`** — re-implement on `tiff` + `img-parts`:
   - `set_tags`: read existing TIFF via `img-parts` `ImageEXIF::exif()` (JPEG `Jpeg`, PNG
     `Png`) — it returns the **bare TIFF** (`II*`/`MM\0*`, no `Exif\0\0`). Parse → in IFD0
     add/replace ASCII (type 2) entries for `ImageDescription 0x010E` / `Artist 0x013B` /
     `Copyright 0x8298` (value = UTF-8 bytes + trailing NUL, `count = len+1`) → serialize →
     `jpeg.set_exif(Some(Bytes::from(tiff)))` → `encoder().write_to(&mut out)`. **No EXIF**
     → build a minimal TIFF (header + IFD0 with just the target tags).
   - `clean_gps`: parse the TIFF → drop the IFD0 `GPS_PTR (0x8825)` entry (don't emit its
     sub-IFD) → serialize → set back. **No EXIF** → return the input bytes unchanged
     (byte-identical no-op — preserve today's behavior).
   - Keep `TagSet`, the public fn signatures, and every `MetadataError` variant identical.
     `run_set`/`run_clean` in `src/cli/mod.rs` must compile unchanged.
   - Re-base the test helpers (`jpeg_with_exif`, PNG variants) to seed EXIF WITHOUT
     `little_exif` — build the seed TIFF with the new writer (or hand-assemble a tiny TIFF)
     and embed via `img-parts`.

3. **`Cargo.toml`** — remove `little_exif = "=0.6.23"` and update its comment block.

4. **`deny.toml`** — delete the `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` entries + their
   comment. **KEEP `RUSTSEC-2024-0436`** (paste) but **correct its comment**: paste is now
   reached only via `rav1e`→`ravif`→`image`/`avif` (drop the now-false "via little_exif"
   clause); revisit when rav1e drops paste.

5. **Tests** — add the 8 tests from the spec's **## Failing Tests** to `src/metadata/mod.rs`
   (`set_preserves_exififd_subtag`, `set_preserves_ifd1_thumbnail`, `set_overwrites_existing_tag`,
   `set_on_no_exif_creates_minimal`, `clean_gps_removes_only_gps`, `clean_gps_no_exif_is_noop`,
   `set_and_clean_preserve_pixels`, `malformed_exif_errors_not_panics`), plus keep all
   existing metadata tests passing. Assert semantics via **`kamadak-exif`**, not byte-compare.

## Gates (all must pass, in the worktree)

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --no-default-features
cargo deny check advisories bans sources licenses     # green with -0194/-0195 removed, -2024-0436 kept
cargo tree | grep -c little_exif    # MUST be 0
cargo tree | grep -c quick-xml      # MUST be 0
cargo tree | grep -c brotli         # MUST be 0
cargo tree -i paste                 # still shows the rav1e path (expected — do NOT try to remove)
```

After `cargo fmt`, re-stage all touched files (`git add -u`) before committing.

## Finish

1. Fill the spec's **## Build Completion** + flip the timeline `build` line to `[x]`.
2. Commit on `feat/spec-045-exif-writer` (include `Cargo.lock`). Message:
   `fix(SPEC-045): in-house TIFF-IFD EXIF writer; drop little_exif (-0194/-0195)`
   ending with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
3. Push the branch; open a PR against `main` with `gh pr create`. PR body: what changed,
   gate results (esp. `cargo tree` little_exif/quick-xml/brotli = 0, and `paste` still via
   rav1e), the hardening approach, and "behavior-preserving; see SPEC-045 / DEC-046". End
   with the Claude Code attribution line. **Do NOT merge.** Report the PR URL + gate output.

## Guardrails

- **Never panic on malformed EXIF** — the `malformed_exif_errors_not_panics` test is the
  gate. No `unwrap()`/`expect()`/unchecked indexing on the byte path; bound every read;
  cap sub-IFD recursion depth.
- Behavior parity via `kamadak-exif` semantics, NOT byte-identity with `little_exif`.
- Do NOT try to remove `paste`/`-2024-0436` (impossible — rav1e/avif path; see DEC-046).
- Do NOT change `strip_all`, `copy_metadata`, `TagSet`, `MetadataError` variants, or the
  `run_set`/`run_clean` CLI surface.
- If a step is genuinely blocked (e.g. `img-parts` PNG `eXIf` behaves unexpectedly), STOP
  and report rather than expanding scope or re-adding a dep.
