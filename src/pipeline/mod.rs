//! Decode-once pipeline executor (DEC-002).
//!
//! `Pipeline` owns an ordered list of `Operation`s and folds them over a
//! single in-memory `Image` without any per-operation disk round-trips
//! (constraint `decode-once-no-per-op-disk`). If any operation fails, the
//! `Err` is returned immediately and later operations do not run.
//!
//! An operation can also *decline* an image (see `Operation::refusal`, e.g. a
//! watermark too large for it). [`Pipeline::run`] treats that as an error;
//! [`Pipeline::run_with`] with [`OnRefusal::Skip`] leaves that one step out,
//! records it, and runs the remaining steps on the unchanged image — what a
//! batch needs to keep every image written.
//!
//! Layering: this module imports `crate::image::Image` and
//! `crate::operation::{Operation, OperationError}`. It must NOT depend on
//! `clap`, `recipe`, `source`, `sink`, `std::fs`, or `std::path`.

use crate::image::Image;
use crate::operation::alpha::{alpha_capacity, stores};
use crate::operation::{Operation, OperationError, Refusal, UnstorableAlpha};

// ─── Refusal policy ─────────────────────────────────────────────────────────

/// What [`Pipeline::run_with`] does when a step declines the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnRefusal {
    /// Stop with [`OperationError::Refused`], as [`Pipeline::run`] does.
    Fail,
    /// Leave the step out, record it, and run the rest.
    Skip,
}

/// A step [`OnRefusal::Skip`] left out, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedStep {
    /// The step's position in the pipeline, from 0.
    pub step: usize,
    /// The step's op name, e.g. `"watermark"`.
    pub op: &'static str,
    /// Why the step declined.
    pub refusal: Refusal,
}

/// The result of [`Pipeline::run_with`]: the final image, and every step
/// that was left out (always empty under [`OnRefusal::Fail`]).
pub struct PipelineRun {
    pub image: Image,
    pub skipped: Vec<SkippedStep>,
}

// ─── Pipeline ───────────────────────────────────────────────────────────────

/// Decode-once executor: owns an ordered list of `Operation`s and folds them
/// over a single in-memory `Image` (DEC-002).
pub struct Pipeline {
    ops: Vec<Box<dyn Operation>>,
}

impl Pipeline {
    /// Create an empty pipeline (no operations queued).
    pub fn new() -> Self {
        Pipeline { ops: Vec::new() }
    }

    /// Whether any step adds pixels with alpha below fully opaque.
    pub fn introduces_alpha(&self) -> bool {
        self.ops.iter().any(|op| op.introduced_alpha().is_some())
    }

    /// The first step, in order, whose introduced alpha an output of `format`
    /// can't store, or `None`. Static: reads the steps, not any pixels.
    ///
    /// The index is the recipe step's index, because a recipe builds one
    /// operation per step in order.
    pub fn unstorable_alpha(&self, format: ::image::ImageFormat) -> Option<UnstorableAlpha> {
        let capacity = alpha_capacity(format);
        self.ops.iter().enumerate().find_map(|(step, op)| {
            let introduced = op.introduced_alpha()?;
            if stores(capacity, introduced.alpha) {
                return None;
            }
            Some(UnstorableAlpha {
                step,
                op: op.name(),
                introduced,
                format,
            })
        })
    }

    /// Append an operation; returns `self` for builder-style chaining.
    pub fn push(mut self, op: Box<dyn Operation>) -> Self {
        self.ops.push(op);
        self
    }

    /// Number of operations currently queued.
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Whether the pipeline has no operations.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Fold every operation over `img` in order, in memory.
    ///
    /// Returns the final `Image` after all operations, or the first
    /// `OperationError` (later operations do **not** run — the `?` operator
    /// provides halt-on-first-error). An empty pipeline returns `img`
    /// unchanged. No disk I/O; no intermediate clone of the image.
    ///
    /// A step that declines the image stops the run with
    /// [`OperationError::Refused`], exactly as `run_with(img, OnRefusal::Fail)`.
    pub fn run(&self, img: Image) -> Result<Image, OperationError> {
        self.run_with(img, OnRefusal::Fail).map(|run| run.image)
    }

    /// Fold every operation over `img`, asking each step first whether it
    /// declines the image. Under [`OnRefusal::Fail`] a refusal is an error;
    /// under [`OnRefusal::Skip`] that step is not applied, is recorded, and
    /// the remaining steps still run. Any other error stops the run either
    /// way.
    pub fn run_with(
        &self,
        img: Image,
        on_refusal: OnRefusal,
    ) -> Result<PipelineRun, OperationError> {
        let mut current = img;
        let mut skipped = Vec::new();
        for (step, op) in self.ops.iter().enumerate() {
            if let Some(refusal) = op.refusal(&current) {
                match on_refusal {
                    OnRefusal::Fail => {
                        return Err(OperationError::Refused {
                            op: op.name(),
                            refusal,
                        });
                    }
                    OnRefusal::Skip => {
                        skipped.push(SkippedStep {
                            step,
                            op: op.name(),
                            refusal,
                        });
                        continue;
                    }
                }
            }
            current = op.apply(current)?;
        }
        Ok(PipelineRun {
            image: current,
            skipped,
        })
    }
}

