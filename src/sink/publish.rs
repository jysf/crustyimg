//! Staged file writes: a file is written whole or not at all.
//!
//! [`publish`] writes `bytes` to a new file beside the destination, closes it,
//! and renames it over the destination only when every byte is out. A write
//! that fails at any step returns its error and leaves the destination exactly
//! as it was (or absent, if it did not exist); the staged file is removed on
//! every early return.
//!
//! The steps run through the [`StageFs`] seam so a test can fail each one in
//! turn. Nothing here calls `fsync`: the guarantee covers errors and killed
//! processes, not power loss.
//!
//! The staged file is named `.<name>.crustyimg-<pid>-<n>.tmp`. The `.tmp`
//! extension keeps it out of crustyimg's own folder and glob inputs (a leading
//! dot does not: those do not skip dotfiles). A killed process can leave one.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Why a staged write failed.
#[derive(Debug)]
pub(crate) enum PublishError {
    /// The staged file could not be created in `dir`: the directory refuses new
    /// files, or seventeen staged names in a row were taken (the first try and
    /// sixteen retries).
    CreateTemp { dir: PathBuf, source: io::Error },
    /// Any later step: the write-probe on an existing destination, the write,
    /// the flush, the permission copy or the rename.
    Write(io::Error),
}

/// The filesystem steps [`publish`] takes, one method each, so a test can make
/// any one of them fail.
pub(crate) trait StageFs {
    /// Create `path` exclusively (it must not exist) for writing.
    fn create_new(&self, path: &Path) -> io::Result<File>;
    /// One write call. It may take fewer bytes than offered; [`publish`] loops.
    fn write(&self, file: &mut File, buf: &[u8]) -> io::Result<usize>;
    fn flush(&self, file: &mut File) -> io::Result<()>;
    fn set_permissions(&self, path: &Path, perm: Permissions) -> io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
}

/// The real filesystem.
pub(crate) struct RealFs;

impl StageFs for RealFs {
    fn create_new(&self, path: &Path) -> io::Result<File> {
        // std's default mode is 0o666 masked by the umask, which is the mode a
        // plain create gave before writes were staged. `create_new` is
        // O_CREAT|O_EXCL: it refuses an existing path and never follows a link.
        OpenOptions::new().write(true).create_new(true).open(path)
    }

    fn write(&self, file: &mut File, buf: &[u8]) -> io::Result<usize> {
        file.write(buf)
    }

    fn flush(&self, file: &mut File) -> io::Result<()> {
        file.flush()
    }

    fn set_permissions(&self, path: &Path, perm: Permissions) -> io::Result<()> {
        fs::set_permissions(path, perm)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }
}

/// Owns a staged file's path and removes the file on drop while armed. Armed
/// until the rename has returned `Ok`, so every `?` before it cleans up.
struct StagedFile {
    path: PathBuf,
    armed: bool,
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Source of the `<n>` in a staged name, shared by every thread so two threads
/// never pick the same name.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// `std::process::id()` panics on `wasm32-unknown-unknown`; nothing there writes
/// a file, so the number only has to exist.
fn process_id() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::process::id()
    }
}

/// How many bytes of the destination's file name a staged name keeps. A
/// 240-byte name plus the suffix would pass the 255-byte limit most filesystems
/// put on a name.
const NAME_KEEP: usize = 100;

fn staged_name(file_name: &str, n: u64) -> String {
    let mut cut = file_name.len().min(NAME_KEEP);
    while !file_name.is_char_boundary(cut) {
        cut -= 1;
    }
    format!(
        ".{}.crustyimg-{}-{}.tmp",
        &file_name[..cut],
        process_id(),
        n
    )
}

