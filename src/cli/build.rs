//! The `build` subcommand: declared multi-target builds, the content-addressed
//! cache, the committed lockfile, and the `--watch` rebuild loop (SPEC-063,
//! SPEC-064, SPEC-066, SPEC-067; DEC-057/DEC-058/DEC-059/DEC-060). Split out of
//! `cli/mod.rs` (SPEC-097) — no behavior change.

use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::error::ImageError;
use crate::operation::OperationRegistry;
use crate::recipe::Recipe;
use crate::sink::Overwrite;
use crate::source::{self, SourceError};

use super::common::{encode_one, load_recipe, write_encoded, BATCH_PROGRESS_TEMPLATE};
use super::{CliError, GlobalArgs};

// ── Build command (SPEC-063, DEC-057) ────────────────────────────────────────

/// A manifest target with everything resolved that could fail before a write:
/// its recipe (parsed, pipeline-probed), its inputs (sources resolved), and the
/// canonical hash of its recipe — computed once per target, not once per input,
/// since every input in a target shares it (SPEC-064).
struct PreparedTarget<'a> {
    target: &'a crate::build::Target,
    recipe: Recipe,
    recipe_hash: crate::build::cache::Hash,
    inputs: Vec<crate::source::Input>,
}

/// How one input's output came to exist, for the build summary (SPEC-064).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Built {
    /// Materialized from a cache entry; no decode, no encode.
    Cached,
    /// Decoded, run through the pipeline, encoded — and stored for next time.
    Rebuilt,
}

/// `"1 target"` / `"2 targets"` — the build summary counts several things and
/// "1 targets" reads like a bug in the tool.
fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// Size-guard, read, and parse the build manifest at `path`.
///
/// Mirrors [`load_recipe`]: the on-disk size is checked before reading, so an
/// oversized manifest is never loaded (DEC-036). A missing file — including the
/// discovered default `crustyimg.build.toml` — is `BuildManifestIo` (exit 3),
/// naming the path; malformed content is a typed `BuildError` (exit 2).
fn load_manifest(path: &str) -> Result<crate::build::BuildManifest, CliError> {
    let io_err = |source: std::io::Error| CliError::BuildManifestIo {
        path: path.to_owned(),
        source,
    };
    let meta = std::fs::metadata(path).map_err(io_err)?;
    if meta.len() > crate::build::BUILD_MANIFEST_MAX_BYTES as u64 {
        return Err(CliError::Build(crate::build::BuildError::TooLarge {
            size: meta.len() as usize,
            max: crate::build::BUILD_MANIFEST_MAX_BYTES,
        }));
    }
    let text = std::fs::read_to_string(path).map_err(io_err)?;
    Ok(crate::build::BuildManifest::from_toml(&text)?)
}

/// Resolve everything a target needs before any output is written: parse its
/// recipe, probe its pipeline (a bad op fails here, exit 1), and resolve its
/// sources (a missing path / empty glob fails here, exit 3/2).
///
/// Manifest paths are relative to the process working directory (DEC-057).
/// Stdin (`-`) is rejected at manifest validation, so an empty reader suffices.
fn prepare_target<'a>(
    target: &'a crate::build::Target,
    registry: &OperationRegistry,
) -> Result<PreparedTarget<'a>, CliError> {
    let recipe = load_recipe(&target.recipe)?;
    recipe.build_pipeline(registry)?;
    let recipe_hash = crate::build::cache::recipe_hash(&recipe)?;

    let mut inputs: Vec<crate::source::Input> = Vec::new();
    for pattern in target.source.as_slice() {
        inputs.extend(source::resolve(pattern, &mut std::io::empty())?);
    }
    if inputs.is_empty() {
        return Err(CliError::Source(SourceError::NotFound(
            target.source.as_slice().join(" "),
        )));
    }

    Ok(PreparedTarget {
        target,
        recipe,
        recipe_hash,
        inputs,
    })
}

/// The `{ext}` stand-in used when computing an output collision key (SPEC-065).
///
/// The real output extension is only knowable after a decode (which is why the
/// cache entry records it — DEC-058), and this check runs at prepare time. So
/// `{ext}` expands to itself: a fixed sentinel that no real extension can equal
/// (an extension never contains braces), and one that stays readable when it
/// reaches the error message. Two inputs whose remaining tokens agree therefore
/// collide **regardless** of output format — over-detection, the safe direction:
/// under-detection would silently miss `a/logo.png` + `b/logo.svg` → `logo.png`.
const EXT_SENTINEL: &str = "{ext}";

