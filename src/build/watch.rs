//! Pure logic for `crustyimg build --watch` (SPEC-067, DEC-060).
//!
//! `--watch` re-runs the whole build whenever a source, recipe, or the manifest
//! changes, and lets the STAGE-021 cache prune the re-run to the affected outputs
//! (DEC-058) — so there is **no** change→target dependency graph here. This module
//! is the *pure, unit-tested* half of that feature: deriving what to watch, the
//! event filter that stops a build from waking itself, and the debounce that turns
//! an editor's write burst into one rebuild. The blocking `notify` wiring and the
//! rebuild loop live in `crate::cli` (`run_build_watching`); everything here is
//! deterministic and filesystem-light so it can be tested without a real watcher.
//!
//! ## The self-trigger crux (the SPEC-066 lesson, repeated)
//!
//! A watcher reports **absolute, OS-canonical** event paths; the manifest-derived
//! excluded set is **manifest-relative** (`dist/`, `.crustyimg`, the lockfile). A
//! raw string prefix check compares an absolute path against a relative one, never
//! matches, and the build's own writes to `out`/`.crustyimg` wake it again — an
//! infinite rebuild loop. [`is_excluded`] normalizes *both* sides (absolutize a
//! relative path against the CWD, then clean it lexically) and compares by path
//! **components** (so `dist` never matches `distortion`). This is the one bug that
//! must not ship, so its test is written first.
//!
//! This module is compiled **unconditionally** (it pulls in no `notify` types), so
//! its unit tests run even in the lean `--no-default-features` build.

use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use thiserror::Error;

use super::cache::DEFAULT_CACHE_DIR;
use super::lock::DEFAULT_LOCK_FILE;
use super::BuildManifest;

// ─── Types ────────────────────────────────────────────────────────────────────

/// What a `build --watch` session watches and what it must ignore.
///
/// Two tiers of roots, because the manifest/recipe live *beside* the build's own
/// output and cache trees at the project root:
///
/// - `recursive` — source-root directories, watched **recursively** (a source tree
///   nests: `assets/**/*.png`). These contain only inputs, never the build's output.
/// - `shallow` — the manifest's and recipes' directories, watched
///   **non-recursively**. We only care about *those files* changing; a recursive
///   watch here would also cover the sibling `.crustyimg/` cache tree, whose
///   high-churn writes on a fresh build can overflow Linux inotify and degrade
///   detection of the source watches. A shallow watch sees the manifest/recipe
///   edits (and top-level dir creation, which is excluded) but not the deep
///   cache/output churn.
///
/// `excluded` are the prefixes whose events are dropped so the build never
/// self-triggers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchSet {
    /// Source-root directories, watched recursively.
    pub recursive: Vec<PathBuf>,
    /// The manifest's and recipes' directories, watched non-recursively.
    pub shallow: Vec<PathBuf>,
    /// Prefixes whose events are ignored: each target's `out` dir, the `.crustyimg`
    /// metadata dir, and the lockfile. A build's own writes land here.
    pub excluded: Vec<PathBuf>,
}

/// A `build --watch` setup failure (SPEC-067).
///
/// The pure logic in this module never fails; these are raised by the `cli`
/// wiring when the OS watcher can't be created or a root can't be registered, and
/// carried as strings so this module stays free of any `notify` type.
#[derive(Debug, Error)]
pub enum WatchError {
    /// The OS filesystem watcher could not be created.
    #[error("could not initialize the filesystem watcher: {0}")]
    Watcher(String),

    /// A watch root could not be registered with the watcher.
    #[error("could not watch {path}: {reason}")]
    Watch {
        /// The root that could not be registered.
        path: String,
        /// The underlying reason from the watcher.
        reason: String,
    },
}

// ─── watch_roots ────────────────────────────────────────────────────────────────

