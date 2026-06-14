//! Integration tests for `crustyimg::source` (SPEC-004).
//!
//! All tests exercise the public API through the crate root:
//! `crustyimg::source::{resolve, looks_like_glob, Input, SourceError}`.
//!
//! Fixtures are created at runtime using `tempfile::TempDir`; no binary
//! files are committed to the repository (AGENTS.md §12).
//!
//! For every non-stdin call we pass `&mut std::io::empty()` as the reader.
//! Results are compared by file stem or file name (not absolute temp paths)
//! because temp dirs sit under an OS-controlled root that varies.

use std::io::Cursor;
use std::path::PathBuf;

use crustyimg::source::{resolve, Input, SourceError};

// ── Fixture helpers ────────────────────────────────────────────────────────────

/// Encode a solid 2×2 RGB PNG in memory and return the bytes.
/// Mirrors the `solid_png` helper in `src/image/mod.rs` tests.
fn solid_png_bytes() -> Vec<u8> {
    use ::image::{DynamicImage, ImageFormat, RgbImage};
    let img = RgbImage::from_pixel(2, 2, ::image::Rgb([128u8, 64, 32]));
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// Write `bytes` to `dir/filename` and return the full path.
fn write_file(dir: &std::path::Path, filename: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(filename);
    std::fs::write(&path, bytes).unwrap();
    path
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn single_file_resolves_to_one_input() {
    let dir = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();
    write_file(dir.path(), "photo.png", &png);
    let path_str = dir.path().join("photo.png").to_string_lossy().into_owned();

    let result = resolve(&path_str, &mut std::io::empty()).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        Input::Path(p) => {
            assert_eq!(p.file_name().unwrap().to_str().unwrap(), "photo.png");
            assert_eq!(result[0].stem(), "photo");
        }
        Input::Stdin { .. } => panic!("expected Input::Path"),
    }
}

#[test]
fn glob_returns_sorted_matches_excluding_nonmatches() {
    let dir = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();

    // Write out of alphabetical order so we can assert sort.
    write_file(dir.path(), "c.png", &png);
    write_file(dir.path(), "a.png", &png);
    write_file(dir.path(), "b.png", &png);
    // A non-image file that must NOT appear in results.
    write_file(dir.path(), "note.txt", b"not an image");

    let pattern = format!("{}/*.png", dir.path().display());
    let result = resolve(&pattern, &mut std::io::empty()).unwrap();

    assert_eq!(result.len(), 3);
    let stems: Vec<&str> = result.iter().map(|i| i.stem()).collect();
    assert_eq!(stems, vec!["a", "b", "c"]);
    // note.txt must be absent.
    assert!(result.iter().all(|i| i.stem() != "note"));
}

#[test]
fn directory_lists_top_level_images_sorted_non_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();

    // Write files out of order.
    write_file(dir.path(), "2.png", &png);
    write_file(dir.path(), "1.png", &png);
    // Non-image file — must be skipped.
    write_file(dir.path(), "readme.txt", b"text");
    // Subdirectory with its own image — must NOT be included (non-recursive).
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    write_file(&sub, "deep.png", &png);

    let dir_str = dir.path().to_string_lossy().into_owned();
    let result = resolve(&dir_str, &mut std::io::empty()).unwrap();

    assert_eq!(result.len(), 2, "expected exactly 1.png and 2.png");
    let stems: Vec<&str> = result.iter().map(|i| i.stem()).collect();
    assert_eq!(stems, vec!["1", "2"]);
    // Confirm no deep.png.
    assert!(result.iter().all(|i| i.stem() != "deep"));
}

#[test]
#[cfg(unix)]
fn directory_skips_symlink_escaping_root() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();

    // Create an "outside" file the symlink will point to.
    let outside_dir = tmp.path().join("outside");
    std::fs::create_dir(&outside_dir).unwrap();
    write_file(&outside_dir, "secret.png", &png);

    // Create the root dir we are resolving.
    let root = tmp.path().join("root");
    std::fs::create_dir(&root).unwrap();

    // A real image inside root.
    write_file(&root, "inside.png", &png);

    // A symlink inside root that escapes to ../outside/secret.png.
    let link = root.join("link.png");
    symlink(outside_dir.join("secret.png"), &link).unwrap();

    let root_str = root.to_string_lossy().into_owned();
    let result = resolve(&root_str, &mut std::io::empty()).unwrap();

    // Only inside.png must appear; the escaping symlink must be dropped.
    assert_eq!(result.len(), 1, "escaping symlink must be excluded");
    assert_eq!(result[0].stem(), "inside");
}

// On Windows we skip the symlink-escape test.
// Creating symlinks requires the SeCreateSymbolicLink privilege, which is
// unavailable in standard CI. The symlink-escape guard in resolve_directory()
// is still present in the code and exercised on Unix.
#[test]
#[cfg(windows)]
fn directory_skips_symlink_escaping_root() {
    eprintln!("directory_skips_symlink_escaping_root: skipped on Windows (symlink creation requires privilege)");
}

#[test]
fn nonimage_entries_are_skipped_not_errored() {
    let dir = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();

    write_file(dir.path(), "ok.png", &png);
    write_file(dir.path(), "data.txt", b"plain text");
    write_file(dir.path(), "weird.bin", b"\x00\x01\x02\x03");

    let dir_str = dir.path().to_string_lossy().into_owned();
    let result = resolve(&dir_str, &mut std::io::empty()).unwrap();

    // Result must not be an error, and must contain only ok.png.
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].stem(), "ok");
}

#[test]
fn resolution_order_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let png = solid_png_bytes();

    write_file(dir.path(), "z.png", &png);
    write_file(dir.path(), "m.png", &png);
    write_file(dir.path(), "a.png", &png);

    let pattern = format!("{}/*.png", dir.path().display());

    let first = resolve(&pattern, &mut std::io::empty()).unwrap();
    let second = resolve(&pattern, &mut std::io::empty()).unwrap();

    assert_eq!(
        first, second,
        "resolution must be deterministic across runs"
    );
    // Also confirm they are sorted.
    let stems: Vec<&str> = first.iter().map(|i| i.stem()).collect();
    assert_eq!(stems, vec!["a", "m", "z"]);
}

#[test]
fn missing_single_file_is_not_found_error() {
    let dir = tempfile::tempdir().unwrap();
    let path_str = dir.path().join("nope.png").to_string_lossy().into_owned();

    let err = resolve(&path_str, &mut std::io::empty()).unwrap_err();

    assert!(
        matches!(err, SourceError::NotFound(_)),
        "expected NotFound, got {err:?}"
    );
}

#[test]
fn empty_glob_match_is_not_found_error() {
    let dir = tempfile::tempdir().unwrap();
    // Empty directory — *.png matches nothing.
    let pattern = format!("{}/*.png", dir.path().display());

    let err = resolve(&pattern, &mut std::io::empty()).unwrap_err();

    assert!(
        matches!(err, SourceError::NotFound(_)),
        "expected NotFound, got {err:?}"
    );
}

#[test]
fn invalid_glob_pattern_is_typed_error() {
    // `[` starts a character class that is never closed → invalid glob pattern.
    // `looks_like_glob` returns true for `[` so resolve dispatches to the glob
    // branch, where glob::glob returns a PatternError.
    let bad_pattern = "a/[".to_string();

    let err = resolve(&bad_pattern, &mut std::io::empty()).unwrap_err();

    assert!(
        matches!(err, SourceError::InvalidPattern { .. }),
        "expected InvalidPattern, got {err:?}"
    );
}