/// The output path one input of `target` would be written to, with `{ext}` left
/// as [`EXT_SENTINEL`] — the equality key for collision detection.
///
/// Mirrors `write_encoded`'s placement exactly (`out` dir + the expanded name
/// template) so the key stands for the real destination. The `out` dir is
/// normalized by path components — `dist`, `dist/`, and `./dist` are one
/// directory — but NOT canonicalized: it need not exist yet.
fn output_collision_key(target: &crate::build::Target, input: &crate::source::Input) -> String {
    let file_name =
        crate::sink::expand_template(target.template(), input.stem(), EXT_SENTINEL, input.path());
    let dir: PathBuf = Path::new(&target.out)
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .collect();
    dir.join(file_name).display().to_string()
}

/// Reject a build whose resolved targets would write two inputs to the same
/// output path (SPEC-065; DEC-057's injective source→output constraint).
///
/// Run **once, globally**, over every prepared target — so a collision between
/// two *different* targets writing the same `out`/name is caught too — and
/// before phase 2 opens the cache or writes anything, so a rejected build leaves
/// the destination untouched.
fn check_output_injective(prepared: &[PreparedTarget]) -> Result<(), CliError> {
    let mut entries: Vec<(String, String)> =
        Vec::with_capacity(prepared.iter().map(|p| p.inputs.len()).sum());
    for p in prepared {
        for input in &p.inputs {
            let label = match input {
                crate::source::Input::Path(path) => path.display().to_string(),
                crate::source::Input::Stdin { stem, .. } => stem.clone(),
            };
            entries.push((output_collision_key(p.target, input), label));
        }
    }

    match crate::build::find_output_collision(&entries) {
        Some(c) => Err(CliError::OutputCollision {
            output: c.output,
            first: c.first,
            second: c.second,
        }),
        None => Ok(()),
    }
}

/// The cache key for one input of a prepared target, plus the input's bytes.
///
/// Returns `Ok(None)` for a stdin input — the manifest rejects `-` sources
/// (DEC-057), so this is unreachable in practice, but a stdin input has no file
/// to hash and simply falls through to an uncached rebuild rather than a panic.
///
/// The file is read here (once) so its content can be hashed *before* any decode
/// — that pre-decode read is exactly what a cache hit exists to make sufficient.
fn cache_key_for(
    prepared: &PreparedTarget,
    input: &crate::source::Input,
    quality: Option<u8>,
) -> Result<Option<crate::build::cache::CacheKey>, CliError> {
    use crate::build::cache;

    let path = match input {
        crate::source::Input::Path(p) => p,
        crate::source::Input::Stdin { .. } => return Ok(None),
    };

    let bytes = std::fs::read(path).map_err(|e| CliError::Image(ImageError::Io(e)))?;
    let input_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    Ok(Some(cache::compute_key(
        crate::version(),
        &cache::feature_signature(),
        &prepared.recipe_hash,
        quality,
        &input_ext,
        &cache::hash_bytes(&bytes),
    )))
}

/// Render a path with `/` separators, whatever the host uses.
///
/// A lockfile is committed and read on every OS, so the paths inside it must not
/// carry a Windows `\`. Only the *lock's* strings are normalized; the bytes are
/// still written through the sink's own platform-native join.
fn to_slash(path: &Path) -> String {
    path.components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// The output path a lock entry names: `out` dir + the expanded name template,
/// with the **real** output extension (known only after the encode, or from the
/// cache entry on a hit).
///
/// The same composition `write_encoded` uses, so a lock entry names exactly the
/// file that was written. It is the post-decode twin of `output_collision_key`,
/// which must leave `{ext}` unexpanded because it runs before any decode.
fn lock_output_path(
    target: &crate::build::Target,
    input: &crate::source::Input,
    ext: &str,
) -> String {
    let file_name =
        crate::sink::expand_template(target.template(), input.stem(), ext, input.path());
    let dir = to_slash(Path::new(&target.out));
    if dir.is_empty() {
        file_name
    } else {
        format!("{dir}/{file_name}")
    }
}

/// The lock entry for one written output.
fn lock_record(
    prepared: &PreparedTarget,
    input: &crate::source::Input,
    key: &crate::build::cache::CacheKey,
    ext: &str,
    bytes: &[u8],
) -> Option<crate::build::lock::LockOutput> {
    // A build's inputs are always `Input::Path` (the manifest rejects `-`,
    // DEC-057), which is also why `cache_key_for` can hand us a key at all.
    let source = match input {
        crate::source::Input::Path(p) => to_slash(p),
        crate::source::Input::Stdin { .. } => return None,
    };
    Some(crate::build::lock::LockOutput {
        path: lock_output_path(prepared.target, input, ext),
        source,
        recipe: prepared.target.recipe.clone(),
        key: key.to_hex(),
        hash: crate::build::cache::hash_bytes(bytes).to_hex(),
        bytes: bytes.len() as u64,
    })
}

/// What every input of a build shares: the operation registry, the (optional)
/// cache, and the global flags the per-input worker reads. Built once in
/// `run_build` and borrowed across the rayon fan-out — all `Sync`.
struct BuildCtx<'a> {
    registry: &'a OperationRegistry,
    /// `None` under `--no-cache`: the store is neither read nor written.
    cache: Option<&'a crate::build::cache::Cache>,
    quality: Option<u8>,
    quiet: bool,
}

