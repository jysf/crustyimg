//! `-v` / `-vv`: a minimal stderr logger for the `log` records our dependencies emit
//! (usvg, resvg, fontdb, the AVIF parser...). Without `-v` nothing is installed, so
//! default output is exactly what it was. crustyimg's own diagnostics stay on
//! `eprintln!`; this only surfaces what libraries say.

use log::{Level, LevelFilter, Log, Metadata, Record};

/// The most verbose level shown for `verbose` `-v` flags; `-Q` wins over any count.
/// `trace` is never shown.
pub(super) fn level_for(verbose: u8, quiet: bool) -> LevelFilter {
    match (quiet, verbose) {
        (true, _) | (false, 0) => LevelFilter::Off,
        (false, 1) => LevelFilter::Warn,
        (false, _) => LevelFilter::Debug,
    }
}

struct StderrLogger;

impl Log for StderrLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = match record.level() {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Info => "info",
            Level::Debug | Level::Trace => "debug",
        };
        // One `eprintln!` per record: stderr is line-locked, so worker threads do not interleave.
        eprintln!("{level}: [{}] {}", record.target(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: StderrLogger = StderrLogger;

/// Install the logger once per process. At `Off` nothing is installed.
pub(super) fn init(verbose: u8, quiet: bool) {
    let level = level_for(verbose, quiet);
    if level == LevelFilter::Off {
        return;
    }
    // A second install can only fail if something already set a logger; keep that one.
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_for_counts() {
        assert_eq!(level_for(0, false), LevelFilter::Off);
        assert_eq!(level_for(1, false), LevelFilter::Warn);
        assert_eq!(level_for(2, false), LevelFilter::Debug);
        assert_eq!(level_for(3, false), LevelFilter::Debug);
    }

    #[test]
    fn level_for_quiet_wins() {
        assert_eq!(level_for(1, true), LevelFilter::Off);
        assert_eq!(level_for(2, true), LevelFilter::Off);
        assert_eq!(level_for(0, true), LevelFilter::Off);
    }
}