fn next_staged_name(file_name: &str) -> String {
    staged_name(file_name, NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Replace `path` with `bytes`, all or nothing. See the module docs.
pub(crate) fn publish(path: &Path, bytes: &[u8], fs: &dyn StageFs) -> Result<(), PublishError> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_owned());

    // An existing destination this user cannot open for writing is refused, as
    // it was when the write truncated in place (a rename would replace it).
    // Opening without `truncate` changes nothing. Its permission bits carry over.
    // A symlink is skipped on both counts: the rename replaces the link itself,
    // so what it points at is neither probed nor the source of the mode (a
    // dangling link would fail the probe, and a link's own mode is not the
    // file's).
    let old_perm = match std::fs::symlink_metadata(path) {
        Ok(meta) if !meta.file_type().is_symlink() => {
            OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(PublishError::Write)?;
            Some(meta.permissions())
        }
        _ => None,
    };

    // Stage beside the destination: a rename is atomic only within one filesystem.
    let mut tries = 0;
    let (mut file, staged_path) = loop {
        let candidate = dir.join(next_staged_name(&name));
        match fs.create_new(&candidate) {
            Ok(file) => break (file, candidate),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && tries < 16 => tries += 1,
            Err(source) => {
                return Err(PublishError::CreateTemp {
                    dir: dir.to_path_buf(),
                    source,
                })
            }
        }
    };
    let mut staged = StagedFile {
        path: staged_path,
        armed: true,
    };

    // Before the first byte: a private file's new bytes must never sit in a
    // file more readable than the one they replace, even if the process dies.
    if let Some(perm) = old_perm {
        fs.set_permissions(&staged.path, perm)
            .map_err(PublishError::Write)?;
    }

    let mut written = 0;
    while written < bytes.len() {
        match fs.write(&mut file, &bytes[written..]) {
            Ok(0) => {
                return Err(PublishError::Write(io::Error::from(
                    io::ErrorKind::WriteZero,
                )))
            }
            Ok(n) => written += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(PublishError::Write(e)),
        }
    }
    fs.flush(&mut file).map_err(PublishError::Write)?;
    // Close before the rename: Windows refuses to rename an open file. On wasm32
    // `File` is a stub with nothing to close, so clippy sees a pointless drop there.
    #[cfg_attr(target_arch = "wasm32", allow(clippy::drop_non_drop))]
    drop(file);

    fs.rename(&staged.path, path).map_err(PublishError::Write)?;
    // Disarm only after the rename succeeded; before it, the file is still ours.
    staged.armed = false;
    Ok(())
}