/// One built output: how it was produced, and what the lockfile records of it.
///
/// `record` is `None` only for a stdin input, which a build manifest cannot
/// declare (DEC-057) — the same unreachable branch `cache_key_for` guards.
struct BuildOutcome {
    built: Built,
    record: Option<crate::build::lock::LockOutput>,
}

/// Build one input: a cache hit materializes the stored bytes, a miss runs the
/// shipped worker and stores the result. Either way it returns the lock record
/// for the bytes it wrote.
///
/// The cache key is computed **unconditionally**, not just when the store is
/// open: every build writes or checks a lockfile, and the key is what the
/// lockfile pins (DEC-058/DEC-059). A `--no-cache` build therefore still reads
/// and hashes each source once — the cost of being lockable.
///
/// Cache failures never fail a build. A `lookup` that errors is treated as a
/// miss (it cannot error by construction, but the executor does not rely on
/// that); a `store` that errors costs the next run a rebuild and is warned about.
fn build_one(
    ctx: &BuildCtx,
    prepared: &PreparedTarget,
    input: &crate::source::Input,
) -> Result<BuildOutcome, CliError> {
    let out_dir = Path::new(&prepared.target.out);
    let template = prepared.target.template();
    // A build owns its declared outputs and overwrites them (DEC-057) — which is
    // also what lets a hit restore an output the user deleted.
    let overwrite = Overwrite::Allow;

    let key = cache_key_for(prepared, input, ctx.quality)?;

    if let (Some(cache), Some(key)) = (ctx.cache, key.as_ref()) {
        if let Ok(Some(hit)) = cache.lookup(key) {
            // The entry is self-describing: it carries the extension the output
            // was encoded as, so no decode is needed to write it to the right path.
            write_encoded(&hit.bytes, &hit.ext, input, out_dir, template, overwrite)?;
            return Ok(BuildOutcome {
                built: Built::Cached,
                record: lock_record(prepared, input, key, &hit.ext, &hit.bytes),
            });
        }
    }

    let (ext, bytes) = encode_one(&prepared.recipe, ctx.registry, input, ctx.quality)?;
    write_encoded(&bytes, ext, input, out_dir, template, overwrite)?;

    if let (Some(cache), Some(key)) = (ctx.cache, key.as_ref()) {
        if let Err(e) = cache.store(key, ext, &bytes) {
            if !ctx.quiet {
                eprintln!("warning: could not cache output: {e}");
            }
        }
    }

    Ok(BuildOutcome {
        built: Built::Rebuilt,
        record: key
            .as_ref()
            .and_then(|k| lock_record(prepared, input, k, ext, &bytes)),
    })
}

/// Read and parse the committed lockfile.
///
/// `Ok(None)` when the file does not exist — under `--check` that is drift, not
/// an error, and the caller says so with an actionable message. Every other I/O
/// failure is `LockIo` (exit 3); a malformed/oversize/over-version lockfile is a
/// typed `LockError` (exit 2). Size is checked on disk before the read, so an
/// oversized lockfile is never brought into memory (DEC-036).
fn load_lock(path: &str) -> Result<Option<crate::build::lock::BuildLock>, CliError> {
    use crate::build::lock::{BuildLock, LockError, LOCK_MAX_BYTES};

    let io_err = |source: std::io::Error| CliError::LockIo {
        path: path.to_owned(),
        source,
    };
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io_err(e)),
    };
    if meta.len() > LOCK_MAX_BYTES as u64 {
        return Err(CliError::Lock(LockError::TooLarge {
            size: meta.len().min(usize::MAX as u64) as usize,
            max: LOCK_MAX_BYTES,
        }));
    }
    let text = std::fs::read_to_string(path).map_err(io_err)?;
    Ok(Some(BuildLock::from_toml(&text)?))
}

