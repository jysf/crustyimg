//! The format auto-decision engine's pure core (SPEC-048): building the
//! candidate shortlist from the `Analysis` verdict and picking the winner.
//!
//! Layering: this module is deliberately free of `sink`, `cli`, and `std::fs`.
//! It depends only on `::image::ImageFormat` and [`crate::analysis::OptBucket`].
//! The CLI passes in the **built + capability-valid** codec set ([`BuiltCodecs`])
//! and does the actual encoding/measuring; this module only *decides*. That seam
//! is what lets the PROJ-003 planner reuse the same engine
//! (`docs/research/proj-002-findings.md §9`).

use std::io::{self, Write};

use ::image::ImageFormat;

use super::{ImageClass, OptBucket};

/// A cross-format switch is only taken when the winner beats the best
/// same-format candidate by at least this fraction — the clear-win guard
/// (DEC-048). Keeps `optimize` from changing a file's extension for a marginal
/// byte gain.
pub const FORMAT_SWITCH_THRESHOLD: f64 = 0.05;

/// The maximum number of candidate formats trial-encoded per image (the big
/// search-cost lever, DEC-048).
pub const MAX_SHORTLIST: usize = 3;

/// The `--profile` bias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Default: modern-format-first; AVIF appended in byte-budget mode.
    Web,
    /// Crisp-text bias: widen the lossless/graphic preference.
    Docs,
    /// Engine off — reproduce today's format-preserving `optimize` exactly.
    Preserve,
}

/// Whether a candidate is encoded lossy (a quality knob) or lossless.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    /// Lossy encode at a searched quality (JPEG / lossy-WebP / AVIF).
    Lossy,
    /// Lossless encode (PNG / lossless-WebP).
    Lossless,
}

/// Which search the active `optimize` mode runs — drives the AVIF gate and the
/// per-candidate solve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The **default** decision (SPEC-084): no search. Each bucket-appropriate
    /// candidate is encoded **once** at its fixed default quality, AVIF is admitted
    /// for lossy-family content at a generous fixed quality, and the smallest that
    /// beats the source wins. This is the mode a plain `optimize` (no
    /// `--target`/`--ssim`/`--max-size`) runs.
    Fast,
    /// Perceptual SSIMULACRA2 target (`--target`/`--ssim`, opt-in).
    Perceptual,
    /// Byte budget (`--max-size`, opt-in).
    SizeBudget,
}

/// One shortlisted candidate: a format + how to encode it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortlistEntry {
    /// The candidate output format.
    pub fmt: ImageFormat,
    /// Lossy or lossless encode of that format.
    pub disposition: Disposition,
}

/// Which optional codecs are compiled in. JPEG / PNG / lossless-WebP are always
/// available; only lossy-WebP and AVIF are feature-gated. Passed from the CLI so
/// this module stays free of `cfg!` and `sink`.
#[derive(Debug, Clone, Copy)]
pub struct BuiltCodecs {
    /// The `webp-lossy` feature (lossy WebP encode + a WebP decoder to score it).
    pub webp_lossy: bool,
    /// The `avif` feature (AVIF encode; still no decoder — byte-budget only).
    pub avif: bool,
}

/// The measured outcome of solving one candidate — the input to [`pick_winner`].
#[derive(Debug, Clone, Copy)]
pub struct CandidateOutcome {
    /// The candidate format.
    pub fmt: ImageFormat,
    /// Lossy or lossless.
    pub disposition: Disposition,
    /// Encoded size in bytes (== the bytes the sink will write, DEC-016).
    pub bytes: u64,
    /// Did this candidate meet the perceptual target / byte budget?
    pub met_target: bool,
}

/// Is AVIF admissible as a fixed-quality candidate for this content bucket?
///
/// AVIF belongs to the **lossy family** — it is the modern replacement for a JPEG,
/// never for a flat graphic (validated: a screenshot → lossless WebP, AVIF is a 4×
/// regression). So it is admitted for [`OptBucket::Lossy`] and the ambiguous
/// [`OptBucket::MixedSafe`] (where `pick_winner`'s smallest-beats-source lets the
/// measured bytes veto it), never for [`OptBucket::LosslessFlat`].
///
/// This is a **bucket predicate, deliberately independent of shortlist position**
/// (SPEC-079's lesson): [`format_shortlist`] truncates to [`MAX_SHORTLIST`], and a
/// last-appended AVIF on a full list would be silently dropped — so the admission
/// rule is about the bucket, not "did AVIF survive the truncation".
pub fn avif_admissible(bucket: OptBucket, built: BuiltCodecs) -> bool {
    built.avif && matches!(bucket, OptBucket::Lossy | OptBucket::MixedSafe)
}