/// `Pipeline::new()` is the natural default (an empty pipeline is a no-op).
/// Required to satisfy clippy's `clippy::new_without_default` lint.
impl Default for Pipeline {
    fn default() -> Self {
        Pipeline::new()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use ::image::{DynamicImage, ImageFormat, RgbaImage};

    use super::*;
    use crate::image::Image;
    use crate::operation::{Identity, Invert, OperationError, OperationParams};

    /// A pipeline of border pads, one per fill, built from recipe text.
    fn pads(fills: &[&str]) -> Pipeline {
        let mut toml = String::from("version = \"2\"\n");
        for fill in fills {
            toml.push_str(&format!(
                "\n[[step]]\nop = \"pad\"\nborder = 0.25\ncolor = \"{fill}\"\n"
            ));
        }
        crate::recipe::Recipe::from_toml(&toml)
            .expect("recipe parses")
            .build_pipeline(&crate::operation::OperationRegistry::with_builtins())
            .expect("pipeline builds")
    }

    #[test]
    fn unstorable_alpha_names_the_first_step() {
        let two = pads(&["#ff000000", "#ff000080"]);
        // GIF stores alpha 00, so the second pad is the first one refused.
        let gif = two.unstorable_alpha(ImageFormat::Gif).expect("refused");
        assert_eq!((gif.step, gif.op), (1, "pad"));
        assert_eq!(gif.introduced.written, "#ff000080");
        // JPEG stores neither, so the first one is.
        let jpeg = two.unstorable_alpha(ImageFormat::Jpeg).expect("refused");
        assert_eq!(jpeg.step, 0);
        assert_eq!(jpeg.introduced.written, "#ff000000");

        let opaque_then_clear = pads(&["#ffffff", "transparent"]);
        let jpeg = opaque_then_clear
            .unstorable_alpha(ImageFormat::Jpeg)
            .expect("refused");
        assert_eq!(jpeg.step, 1);
        assert!(opaque_then_clear
            .unstorable_alpha(ImageFormat::Gif)
            .is_none());
        assert!(opaque_then_clear
            .unstorable_alpha(ImageFormat::Png)
            .is_none());

        assert!(pads(&["#ffffff"])
            .unstorable_alpha(ImageFormat::Jpeg)
            .is_none());
        assert!(Pipeline::new()
            .unstorable_alpha(ImageFormat::Jpeg)
            .is_none());
    }

    #[test]
    fn introduces_alpha_is_any_translucent_step() {
        assert!(pads(&["#ffffff", "#ff000080"]).introduces_alpha());
        assert!(pads(&["transparent"]).introduces_alpha());
        assert!(!pads(&["#ffffff", "#123456ff"]).introduces_alpha());
        assert!(!Pipeline::new().introduces_alpha());
    }

    /// Build a small in-memory RGBA `Image` for testing.
    fn make_image(w: u32, h: u32) -> Image {
        let buf = RgbaImage::from_fn(w, h, |x, y| {
            ::image::Rgba([(x * 10 + 5) as u8, (y * 10 + 5) as u8, 50, 200])
        });
        Image::from_parts(DynamicImage::ImageRgba8(buf), ImageFormat::Png, None)
    }

    // ── test-only helper: records application order ───────────────────────

    /// Records its label in a shared `Vec` on every `apply` — lets tests
    /// verify that the fold runs ops in the correct order.
    struct RecordOrder {
        label: &'static str,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Operation for RecordOrder {
        fn name(&self) -> &'static str {
            self.label
        }

        fn params(&self) -> OperationParams {
            OperationParams::empty()
        }

        fn apply(&self, img: Image) -> Result<Image, OperationError> {
            self.log.borrow_mut().push(self.label);
            Ok(img)
        }
    }

    // ── test-only helper: always fails ───────────────────────────────────

    /// Always returns `OperationError::Apply` — lets tests verify that a
    /// failing op halts the pipeline and propagates the typed error.
    struct AlwaysFails;

    impl Operation for AlwaysFails {
        fn name(&self) -> &'static str {
            "always_fails"
        }

        fn params(&self) -> OperationParams {
            OperationParams::empty()
        }