// Two attributes, not `all(test, …)`: clippy's test exemption looks for a bare `cfg(test)`.
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use std::cell::Cell;

    const OLD: &[u8] = b"OLD CONTENT";
    /// Five 1,024-byte chunks (and a bit) through `FailingFs`.
    const CHUNK: usize = 1024;

    fn payload() -> Vec<u8> {
        (0..5_000u32).map(|i| (i % 251) as u8).collect()
    }

    #[derive(Clone, Copy)]
    enum FailAt {
        /// Never fails: chunked writes only.
        Never,
        Create,
        /// The nth `write` call (1-based).
        Write(usize),
        Flush,
        SetPermissions,
        Rename,
    }

    /// `RealFs` that fails at one step and writes at most `CHUNK` bytes a call,
    /// so "the second write fails" is a state with bytes already staged.
    struct FailingFs {
        at: FailAt,
        writes: Cell<usize>,
    }

    impl FailingFs {
        fn new(at: FailAt) -> Self {
            FailingFs {
                at,
                writes: Cell::new(0),
            }
        }
        fn boom() -> io::Error {
            io::Error::other("injected failure")
        }
    }

    impl StageFs for FailingFs {
        fn create_new(&self, path: &Path) -> io::Result<File> {
            if matches!(self.at, FailAt::Create) {
                return Err(Self::boom());
            }
            RealFs.create_new(path)
        }
        fn write(&self, file: &mut File, buf: &[u8]) -> io::Result<usize> {
            self.writes.set(self.writes.get() + 1);
            if matches!(self.at, FailAt::Write(n) if n == self.writes.get()) {
                return Err(Self::boom());
            }
            RealFs.write(file, &buf[..buf.len().min(CHUNK)])
        }
        fn flush(&self, file: &mut File) -> io::Result<()> {
            if matches!(self.at, FailAt::Flush) {
                return Err(Self::boom());
            }
            RealFs.flush(file)
        }
        fn set_permissions(&self, path: &Path, perm: Permissions) -> io::Result<()> {
            if matches!(self.at, FailAt::SetPermissions) {
                return Err(Self::boom());
            }
            RealFs.set_permissions(path, perm)
        }
        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            if matches!(self.at, FailAt::Rename) {
                return Err(Self::boom());
            }
            RealFs.rename(from, to)
        }
    }

    fn staged_left(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".crustyimg-"))
            .collect()
    }

    /// Fail at `at`, over an existing `OLD` file and onto a new path: `Err`, the
    /// old bytes kept (or no file), no staged file.
    fn failure_keeps_everything(at: FailAt, existing_only: bool) {
        for existing in [true, false] {
            if existing_only && !existing {
                continue;
            }
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("a.png");
            if existing {
                fs::write(&path, OLD).unwrap();
            }
            let result = publish(&path, &payload(), &FailingFs::new(at));
            assert!(result.is_err(), "existing={existing}: expected Err");
            if existing {
                assert_eq!(fs::read(&path).unwrap(), OLD, "the old bytes are kept");
            } else {
                assert!(!path.exists(), "no file appears at a new path");
            }
            assert_eq!(staged_left(dir.path()), Vec::<String>::new());
        }
    }

    // FT-U1
    #[test]
    fn a_failed_create_keeps_the_old_file() {
        failure_keeps_everything(FailAt::Create, false);
    }

    // FT-U2
    #[test]
    fn a_failed_first_write_keeps_the_old_file() {
        failure_keeps_everything(FailAt::Write(1), false);
    }

    // FT-U3
    #[test]
    fn a_failed_second_write_keeps_the_old_file() {
        failure_keeps_everything(FailAt::Write(2), false);
    }

    // FT-U4
    #[test]
    fn a_failed_flush_keeps_the_old_file() {
        failure_keeps_everything(FailAt::Flush, false);
    }

    // FT-U5: set-permissions runs only for an existing file.
    #[test]
    fn a_failed_set_permissions_keeps_the_old_file() {
        failure_keeps_everything(FailAt::SetPermissions, true);
    }

    // FT-U6
    #[test]
    fn a_failed_rename_keeps_the_old_file() {
        failure_keeps_everything(FailAt::Rename, false);
    }

    // FT-U7: proves U3's state is real: bytes are already staged when the write fails.
    #[test]
    fn a_second_write_failure_leaves_partial_bytes_before_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let staged_path = dir.path().join("staged");
        let fs_ = FailingFs::new(FailAt::Write(2));
        let mut file = fs_.create_new(&staged_path).unwrap();
        let data = payload();
        let first = fs_.write(&mut file, &data).unwrap();
        assert_eq!(first, CHUNK, "the first write is one chunk");
        assert!(fs_.write(&mut file, &data[first..]).is_err());
        drop(file);
        assert_eq!(fs::metadata(&staged_path).unwrap().len(), CHUNK as u64);
    }

    // FT-U8
    #[test]
    fn a_successful_write_replaces_the_file_and_leaves_no_staged_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        fs::write(&path, OLD).unwrap();
        let fs_ = FailingFs::new(FailAt::Never);
        publish(&path, &payload(), &fs_).unwrap();
        assert_eq!(fs::read(&path).unwrap(), payload());
        assert_eq!(fs_.writes.get(), 5, "5,000 bytes take five chunked writes");
        assert_eq!(staged_left(dir.path()), Vec::<String>::new());
    }

    // FT-U9
    #[test]
    fn a_long_file_name_still_writes() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("{}.png", "a".repeat(240));
        assert_eq!(name.len(), 244);
        let path = dir.path().join(name);
        publish(&path, &payload(), &RealFs).unwrap();
        assert_eq!(fs::read(&path).unwrap(), payload());
        assert_eq!(staged_left(dir.path()), Vec::<String>::new());
    }

    // FT-U10: the uniqueness pin (the CLI row cannot see a microsecond collision).
    #[test]
    fn staged_names_differ_across_threads() {
        let names: Vec<String> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..8)
                .map(|_| s.spawn(|| next_staged_name("a.png")))
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "all eight names differ: {names:?}");
    }

    // FT-U11
    #[cfg(unix)]
    #[test]
    fn a_replaced_file_keeps_its_mode() {
        use std::os::unix::fs::PermissionsExt;
        for mode in [0o600, 0o640] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("a.png");
            fs::write(&path, OLD).unwrap();
            fs::set_permissions(&path, Permissions::from_mode(mode)).unwrap();
            publish(&path, &payload(), &RealFs).unwrap();
            let got = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(got, mode);
            assert_eq!(fs::read(&path).unwrap(), payload());
        }
    }

    // FT-U9b: the byte cut must land on a character boundary. 99 `a`s then `é`
    // (two bytes) puts byte 100 inside the `é`.
    #[test]
    fn a_name_with_a_multibyte_character_at_the_cut_still_writes() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("{}é.png", "a".repeat(99));
        assert!(
            !name.is_char_boundary(NAME_KEEP),
            "precondition: byte 100 is mid-character"
        );
        let kept = staged_name(&name, 0);
        let prefix = kept
            .strip_prefix('.')
            .and_then(|k| k.split(".crustyimg-").next())
            .unwrap();
        assert!(prefix.len() <= NAME_KEEP, "kept {} bytes", prefix.len());
        assert!(name.starts_with(prefix), "the prefix is a cut of the name");
        let path = dir.path().join(&name);
        publish(&path, &payload(), &RealFs).unwrap();
        assert_eq!(fs::read(&path).unwrap(), payload());
        assert_eq!(staged_left(dir.path()), Vec::<String>::new());
    }

    // FT-U13: the half of the `.tmp` contract the integration row cannot reach:
    // the names the code makes end in `.tmp`, and the source resolver skips them.
    #[test]
    fn a_staged_name_ends_in_tmp_and_is_not_an_input() {
        let name = next_staged_name("a.png");
        assert!(name.starts_with('.'), "{name}");
        assert!(name.ends_with(".tmp"), "{name}");
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(&name), OLD).unwrap();
        let inputs =
            crate::source::resolve(dir.path().to_str().unwrap(), &mut io::empty()).unwrap();
        assert!(inputs.is_empty(), "the resolver took {name} for an image");
    }

    // FT-U14: the old mode is on the staged file before its first byte is written.
    #[cfg(unix)]
    #[test]
    fn the_old_mode_is_on_the_staged_file_before_the_first_write() {
        use std::os::unix::fs::PermissionsExt;
        struct Spy {
            staged: std::cell::RefCell<Option<PathBuf>>,
            mode_at_first_write: Cell<Option<u32>>,
        }
        impl StageFs for Spy {
            fn create_new(&self, path: &Path) -> io::Result<File> {
                *self.staged.borrow_mut() = Some(path.to_path_buf());
                RealFs.create_new(path)
            }
            fn write(&self, file: &mut File, buf: &[u8]) -> io::Result<usize> {
                if self.mode_at_first_write.get().is_none() {
                    let path = self.staged.borrow().clone().unwrap();
                    let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
                    self.mode_at_first_write.set(Some(mode));
                }
                RealFs.write(file, buf)
            }
            fn flush(&self, file: &mut File) -> io::Result<()> {
                RealFs.flush(file)
            }
            fn set_permissions(&self, path: &Path, perm: Permissions) -> io::Result<()> {
                RealFs.set_permissions(path, perm)
            }
            fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
                RealFs.rename(from, to)
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        fs::write(&path, OLD).unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
        let spy = Spy {
            staged: Default::default(),
            mode_at_first_write: Cell::new(None),
        };
        publish(&path, &payload(), &spy).unwrap();
        assert_eq!(spy.mode_at_first_write.get(), Some(0o600));
    }

    // FT-U12
    #[cfg(unix)]
    #[test]
    fn a_read_only_destination_is_refused_and_kept() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.png");
        fs::write(&path, OLD).unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o444)).unwrap();
        assert!(
            OpenOptions::new().write(true).open(&path).is_err(),
            "precondition: running as root? a read-only file cannot be made here"
        );
        let result = publish(&path, &payload(), &RealFs);
        assert!(matches!(result, Err(PublishError::Write(_))));
        assert_eq!(fs::read(&path).unwrap(), OLD);
        assert_eq!(staged_left(dir.path()), Vec::<String>::new());
    }
}
