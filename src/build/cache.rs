//! The content-addressed build cache (SPEC-064, DEC-058).
//!
//! `crustyimg build` re-runs a declared build over an asset tree. Most of that
//! work is usually redundant: the sources, the recipe, and the binary are the
//! same as last time, so the output bytes will be too. This module is the two
//! halves that let a re-run skip it:
//!
//! 1. **The cache key** ([`compute_key`]) — a SHA-256 over *every* input that
//!    can change an output's bytes. Miss one and the build ships a stale
//!    artifact, silently; that is the correctness core of this module.
//! 2. **The store** ([`Cache`]) — a local, content-addressed, on-disk store
//!    (`.crustyimg/cache/`) of output bytes, keyed by (1).
//!
//! The executor (`run_build` in [`crate::cli`]) does the wiring: hash the input,
//! build the key, [`Cache::lookup`]; on a hit, materialize the entry's bytes to
//! the target path and skip the whole decode→pipeline→encode; on a miss, run the
//! shipped worker and [`Cache::store`] the result.
//!
//! Layering: library-only. Depends on `sha2` + [`crate::recipe`]; no `clap`, no
//! pixel decode, no knowledge of where an output is written.
//!
//! ## What is in the key, and what deliberately is not
//!
//! In, domain-separated so no two field values can concatenate into a third:
//! the [`CACHE_SCHEMA_VERSION`], the crustyimg version, the compiled-in
//! [`feature_signature`], the canonical [`recipe_hash`], the encode quality, the
//! input's lowercased **extension**, and the input's **content hash**.
//!
//! The extension is load-bearing, not decoration: crustyimg routes decode by
//! extension (RAW previews, SPEC-061/DEC-055), so the same bytes named `.nef`
//! and `.jpg` decode to different pixels.
//!
//! The **output format is NOT in the key.** It is a pure function of the input
//! bytes and extension — both already keyed — so a hit implies the same format,
//! and computing it up front would need exactly the decode a hit exists to skip.
//! The entry is **self-describing** instead: it records its own output
//! extension. That inversion is what lets a hit skip decode entirely.
//!
//! The output **destination** (`out` dir, `name` template) is not in the key
//! either. Identical inputs produce identical bytes wherever they land, so one
//! entry materializes to N paths. The key identifies the *bytes*; the manifest
//! decides where they go.
//!
//! ## Hardening (`untrusted-input-hardening`)
//!
//! The store lives under the user's tree, so a *read* treats it as untrusted:
//! entry paths are hex-only (no caller-controlled string reaches a path
//! component, so a key cannot traverse), reads are bounded by
//! [`CACHE_ENTRY_MAX_BYTES`], non-regular files (symlinks) are refused, and
//! every entry carries a hash of its own payload that [`Cache::lookup`]
//! re-verifies before returning it. Any anomaly — truncation, corruption, a bad
//! frame, a symlink, an oversize entry — is reported as a plain **miss**
//! (`Ok(None)`), never a panic and never unverified bytes. The executor answers
//! a miss by rebuilding, which is always correct.
//!
//! Writes are atomic: an entry is staged in `tmp/` under the cache root and
//! `rename`d into place, so a crashed or concurrent writer never leaves a
//! half-entry that a later run would trust. Content-addressing makes the rayon
//! fan-out safe by construction — the same key implies identical bytes, so
//! last-writer-wins is harmless.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest as _, Sha256};

use crate::recipe::{Recipe, RecipeError};

// ─── Constants ──────────────────────────────────────────────────────────────

/// The on-disk cache format version. Bump to invalidate every existing entry
/// when the *cache logic itself* changes (key composition, entry framing) in a
/// way that would otherwise let a stale entry look valid.
pub const CACHE_SCHEMA_VERSION: u32 = 1;

/// The cache root, relative to the process working directory — the same
/// convention the manifest's own paths follow (DEC-057).
pub const DEFAULT_CACHE_DIR: &str = ".crustyimg/cache";

/// The largest entry payload the cache will store or read back (256 MiB).
///
/// A `lookup` reads at most this much: an entry claiming to be larger is a miss
/// rather than an unbounded allocation. An output bigger than this is simply
/// never cached — the build still writes it, it just rebuilds it every run.
pub const CACHE_ENTRY_MAX_BYTES: usize = 256 * 1024 * 1024;