        fn apply(&self, _img: Image) -> Result<Image, OperationError> {
            Err(OperationError::Apply {
                op: "always_fails",
                reason: "intentional test failure".to_string(),
            })
        }
    }

    // ── test-only helper: a mark that may decline ───────────────────────

    /// Stands in for a watermark: when `refuse` is set it declines every
    /// image (and `apply` refuses too, as the real op does); otherwise it
    /// passes the image through untouched.
    struct Mark {
        refuse: bool,
    }

    fn mark_refusal() -> crate::operation::Refusal {
        crate::operation::Refusal::DoesNotFit {
            mark: (50, 10),
            margin: 0,
            image: (3, 3),
        }
    }

    impl Operation for Mark {
        fn name(&self) -> &'static str {
            "watermark"
        }

        fn params(&self) -> OperationParams {
            OperationParams::empty()
        }

        fn refusal(&self, _img: &Image) -> Option<crate::operation::Refusal> {
            self.refuse.then(mark_refusal)
        }

        fn apply(&self, img: Image) -> Result<Image, OperationError> {
            match self.refusal(&img) {
                Some(refusal) => Err(OperationError::Refused {
                    op: "watermark",
                    refusal,
                }),
                None => Ok(img),
            }
        }
    }

    /// `[invert, mark, invert]`: the image comes back unchanged whenever the
    /// mark is skipped or passes through.
    fn invert_mark_invert(refuse: bool) -> Pipeline {
        Pipeline::new()
            .push(Box::new(Invert))
            .push(Box::new(Mark { refuse }))
            .push(Box::new(Invert))
    }

    #[test]
    fn run_fails_on_a_refusal_and_stops() {
        for result in [
            invert_mark_invert(true).run(make_image(3, 3)),
            invert_mark_invert(true)
                .run_with(make_image(3, 3), OnRefusal::Fail)
                .map(|run| run.image),
        ] {
            match result {
                Err(OperationError::Refused {
                    op: "watermark",
                    refusal,
                }) => assert_eq!(refusal, mark_refusal()),
                other => panic!("expected Refused, got {:?}", other.map(|_| "an image")),
            }
        }
    }

    #[test]
    fn run_with_skip_skips_the_step_and_runs_the_rest() {
        let img = make_image(3, 3);
        let original_raw = img.pixels().to_rgba8().into_raw();
        let run = invert_mark_invert(true)
            .run_with(img, OnRefusal::Skip)
            .expect("a skipped step is not an error");
        // Both inverts ran, so the image is back where it started.
        assert_eq!(run.image.pixels().to_rgba8().into_raw(), original_raw);
        assert_eq!(
            run.skipped,
            vec![SkippedStep {
                step: 1,
                op: "watermark",
                refusal: mark_refusal(),
            }]
        );
    }

    #[test]
    fn policies_agree_when_nothing_refuses() {
        let original_raw = make_image(3, 3).pixels().to_rgba8().into_raw();
        let plain = invert_mark_invert(false).run(make_image(3, 3)).unwrap();
        assert_eq!(plain.pixels().to_rgba8().into_raw(), original_raw);
        for policy in [OnRefusal::Fail, OnRefusal::Skip] {
            let run = invert_mark_invert(false)
                .run_with(make_image(3, 3), policy)
                .unwrap();
            assert_eq!(run.image.pixels().to_rgba8().into_raw(), original_raw);
            assert!(run.skipped.is_empty());
        }
    }

    // ── unit tests ───────────────────────────────────────────────────────

    #[test]
    fn new_pipeline_is_empty() {
        let p = Pipeline::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn push_increments_len() {
        let p = Pipeline::new()
            .push(Box::new(Identity))
            .push(Box::new(Invert));
        assert_eq!(p.len(), 2);
        assert!(!p.is_empty());
    }

    #[test]
    fn empty_pipeline_returns_image_unchanged() {
        let img = make_image(3, 3);
        let original_raw = img.pixels().to_rgba8().into_raw();
        let result = Pipeline::new().run(img).unwrap();
        assert_eq!(result.pixels().to_rgba8().into_raw(), original_raw);
    }

    #[test]
    fn single_identity_returns_equal_pixels() {
        let img = make_image(4, 4);
        let original_raw = img.pixels().to_rgba8().into_raw();
        let result = Pipeline::new().push(Box::new(Identity)).run(img).unwrap();
        assert_eq!(result.pixels().to_rgba8().into_raw(), original_raw);
    }

    #[test]
    fn double_invert_round_trips() {
        let img = make_image(3, 3);
        let original_raw = img.pixels().to_rgba8().into_raw();
        let result = Pipeline::new()
            .push(Box::new(Invert))
            .push(Box::new(Invert))
            .run(img)
            .unwrap();
        assert_eq!(result.pixels().to_rgba8().into_raw(), original_raw);
    }

    #[test]
    fn order_is_preserved() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
        let img = make_image(2, 2);
        let result = Pipeline::new()
            .push(Box::new(RecordOrder {
                label: "A",
                log: Rc::clone(&log),
            }))
            .push(Box::new(RecordOrder {
                label: "B",
                log: Rc::clone(&log),
            }))
            .run(img);
        assert!(result.is_ok());
        assert_eq!(*log.borrow(), vec!["A", "B"]);
    }

    #[test]
    fn failing_op_halts_and_propagates() {
        let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
        let img = make_image(2, 2);
        let result = Pipeline::new()
            .push(Box::new(Identity))
            .push(Box::new(AlwaysFails))
            .push(Box::new(RecordOrder {
                label: "after",
                log: Rc::clone(&log),
            }))
            .run(img);
        // The error propagates as the typed Apply variant.
        assert!(matches!(
            result,
            Err(OperationError::Apply {
                op: "always_fails",
                ..
            })
        ));
        // The op after the failing one must NOT have run.
        assert!(!log.borrow().contains(&"after"));
    }
}
