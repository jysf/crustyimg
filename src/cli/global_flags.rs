//! The guard for global flags a subcommand would otherwise accept and ignore
//! (SPEC-149). Global flags are `global = true` so they parse anywhere; this table
//! says where each one is real, and `dispatch` refuses it (exit 2) with a reason
//! everywhere else. Scopes are defined by exclusion (`AllButBuild`), so a verb added
//! later is refused without an edit here.

use super::{Cli, Commands, GlobalArgs};

/// Where a row applies.
#[derive(Clone, Copy)]
enum RefusedOn {
    /// Every command except `build`.
    AllButBuild,
    /// `build` only.
    Build,
    /// `responsive` only.
    Responsive,
    /// Every command, `build` included.
    All,
}

struct GlobalFlagRule {
    /// Whether the user typed the flag.
    given: fn(&GlobalArgs) -> bool,
    on: RefusedOn,
    /// Printed after `error: `.
    message: &'static str,
}

// The `--watch` message is SPEC-071's, moved here unchanged.
const R1_WATCH: &str = "--watch is only valid with `build`: there is no rebuild loop to run for \
                        this subcommand";
const R2_CHECK: &str = "--check (or --frozen/--locked) is only valid with `build`: there is no \
                        build lockfile to check for this subcommand";
const R3_STRICT: &str = "--strict is only valid with `build --check`: there is no build lockfile \
                         to check for this subcommand";
const R4_STRICT_ALONE: &str =
    "--strict only applies with --check: run `build --check --strict`, or drop --strict";
const R5_NO_CACHE: &str =
    "--no-cache is only valid with `build`: there is no build cache for this subcommand";
const R6_KEEP_GPS: &str = "--keep-gps is not supported yet: pixel-lane encodes drop all \
                           metadata, GPS included; to keep it, copy the metadata back after the \
                           encode with `crustyimg meta copy --from <in> --to <out> -y` (JPEG \
                           only)";
const R7_NAME_TEMPLATE: &str =
    "responsive does not take --name-template: its output names are fixed as {stem}-{w}w.{ext}";
const R8_FORMAT: &str = "responsive does not take --format: choose its output formats with \
                         --formats (e.g. --formats webp,jpeg)";

/// In report order. `--strict` appears twice: R3 off `build`, R4 on `build` without `--check`.
const GLOBAL_FLAG_RULES: &[GlobalFlagRule] = &[
    GlobalFlagRule {
        given: |g| g.watch,
        on: RefusedOn::AllButBuild,
        message: R1_WATCH,
    },
    GlobalFlagRule {
        given: |g| g.check,
        on: RefusedOn::AllButBuild,
        message: R2_CHECK,
    },
    GlobalFlagRule {
        given: |g| g.strict,
        on: RefusedOn::AllButBuild,
        message: R3_STRICT,
    },
    GlobalFlagRule {
        given: |g| g.strict && !g.check,
        on: RefusedOn::Build,
        message: R4_STRICT_ALONE,
    },
    GlobalFlagRule {
        given: |g| g.no_cache,
        on: RefusedOn::AllButBuild,
        message: R5_NO_CACHE,
    },
    GlobalFlagRule {
        given: |g| g.keep_gps,
        on: RefusedOn::All,
        message: R6_KEEP_GPS,
    },
    GlobalFlagRule {
        given: |g| g.name_template.is_some(),
        on: RefusedOn::Responsive,
        message: R7_NAME_TEMPLATE,
    },
    GlobalFlagRule {
        given: |g| g.format.is_some(),
        on: RefusedOn::Responsive,
        message: R8_FORMAT,
    },
];

/// The refusal messages that apply to this invocation, in table order.
pub(super) fn refused_global_flags(cli: &Cli) -> Vec<&'static str> {
    let is_build = matches!(cli.command, Commands::Build { .. });
    let is_responsive = matches!(cli.command, Commands::Responsive { .. });
    GLOBAL_FLAG_RULES
        .iter()
        .filter(|rule| {
            let applies = match rule.on {
                RefusedOn::AllButBuild => !is_build,
                RefusedOn::Build => is_build,
                RefusedOn::Responsive => is_responsive,
                RefusedOn::All => true,
            };
            applies && (rule.given)(&cli.global)
        })
        .map(|rule| rule.message)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn refused(args: &[&str]) -> Vec<&'static str> {
        let mut argv = vec!["crustyimg"];
        argv.extend_from_slice(args);
        refused_global_flags(&Cli::try_parse_from(argv).expect("argv parses"))
    }

    #[test]
    fn refused_global_flags_reports_every_row_in_order() {
        assert_eq!(
            refused(&[
                "resize",
                "a",
                "--max",
                "8",
                "--check",
                "--no-cache",
                "--keep-gps"
            ]),
            [R2_CHECK, R5_NO_CACHE, R6_KEEP_GPS]
        );
        assert_eq!(refused(&["meta", "strip", "a", "--strict"]), [R3_STRICT]);
        assert_eq!(refused(&["build", "--strict"]), [R4_STRICT_ALONE]);
        assert!(refused(&["build", "--check", "--strict"]).is_empty());
        assert!(refused(&["build", "--no-cache", "--check"]).is_empty());
        assert_eq!(
            refused(&["build", "--watch", "--strict"]),
            [R4_STRICT_ALONE]
        );
        assert_eq!(
            refused(&[
                "responsive",
                "a",
                "--widths",
                "8",
                "--format",
                "webp",
                "--name-template",
                "x"
            ]),
            [R7_NAME_TEMPLATE, R8_FORMAT]
        );
        assert!(refused(&[
            "resize",
            "a",
            "--max",
            "8",
            "--name-template",
            "x",
            "--format",
            "png"
        ])
        .is_empty());
        assert!(refused(&["resize", "a", "--max", "8"]).is_empty());
    }
}