/// Frame magic. Carries the schema version so a v1 reader cannot even begin to
/// parse a future v2 entry that survived at the same path.
const ENTRY_MAGIC: &[u8; 12] = b"CRUSTYCACHE1";

/// The longest output extension an entry may record. `"tiff"` is 4; the cap
/// exists only to bound a malformed frame's claimed length.
const MAX_EXT_LEN: usize = 16;

/// The subdirectory under the cache root that atomic writes stage through.
const TMP_SUBDIR: &str = "tmp";

/// Lowercase hex digits, indexed by nibble.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Distinguishes concurrent staged writes within one process. Paired with the
/// pid it makes a staging file unique across every writer of a given key, so two
/// rayon tasks storing the same key never share a temp path.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

// ─── Errors ─────────────────────────────────────────────────────────────────

/// Errors from the cache store (DEC-007: typed here, exit-code mapped in `cli`).
///
/// Deliberately small. [`Cache::lookup`] never fails — every anomaly degrades to
/// a miss — and a [`Cache::store`] failure costs an optimization, not a build.
/// Only [`Cache::open`] produces an error the executor cannot continue past.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// The cache root (or its `tmp/` staging dir) could not be created.
    #[error("could not open build cache at {path}: {source}")]
    Open {
        path: String,
        source: std::io::Error,
    },

    /// Staging or committing a cache entry failed.
    #[error("could not write build-cache entry: {0}")]
    Io(std::io::Error),
}

// ─── Hash + key ─────────────────────────────────────────────────────────────

/// A 32-byte SHA-256 digest: of source bytes, of a canonical recipe, or of a
/// stored entry's payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hash([u8; 32]);

impl Hash {
    /// The lowercase hex form (64 characters). Entry paths are built from this,
    /// which is why it must contain nothing outside `[0-9a-f]`.
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            // Both indices are nibbles, so always in 0..16.
            s.push(HEX_DIGITS[(b >> 4) as usize] as char);
            s.push(HEX_DIGITS[(b & 0x0f) as usize] as char);
        }
        s
    }

    /// The raw digest bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// SHA-256 of `bytes` — a source file's contents, a canonical recipe, or an
/// entry's stored payload.
pub fn hash_bytes(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Hash(hasher.finalize().into())
}

/// A cache key: the digest of every output-affecting input (see [`compute_key`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheKey(Hash);

impl CacheKey {
    /// The lowercase-hex form of this key — its identity on disk.
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }
}

/// Hash the **canonical parsed** recipe: its ordered ops and their params, via
/// the round-tripping TOML serialization (DEC-005).
///
/// Hashing the parsed form rather than the recipe file's raw bytes is what makes
/// a comment or whitespace edit a cache *hit* while a changed param is a miss.
pub fn recipe_hash(recipe: &Recipe) -> Result<Hash, RecipeError> {
    Ok(hash_bytes(recipe.to_toml()?.as_bytes()))
}

/// The compiled-in cargo features that can change an encode's output bytes,
/// sorted and comma-joined (`""` for the default build).
///
/// Over-inclusion is a *safe* over-invalidation: a feature listed here that
/// turns out not to affect bytes only costs a rebuild. Omitting one that does
/// affect bytes serves a stale artifact — so when in doubt, list it.
pub fn feature_signature() -> String {
    let mut features: Vec<&str> = Vec::new();
    if cfg!(feature = "avif") {
        features.push("avif");
    }
    if cfg!(feature = "heic") {
        features.push("heic");
    }
    if cfg!(feature = "webp-lossy") {
        features.push("webp-lossy");
    }
    features.sort_unstable();
    features.join(",")
}