/// Write the lockfile atomically: stage beside it, then `rename` into place.
///
/// A reader (a concurrent `--check`, a `git status`) therefore never sees a
/// half-written lockfile — the same temp→rename discipline the cache store uses.
fn write_lock(path: &str, contents: &str) -> Result<(), CliError> {
    let write_err = |source: std::io::Error| CliError::LockWrite {
        path: path.to_owned(),
        source,
    };
    let tmp = format!("{path}.tmp{}", std::process::id());
    std::fs::write(&tmp, contents).map_err(write_err)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(write_err(e));
    }
    Ok(())
}

/// The `build` path: run every `[[target]]` in a declared build manifest.
///
/// `run_apply` generalized from one (recipe, inputs, out) to N declared targets.
/// Two phases, deliberately:
///
/// 1. **Prepare every target** — parse each recipe, probe each pipeline, resolve
///    each source set, then check that the resolved targets map sources to outputs
///    **injectively** (SPEC-065). Any failure here aborts the build having written
///    nothing, so target #2's typo can't leave target #1's half-built outputs on disk.
/// 2. **Execute** — for each target, fan its inputs out over rayon into
///    [`build_one`], writing to that target's `out` dir under its name template.
///    The registry is built ONCE and shared (fn ptrs → `Sync`).
///
/// Each input goes through the **content-addressed cache** (SPEC-064): its key
/// is computed from the source bytes + extension, the canonical recipe, the
/// quality, and this binary's version + features; a hit materializes the stored
/// output and skips decode→pipeline→encode entirely, a miss runs the worker and
/// stores the result. `--no-cache` bypasses the store in both directions.
///
/// Every build ends at the **lockfile** (SPEC-066): by default it writes
/// `crustyimg.build.lock`, pinning each output's cache key and recording its
/// observed bytes + the env they were observed under. Under
/// `--check`/`--frozen`/`--locked` it instead diffs the build against the
/// committed lockfile and returns [`CliError::CheckFailed`] (exit 7) on drift,
/// without ever writing the lockfile. The committed lockfile is loaded in phase
/// 1, so a malformed one fails before any output is written.
///
/// A build **owns its declared outputs**, so it writes with [`Overwrite::Allow`]
/// and needs no `--yes` (the deliberate difference from `apply`; the sink still
/// blocks name-template escapes, so it only ever writes inside `out`). Per-output
/// decode/encode failures are collected and reported → exit 6 (DEC-015), not a
/// hard abort. `--jobs` bounds the pool and `--quiet` hides progress + summary,
/// exactly as in `run_apply`.
pub(super) fn run_build(file: Option<&str>, global: &GlobalArgs) -> Result<(), CliError> {
    use crate::build::lock::{self, DEFAULT_LOCK_FILE};

    let path = file.unwrap_or(crate::build::DEFAULT_MANIFEST_FILE);
    let manifest = load_manifest(path)?;

    let registry = OperationRegistry::with_builtins();

    // ── Phase 1: prepare all targets (nothing is written yet) ────────────────
    let mut prepared: Vec<PreparedTarget> = Vec::with_capacity(manifest.target.len());
    for (i, target) in manifest.target.iter().enumerate() {
        match prepare_target(target, &registry) {
            Ok(p) => prepared.push(p),
            Err(e) => {
                // Name the offending target; `run()` prints the error itself.
                eprintln!(
                    "error: target #{i} (recipe {}, out {})",
                    target.recipe, target.out
                );
                return Err(e);
            }
        }
    }

    // A build must map sources to outputs injectively, or two inputs race one
    // destination (DEC-057). Checked here — after every target is resolved, before
    // the store opens and before any write — so a collision costs nothing on disk.
    check_output_injective(&prepared)?;

    // Load the committed lockfile before anything is written, so a malformed one
    // (or, under `--check`, a missing one) costs nothing on disk. `None` here is
    // "no lockfile"; only `--check` cares.
    let committed = if global.check {
        let Some(committed) = load_lock(DEFAULT_LOCK_FILE)? else {
            eprintln!(
                "error: no lockfile \"{DEFAULT_LOCK_FILE}\"; run `crustyimg build` to create one"
            );
            return Err(CliError::CheckFailed);
        };
        Some(committed)
    } else {
        None
    };

    let total: usize = prepared.iter().map(|p| p.inputs.len()).sum();

    // Open the store once, before anything is written. This is the only cache
    // error that can reach the boundary (exit 5); `--no-cache` skips it entirely,
    // so a bypassed build creates no `.crustyimg/` directory at all.
    let cache = if global.no_cache {
        None
    } else {
        Some(crate::build::cache::Cache::open(
            crate::build::cache::DEFAULT_CACHE_DIR,
        )?)
    };

    // ── Phase 2: execute ─────────────────────────────────────────────────────
    let bar = if global.quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(total as u64);
        let style = ProgressStyle::with_template(BATCH_PROGRESS_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_bar());
        pb.set_style(style);
        pb
    };

    let ctx = BuildCtx {
        registry: &registry,
        cache: cache.as_ref(),
        quality: global.quality,
        quiet: global.quiet,
    };

    let execute = || {
        let mut outcomes: Vec<Result<BuildOutcome, CliError>> = Vec::with_capacity(total);
        for p in &prepared {
            let results: Vec<Result<BuildOutcome, CliError>> = p
                .inputs
                .par_iter()
                .map(|input| {
                    let r = build_one(&ctx, p, input);
                    if let Err(ref e) = r {
                        let label = match input {
                            crate::source::Input::Path(path) => path.display().to_string(),
                            crate::source::Input::Stdin { stem, .. } => stem.clone(),
                        };
                        eprintln!("error: {label}: {e}");
                    }
                    bar.inc(1);
                    r
                })
                .collect();
            outcomes.extend(results);
        }
        outcomes
    };

    let outcomes = if let Some(n) = global.jobs {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .map_err(|e| CliError::Usage(format!("could not build thread pool: {e}")))?;
        pool.install(execute)
    } else {
        execute()
    };

    bar.finish_and_clear();

    let failed = outcomes.iter().filter(|r| r.is_err()).count();
    let cached = outcomes
        .iter()
        .filter(|r| matches!(r, Ok(o) if o.built == Built::Cached))
        .count();
    let rebuilt = outcomes
        .iter()
        .filter(|r| matches!(r, Ok(o) if o.built == Built::Rebuilt))
        .count();

    // ── Summary (stderr; stdout stays clean for pipes) ───────────────────────
    if !global.quiet {
        for (i, p) in prepared.iter().enumerate() {
            eprintln!(
                "target #{i}: {} × {} → {}",
                plural(p.inputs.len(), "input"),
                p.target.recipe,
                p.target.out
            );
        }
        // A no-change re-run reads "(N cached, 0 rebuilt)" — the zero-work signal.
        let summary = format!(
            "built {}, {} ({cached} cached, {rebuilt} rebuilt)",
            plural(prepared.len(), "target"),
            plural(cached + rebuilt, "output")
        );
        if failed > 0 {
            eprintln!("{summary}, {failed} failed");
        } else {
            eprintln!("{summary}");
        }
    }

    // A partial build has nothing coherent to lock or check: some outputs are
    // missing, so a lock written now would pin a half-built tree and a `--check`
    // would report the failures twice. Exit 6 (DEC-015), lockfile untouched.
    if failed > 0 {
        return Err(CliError::PartialBatch { failed, total });
    }

    // ── Phase 3: the lockfile (SPEC-066, DEC-059) ────────────────────────────
    let records: Vec<crate::build::lock::LockOutput> = outcomes
        .into_iter()
        .filter_map(|o| o.ok().and_then(|o| o.record))
        .collect();

    // The lockfile's second line of defense against a non-injective build. The
    // prepare-phase check (SPEC-065) cannot expand `{ext}` before a decode, so a
    // target naming a *literal* extension (`{stem}.png`) can still collide with
    // one naming `{stem}.{ext}` — undetectably, pre-decode. Here the real
    // extensions are known, so two outputs claiming one path are rejected rather
    // than pinned. The outputs are already written (the collision raced), but the
    // build fails loudly with the same typed error and exit 2, and no lockfile
    // records the ambiguity.
    let entries: Vec<(String, String)> = records
        .iter()
        .map(|r| (r.path.clone(), r.source.clone()))
        .collect();
    if let Some(c) = crate::build::find_output_collision(&entries) {
        return Err(CliError::OutputCollision {
            output: c.output,
            first: c.first,
            second: c.second,
        });
    }

    let current = lock::BuildLock::new(lock::current_env(), records);

    match committed {
        // `--check` / `--frozen` / `--locked`: assert, never write.
        Some(committed) => {
            let d = lock::diff(&committed, &current, global.strict);
            for change in &d.changes {
                eprintln!("{change}");
            }
            if d.drifted {
                return Err(CliError::CheckFailed);
            }
            if !global.quiet && d.is_clean() {
                eprintln!("lockfile {DEFAULT_LOCK_FILE} is up to date");
            }
        }
        // The default: refresh the lockfile from what this build produced —
        // UNLESS this is a `--watch` cycle. A dev loop is pre-commit iteration, not
        // a commit; silently rewriting the committed `crustyimg.build.lock` in the
        // working tree mid-edit is exactly what a watching author does not want
        // (SPEC-067, DEC-060). `--watch` is incompatible with `--check`, so under
        // watch `committed` is always `None` and this is the only branch reached.
        None if global.watch => {}
        None => write_lock(DEFAULT_LOCK_FILE, &current.to_toml()?)?,
    }

    Ok(())
}