/// Build the ordered ≤[`MAX_SHORTLIST`] candidate shortlist for a decoded image
/// (SPEC-048 / DEC-048).
///
/// Only built + capability-valid entries are returned. **AVIF is admitted for the
/// lossy-family buckets** (see [`avif_admissible`]) in the two non-perceptual
/// modes: [`Mode::SizeBudget`] (the byte-budget search, appended) and
/// [`Mode::Fast`] (the default decision, prepended so the [`MAX_SHORTLIST`]
/// truncation can never drop it). It never appears in [`Mode::Perceptual`] — it has
/// no decoder to score a round-trip against (DEC-020). The result is never empty —
/// it always contains at least one always-available lossless entry (PNG).
pub fn format_shortlist(
    bucket: OptBucket,
    has_alpha: bool,
    profile: Profile,
    mode: Mode,
    built: BuiltCodecs,
) -> Vec<ShortlistEntry> {
    use Disposition::{Lossless, Lossy};
    use ImageFormat::{Jpeg, Png, WebP};

    let lossy = |fmt| ShortlistEntry {
        fmt,
        disposition: Lossy,
    };
    let lossless = |fmt| ShortlistEntry {
        fmt,
        disposition: Lossless,
    };

    let mut out: Vec<ShortlistEntry> = Vec::new();

    // `docs` widens the graphic bias: treat the ambiguous bucket as lossless-flat.
    let effective = match (profile, bucket) {
        (Profile::Docs, OptBucket::MixedSafe) => OptBucket::LosslessFlat,
        (_, b) => b,
    };

    match effective {
        OptBucket::Lossy => {
            if built.webp_lossy {
                out.push(lossy(WebP));
            }
            if has_alpha {
                // JPEG has no alpha — a lossless RGBA format preserves it.
                out.push(lossless(Png));
            } else {
                out.push(lossy(Jpeg));
            }
        }
        OptBucket::LosslessFlat => {
            out.push(lossless(WebP));
            out.push(lossless(Png));
        }
        OptBucket::MixedSafe => {
            if built.webp_lossy {
                out.push(lossy(WebP));
            }
            out.push(lossless(WebP));
            out.push(lossless(Png));
            if !has_alpha {
                out.push(lossy(Jpeg));
            }
        }
    }

    // AVIF admission (built + lossy-family bucket only — the bucket predicate,
    // NOT shortlist position). Two non-perceptual modes admit it, differently:
    //   * SizeBudget: appended — its byte-budget search comparison-shops it against
    //     the same-family lossy candidates (unchanged from SPEC-018).
    //   * Fast: PREPENDED — it is the preferred modern candidate for the default
    //     decision, and prepending makes it immune to the MAX_SHORTLIST truncation
    //     that would otherwise silently drop a last-appended AVIF (SPEC-079's
    //     lesson: admit by bucket, and don't let truncation position decide it).
    if avif_admissible(effective, built) {
        match mode {
            Mode::SizeBudget => out.push(lossy(ImageFormat::Avif)),
            Mode::Fast => out.insert(0, lossy(ImageFormat::Avif)),
            Mode::Perceptual => {}
        }
    }

    out.truncate(MAX_SHORTLIST);

    // Always keep at least one buildable lossless candidate (PNG is always on).
    if out.is_empty() {
        out.push(lossless(Png));
    }
    out
}

/// Pick the winning candidate (SPEC-048 / DEC-048), or `None` for passthrough
/// (keep the source file unchanged).
///
/// Winner = the smallest-bytes candidate that **met its target** AND **beats the
/// source file**; ties break by shortlist order. The **clear-win guard**: if the
/// smallest is a *different* format from the source, it is only chosen when it
/// beats the best same-format candidate by [`FORMAT_SWITCH_THRESHOLD`] — else the
/// same-format candidate is kept (no surprising extension change for a marginal
/// gain). Returns `None` when nothing met its target and beat the source.
pub fn pick_winner(
    cands: &[CandidateOutcome],
    source_bytes: u64,
    source_format: ImageFormat,
) -> Option<usize> {
    let eligible: Vec<usize> = (0..cands.len())
        .filter(|&i| cands[i].met_target && cands[i].bytes < source_bytes)
        .collect();
    if eligible.is_empty() {
        return None;
    }

    // Smallest bytes, then earliest shortlist order (index is unique ⇒ total order).
    let best = *eligible
        .iter()
        .min_by_key(|&&i| (cands[i].bytes, i))
        .expect("eligible is non-empty");

    if cands[best].fmt == source_format {
        return Some(best);
    }

    // The best is a different format — apply the clear-win guard against the best
    // same-format candidate (if any).
    let same_best = eligible
        .iter()
        .copied()
        .filter(|&i| cands[i].fmt == source_format)
        .min_by_key(|&i| (cands[i].bytes, i));

    match same_best {
        Some(s) => {
            let win = (cands[s].bytes as f64 - cands[best].bytes as f64) / cands[s].bytes as f64;
            if win >= FORMAT_SWITCH_THRESHOLD {
                Some(best)
            } else {
                Some(s)
            }
        }
        // No same-format candidate to fall back to: the switch already beat the
        // source, so take it.
        None => Some(best),
    }
}

// ── Explain trace (SPEC-049 / DEC-049) ───────────────────────────────────────

/// One candidate as recorded in an [`ExplainTrace`].
#[derive(Debug, Clone, Copy)]
pub struct CandidateTrace {
    /// The candidate format.
    pub fmt: ImageFormat,
    /// Lossy or lossless.
    pub disposition: Disposition,
    /// The chosen encoder quality (`None` for a lossless candidate).
    pub quality: Option<u8>,
    /// Encoded size in bytes.
    pub bytes: u64,
    /// Whether it met the perceptual target / byte budget.
    pub met_target: bool,
}