/// Absorb one field into the running digest, tagged and length-prefixed.
///
/// The tag and length are what make the composition injective: without them
/// `("ab", "c")` and `("a", "bc")` would hash identically, so one field's change
/// could be masked by another's.
fn absorb(hasher: &mut Sha256, tag: u8, bytes: &[u8]) {
    hasher.update([tag]);
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

/// Compose the cache key from every output-affecting input (DEC-058).
///
/// - `version` — the crustyimg version (`env!("CARGO_PKG_VERSION")`); a new
///   binary may encode differently, so it invalidates.
/// - `features` — [`feature_signature`] of this build.
/// - `recipe_hash` — [`recipe_hash`] of the resolved recipe.
/// - `quality` — the encode quality. `None` gets a distinct sentinel, so "no
///   `-q`" and `-q 0` are different keys.
/// - `input_ext` — the input's lowercased extension (decode is extension-routed).
/// - `input_hash` — [`hash_bytes`] of the input file's contents.
///
/// [`CACHE_SCHEMA_VERSION`] is folded in as well, so bumping the const
/// invalidates every entry without touching a caller.
pub fn compute_key(
    version: &str,
    features: &str,
    recipe_hash: &Hash,
    quality: Option<u8>,
    input_ext: &str,
    input_hash: &Hash,
) -> CacheKey {
    compute_key_with_schema(
        CACHE_SCHEMA_VERSION,
        version,
        features,
        recipe_hash,
        quality,
        input_ext,
        input_hash,
    )
}

/// [`compute_key`] with the schema version as a parameter.
///
/// Exists so the unit tests can prove the schema version is load-bearing: it is
/// a compiled-in const, so no test running one binary can observe two values of
/// it through the public [`compute_key`].
fn compute_key_with_schema(
    schema: u32,
    version: &str,
    features: &str,
    recipe_hash: &Hash,
    quality: Option<u8>,
    input_ext: &str,
    input_hash: &Hash,
) -> CacheKey {
    let mut hasher = Sha256::new();
    absorb(&mut hasher, 0, &schema.to_le_bytes());
    absorb(&mut hasher, 1, version.as_bytes());
    absorb(&mut hasher, 2, features.as_bytes());
    absorb(&mut hasher, 3, recipe_hash.as_bytes());
    // A one-byte sentinel tag for `None` that no `Some(q)` encoding can equal.
    match quality {
        Some(q) => absorb(&mut hasher, 4, &[q]),
        None => absorb(&mut hasher, 5, &[]),
    }
    absorb(&mut hasher, 6, input_ext.as_bytes());
    absorb(&mut hasher, 7, input_hash.as_bytes());
    CacheKey(Hash(hasher.finalize().into()))
}

// ─── Store ──────────────────────────────────────────────────────────────────

/// A cached output: the encoded bytes plus the extension they should be written
/// under. Self-describing, so a hit needs no decode to know its own format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedOutput {
    /// The output's lowercase extension (`"png"`, `"jpg"`, …).
    pub ext: String,
    /// The verified encoded output bytes.
    pub bytes: Vec<u8>,
}