// ── Build --watch (SPEC-067, DEC-060) ────────────────────────────────────────

/// The debounce quiet-window. An editor's save burst (an atomic temp-write, a
/// rename, and metadata touches) fires many events within a few milliseconds;
/// 200 ms coalesces them into one rebuild while staying responsive (DEC-060).
#[cfg(feature = "watch")]
const WATCH_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(200);

/// `crustyimg build --watch`: an initial build, then a debounced rebuild loop over
/// the shipped [`run_build`] on any change to a source, recipe, or the manifest
/// (SPEC-067). The STAGE-021 cache makes each full re-run incremental (DEC-058), so
/// there is no dependency graph — just a resilient loop.
///
/// This entry handles the two decisions that must hold in *every* build (including
/// the lean `--no-default-features` one, where the watcher is not compiled in): the
/// `--watch` + verify-mode usage error, and the "built without watch support"
/// message. The real loop is [`watch_impl`], behind the `watch` feature.
pub(super) fn run_build_watching(file: Option<&str>, global: &GlobalArgs) -> Result<(), CliError> {
    // A watch loop *refreshes* outputs; `--check`/`--frozen`/`--locked` is a
    // one-shot lockfile assertion. The two are incompatible — reject before any
    // watching begins, and before the lean-build check, so the message is the same
    // on every build (`--frozen`/`--locked` are clap aliases of `--check`, so this
    // one field covers all three). Exit 2 (usage), never a leaked check-mode code.
    if global.check {
        return Err(CliError::Usage(
            "--watch cannot be combined with --check/--frozen/--locked: a watch loop \
             refreshes outputs, while --check is a one-shot lockfile assertion"
                .to_owned(),
        ));
    }

    #[cfg(not(feature = "watch"))]
    {
        let _ = file;
        Err(CliError::Usage(
            "this binary was built --no-default-features, so `--watch` is not compiled \
             in; rebuild with the `watch` feature (e.g. `cargo build --features watch`)"
                .to_owned(),
        ))
    }

    #[cfg(feature = "watch")]
    {
        watch_impl(file, global)
    }
}