/// Derive the [`WatchSet`] for a manifest: what to watch, and what to ignore.
///
/// Purely lexical and filesystem-free, so it is deterministic in a unit test. Every
/// root is a *directory* (never a bare file), because watching a directory catches
/// an editor's atomic save — a temp-write + rename — that a single-file watch would
/// miss when the original inode is replaced. Over-watching is the safe direction:
/// the cache turns a redundant rebuild into a no-op, whereas *under*-watching would
/// silently miss a real edit. Source roots are recursive; the manifest/recipe dirs
/// are shallow (see [`WatchSet`] for why).
pub fn watch_roots(manifest: &BuildManifest, manifest_path: &Path) -> WatchSet {
    let mut recursive: Vec<PathBuf> = Vec::new();
    let mut shallow: Vec<PathBuf> = Vec::new();
    let mut excluded: Vec<PathBuf> = Vec::new();

    // The manifest's own directory — editing the manifest is a rebuild trigger, and
    // `run_build` re-reads it each cycle. Shallow: we only care about this one file,
    // not the output/cache tree that sits beside it.
    push_unique(&mut shallow, containing_dir(manifest_path));

    for target in &manifest.target {
        for src in target.source.as_slice() {
            push_unique(&mut recursive, source_root(src));
        }
        push_unique(&mut shallow, containing_dir(Path::new(&target.recipe)));
        // A target's outputs must not wake the watcher.
        push_unique(&mut excluded, lexical_clean(Path::new(&target.out)));
    }

    // A directory that is already a recursive root needs no shallow watch too (a
    // recursive watch is a superset). This also avoids registering the same inode
    // twice, which on Linux inotify can share/clobber a watch descriptor.
    shallow.retain(|s| !recursive.contains(s));

    // The build's own metadata dir (cache entries + temp files) and the lockfile.
    push_unique(&mut excluded, cache_exclusion());
    push_unique(&mut excluded, lexical_clean(Path::new(DEFAULT_LOCK_FILE)));

    WatchSet {
        recursive,
        shallow,
        excluded,
    }
}

// ─── is_excluded (the correctness crux) ─────────────────────────────────────────

/// Whether a watcher event `path` falls under any `excluded` prefix — i.e. it is
/// the build's own write and must not trigger a rebuild.
///
/// **Normalizes both sides** so an absolute/canonical event path from the watcher
/// still matches a manifest-relative excluded entry: a relative path is absolutized
/// against the current working directory, then both are cleaned lexically, then
/// compared by path **components** (`Path::starts_with`), so `dist` matches
/// `dist/a.png` but never `distortion/a.png`. Without this the build self-triggers
/// forever (see the module docs).
pub fn is_excluded(path: &Path, excluded: &[PathBuf]) -> bool {
    let event = normalize_abs(path);
    excluded
        .iter()
        .any(|ex| event.starts_with(normalize_abs(ex)))
}

// ─── debounce ─────────────────────────────────────────────────────────────────

/// Coalesce a burst of events into a single rebuild signal.
///
/// Blocks until the first event, then keeps draining the channel until a quiet
/// `window` elapses with no new event, returning the whole batch. An editor's
/// save fires many events (the atomic temp-write + rename, plus metadata touches)
/// within milliseconds; this collapses them into one rebuild. Returns `None` when
/// the sender has hung up (the watcher thread died) so the caller's loop can stop.
///
/// Written over an injected `Receiver` + `window` so it is testable with a
/// synthetic channel: pre-queue a burst, assert one batch; send one more after the
/// window, assert a second batch — no wall-clock flakiness beyond the short window.
pub fn debounce(rx: &Receiver<PathBuf>, window: Duration) -> Option<Vec<PathBuf>> {
    // Block for the first event; `Err` means the sender is gone → stop the loop.
    let first = rx.recv().ok()?;
    let mut batch = vec![first];
    // Drain everything that arrives within `window` of the previous event.
    while let Ok(next) = rx.recv_timeout(window) {
        batch.push(next);
    }
    Some(batch)
}