/// A local, content-addressed store of build outputs, rooted at a directory
/// (`.crustyimg/cache/` by default). Local only — there is no network path here
/// and none is planned (the no-service guardrail, `docs/territory.md`).
#[derive(Debug)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Open (creating if absent) the cache root and its `tmp/` staging dir.
    ///
    /// The one cache failure the executor cannot shrug off: if the store cannot
    /// be created, `--no-cache` is the user's way past it.
    pub fn open(root: impl AsRef<Path>) -> Result<Cache, CacheError> {
        let root = root.as_ref().to_path_buf();
        let open_err = |path: &Path| {
            let path = path.display().to_string();
            move |source| CacheError::Open { path, source }
        };
        std::fs::create_dir_all(&root).map_err(open_err(&root))?;
        let tmp = root.join(TMP_SUBDIR);
        std::fs::create_dir_all(&tmp).map_err(open_err(&tmp))?;
        Ok(Cache { root })
    }

    /// This cache's root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The on-disk path for `key`: `<root>/<first-2-hex>/<full-64-hex>`.
    ///
    /// Sharded by the leading byte so one flat directory never holds every
    /// entry. Both components come from [`CacheKey::to_hex`] and are therefore
    /// hex only — no caller-controlled string reaches a path component, which is
    /// why a key can never traverse out of the root.
    pub fn path_for(&self, key: &CacheKey) -> PathBuf {
        let hex = key.to_hex();
        self.root.join(&hex[..2]).join(&hex)
    }

    /// Look `key` up. `Ok(Some(_))` only for an entry that exists, is a regular
    /// file, is within [`CACHE_ENTRY_MAX_BYTES`], parses, and whose payload
    /// re-hashes to the hash it recorded when stored.
    ///
    /// **Every** other outcome — absent, truncated, corrupt, symlinked,
    /// oversize, unreadable — is `Ok(None)`: a miss, which the executor answers
    /// by rebuilding. This never fails and never panics; serving an unverified
    /// byte is the one thing a build cache must not do.
    pub fn lookup(&self, key: &CacheKey) -> Result<Option<CachedOutput>, CacheError> {
        Ok(read_entry(&self.path_for(key), CACHE_ENTRY_MAX_BYTES))
    }

    /// Store `bytes` (an output encoded as `ext`) under `key`.
    ///
    /// Staged in `tmp/` and `rename`d into place, so a reader only ever sees a
    /// complete entry. Re-storing an existing key is a harmless overwrite:
    /// content-addressing means the bytes are the same.
    ///
    /// An output over [`CACHE_ENTRY_MAX_BYTES`] is not stored — the reader would
    /// refuse it anyway, so writing it would only waste disk.
    pub fn store(&self, key: &CacheKey, ext: &str, bytes: &[u8]) -> Result<(), CacheError> {
        self.store_bounded(key, ext, bytes, CACHE_ENTRY_MAX_BYTES)
    }

    /// [`Cache::store`] with the size bound as a parameter, so tests can exercise
    /// the refusal without materializing a 256 MiB payload.
    fn store_bounded(
        &self,
        key: &CacheKey,
        ext: &str,
        bytes: &[u8],
        max: usize,
    ) -> Result<(), CacheError> {
        if bytes.len() > max || ext.len() > MAX_EXT_LEN {
            return Ok(());
        }

        let final_path = self.path_for(key);
        let shard = final_path.parent().unwrap_or(&self.root);
        std::fs::create_dir_all(shard).map_err(CacheError::Io)?;

        let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = self.root.join(TMP_SUBDIR).join(format!(
            "{}.{}.{n}.tmp",
            key.to_hex(),
            std::process::id()
        ));

        let committed = write_entry(&tmp_path, ext, bytes)
            .and_then(|()| std::fs::rename(&tmp_path, &final_path));
        if committed.is_err() {
            // A leaked staging file is never read (only `<shard>/<hex>` is), but
            // it would accumulate.
            let _ = std::fs::remove_file(&tmp_path);
        }
        committed.map_err(CacheError::Io)
    }
}

/// Read and fully verify the entry at `path`, or `None` for any anomaly.
fn read_entry(path: &Path, max: usize) -> Option<CachedOutput> {
    // Refuse anything that is not a regular file — notably a symlink, which
    // could otherwise aim the read at an arbitrary file. `symlink_metadata` does
    // not follow, so this rejects the link itself.
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > max as u64 {
        return None;
    }

    // Bound the read independently of the size just checked, so a file that grows
    // between the two calls is still capped. Reading `max + 1` lets an oversize
    // file be *detected* rather than silently truncated into a "valid" entry.
    let file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut buf).ok()?;
    if buf.len() > max {
        return None;
    }

    parse_entry(&buf)
}

/// Serialize an entry and write it to `path`.
///
/// Frame: `MAGIC | ext_len:u8 | ext | payload_len:u64 LE | payload_hash:32 | payload`.
/// The payload hash is what [`parse_entry`] re-checks; the lengths let a
/// truncated file be detected as such rather than read as a short payload.
fn write_entry(path: &Path, ext: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut frame = Vec::with_capacity(ENTRY_MAGIC.len() + 41 + ext.len() + bytes.len());
    frame.extend_from_slice(ENTRY_MAGIC);
    frame.push(ext.len() as u8); // bounded by MAX_EXT_LEN at the call site
    frame.extend_from_slice(ext.as_bytes());
    frame.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    frame.extend_from_slice(hash_bytes(bytes).as_bytes());
    frame.extend_from_slice(bytes);

    // The file is closed when `file` drops at the end of this function, before
    // the caller renames it — Windows will not rename an open file.
    let mut file = std::fs::File::create(path)?;
    file.write_all(&frame)
}