/// Wall-clock timing for one image's audit run, in milliseconds (SPEC-088).
///
/// `decode` and `encode` are the two dominant sub-costs; `total` is the whole
/// per-image decide→encode→(score) span, so `total >= encode` and `total >=
/// decode` always hold. Only populated when `--timing` is requested — a non-timing
/// run leaves [`ExplainTrace::timing`] `None`, so the JSON stays byte-identical
/// (the same additive/gated discipline as `verify_score`, DEC-071).
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// Time spent decoding the source image.
    pub decode_ms: f64,
    /// Time spent encoding the candidate(s).
    pub encode_ms: f64,
    /// Whole per-image span (decode + analysis + encode + any score).
    pub total_ms: f64,
}

/// A human- and machine-readable record of one `optimize` auto-decision
/// (SPEC-049). A forward-compatible **subset** of the planner/manifest schema
/// (`docs/research/proj-002-design-planner.md`): PROJ-003 (planner objective /
/// warnings) and PROJ-005 (the manifest `optimization` field) extend it
/// additively. Deterministic — no paths or timestamps, floats rendered at fixed
/// precision — so it is golden-testable across platforms.
#[derive(Debug, Clone)]
pub struct ExplainTrace {
    /// The input's format.
    pub source_format: ImageFormat,
    /// The internal class label (cosmetic).
    pub class: ImageClass,
    /// Luma entropy (bits).
    pub entropy: f32,
    /// Edge-pixel fraction.
    pub edge_ratio: f32,
    /// Flat-pixel fraction.
    pub flat_ratio: f32,
    /// Distinct-colour count (capped).
    pub unique_colors: u32,
    /// Whether the colour count saturated at the cap.
    pub unique_saturated: bool,
    /// Whether the image carries alpha.
    pub has_alpha: bool,
    /// The active `--profile`.
    pub profile: Profile,
    /// Perceptual vs byte-budget mode.
    pub mode: Mode,
    /// The source file size in bytes (the "beat source" reference).
    pub source_bytes: u64,
    /// Every evaluated candidate, in shortlist order.
    pub candidates: Vec<CandidateTrace>,
    /// Index into `candidates`, or `None` for passthrough (source kept).
    pub winner: Option<usize>,
    /// Shipped bytes (the winner's, or `source_bytes` on passthrough).
    pub out_bytes: u64,
    /// The winner's achieved SSIMULACRA2 score, when it was measured this run
    /// (`web` always, `optimize --verify` on request — SPEC-085/086); `None` when
    /// scoring was off, or the winner is lossless / a passthrough. Surfaced in the
    /// JSON explain only when `Some`, so a non-verify run's schema is unchanged.
    pub verify_score: Option<f64>,
    /// Decode/encode/total timing for this run, when `--timing` was requested
    /// (SPEC-088). Rides the JSON and the human report only when `Some`, so a
    /// non-`--timing` run's output is byte-identical (same discipline as
    /// [`Self::verify_score`]).
    pub timing: Option<Timing>,
}

impl ExplainTrace {
    /// Savings as a whole-percent integer: positive when the output is smaller,
    /// **negative when it is larger** (`0` only for an empty source or an exact
    /// break-even). A larger output is never clamped to `0` — the metadata-forced
    /// fallback (SPEC-084) can ship a re-encode that could not beat the source, and
    /// the report must tell the truth about that (see [`Self::size_delta_phrase`]).
    pub fn savings_percent(&self) -> i64 {
        if self.source_bytes == 0 {
            return 0;
        }
        let frac = 1.0 - (self.out_bytes as f64 / self.source_bytes as f64);
        (frac * 100.0).round() as i64
    }

    /// The size change rendered honestly: `"40% smaller"`, `"17% larger"`, or
    /// `"0% smaller"` at break-even. A forced re-encode that could not beat the
    /// source (SPEC-084 never-bigger) reads as "larger", never a clamped
    /// "0% smaller".
    fn size_delta_phrase(&self) -> String {
        let p = self.savings_percent();
        if p < 0 {
            format!("{}% larger", -p)
        } else {
            format!("{p}% smaller")
        }
    }

    /// The default one-line summary (chosen format + savings) shown when
    /// `--explain` is not set (SPEC-048). Path-free and deterministic.
    pub fn summary_line(&self) -> String {
        match self.winner {
            Some(i) => format!(
                "{} \u{2192} {} \u{b7} {} \u{2192} {} B ({})",
                format_name(self.source_format),
                format_name(self.candidates[i].fmt),
                self.source_bytes,
                self.out_bytes,
                self.size_delta_phrase(),
            ),
            None => format!(
                "kept {} ({} B, already optimal)",
                format_name(self.source_format),
                self.source_bytes,
            ),
        }
    }

    /// One-line reason for the outcome (used by both renderers). Mode-aware: the
    /// default (fast) path has no perceptual *target*, so it never claims one was
    /// "met" (SPEC-084) — it picks the smallest re-encode that beats the source by
    /// bytes. A forced winner that could NOT beat the source (metadata stripped /
    /// orientation baked, so the raw source could not ship) is named as such.
    fn win_reason(&self) -> String {
        match self.winner {
            Some(i) => {
                let fmt = format_name(self.candidates[i].fmt);
                if self.out_bytes >= self.source_bytes {
                    format!(
                        "shipped the smallest correct re-encode ({fmt}); no candidate beat the \
                         source, but the source could not ship unchanged (metadata stripped / \
                         orientation baked)"
                    )
                } else {
                    match self.mode {
                        Mode::Fast => {
                            format!("smallest fixed-quality re-encode that beat the source ({fmt})")
                        }
                        Mode::Perceptual | Mode::SizeBudget => format!(
                            "smallest candidate that met the target and beat the source ({fmt})"
                        ),
                    }
                }
            }
            None => match self.mode {
                Mode::Fast => "kept source — no candidate beat it".to_owned(),
                Mode::Perceptual | Mode::SizeBudget => {
                    "kept source — no candidate met the target and beat it".to_owned()
                }
            },
        }
    }

