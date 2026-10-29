//! The shipped builds must keep SVG input.
//!
//! `svg` is an optional feature, and every SVG test is gated on it. So if `svg`
//! were dropped from the default features, or from the feature list the shipped
//! wasm package is built with, those tests would simply not compile in and every
//! suite would stay green while SVG input vanished from the release. This test is
//! not gated: it reads the two feature lists from source and fails if either loses
//! `svg` (or `avif`, which is in both lists for the same reason).

/// The quoted names inside the brackets of the one line starting with `prefix`.
fn feature_names(source: &str, prefix: &str, what: &str) -> Vec<String> {
    // Normalize CRLF so a Windows checkout parses the same way.
    let source = source.replace("\r\n", "\n");
    let lines: Vec<&str> = source
        .lines()
        .filter(|l| l.trim_start().starts_with(prefix))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one `{prefix}` line in {what}, found {lines:?}"
    );
    // `default = ["display", "svg"]`: every other `"`-split piece is a name.
    lines[0]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

#[test]
fn default_features_keep_svg_and_avif() {
    let names = feature_names(include_str!("../Cargo.toml"), "default = [", "Cargo.toml");
    for f in ["svg", "avif"] {
        assert!(
            names.iter().any(|n| n == f),
            "`{f}` missing from Cargo.toml default features: {names:?}"
        );
    }
}

#[test]
fn shipped_wasm_features_keep_svg_and_avif() {
    // `_wasm_features` is a single quoted string such as
    // "--no-default-features --features avif,svg"; take the part after `--features`.
    let source = include_str!("../justfile").replace("\r\n", "\n");
    let line = source
        .lines()
        .find(|l| l.starts_with("_wasm_features :="))
        .expect("justfile defines `_wasm_features :=`");
    let value = line
        .split('"')
        .nth(1)
        .expect("`_wasm_features` is a quoted string");
    let listed = value
        .split_once("--features")
        .map(|(_, rest)| rest.trim())
        .expect("`_wasm_features` names a `--features` list");
    let names: Vec<&str> = listed.split([',', ' ']).filter(|s| !s.is_empty()).collect();
    for f in ["svg", "avif"] {
        assert!(
            names.contains(&f),
            "`{f}` missing from the justfile's `_wasm_features`: {value:?}"
        );
    }
}