/// Parse and verify a framed entry. `None` on any malformation.
fn parse_entry(buf: &[u8]) -> Option<CachedOutput> {
    let rest = buf.strip_prefix(ENTRY_MAGIC)?;

    let (&ext_len, rest) = rest.split_first()?;
    let ext_len = ext_len as usize;
    if ext_len > MAX_EXT_LEN {
        return None;
    }
    let (ext_bytes, rest) = rest.split_at_checked(ext_len)?;
    let ext = std::str::from_utf8(ext_bytes).ok()?;

    let (len_bytes, rest) = rest.split_at_checked(8)?;
    let payload_len = u64::from_le_bytes(len_bytes.try_into().ok()?);

    let (recorded_hash, payload) = rest.split_at_checked(32)?;

    // A truncated payload — or one with trailing garbage — fails here rather
    // than being re-hashed as if it were whole.
    if payload.len() as u64 != payload_len {
        return None;
    }
    // Verify-on-read: the corrupt→miss guarantee. Any altered byte lands here.
    if hash_bytes(payload).as_bytes() != recorded_hash {
        return None;
    }

    Some(CachedOutput {
        ext: ext.to_owned(),
        bytes: payload.to_vec(),
    })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed set of key components; each test perturbs exactly one.
    struct Base {
        version: String,
        features: String,
        recipe: Hash,
        quality: Option<u8>,
        ext: String,
        input: Hash,
    }

    impl Base {
        fn new() -> Base {
            Base {
                version: "0.4.0".into(),
                features: "avif".into(),
                recipe: hash_bytes(b"recipe-canonical-toml"),
                quality: Some(80),
                ext: "png".into(),
                input: hash_bytes(b"source-file-bytes"),
            }
        }

        fn key(&self) -> CacheKey {
            compute_key(
                &self.version,
                &self.features,
                &self.recipe,
                self.quality,
                &self.ext,
                &self.input,
            )
        }
    }

    fn temp_cache() -> (tempfile::TempDir, Cache) {
        let dir = tempfile::TempDir::new().unwrap();
        let cache = Cache::open(dir.path().join("cache")).unwrap();
        (dir, cache)
    }

    // ── Key composition ──────────────────────────────────────────────────

    #[test]
    fn key_is_stable_for_identical_inputs() {
        let base = Base::new();
        assert_eq!(base.key(), base.key());
        // And across two independently-constructed component sets.
        assert_eq!(Base::new().key(), Base::new().key());
    }

    #[test]
    fn key_changes_with_each_output_affecting_input() {
        // The correctness core: every enumerated input must be load-bearing.
        // Miss one and `build` serves a stale artifact.
        let b = Base::new();
        let k = b.key();

        // 1. input content hash
        assert_ne!(
            k,
            compute_key(
                &b.version,
                &b.features,
                &b.recipe,
                b.quality,
                &b.ext,
                &hash_bytes(b"different-source-bytes")
            ),
            "input content must affect the key"
        );

        // 2. input extension — decode is extension-routed (SPEC-061/DEC-055), so
        //    the same bytes named `.nef` decode differently than `.jpg`.
        assert_ne!(
            k,
            compute_key(
                &b.version,
                &b.features,
                &b.recipe,
                b.quality,
                "nef",
                &b.input
            ),
            "input extension must affect the key"
        );

        // 3. recipe
        assert_ne!(
            k,
            compute_key(
                &b.version,
                &b.features,
                &hash_bytes(b"a-different-recipe"),
                b.quality,
                &b.ext,
                &b.input
            ),
            "recipe must affect the key"
        );

        // 4. quality — including `None` vs `Some`, which must not collide.
        assert_ne!(
            k,
            compute_key(
                &b.version,
                &b.features,
                &b.recipe,
                Some(81),
                &b.ext,
                &b.input
            ),
            "quality value must affect the key"
        );
        assert_ne!(
            k,
            compute_key(&b.version, &b.features, &b.recipe, None, &b.ext, &b.input),
            "quality None must not collide with Some(80)"
        );
        assert_ne!(
            compute_key(&b.version, &b.features, &b.recipe, None, &b.ext, &b.input),
            compute_key(
                &b.version,
                &b.features,
                &b.recipe,
                Some(0),
                &b.ext,
                &b.input
            ),
            "quality None must not collide with Some(0)"
        );

        // 5. crustyimg version. This is the ONLY place version-invalidation can
        //    be proven: the shipped key folds in `env!("CARGO_PKG_VERSION")`, a
        //    compile-time const, so no integration test driving one binary can
        //    ever observe two values of it.
        assert_ne!(
            k,
            compute_key("9.9.9", &b.features, &b.recipe, b.quality, &b.ext, &b.input),
            "crustyimg version must affect the key"
        );

        // 6. feature signature
        assert_ne!(
            k,
            compute_key(
                &b.version,
                "avif,heic",
                &b.recipe,
                b.quality,
                &b.ext,
                &b.input
            ),
            "feature signature must affect the key"
        );

        // 7. cache-schema version. Same argument as the crustyimg version: a
        //    const, reachable only through the schema-parameterized composition
        //    the public `compute_key` delegates to.
        let at_schema = |s| {
            compute_key_with_schema(
                s,
                &b.version,
                &b.features,
                &b.recipe,
                b.quality,
                &b.ext,
                &b.input,
            )
        };
        assert_eq!(
            at_schema(CACHE_SCHEMA_VERSION),
            k,
            "compute_key must fold in the schema const"
        );
        assert_ne!(
            at_schema(CACHE_SCHEMA_VERSION + 1),
            k,
            "cache-schema version must affect the key"
        );
    }

    #[test]
    fn key_fields_are_domain_separated() {
        // Without a length prefix per field, moving a character across a field
        // boundary would leave the concatenation — and so the key — unchanged.
        let h = hash_bytes(b"x");
        assert_ne!(
            compute_key("1.0", "avif", &h, None, "png", &h),
            compute_key("1.0a", "vif", &h, None, "png", &h),
            "adjacent string fields must not be able to shift"
        );
    }

    #[test]
    fn recipe_hash_is_canonical_not_textual() {
        // Cosmetic edits (comments, whitespace) parse to the same Recipe and so
        // must hash the same; a changed param must not.
        let a = Recipe::from_toml("version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n").unwrap();
        let b = Recipe::from_toml(
            "# a comment\nversion   =   \"1\"\n[[step]]\n\nop = \"auto-orient\"\n\n",
        )
        .unwrap();
        assert_eq!(recipe_hash(&a).unwrap(), recipe_hash(&b).unwrap());

        let c = Recipe::from_toml(
            "version = \"1\"\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 16\n",
        )
        .unwrap();
        assert_ne!(recipe_hash(&a).unwrap(), recipe_hash(&c).unwrap());
    }

    #[test]
    fn feature_signature_is_sorted_and_reflects_this_build() {
        let sig = feature_signature();
        let parts: Vec<&str> = sig.split(',').filter(|s| !s.is_empty()).collect();
        let mut sorted = parts.clone();
        sorted.sort_unstable();
        assert_eq!(parts, sorted, "feature signature must be sorted");
        assert_eq!(sig.contains("avif"), cfg!(feature = "avif"));
        assert_eq!(sig.contains("heic"), cfg!(feature = "heic"));
        assert_eq!(sig.contains("webp-lossy"), cfg!(feature = "webp-lossy"));
    }

    #[test]
    fn hash_bytes_is_stable_and_hex_is_lowercase_64() {
        assert_eq!(hash_bytes(b"abc"), hash_bytes(b"abc"));
        assert_ne!(hash_bytes(b"abc"), hash_bytes(b"abd"));
        let hex = hash_bytes(b"abc").to_hex();
        assert_eq!(hex.len(), 64);
        // The published SHA-256 of "abc": a wrong hasher (or a wrong nibble
        // order) fails here rather than producing a self-consistent cache.
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // ── Store ────────────────────────────────────────────────────────────

    #[test]
    fn store_then_lookup_roundtrips() {
        let (_d, cache) = temp_cache();
        let key = Base::new().key();
        let bytes = b"\x89PNG\r\n\x1a\n-pretend-encoded-output".to_vec();

        cache.store(&key, "png", &bytes).unwrap();
        let got = cache.lookup(&key).unwrap().expect("a stored key must hit");

        assert_eq!(got.ext, "png");
        assert_eq!(got.bytes, bytes);
    }

    #[test]
    fn lookup_unknown_key_is_none() {
        let (_d, cache) = temp_cache();
        let never_stored = compute_key(
            "0.0.0",
            "",
            &hash_bytes(b"n"),
            None,
            "png",
            &hash_bytes(b"o"),
        );
        assert!(cache.lookup(&never_stored).unwrap().is_none());
    }

    #[test]
    fn corrupt_entry_is_a_miss() {
        // The load-bearing guarantee: an entry whose payload no longer matches
        // the hash it recorded is a MISS, so the executor rebuilds. It is never
        // served, and reading it never panics.
        let (_d, cache) = temp_cache();
        let key = Base::new().key();
        cache
            .store(&key, "png", b"the-original-output-bytes")
            .unwrap();
        assert!(cache.lookup(&key).unwrap().is_some(), "sanity: it stored");

        let path = cache.path_for(&key);
        let good = std::fs::read(&path).unwrap();

        // (a) a flipped payload byte
        let mut flipped = good.clone();
        let last = flipped.len() - 1;
        flipped[last] ^= 0xff;
        std::fs::write(&path, &flipped).unwrap();
        assert!(
            cache.lookup(&key).unwrap().is_none(),
            "an altered payload must miss"
        );

        // (b) a truncated entry
        std::fs::write(&path, &good[..good.len() - 3]).unwrap();
        assert!(
            cache.lookup(&key).unwrap().is_none(),
            "a truncated entry must miss"
        );

        // (c) trailing garbage appended
        let mut extended = good.clone();
        extended.extend_from_slice(b"junk");
        std::fs::write(&path, &extended).unwrap();
        assert!(
            cache.lookup(&key).unwrap().is_none(),
            "an extended entry must miss"
        );

        // (d) files that are not entries at all
        std::fs::write(&path, b"").unwrap();
        assert!(
            cache.lookup(&key).unwrap().is_none(),
            "an empty entry must miss"
        );
        std::fs::write(&path, b"not a cache entry, just some bytes").unwrap();
        assert!(cache.lookup(&key).unwrap().is_none(), "bad magic must miss");

        // Restoring the good bytes hits again — the miss was the content, not
        // some sticky state.
        std::fs::write(&path, &good).unwrap();
        assert!(cache.lookup(&key).unwrap().is_some());
    }

    #[test]
    fn missing_sidecar_or_metadata_is_a_miss() {
        // The entry is self-describing: its ext + payload hash live IN the frame.
        // A frame missing that metadata cannot describe an output, so it misses.
        let (_d, cache) = temp_cache();
        let key = Base::new().key();
        cache.store(&key, "png", b"payload").unwrap();
        let path = cache.path_for(&key);

        // Magic only: no ext, no payload length, no recorded hash.
        std::fs::write(&path, ENTRY_MAGIC).unwrap();
        assert!(cache.lookup(&key).unwrap().is_none());

        // Magic + ext, but the payload-length / recorded-hash metadata is gone.
        let mut partial = ENTRY_MAGIC.to_vec();
        partial.push(3);
        partial.extend_from_slice(b"png");
        std::fs::write(&path, &partial).unwrap();
        assert!(cache.lookup(&key).unwrap().is_none());

        // Magic + ext + length, but the recorded hash is truncated.
        partial.extend_from_slice(&7u64.to_le_bytes());
        partial.extend_from_slice(&[0u8; 16]);
        std::fs::write(&path, &partial).unwrap();
        assert!(cache.lookup(&key).unwrap().is_none());
    }

    #[test]
    fn oversize_entry_is_a_miss() {
        // Bounds are exercised against a small `max` rather than by writing
        // 256 MiB to a temp dir; `store`/`lookup` pass the real const through to
        // these same two functions.
        let (_d, cache) = temp_cache();
        let key = Base::new().key();

        // A payload over the bound is never stored, so there is nothing to read.
        cache.store_bounded(&key, "png", b"0123456789", 4).unwrap();
        assert!(
            !cache.path_for(&key).exists(),
            "an oversize output must not be stored"
        );

        // And an entry FILE over the bound is refused on read — a bounded read,
        // never an unbounded load.
        cache.store(&key, "png", b"0123456789").unwrap();
        let path = cache.path_for(&key);
        assert!(
            read_entry(&path, CACHE_ENTRY_MAX_BYTES).is_some(),
            "sanity: it reads"
        );
        let on_disk = std::fs::symlink_metadata(&path).unwrap().len() as usize;
        assert!(
            read_entry(&path, on_disk - 1).is_none(),
            "an oversize entry must miss"
        );

        // The shipped bound is generous enough for real outputs, and the store
        // never commits an entry larger than it.
        assert!(on_disk <= CACHE_ENTRY_MAX_BYTES);
        assert!(cache.lookup(&key).unwrap().is_some());
    }

    #[test]
    fn store_is_atomic_no_partial_entry() {
        let (_d, cache) = temp_cache();
        let key = Base::new().key();
        cache.store(&key, "jpg", b"complete-output").unwrap();

        // The committed entry is whole and verifies.
        assert_eq!(
            cache.lookup(&key).unwrap().unwrap().bytes,
            b"complete-output"
        );

        // Staging happens under `tmp/` and is renamed away: no temp file
        // survives, and nothing under `tmp/` is on the lookup path.
        let tmp_dir = cache.root().join(TMP_SUBDIR);
        let leftovers: Vec<_> = std::fs::read_dir(&tmp_dir).unwrap().flatten().collect();
        assert!(
            leftovers.is_empty(),
            "store must leave no staging file behind, found {leftovers:?}"
        );

        // A file dropped into `tmp/` is never mistaken for a committed entry.
        std::fs::write(tmp_dir.join("stray.tmp"), b"garbage").unwrap();
        assert_eq!(
            cache.lookup(&key).unwrap().unwrap().bytes,
            b"complete-output"
        );
    }

    #[test]
    fn key_path_is_hex_sharded_and_contained() {
        let (_d, cache) = temp_cache();
        let key = Base::new().key();
        let hex = key.to_hex();
        let path = cache.path_for(&key);

        // Sharded: <root>/<first 2 hex>/<full 64 hex>.
        assert_eq!(path.parent().unwrap(), cache.root().join(&hex[..2]));
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), hex);

        // Every path component below the root is hex only — no user-controlled
        // string (stem, template, extension) reaches a path, so a key can never
        // traverse out of the store.
        let rel = path.strip_prefix(cache.root()).unwrap();
        for component in rel.components() {
            let s = component.as_os_str().to_str().unwrap();
            assert!(
                s.bytes().all(|c| HEX_DIGITS.contains(&c)),
                "non-hex path component: {s}"
            );
        }
        assert!(path.starts_with(cache.root()));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_entry_is_a_miss() {
        // A symlink where an entry should be is refused rather than followed —
        // otherwise the bounded read could be aimed at an arbitrary file.
        let (dir, cache) = temp_cache();
        let key = Base::new().key();
        cache.store(&key, "png", b"real-output").unwrap();
        let path = cache.path_for(&key);

        // Aim the entry path at a *valid* entry stored elsewhere: even a
        // well-formed target must not be served through a link.
        let elsewhere = dir.path().join("elsewhere");
        std::fs::copy(&path, &elsewhere).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &path).unwrap();

        assert!(
            cache.lookup(&key).unwrap().is_none(),
            "a symlinked entry must miss"
        );
    }

    #[test]
    fn open_is_idempotent_and_creates_the_store() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path().join("nested/.crustyimg/cache");
        let cache = Cache::open(&root).unwrap();
        assert!(root.is_dir());
        assert!(root.join(TMP_SUBDIR).is_dir());
        assert_eq!(cache.root(), root);

        // Re-opening an existing store is fine — every build does it.
        assert!(Cache::open(&root).is_ok());
    }

    #[test]
    fn open_reports_a_typed_error_when_the_root_cannot_be_created() {
        let dir = tempfile::TempDir::new().unwrap();
        // A regular file sitting where the cache root should be.
        let blocked = dir.path().join("blocked");
        std::fs::write(&blocked, b"not a directory").unwrap();

        let err = Cache::open(&blocked).unwrap_err();
        assert!(matches!(err, CacheError::Open { .. }));
        assert!(err.to_string().contains("blocked"));
    }
}