// ─── Private helpers (purely lexical) ────────────────────────────────────────────

/// The directory to watch for a source argument (glob / dir / path).
///
/// - a glob (`assets/**/*.png`, `a/*.png`, `*.png`) → the directory of its literal
///   prefix (`assets`, `a`, `.`);
/// - a trailing-separator directory spelling (`photos/`) → that directory;
/// - a bare path (`logo.png`, or a slash-less directory name) → its parent, so an
///   atomic save inside that directory is still observed.
fn source_root(src: &str) -> PathBuf {
    let literal = match src.find(['*', '?', '[']) {
        Some(pos) => &src[..pos],
        None => src,
    };
    let had_glob = literal.len() < src.len();
    let ends_with_sep = literal.ends_with('/') || literal.ends_with('\\');

    if ends_with_sep {
        // `assets/` or `assets/**/...` → watch the directory `assets`.
        let dir = literal.trim_end_matches(['/', '\\']);
        if dir.is_empty() {
            PathBuf::from(".")
        } else {
            lexical_clean(Path::new(dir))
        }
    } else if had_glob {
        // `a/b*.png` → literal `a/b`, watch its parent dir `a`; `*.png` → `.`.
        containing_dir(Path::new(literal))
    } else {
        // A bare path or slash-less directory name → watch its parent.
        containing_dir(Path::new(literal))
    }
}

/// The directory that contains `p` (its parent), or `.` when `p` has no parent.
fn containing_dir(p: &Path) -> PathBuf {
    match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => lexical_clean(parent),
        _ => PathBuf::from("."),
    }
}

/// The exclusion prefix for the build's metadata directory: the first component of
/// [`DEFAULT_CACHE_DIR`] (`.crustyimg/cache` → `.crustyimg`), so *everything* the
/// build writes under it — cache entries and their temp files — is filtered.
fn cache_exclusion() -> PathBuf {
    match Path::new(DEFAULT_CACHE_DIR).components().next() {
        Some(first) => PathBuf::from(first.as_os_str()),
        None => lexical_clean(Path::new(DEFAULT_CACHE_DIR)),
    }
}

/// Absolutize `p` against the CWD if it is relative, then clean it lexically.
///
/// Deliberately does **not** canonicalize: a canonicalize would fail on a path the
/// build just deleted, and would resolve symlinks the watcher may not have. The
/// CWD from `current_dir()` is already symlink-resolved on Unix, so a lexical
/// absolutize aligns with the watcher's absolute event paths.
fn normalize_abs(p: &Path) -> PathBuf {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(p),
            Err(_) => p.to_path_buf(),
        }
    };
    lexical_clean(&abs)
}