/// Create a fresh OS watcher whose events are forwarded (path by path) over `tx`.
///
/// The `tx` is cloned so the caller can re-create the watcher on a manifest change
/// without losing the receiver. A watcher-init failure is a typed [`WatchError`].
#[cfg(feature = "watch")]
fn make_watcher(
    tx: &std::sync::mpsc::Sender<PathBuf>,
) -> Result<notify::RecommendedWatcher, CliError> {
    use crate::build::watch::WatchError;

    let tx = tx.clone();
    let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        // A watch error mid-run (e.g. a transient backend hiccup) is dropped rather
        // than crashing the loop; a real edit fires another event. Each event's
        // paths are forwarded for the debounce + exclusion filter to judge.
        if let Ok(event) = res {
            for p in event.paths {
                let _ = tx.send(p);
            }
        }
    })
    .map_err(|e| CliError::from(WatchError::Watcher(e.to_string())))?;
    Ok(watcher)
}

/// Register every root in `set` with `watcher`: source roots recursively, the
/// manifest/recipe dirs non-recursively (so the sibling output/cache tree is not
/// recursively watched — see `WatchSet`). A root that cannot be watched (e.g. it
/// does not exist yet) is a warning, not a failure — the others still watch, and a
/// later manifest edit can bring it into range.
#[cfg(feature = "watch")]
fn register_roots(watcher: &mut notify::RecommendedWatcher, set: &crate::build::watch::WatchSet) {
    use notify::{RecursiveMode, Watcher};
    for (roots, mode) in [
        (&set.recursive, RecursiveMode::Recursive),
        (&set.shallow, RecursiveMode::NonRecursive),
    ] {
        for root in roots {
            if let Err(e) = watcher.watch(root, mode) {
                eprintln!("warning: cannot watch {}: {e}", root.display());
            }
        }
    }
}

