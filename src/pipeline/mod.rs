//! Decode-once pipeline executor (DEC-002).
//!
//! `Pipeline` owns an ordered list of `Operation`s and folds them over a
//! single in-memory `Image` without any per-operation disk round-trips
//! (constraint `decode-once-no-per-op-disk`). If any operation fails, the
//! `Err` is returned immediately and later operations do not run.
//!
//! Layering: this module imports `crate::image::Image` and
//! `crate::operation::{Operation, OperationError}`. It must NOT depend on
//! `clap`, `recipe`, `source`, `sink`, `std::fs`, or `std::path`.

use crate::image::Image;
use crate::operation::{Operation, OperationError};

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
    pub fn run(&self, img: Image) -> Result<Image, OperationError> {
        let mut current = img;
        for op in &self.ops {
            current = op.apply(current)?;
        }
        Ok(current)
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