    /// Render a concise, human-readable trace (SPEC-049). Goes to stderr so
    /// stdout stays pipe-clean (AGENTS §11).
    pub fn render_human(&self, w: &mut impl Write) -> io::Result<()> {
        let colors = if self.unique_saturated {
            format!("{}+", self.unique_colors)
        } else {
            self.unique_colors.to_string()
        };
        writeln!(
            w,
            "optimize: {} \u{2192} {} ({} \u{2192} {} B, {})",
            format_name(self.source_format),
            self.winner
                .map(|i| format_name(self.candidates[i].fmt))
                .unwrap_or_else(|| format_name(self.source_format)),
            self.source_bytes,
            self.out_bytes,
            self.size_delta_phrase(),
        )?;
        writeln!(
            w,
            "  class={} entropy={:.2} edges={:.2} flat={:.2} colors={} alpha={} profile={} mode={}",
            class_name(self.class),
            self.entropy,
            self.edge_ratio,
            self.flat_ratio,
            colors,
            self.has_alpha,
            profile_name(self.profile),
            mode_name(self.mode),
        )?;
        for (i, c) in self.candidates.iter().enumerate() {
            let marker = if self.winner == Some(i) { '*' } else { ' ' };
            let q = c
                .quality
                .map(|q| q.to_string())
                .unwrap_or_else(|| "-".to_owned());
            writeln!(
                w,
                "  {marker} {} {} q={q} {} B met={}",
                format_name(c.fmt),
                disposition_name(c.disposition),
                c.bytes,
                c.met_target,
            )?;
        }
        writeln!(w, "  reason: {}", self.win_reason())?;
        // Timing rides the human trace only under `--timing` (SPEC-088).
        if let Some(t) = self.timing {
            writeln!(
                w,
                "  timing: decode {:.1} ms · encode {:.1} ms · total {:.1} ms",
                t.decode_ms, t.encode_ms, t.total_ms,
            )?;
        }
        Ok(())
    }

    /// Render the trace as single-line, hand-rolled JSON (SPEC-049 / DEC-049) —
    /// no `serde_json` runtime dependency. Floats at 2 decimals so the output is
    /// stable across platforms.
    pub fn write_json(&self, w: &mut impl Write) -> io::Result<()> {
        write!(
            w,
            "{{\"schema\":\"crustyimg.optimize.explain/v1\",\
             \"source_format\":\"{}\",\"class\":\"{}\",\"profile\":\"{}\",\"mode\":\"{}\",\
             \"features\":{{\"entropy\":{:.2},\"edge_ratio\":{:.2},\"flat_ratio\":{:.2},\
             \"unique_colors\":{},\"unique_saturated\":{},\"has_alpha\":{}}},\
             \"source_bytes\":{},\"candidates\":[",
            format_name(self.source_format),
            class_name(self.class),
            profile_name(self.profile),
            mode_name(self.mode),
            self.entropy,
            self.edge_ratio,
            self.flat_ratio,
            self.unique_colors,
            self.unique_saturated,
            self.has_alpha,
            self.source_bytes,
        )?;
        for (i, c) in self.candidates.iter().enumerate() {
            if i > 0 {
                write!(w, ",")?;
            }
            write!(
                w,
                "{{\"format\":\"{}\",\"disposition\":\"{}\",\"quality\":{},\"bytes\":{},\"met_target\":{}}}",
                format_name(c.fmt),
                disposition_name(c.disposition),
                c.quality.map(|q| q.to_string()).unwrap_or_else(|| "null".to_owned()),
                c.bytes,
                c.met_target,
            )?;
        }
        write!(
            w,
            "],\"winner\":{},\"out_bytes\":{},\"savings_percent\":{}",
            self.winner
                .map(|i| i.to_string())
                .unwrap_or_else(|| "null".to_owned()),
            self.out_bytes,
            self.savings_percent(),
        )?;
        // The SSIMULACRA2 readout rides the JSON only when it was actually measured
        // (`web` / `optimize --verify`); omitting it otherwise keeps a non-verify
        // run's schema byte-identical (SPEC-086).
        if let Some(s) = self.verify_score {
            write!(w, ",\"ssim\":{s:.1}")?;
        }
        // The timing object rides the JSON only under `--timing`; omitting it
        // otherwise keeps a non-timing run's schema byte-identical (SPEC-088).
        if let Some(t) = self.timing {
            write!(
                w,
                ",\"timing\":{{\"decode_ms\":{:.2},\"encode_ms\":{:.2},\"total_ms\":{:.2}}}",
                t.decode_ms, t.encode_ms, t.total_ms,
            )?;
        }
        write!(w, "}}")
    }
}

/// Stable lowercase name for a candidate format (the engine emits only these).
fn format_name(f: ImageFormat) -> &'static str {
    match f {
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        ImageFormat::Avif => "avif",
        ImageFormat::Gif => "gif",
        _ => "other",
    }
}