/// The real `--watch` loop (behind the `watch` feature): set up the watcher, run an
/// initial build, then rebuild on each debounced, non-self-triggered change.
#[cfg(feature = "watch")]
fn watch_impl(file: Option<&str>, global: &GlobalArgs) -> Result<(), CliError> {
    use crate::build::watch;

    let path = file.unwrap_or(crate::build::DEFAULT_MANIFEST_FILE);

    // A missing/unparseable manifest at start is a HARD exit — there is nothing to
    // derive a watch set from. (A *build* failure with a valid manifest is NOT a
    // hard exit; see the initial build below.)
    let manifest = load_manifest(path)?;
    let mut set = watch::watch_roots(&manifest, Path::new(path));

    let (tx, rx) = std::sync::mpsc::channel::<PathBuf>();
    let mut watcher = make_watcher(&tx)?;
    register_roots(&mut watcher, &set);

    // Initial build. A build FAILURE (bad recipe, undecodable input) with a valid
    // manifest still enters the loop — print and wait so the user can fix it and
    // watch it recover (the first cycle is not special). `run_build` suppresses the
    // lockfile write because `global.watch` is set.
    if let Err(e) = run_build(file, global) {
        eprintln!("error: {e}");
    }
    if !global.quiet {
        eprintln!("watching for changes (Ctrl-C to stop)…");
    }

    // The debounced rebuild loop. `debounce` returns `None` when the watcher hangs
    // up (its thread died), which ends the loop. Ctrl-C exits via the default SIGINT
    // (exit 130) — no `ctrlc` dependency (DEC-060).
    let manifest_prefix = [PathBuf::from(path)];
    while let Some(batch) = watch::debounce(&rx, WATCH_DEBOUNCE) {
        // Drop a batch that is entirely the build's OWN writes, so a build never
        // wakes itself (the correctness crux; see `watch::is_excluded`).
        if batch.iter().all(|p| watch::is_excluded(p, &set.excluded)) {
            continue;
        }

        if let Err(e) = run_build(file, global) {
            eprintln!("error: {e}");
        }

        // A manifest edit can change the target set → re-derive the watch set and
        // re-register the watcher (acceptable per DEC-060). `run_build` already
        // re-reads the manifest each cycle, so the BUILD is always current; this
        // only refreshes WHICH directories are watched. Reusing `is_excluded` with
        // the manifest as the sole prefix answers "did this batch touch the
        // manifest?" with the same path-normalization the exclusion uses.
        let manifest_changed = batch
            .iter()
            .any(|p| watch::is_excluded(p, &manifest_prefix));
        if manifest_changed {
            if let Ok(m) = load_manifest(path) {
                set = watch::watch_roots(&m, Path::new(path));
                watcher = make_watcher(&tx)?;
                register_roots(&mut watcher, &set);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_agrees_with_its_count() {
        assert_eq!(plural(0, "target"), "0 targets");
        assert_eq!(plural(1, "target"), "1 target");
        assert_eq!(plural(2, "output"), "2 outputs");
    }

    /// A `Target` writing `out` under `name` (or the default template).
    fn target(out: &str, name: Option<&str>) -> crate::build::Target {
        crate::build::Target {
            source: crate::build::SourceSpec::One("src/*.png".into()),
            recipe: "r.toml".into(),
            out: out.into(),
            name: name.map(str::to_owned),
        }
    }

    fn key(out: &str, name: Option<&str>, path: &str) -> String {
        output_collision_key(
            &target(out, name),
            &crate::source::Input::Path(PathBuf::from(path)),
        )
    }

    #[test]
    fn collision_key_ignores_output_ext_but_not_the_rest() {
        // Same stem, different input format → the SAME key: the output ext is
        // unknowable pre-decode, so this must over-detect (both may → logo.png).
        assert_eq!(
            key("dist", None, "a/logo.png"),
            key("dist", None, "b/logo.svg")
        );

        // {parent} disambiguates; distinct stems and distinct out dirs do too.
        let tpl = Some("{parent}_{stem}.{ext}");
        assert_ne!(
            key("dist", tpl, "a/logo.png"),
            key("dist", tpl, "b/logo.png")
        );
        assert_ne!(
            key("dist", None, "a/one.png"),
            key("dist", None, "a/two.png")
        );
        assert_ne!(
            key("dist", None, "a/logo.png"),
            key("other", None, "a/logo.png")
        );
    }

    #[test]
    fn collision_key_normalizes_the_out_dir() {
        // `dist`, `dist/`, and `./dist` are one directory — two targets naming it
        // differently still collide. (Not canonicalized: it may not exist yet.)
        let expected = Path::new("dist").join("logo.{ext}").display().to_string();
        assert_eq!(key("dist", None, "a/logo.png"), expected);
        assert_eq!(key("dist/", None, "a/logo.png"), expected);
        assert_eq!(key("./dist", None, "a/logo.png"), expected);
    }

    // ── the lockfile's output path (SPEC-066) ────────────────────────────────

    #[test]
    fn lock_output_path_expands_the_real_ext_and_slashes() {
        let input = crate::source::Input::Path(PathBuf::from("src/a.png"));

        // The real output extension is substituted — the post-decode twin of the
        // collision key, which must leave `{ext}` as a sentinel.
        assert_eq!(
            lock_output_path(&target("dist", None), &input, "webp"),
            "dist/a.webp"
        );
        assert_eq!(
            lock_output_path(&target("dist/img", Some("{stem}_web.{ext}")), &input, "jpg"),
            "dist/img/a_web.jpg"
        );

        // `./dist` normalizes; a bare `.` out dir leaves just the file name. The
        // separator is always `/`, so a lockfile committed on one OS reads on another.
        assert_eq!(
            lock_output_path(&target("./dist", None), &input, "png"),
            "dist/a.png"
        );
        assert_eq!(lock_output_path(&target(".", None), &input, "png"), "a.png");
    }

    #[test]
    fn to_slash_normalizes_separators_and_drops_cur_dir() {
        assert_eq!(to_slash(&PathBuf::from("src").join("a.png")), "src/a.png");
        assert_eq!(to_slash(Path::new("./src/a.png")), "src/a.png");
        assert_eq!(to_slash(Path::new("a.png")), "a.png");
    }

    #[test]
    fn lock_record_carries_key_hash_and_size() {
        use crate::build::cache;

        let recipe = Recipe::from_toml("version = \"1\"\n").expect("empty recipe parses");
        let t = target("dist", None);
        let prepared = PreparedTarget {
            target: &t,
            recipe_hash: cache::recipe_hash(&recipe).expect("recipe hashes"),
            recipe,
            inputs: Vec::new(),
        };
        let input = crate::source::Input::Path(PathBuf::from("src/a.png"));
        let key = cache::compute_key(
            "0.0.0",
            "",
            &prepared.recipe_hash,
            None,
            "png",
            &cache::hash_bytes(b"in"),
        );

        let rec = lock_record(&prepared, &input, &key, "png", b"out-bytes")
            .expect("a path input always records");
        assert_eq!(rec.path, "dist/a.png");
        assert_eq!(rec.source, "src/a.png");
        assert_eq!(rec.recipe, "r.toml");
        assert_eq!(rec.key, key.to_hex());
        assert_eq!(rec.hash, cache::hash_bytes(b"out-bytes").to_hex());
        assert_eq!(rec.bytes, 9);

        // A stdin input has no lock record — a manifest cannot declare one (DEC-057).
        let stdin = crate::source::Input::Stdin {
            bytes: Vec::new(),
            stem: "x".into(),
        };
        assert!(lock_record(&prepared, &stdin, &key, "png", b"").is_none());
    }
}
