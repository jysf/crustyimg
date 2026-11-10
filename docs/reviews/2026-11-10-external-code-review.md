# External code review, 2026-11-10

An outside review of `main` at `c2aa9de` (0.8.2 plus unreleased work). It recommended **holding
0.8.3** for the four P1 findings. This file keeps the evidence so the frame sessions for SPEC-154
to SPEC-158 can start from it. The reviewer's reproductions ran in a separate checkout and have
**not** been re-driven here; each frame must drive its own finding on `main` before it relies on
a number below.

On 2026-11-10 the orchestrator read the cited code in this repo and confirmed that the code each
finding points at is present (the sink, the JPEG strip loop, the collision key, `LOCK_MAX_BYTES`,
the double read, the classifier regex). That confirms the cause, not the symptom.

## Findings

| Pri | Finding | Evidence (from the review) | Fix asked for | Spec |
|---|---|---|---|---|
| P1 | File writes can return success after losing data | The file and directory sinks truncate the destination, then return after `BufWriter::write_all` without checking flush. With a one-byte file-size limit, `convert` exited 0 and left a one-byte PNG. | Write directly when the bytes are already buffered, propagate write errors, stage replacements before publishing so a failed write preserves the existing file. `src/sink/mod.rs` (pixel sink ~498, byte sink ~640). | SPEC-155 |
| P1 | `build` can overwrite a declared source | A resize target with `source = "src/photo.png"` and `out = "src"` replaced the 2×2 original with a 1×1 image and exited 0. An immediate `build --check` exited 7. | Reject destinations that overlap any declared source before writing, including overlaps across targets. `src/cli/build.rs` (preflight ~384, overwrite policy ~544). | SPEC-157 |
| P1 | Aliased destinations bypass collision detection and produce a false lockfile | With an existing `alias/` directory, targets writing `dist/result.png` and `alias/../dist/result.png` exited 0. One file existed; the lockfile recorded two paths and two hashes. | One destination identity for planning and writing; reject collisions before publishing either output. `src/cli/build.rs` (preflight key ~367, post-write check ~882). | SPEC-157 |
| P1 | `meta strip` changes how Adobe YCCK JPEGs render | The strip loop removes Adobe APP14, which carries the colour transform. On `tests/fixtures/lint/cmyk_16x16.jpg`, `crustyimg diff` scored the stripped file 41.94 SSIMULACRA2 against the original. | Preserve the Adobe marker; the regression compares decoded pixels as well as checking metadata removal. `src/metadata/mod.rs` (~166). | SPEC-156 |
| P2 | A successful build can generate a lockfile its own verifier refuses | A 22,000-image build exited 0 and wrote a 5,191,870-byte lockfile; `build --check` exited 2 because the reader caps lockfiles at 4 MiB. | One size contract for reader and writer; or report the limit before publishing an unusable lockfile. `src/cli/build.rs` (~654), `src/build/lock.rs` (~267). | SPEC-158 |
| P2 | A cache miss can hash one version of a source and encode another | By inspection: `build` reads the source to compute its key, then the encoder reopens the path. An edit or atomic replace between the reads stores new pixels under the old hash. | Decode the same byte snapshot that was hashed, including on the auto-optimize path. `src/cli/build.rs` (~415), `src/cli/common.rs` (~72). | SPEC-158 |
| P2 | The CI path filter classifies real build inputs as docs-only | The classifier returned `false` for the bundled web recipe, the bundled font, `clippy.toml` and `deny.toml`. | Add those paths and regression cases; align the Pages triggers for shared inputs. `scripts/ci-code-paths.sh` (~15), `.github/workflows/ci.yml` (~169). | SPEC-154 |

## Notes from the review's five perspectives

- **Rust engineer:** `encode_one` clones the decoded image only to keep the source format
  (`src/cli/common.rs` ~79). Capture the format before moving `img` into the pipeline. (SPEC-158)
- **Staff engineer:** one build flow for destination resolution, source snapshotting, encoding and
  publishing, so the sink, the cache and the lockfile consume the same resolved paths and byte
  identities. This is the common cause behind several findings. (SPEC-157 and SPEC-158)
- **Principal engineer:** DEC-061 accepts symlinked output directories as a compatibility tradeoff,
  and the reviewer reproduced an out-of-tree write through one. Resolve it deliberately alongside
  destination identity, and document what `--check` writes. (SPEC-157)
- **Engineering manager:** two tracks, sink and metadata fidelity first, then build destination
  safety and lockfile integrity. Fix the CI classifier early. Before tagging, run one manifest
  through a cold build, a warm build and `--check`, and verify the artifacts on every release
  channel. (STAGE-063's ordering and the release cycle)
- **Product manager:** the downstream photo-site job needs "every image was watermarked or the
  build failed". PROJ-011's recorded seven-image trial left one image unwatermarked, and a warm
  build suppresses that skip warning. Wants a strict mode, measured on cold and warm builds. (Not
  a STAGE-063 spec; the watermark items belong to PROJ-017 or PROJ-013.)

## Reviewer's verification

macOS arm64: format and Clippy passed. Native: 588 library tests passed, then `build_watch` stopped
with 4 passed and 10 timed out waiting for rebuilds. All other native integration targets passed
(687 tests). Full and slim Wasm suites passed (55 and 53). The watch timeouts are unexplained.
Linux, Windows, optional HEIC and live publishing were not run.