fn class_name(c: ImageClass) -> &'static str {
    match c {
        ImageClass::Photograph => "photograph",
        ImageClass::GraphicLogo => "graphic-logo",
        ImageClass::Icon => "icon",
        ImageClass::Document => "document",
        ImageClass::UiScreenshot => "ui-screenshot",
    }
}

fn profile_name(p: Profile) -> &'static str {
    match p {
        Profile::Web => "web",
        Profile::Docs => "docs",
        Profile::Preserve => "preserve",
    }
}

fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::Fast => "fast",
        Mode::Perceptual => "perceptual",
        Mode::SizeBudget => "size-budget",
    }
}

fn disposition_name(d: Disposition) -> &'static str {
    match d {
        Disposition::Lossy => "lossy",
        Disposition::Lossless => "lossless",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ImageFormat::{Avif, Jpeg, Png, WebP};

    const ALL_BUILT: BuiltCodecs = BuiltCodecs {
        webp_lossy: true,
        avif: true,
    };
    const NONE_BUILT: BuiltCodecs = BuiltCodecs {
        webp_lossy: false,
        avif: false,
    };

    fn oc(fmt: ImageFormat, bytes: u64, met: bool) -> CandidateOutcome {
        CandidateOutcome {
            fmt,
            disposition: Disposition::Lossy,
            bytes,
            met_target: met,
        }
    }

    // ── pick_winner ──────────────────────────────────────────────────────────

    #[test]
    fn winner_smallest_met_that_beats_source() {
        let cands = [oc(Jpeg, 900, true), oc(WebP, 700, true), oc(Png, 950, true)];
        // source is PNG so the guard compares against a same-format candidate.
        let w = pick_winner(&cands, 1000, Png).unwrap();
        // best raw = WebP(700); same-format PNG(950); win = (950-700)/950 = 0.26 ≥ 5% → switch.
        assert_eq!(cands[w].fmt, WebP);
    }

    #[test]
    fn winner_none_beats_source_is_passthrough() {
        let cands = [oc(Jpeg, 1200, true), oc(WebP, 1100, true)];
        assert_eq!(pick_winner(&cands, 1000, Jpeg), None);
    }

    #[test]
    fn winner_tie_break_is_shortlist_order() {
        let cands = [oc(WebP, 800, true), oc(Jpeg, 800, true)];
        let w = pick_winner(&cands, 1000, Jpeg).unwrap();
        // Equal bytes → earliest index (0 = WebP) wins; but WebP differs from
        // source JPEG and there IS a same-format JPEG at equal bytes → guard: win
        // = 0% < 5% → keep the source-format JPEG.
        assert_eq!(cands[w].fmt, Jpeg);
    }

    #[test]
    fn winner_unmet_target_excluded() {
        let cands = [oc(WebP, 500, false), oc(Jpeg, 900, true)];
        let w = pick_winner(&cands, 1000, Jpeg).unwrap();
        assert_eq!(cands[w].fmt, Jpeg); // the smaller WebP did not meet its target
    }

    #[test]
    fn clear_win_guard_below_threshold_keeps_source_format() {
        // Source PNG(1000); WebP(980) is only 2% smaller → keep PNG.
        let cands = [oc(Png, 1000, true), oc(WebP, 980, true)];
        let w = pick_winner(&cands, 2000, Png).unwrap();
        assert_eq!(cands[w].fmt, Png);
    }

    #[test]
    fn clear_win_guard_at_threshold_takes_switch() {
        // Source PNG(1000); WebP(900) is 10% smaller → switch.
        let cands = [oc(Png, 1000, true), oc(WebP, 900, true)];
        let w = pick_winner(&cands, 2000, Png).unwrap();
        assert_eq!(cands[w].fmt, WebP);
    }

    #[test]
    fn clear_win_guard_no_same_format_takes_switch() {
        // Source is GIF (not a candidate) — the switch beat the source, take it.
        let cands = [oc(WebP, 980, true)];
        let w = pick_winner(&cands, 1000, ImageFormat::Gif).unwrap();
        assert_eq!(cands[w].fmt, WebP);
    }

    // ── format_shortlist ─────────────────────────────────────────────────────

    #[test]
    fn shortlist_photo_no_alpha() {
        let s = format_shortlist(
            OptBucket::Lossy,
            false,
            Profile::Web,
            Mode::Perceptual,
            ALL_BUILT,
        );
        assert_eq!(
            s,
            vec![
                ShortlistEntry {
                    fmt: WebP,
                    disposition: Disposition::Lossy
                },
                ShortlistEntry {
                    fmt: Jpeg,
                    disposition: Disposition::Lossy
                },
            ]
        );
    }

    #[test]
    fn shortlist_few_colour_graphic_is_lossless() {
        let s = format_shortlist(
            OptBucket::LosslessFlat,
            false,
            Profile::Web,
            Mode::Perceptual,
            ALL_BUILT,
        );
        assert_eq!(
            s,
            vec![
                ShortlistEntry {
                    fmt: WebP,
                    disposition: Disposition::Lossless
                },
                ShortlistEntry {
                    fmt: Png,
                    disposition: Disposition::Lossless
                },
            ]
        );
    }

    #[test]
    fn shortlist_alpha_photo_excludes_jpeg() {
        let s = format_shortlist(
            OptBucket::Lossy,
            true,
            Profile::Web,
            Mode::Perceptual,
            ALL_BUILT,
        );
        assert!(s.iter().all(|e| e.fmt != Jpeg), "JPEG has no alpha: {s:?}");
        assert!(s.iter().any(|e| e.fmt == Png)); // lossless alpha-safe fallback
    }

    #[test]
    fn shortlist_avif_never_in_perceptual_and_gated_on_built() {
        // Perceptual never admits AVIF (no decoder to score a round-trip, DEC-020).
        let perceptual = format_shortlist(
            OptBucket::Lossy,
            false,
            Profile::Web,
            Mode::Perceptual,
            ALL_BUILT,
        );
        assert!(
            perceptual.iter().all(|e| e.fmt != Avif),
            "no AVIF perceptual"
        );

        // Both non-perceptual modes admit AVIF for a lossy-family bucket when built.
        for mode in [Mode::SizeBudget, Mode::Fast] {
            let built = format_shortlist(OptBucket::Lossy, false, Profile::Web, mode, ALL_BUILT);
            assert!(
                built.iter().any(|e| e.fmt == Avif),
                "AVIF in {mode:?} when built: {built:?}"
            );
            let unbuilt = format_shortlist(OptBucket::Lossy, false, Profile::Web, mode, NONE_BUILT);
            assert!(
                unbuilt.iter().all(|e| e.fmt != Avif),
                "no AVIF unbuilt in {mode:?}: {unbuilt:?}"
            );
        }

        // AVIF is never admitted for a purely lossless/graphic bucket, even in Fast.
        let graphic = format_shortlist(
            OptBucket::LosslessFlat,
            false,
            Profile::Web,
            Mode::Fast,
            ALL_BUILT,
        );
        assert!(
            graphic.iter().all(|e| e.fmt != Avif),
            "no AVIF for a graphic bucket: {graphic:?}"
        );
    }

    // ── SPEC-084: the default (fast) decision mode ────────────────────────────

    /// The default (fast) decision admits AVIF for a photographic (lossy) bucket.
    #[test]
    fn default_decision_admits_avif_for_photo() {
        let s = format_shortlist(OptBucket::Lossy, false, Profile::Web, Mode::Fast, ALL_BUILT);
        assert!(
            s.iter()
                .any(|e| e.fmt == Avif && e.disposition == Disposition::Lossy),
            "fast mode must offer a lossy AVIF candidate for a photo: {s:?}"
        );
    }

    /// The default (fast) decision keeps a graphic bucket lossless — never AVIF.
    #[test]
    fn default_decision_keeps_graphic_lossless() {
        let s = format_shortlist(
            OptBucket::LosslessFlat,
            false,
            Profile::Web,
            Mode::Fast,
            ALL_BUILT,
        );
        assert!(
            s.iter().all(|e| e.disposition == Disposition::Lossless),
            "a graphic bucket must stay lossless in fast mode: {s:?}"
        );
        assert!(
            s.iter().all(|e| e.fmt != Avif),
            "never AVIF for a graphic: {s:?}"
        );
    }

    /// The default decision returns passthrough (`None`) when no fixed-quality
    /// candidate beats the source — the never-bigger guarantee, mode-independent
    /// (`pick_winner` compares measured bytes to the source regardless of how the
    /// candidates were produced).
    #[test]
    fn default_decision_passthrough_when_nothing_beats_source() {
        // An already-optimal source: every fast candidate is ≥ the source bytes.
        let cands = [
            oc(Avif, 1_050, true),
            oc(Jpeg, 1_100, true),
            oc(WebP, 1_010, true),
        ];
        assert_eq!(
            pick_winner(&cands, 1_000, Jpeg),
            None,
            "nothing beats the source → passthrough"
        );
    }

    /// The AVIF admission is a **bucket predicate**, not shortlist membership: even
    /// when the pre-AVIF candidate list already fills `MAX_SHORTLIST`, AVIF survives
    /// (it is prepended, not appended-then-truncated — the SPEC-079 footgun).
    #[test]
    fn avif_admission_survives_shortlist_truncation() {
        // MixedSafe + no-alpha + ALL_BUILT already yields four pre-AVIF candidates
        // (lossy WebP, lossless WebP, lossless PNG, JPEG) — more than MAX_SHORTLIST.
        // An appended AVIF would be truncated away; a prepended one must not be.
        let s = format_shortlist(
            OptBucket::MixedSafe,
            false,
            Profile::Web,
            Mode::Fast,
            ALL_BUILT,
        );
        assert!(s.len() <= MAX_SHORTLIST, "still bounded: {s:?}");
        assert!(
            s.iter().any(|e| e.fmt == Avif),
            "AVIF must survive truncation on a full lossy-family shortlist: {s:?}"
        );
        // And the predicate agrees independently of the shortlist.
        assert!(avif_admissible(OptBucket::MixedSafe, ALL_BUILT));
        assert!(!avif_admissible(OptBucket::LosslessFlat, ALL_BUILT));
        assert!(!avif_admissible(OptBucket::Lossy, NONE_BUILT));
    }

    #[test]
    fn shortlist_is_always_nonempty_bounded_and_buildable() {
        let buckets = [
            OptBucket::Lossy,
            OptBucket::LosslessFlat,
            OptBucket::MixedSafe,
        ];
        let profiles = [Profile::Web, Profile::Docs];
        let modes = [Mode::Fast, Mode::Perceptual, Mode::SizeBudget];
        let builts = [
            ALL_BUILT,
            NONE_BUILT,
            BuiltCodecs {
                webp_lossy: true,
                avif: false,
            },
        ];
        for &bucket in &buckets {
            for &alpha in &[false, true] {
                for &profile in &profiles {
                    for &mode in &modes {
                        for &built in &builts {
                            let s = format_shortlist(bucket, alpha, profile, mode, built);
                            assert!(!s.is_empty(), "empty {bucket:?}/{profile:?}/{mode:?}");
                            assert!(s.len() <= MAX_SHORTLIST, "over cap: {s:?}");
                            for e in &s {
                                // Every entry must be buildable given `built`.
                                let ok = match (e.fmt, e.disposition) {
                                    (Jpeg, _) | (Png, _) => true,
                                    (WebP, Disposition::Lossless) => true,
                                    (WebP, Disposition::Lossy) => built.webp_lossy,
                                    // AVIF is admitted in the two non-perceptual
                                    // modes, for the lossy-family buckets only.
                                    (Avif, _) => {
                                        built.avif && matches!(mode, Mode::SizeBudget | Mode::Fast)
                                    }
                                    _ => false,
                                };
                                assert!(ok, "unbuildable entry {e:?} with {built:?}");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn docs_profile_makes_mixed_lossless() {
        let s = format_shortlist(
            OptBucket::MixedSafe,
            false,
            Profile::Docs,
            Mode::Perceptual,
            ALL_BUILT,
        );
        assert!(
            s.iter().all(|e| e.disposition == Disposition::Lossless),
            "docs widens to lossless: {s:?}"
        );
    }

    // ── ExplainTrace (SPEC-049) ──────────────────────────────────────────────

    fn sample_trace() -> ExplainTrace {
        ExplainTrace {
            source_format: Png,
            class: ImageClass::Photograph,
            entropy: 6.234,
            edge_ratio: 0.012,
            flat_ratio: 0.987,
            unique_colors: 4096,
            unique_saturated: true,
            has_alpha: false,
            profile: Profile::Web,
            mode: Mode::Perceptual,
            source_bytes: 10_000,
            candidates: vec![
                CandidateTrace {
                    fmt: WebP,
                    disposition: Disposition::Lossy,
                    quality: Some(82),
                    bytes: 6000,
                    met_target: true,
                },
                CandidateTrace {
                    fmt: Jpeg,
                    disposition: Disposition::Lossy,
                    quality: Some(90),
                    bytes: 7000,
                    met_target: true,
                },
            ],
            winner: Some(0),
            out_bytes: 6000,
            verify_score: None,
            timing: None,
        }
    }

    #[test]
    fn explain_json_golden() {
        let mut buf = Vec::new();
        sample_trace().write_json(&mut buf).unwrap();
        let json = String::from_utf8(buf).unwrap();
        assert_eq!(
            json,
            r#"{"schema":"crustyimg.optimize.explain/v1","source_format":"png","class":"photograph","profile":"web","mode":"perceptual","features":{"entropy":6.23,"edge_ratio":0.01,"flat_ratio":0.99,"unique_colors":4096,"unique_saturated":true,"has_alpha":false},"source_bytes":10000,"candidates":[{"format":"webp","disposition":"lossy","quality":82,"bytes":6000,"met_target":true},{"format":"jpeg","disposition":"lossy","quality":90,"bytes":7000,"met_target":true}],"winner":0,"out_bytes":6000,"savings_percent":40}"#
        );
    }

    #[test]
    fn explain_json_is_deterministic() {
        let (mut a, mut b) = (Vec::new(), Vec::new());
        sample_trace().write_json(&mut a).unwrap();
        sample_trace().write_json(&mut b).unwrap();
        assert_eq!(a, b);
    }

    /// SPEC-086: `optimize --verify` / `web` surface the measured score as a trailing
    /// `"ssim"` field; a non-scored run omits it entirely (schema unchanged).
    #[test]
    fn explain_json_includes_ssim_only_when_verified() {
        let mut off = Vec::new();
        sample_trace().write_json(&mut off).unwrap();
        assert!(
            !String::from_utf8(off).unwrap().contains("\"ssim\""),
            "a non-verify trace must not emit an ssim field"
        );

        let mut trace = sample_trace();
        trace.verify_score = Some(88.42);
        let mut on = Vec::new();
        trace.write_json(&mut on).unwrap();
        let json = String::from_utf8(on).unwrap();
        assert!(
            json.contains("\"ssim\":88.4"),
            "a verified trace must emit the rounded ssim: {json}"
        );
        assert!(
            json.trim_end().ends_with("}"),
            "the ssim field must stay inside the JSON object: {json}"
        );
    }

    /// SPEC-088: `--timing` adds a `timing` object to the JSON with fixed-precision
    /// decode/encode/total fields; a non-timing trace omits it entirely (schema
    /// unchanged), holding the byte-identity guarantee.
    #[test]
    fn explain_json_includes_timing_only_when_set() {
        let mut off = Vec::new();
        sample_trace().write_json(&mut off).unwrap();
        assert!(
            !String::from_utf8(off).unwrap().contains("\"timing\""),
            "a non-timing trace must not emit a timing field"
        );

        let mut trace = sample_trace();
        trace.timing = Some(Timing {
            decode_ms: 12.345,
            encode_ms: 56.7,
            total_ms: 78.912,
        });
        let mut on = Vec::new();
        trace.write_json(&mut on).unwrap();
        let json = String::from_utf8(on).unwrap();
        assert!(
            json.contains(
                "\"timing\":{\"decode_ms\":12.35,\"encode_ms\":56.70,\"total_ms\":78.91}"
            ),
            "timing must render at 2dp inside the object: {json}"
        );
        assert!(
            json.trim_end().ends_with('}'),
            "timing must stay inside the JSON object: {json}"
        );
    }

    /// SPEC-088: timing and ssim can co-exist and both stay inside the object, in a
    /// stable order (ssim then timing) — the additive-field discipline composes.
    #[test]
    fn explain_json_timing_and_ssim_compose() {
        let mut trace = sample_trace();
        trace.verify_score = Some(88.42);
        trace.timing = Some(Timing {
            decode_ms: 1.0,
            encode_ms: 2.0,
            total_ms: 3.0,
        });
        let mut buf = Vec::new();
        trace.write_json(&mut buf).unwrap();
        let json = String::from_utf8(buf).unwrap();
        let ssim_at = json.find("\"ssim\"").expect("ssim present");
        let timing_at = json.find("\"timing\"").expect("timing present");
        assert!(ssim_at < timing_at, "ssim must precede timing: {json}");
        assert!(json.trim_end().ends_with('}'));
    }

    /// SPEC-088: the human trace gains a timing line only under `--timing`.
    #[test]
    fn explain_human_shows_timing_only_when_set() {
        let mut off = Vec::new();
        sample_trace().render_human(&mut off).unwrap();
        assert!(!String::from_utf8(off).unwrap().contains("timing"));

        let mut trace = sample_trace();
        trace.timing = Some(Timing {
            decode_ms: 4.0,
            encode_ms: 8.0,
            total_ms: 12.0,
        });
        let mut on = Vec::new();
        trace.render_human(&mut on).unwrap();
        let s = String::from_utf8(on).unwrap();
        assert!(s.contains("timing:"), "human trace must show timing: {s}");
        assert!(s.contains("total 12.0 ms"), "{s}");
    }

    #[test]
    fn explain_human_lists_candidates_winner_savings() {
        let mut buf = Vec::new();
        sample_trace().render_human(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("webp"), "{s}");
        assert!(s.contains("jpeg"), "{s}");
        assert!(s.contains("40% smaller"), "{s}");
        assert!(s.contains("reason:"), "{s}");
        assert!(s.contains('*'), "winner marker missing: {s}");
    }

    #[test]
    fn explain_passthrough_renders_kept_source() {
        let mut trace = sample_trace();
        trace.winner = None;
        trace.out_bytes = trace.source_bytes;
        assert_eq!(trace.savings_percent(), 0);

        let mut human = Vec::new();
        trace.render_human(&mut human).unwrap();
        assert!(String::from_utf8(human).unwrap().contains("kept source"));

        let mut json = Vec::new();
        trace.write_json(&mut json).unwrap();
        assert!(String::from_utf8(json).unwrap().contains("\"winner\":null"));
    }

    // ── SPEC-084 Finding 1: honest reporting of a forced re-encode that is LARGER ──

    /// A forced winner whose output EXCEEDS the source (the metadata-stripped
    /// re-encode couldn't beat an already-tight source) reports the true delta as
    /// "larger" — `savings_percent` goes negative, never clamped to a break-even 0.
    #[test]
    fn larger_forced_output_is_reported_honestly_not_clamped() {
        let mut trace = sample_trace();
        // winner Some(0), but its bytes (12_000) exceed the 10_000-byte source.
        trace.out_bytes = 12_000;
        trace.candidates[0].bytes = 12_000;

        assert_eq!(
            trace.savings_percent(),
            -20,
            "a 20% larger output must report -20, not a clamped 0"
        );
        assert!(
            trace.summary_line().contains("20% larger"),
            "summary must say 'larger', not '0% smaller': {}",
            trace.summary_line()
        );
        assert!(
            !trace.summary_line().contains("smaller"),
            "a larger output must not read as 'smaller': {}",
            trace.summary_line()
        );

        let mut human = Vec::new();
        trace.render_human(&mut human).unwrap();
        let h = String::from_utf8(human).unwrap();
        assert!(h.contains("20% larger"), "{h}");
        assert!(
            h.contains("could not ship unchanged"),
            "the reason must explain the forced re-encode: {h}"
        );

        let mut json = Vec::new();
        trace.write_json(&mut json).unwrap();
        assert!(
            String::from_utf8(json)
                .unwrap()
                .contains("savings_percent\":-20"),
            "JSON savings_percent must be honestly negative"
        );
    }

    /// The default (fast) path has no perceptual *target*, so its reason must not
    /// claim one was "met" (SPEC-084 Finding 4) — for both a winning and a
    /// passthrough outcome.
    #[test]
    fn fast_mode_reason_never_claims_a_met_target() {
        let mut win = sample_trace();
        win.mode = Mode::Fast;
        assert!(
            !win.win_reason().contains("met the target"),
            "fast win reason must not mention a target: {}",
            win.win_reason()
        );
        assert!(
            win.win_reason().contains("beat the source"),
            "fast win reason should still explain the byte decision: {}",
            win.win_reason()
        );

        let mut pass = sample_trace();
        pass.mode = Mode::Fast;
        pass.winner = None;
        pass.out_bytes = pass.source_bytes;
        assert!(
            !pass.win_reason().contains("met the target"),
            "fast passthrough reason must not mention a target: {}",
            pass.win_reason()
        );
    }
}
