//! Bundled recipe registry (SPEC-085).
//!
//! A small in-binary catalogue of shipped recipes (`web`, `gallery`, `product`)
//! so `apply --recipe <name>` runs a maintained flow without a file on disk, and
//! the `web` verb == `apply --recipe web`. The TOMLs live in the repo's `recipes/`
//! directory and are embedded via [`include_str!`], so they are the exact same
//! recipe format the file path and the wasm build use (DEC-005) — just shipped in
//! the binary.
//!
//! ## Resolution precedence (used by `run_apply` / `load_recipe`)
//!
//! A **real file on disk always wins**: `apply --recipe <arg>` treats `<arg>` as a
//! path first, and only falls back to this registry when no such file exists. So a
//! local `web.toml` (or a file literally named `web`) is unambiguous — it shadows
//! the bundled `web`. This keeps every existing file-path recipe working exactly as
//! before and makes the bundled names a convenience layer on top, never an override.
//!
//! ## The terminal `optimize` step
//!
//! Each bundled flow ends with an `op = "optimize"` step — the reserved terminal
//! marker that makes the recipe encode via the fast AVIF-aware decision
//! (`Mode::Fast`, beats-the-downscaled-image, score) instead of a plain
//! format-preserving sink write. Because these flows downscale first, an
//! already-small source above the bound can re-encode larger than the original;
//! that is reported honestly (SPEC-090, DEC-075), not hidden. It is handled in the
//! CLI's apply path, not the operation registry, so a bundled recipe modernizes
//! format the way the `web` verb does.

/// One bundled recipe: its lookup `name` and the embedded TOML `text`.
struct Bundled {
    name: &'static str,
    text: &'static str,
}

/// The shipped recipes, embedded at compile time from `recipes/`.
///
/// `include_str!` paths are relative to THIS file (`src/recipe/bundled.rs`), so the
/// repo-root `recipes/` dir is two levels up.
const BUNDLED: &[Bundled] = &[
    Bundled {
        name: "web",
        text: include_str!("../../recipes/web.toml"),
    },
    Bundled {
        name: "gallery",
        text: include_str!("../../recipes/gallery.toml"),
    },
    Bundled {
        name: "product",
        text: include_str!("../../recipes/product.toml"),
    },
];

/// Resolve a bundled recipe NAME to its embedded TOML text, or `None` if no
/// bundled recipe carries that name. Case-sensitive, exact match.
pub fn resolve(name: &str) -> Option<&'static str> {
    BUNDLED.iter().find(|b| b.name == name).map(|b| b.text)
}

/// The bundled recipe names, in catalogue order — for `--help` and diagnostics.
pub fn names() -> Vec<&'static str> {
    BUNDLED.iter().map(|b| b.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::Recipe;

    /// Every bundled recipe resolves to a NAME and parses as a valid `Recipe`
    /// (Failing Test `bundled_recipe_names_resolve`). The pixel steps (everything
    /// before the terminal `optimize`) must also build a real pipeline — the
    /// terminal `optimize` marker is not a registry op, so it is stripped first,
    /// mirroring the CLI apply path.
    #[test]
    fn bundled_recipe_names_resolve() {
        use crate::operation::OperationRegistry;
        let registry = OperationRegistry::with_builtins();
        for name in ["web", "gallery", "product"] {
            let text = resolve(name).unwrap_or_else(|| panic!("{name} must resolve"));
            let recipe = Recipe::from_toml(text)
                .unwrap_or_else(|e| panic!("{name} must parse as a Recipe: {e}"));
            assert_eq!(
                recipe.name.as_deref(),
                Some(name),
                "recipe carries its name"
            );

            // The last step is the terminal optimize marker.
            let last = recipe.steps.last().expect("recipe has steps");
            assert_eq!(last.op, "optimize", "{name} ends with the optimize marker");

            // The preceding pixel steps build a working pipeline.
            let mut pixel = recipe.clone();
            pixel.steps.pop();
            pixel
                .build_pipeline(&registry)
                .unwrap_or_else(|e| panic!("{name} pixel pipeline must build: {e}"));
        }
    }

    /// `names()` lists the three shipped flows in catalogue order.
    #[test]
    fn names_lists_bundled_flows() {
        assert_eq!(names(), vec!["web", "gallery", "product"]);
    }

    /// An unknown name resolves to `None` (so the caller can fall through to a
    /// not-found error), never a panic.
    #[test]
    fn unknown_name_resolves_none() {
        assert!(resolve("does-not-exist").is_none());
        assert!(resolve("").is_none());
    }
}