/// Remove `.` components and resolve `..` lexically (never touching the disk), so
/// two spellings of the same path compare equal by component.
fn lexical_clean(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                // Pop a preceding normal component; otherwise keep `..` (we can't
                // resolve above a relative root without touching the filesystem).
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                } else {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

/// Push `p` onto `v` only if not already present (order-preserving dedup).
fn push_unique(v: &mut Vec<PathBuf>, p: PathBuf) {
    if !v.contains(&p) {
        v.push(p);
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    // ── is_excluded: the load-bearing correctness detail (written FIRST) ─────────

    #[test]
    fn is_excluded_matches_across_absolute_and_relative_paths() {
        let cwd = std::env::current_dir().expect("a cwd");

        // The excluded set holds a manifest-RELATIVE dir, while the event arrives
        // as an ABSOLUTE path under the cwd — exactly how `notify` reports it.
        let excluded = vec![PathBuf::from("dist")];
        let abs_event = cwd.join("dist").join("a.png");
        assert!(
            is_excluded(&abs_event, &excluded),
            "an absolute event under a relative excluded dir must be excluded — \
             else the build self-triggers forever"
        );

        // And the mirror: an absolute excluded entry vs a relative event.
        let excluded_abs = vec![cwd.join("dist")];
        assert!(is_excluded(Path::new("dist/a.png"), &excluded_abs));

        // A source event (under the cwd, but not under an excluded prefix) is kept.
        let source_event = cwd.join("assets").join("a.png");
        assert!(!is_excluded(&source_event, &excluded));
    }

    #[test]
    fn is_excluded_drops_output_and_cache_events_keeps_source() {
        let excluded = vec![
            PathBuf::from("dist/img"),
            PathBuf::from(".crustyimg"),
            PathBuf::from("crustyimg.build.lock"),
        ];

        // Output, cache, and lockfile events are the build's own writes → dropped.
        assert!(is_excluded(Path::new("dist/img/logo.png"), &excluded));
        assert!(is_excluded(
            Path::new(".crustyimg/cache/ab/cd.bin"),
            &excluded
        ));
        assert!(is_excluded(Path::new("crustyimg.build.lock"), &excluded));

        // A source edit is kept (it must trigger a rebuild).
        assert!(!is_excluded(Path::new("assets/logo.png"), &excluded));

        // Component match, NOT string prefix: `dist/img` must not swallow a sibling
        // that merely shares a textual prefix.
        assert!(!is_excluded(Path::new("dist/imgology/x.png"), &excluded));
    }

    // ── watch_roots ──────────────────────────────────────────────────────────────

    fn manifest_2() -> BuildManifest {
        BuildManifest::from_toml(
            r#"
version = 1

[[target]]
source = "assets/**/*.png"
recipe = "recipes/web.toml"
out = "dist/img"

[[target]]
source = ["photos/", "logo.png"]
recipe = "recipes/thumb.toml"
out = "dist/thumb"
"#,
        )
        .expect("a valid 2-target manifest")
    }

    #[test]
    fn watch_roots_covers_sources_recipes_manifest() {
        let m = manifest_2();
        let set = watch_roots(&m, Path::new("crustyimg.build.toml"));

        // Source-root directories are watched RECURSIVELY: the glob prefix, the
        // trailing-slash dir, and the bare `logo.png` source's parent (".").
        assert!(
            set.recursive.contains(&PathBuf::from("assets")),
            "{:?}",
            set.recursive
        );
        assert!(
            set.recursive.contains(&PathBuf::from("photos")),
            "{:?}",
            set.recursive
        );
        assert!(
            set.recursive.contains(&PathBuf::from(".")),
            "{:?}",
            set.recursive
        );
        // Each recipe's directory is watched SHALLOW (non-recursively).
        assert!(
            set.shallow.contains(&PathBuf::from("recipes")),
            "{:?}",
            set.shallow
        );
        // The manifest's own dir is "." — but "." is already a recursive root (the
        // `logo.png` source), so it is de-duplicated OUT of the shallow set (a
        // recursive watch is a superset). It must still be watched somewhere.
        assert!(
            !set.shallow.contains(&PathBuf::from(".")),
            "'.' is recursive, so it must not also be shallow: {:?}",
            set.shallow
        );
    }

    #[test]
    fn watch_roots_keeps_manifest_dir_shallow_when_no_source_shares_it() {
        // A manifest in its own dir, sources elsewhere: the manifest dir is a
        // SHALLOW root (so the sibling cache/output tree isn't recursively watched).
        let m = BuildManifest::from_toml(
            "version = 1\n\n[[target]]\nsource = \"src/*.png\"\nrecipe = \"r.toml\"\nout = \"dist\"\n",
        )
        .unwrap();
        let set = watch_roots(&m, Path::new("crustyimg.build.toml"));
        assert!(
            set.recursive.contains(&PathBuf::from("src")),
            "{:?}",
            set.recursive
        );
        // Manifest dir "." and recipe dir "." are shallow, and "." is NOT recursive
        // here (no source resolves to it), so the output/cache tree at "." is only
        // shallow-watched.
        assert!(
            set.shallow.contains(&PathBuf::from(".")),
            "{:?}",
            set.shallow
        );
        assert!(
            !set.recursive.contains(&PathBuf::from(".")),
            "{:?}",
            set.recursive
        );
    }

    #[test]
    fn watch_roots_excludes_outputs_cache_and_lock() {
        let m = manifest_2();
        let set = watch_roots(&m, Path::new("crustyimg.build.toml"));

        // Each target's output directory.
        assert!(
            set.excluded.contains(&PathBuf::from("dist/img")),
            "{:?}",
            set.excluded
        );
        assert!(
            set.excluded.contains(&PathBuf::from("dist/thumb")),
            "{:?}",
            set.excluded
        );
        // The whole `.crustyimg` metadata dir (not just `.crustyimg/cache`), so a
        // cache temp file at its root is excluded too.
        assert!(
            set.excluded.contains(&PathBuf::from(".crustyimg")),
            "{:?}",
            set.excluded
        );
        // The lockfile.
        assert!(
            set.excluded
                .contains(&PathBuf::from("crustyimg.build.lock")),
            "{:?}",
            set.excluded
        );
    }

    #[test]
    fn source_root_derivation_covers_glob_dir_and_file_spellings() {
        assert_eq!(source_root("assets/**/*.png"), PathBuf::from("assets"));
        assert_eq!(source_root("a/*.png"), PathBuf::from("a"));
        assert_eq!(source_root("*.png"), PathBuf::from("."));
        assert_eq!(source_root("a/b*.png"), PathBuf::from("a"));
        assert_eq!(source_root("photos/"), PathBuf::from("photos"));
        // A bare file → its parent (so an atomic save in that dir is seen).
        assert_eq!(source_root("logo.png"), PathBuf::from("."));
        assert_eq!(source_root("src/logo.png"), PathBuf::from("src"));
    }

    #[test]
    fn empty_manifest_still_watches_its_own_directory() {
        let m = BuildManifest::from_toml("version = 1\n").unwrap();
        let set = watch_roots(&m, Path::new("sub/crustyimg.build.toml"));
        // Nothing to build, but the manifest's dir is watched (shallow) so a later
        // edit that adds a target is seen.
        assert!(
            set.shallow.contains(&PathBuf::from("sub")),
            "{:?}",
            set.shallow
        );
        // The metadata dir + lockfile are always excluded.
        assert!(set.excluded.contains(&PathBuf::from(".crustyimg")));
        assert!(set
            .excluded
            .contains(&PathBuf::from("crustyimg.build.lock")));
    }

    // ── debounce ─────────────────────────────────────────────────────────────────

    #[test]
    fn debounce_coalesces_a_burst_into_one_signal() {
        let (tx, rx) = channel();
        let window = Duration::from_millis(30);

        // A burst of N events, all queued before the debounce runs.
        for i in 0..5 {
            tx.send(PathBuf::from(format!("f{i}.png"))).unwrap();
        }
        let batch = debounce(&rx, window).expect("a burst yields one signal");
        assert_eq!(batch.len(), 5, "the whole burst coalesces into one batch");

        // After the quiet window, a later event is a SECOND, separate signal.
        tx.send(PathBuf::from("later.png")).unwrap();
        let batch2 = debounce(&rx, window).expect("a later event is a second signal");
        assert_eq!(batch2, vec![PathBuf::from("later.png")]);

        // When the sender hangs up, debounce returns None so the loop can stop.
        drop(tx);
        assert!(debounce(&rx, window).is_none());
    }

    // ── WatchError ───────────────────────────────────────────────────────────────

    #[test]
    fn watch_error_messages_are_actionable() {
        let e = WatchError::Watcher("backend unavailable".into());
        assert!(e.to_string().contains("backend unavailable"));

        let e = WatchError::Watch {
            path: "assets".into(),
            reason: "no such directory".into(),
        };
        let msg = e.to_string();
        assert!(
            msg.contains("assets") && msg.contains("no such directory"),
            "{msg}"
        );
    }
}
