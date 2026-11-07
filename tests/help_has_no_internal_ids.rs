//! No text a user can see carries an internal spec or decision id.
//!
//! Four nets:
//! - every help page the parser can print (`-h` and `--help`, root, groups, leaves),
//!   rendered through the parser as the binary does, plus a check that the walk
//!   reaches every page listed in the rendered `Commands:` sections;
//! - every shell-completion script;
//! - every string literal in production source (comments and `#[cfg(test)]` items
//!   excluded), which sees runtime-formatted messages no help walk can;
//! - the browser demo's visible text, and every file the binary embeds.
//!
//! The id prefixes are assembled with `concat!` so this file never matches its own
//! pattern, should a later scan include `tests/`.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::error::ErrorKind;
use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use crustyimg::cli::Cli;

const SPEC_PREFIX: &str = concat!("SP", "EC-");
const DEC_PREFIX: &str = concat!("DE", "C-");

/// Whether `text` holds an id: a prefix followed by an ASCII digit.
fn has_id(text: &str) -> bool {
    [SPEC_PREFIX, DEC_PREFIX].iter().any(|prefix| {
        text.match_indices(prefix).any(|(at, _)| {
            text[at + prefix.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        })
    })
}

/// The lines of `text` (CRLF normalised) that hold an id, as `(1-based line, line)`.
fn id_lines(text: &str) -> Vec<(usize, String)> {
    text.replace("\r\n", "\n")
        .lines()
        .enumerate()
        .filter(|(_, line)| has_id(line))
        .map(|(i, line)| (i + 1, line.to_owned()))
        .collect()
}

// ── Help pages ──────────────────────────────────────────────────────────────────

/// Every command path of the real clap tree, root (`[]`) and groups included, hidden
/// commands and `help` excluded.
fn walk_paths() -> Vec<Vec<String>> {
    fn walk(cmd: &clap::Command, prefix: &[String], out: &mut Vec<Vec<String>>) {
        out.push(prefix.to_vec());
        for sub in cmd.get_subcommands() {
            if sub.get_name() == "help" || sub.is_hide_set() {
                continue;
            }
            let mut path = prefix.to_vec();
            path.push(sub.get_name().to_owned());
            walk(sub, &path, out);
        }
    }
    let mut out = Vec::new();
    walk(&Cli::command(), &[], &mut out);
    out
}

/// The page the binary prints for `crustyimg <path> <flag>`. Global flags reach a
/// subcommand's page only when the parser builds it, so the page is rendered from the
/// parse error, never from `render_help()` on a sub-`Command`.
fn rendered_page(path: &[String], flag: &str) -> String {
    let mut argv = vec!["crustyimg".to_owned()];
    argv.extend(path.iter().cloned());
    argv.push(flag.to_owned());
    let err = Cli::command()
        .try_get_matches_from(&argv)
        .expect_err("a help flag should stop the parse");
    assert_eq!(
        err.kind(),
        ErrorKind::DisplayHelp,
        "`{}` did not display help",
        argv.join(" ")
    );
    err.render().to_string().replace("\r\n", "\n")
}

/// The command names in a rendered page's `Commands:` section (`help` skipped).
fn listed_commands(page: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;
    for line in page.lines() {
        if line.trim_end() == "Commands:" {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        // An entry is indented by exactly two spaces; a wrapped description by more.
        if let Some(rest) = line.strip_prefix("  ") {
            if rest.starts_with(|c: char| !c.is_whitespace()) {
                let name = rest.split_whitespace().next().unwrap_or_default();
                if name != "help" {
                    names.push(name.to_owned());
                }
            }
        }
    }
    names
}

/// The command paths reachable by reading the rendered pages alone.
fn derived_paths() -> BTreeSet<Vec<String>> {
    fn visit(path: Vec<String>, out: &mut BTreeSet<Vec<String>>) {
        let page = rendered_page(&path, "--help");
        out.insert(path.clone());
        for name in listed_commands(&page) {
            let mut child = path.clone();
            child.push(name);
            visit(child, out);
        }
    }
    let mut out = BTreeSet::new();
    visit(Vec::new(), &mut out);
    out
}

#[test]
fn no_help_page_shows_an_internal_id() {
    let paths = walk_paths();
    assert!(!paths.is_empty(), "the walk found no command");
    let mut pages = 0;
    let mut hits = Vec::new();
    for path in &paths {
        for flag in ["-h", "--help"] {
            pages += 1;
            for (_, line) in id_lines(&rendered_page(path, flag)) {
                hits.push(format!(
                    "crustyimg {} {flag}: {}",
                    path.join(" "),
                    line.trim()
                ));
            }
        }
    }
    println!(
        "help walk: {} command paths, {pages} pages, {} id lines",
        paths.len(),
        hits.len()
    );
    assert!(
        hits.is_empty(),
        "{} id lines over {pages} pages:\n{}",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn the_help_walk_visits_every_page_the_user_can_reach() {
    let walked: BTreeSet<Vec<String>> = walk_paths().into_iter().collect();
    let derived = derived_paths();
    println!(
        "help pages: {} paths derived from the rendered text, {} pages",
        derived.len(),
        derived.len() * 2
    );
    assert!(
        derived.iter().any(|p| p.len() == 2),
        "the rendered pages list no nested command (e.g. `meta set`): {derived:?}"
    );
    assert_eq!(
        walked, derived,
        "the walk and the rendered Commands: sections disagree"
    );
}

// ── Completion scripts ──────────────────────────────────────────────────────────

#[test]
fn no_completion_script_shows_an_internal_id() {
    let shells = Shell::value_variants();
    let mut seen = 0;
    let mut hits = Vec::new();
    for shell in shells {
        let mut buf = Vec::new();
        clap_complete::generate(*shell, &mut Cli::command(), "crustyimg", &mut buf);
        let script = String::from_utf8_lossy(&buf).replace("\r\n", "\n");
        assert!(
            !script.is_empty() && script.contains("crustyimg"),
            "{shell}: empty or foreign completion script"
        );
        seen += 1;
        let lines = id_lines(&script);
        if !lines.is_empty() {
            hits.push(format!(
                "{shell}: {} id lines, first: {}",
                lines.len(),
                lines[0].1.trim()
            ));
        }
    }
    assert_eq!(seen, shells.len());
    assert!(
        hits.is_empty(),
        "completion scripts carry ids:\n{}",
        hits.join("\n")
    );
}

// ── Production string literals ──────────────────────────────────────────────────

/// Every string literal in Rust source `text`, as `(line it starts on, contents)`,
/// skipping comments and every `#[cfg(test)]` item written as a block.
///
/// A character lexer: line comments, nested block comments, strings (`\` escapes the
/// next character), raw strings (`r#"…"#`, also `br`), char literals (`'"'`, `'\''`)
/// told apart from lifetimes. `#[cfg(test)]` arms a skip; the next `{` at code level
/// opens a skipped region up to its matching `}`; a `;` first disarms it (an
/// attribute on an item with no block). Code after a skipped region is scanned.
fn production_literals(text: &str) -> Vec<(usize, String)> {
    const CFG_TEST: &str = "#[cfg(test)]";
    let src: Vec<char> = text.replace("\r\n", "\n").chars().collect();
    let at = |k: usize| src.get(k).copied();
    let is_ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    let mut out = Vec::new();
    let mut line = 1;
    let mut depth = 0usize;
    let mut armed = false;
    let mut skip_at: Option<usize> = None;
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        let next = at(i + 1);
        let prev = if i == 0 { None } else { at(i - 1) };

        // Line comment: skip to the newline (left for the main loop to count).
        if c == '/' && next == Some('/') {
            while i < src.len() && src[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Block comment, nested.
        if c == '/' && next == Some('*') {
            let mut level = 1;
            i += 2;
            while i < src.len() && level > 0 {
                if src[i] == '/' && at(i + 1) == Some('*') {
                    level += 1;
                    i += 2;
                } else if src[i] == '*' && at(i + 1) == Some('/') {
                    level -= 1;
                    i += 2;
                } else {
                    if src[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            continue;
        }
        // Raw string: `r#*"` or `br#*"`, not inside an identifier.
        let raw_r = if !is_ident(prev) && c == 'r' {
            Some(i)
        } else if !is_ident(prev) && c == 'b' && next == Some('r') {
            Some(i + 1)
        } else {
            None
        };
        if let Some(r) = raw_r {
            let mut j = r + 1;
            let mut hashes = 0;
            while at(j) == Some('#') {
                hashes += 1;
                j += 1;
            }
            if at(j) == Some('"') {
                let start_line = line;
                let mut body = String::new();
                j += 1;
                loop {
                    match at(j) {
                        None => break,
                        Some('"') if (1..=hashes).all(|h| at(j + h) == Some('#')) => {
                            j += 1 + hashes;
                            break;
                        }
                        Some(ch) => {
                            if ch == '\n' {
                                line += 1;
                            }
                            body.push(ch);
                            j += 1;
                        }
                    }
                }
                if skip_at.is_none() {
                    out.push((start_line, body));
                }
                i = j;
                continue;
            }
        }
        // String (plain or byte).
        if c == '"' {
            let start_line = line;
            let mut body = String::new();
            i += 1;
            while i < src.len() {
                let ch = src[i];
                if ch == '\\' {
                    body.push(ch);
                    if let Some(esc) = at(i + 1) {
                        if esc == '\n' {
                            line += 1;
                        }
                        body.push(esc);
                    }
                    i += 2;
                    continue;
                }
                i += 1;
                if ch == '"' {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                }
                body.push(ch);
            }
            if skip_at.is_none() {
                out.push((start_line, body));
            }
            continue;
        }
        // Char literal or lifetime.
        if c == '\'' {
            if next == Some('\\') {
                // Escaped char literal: skip the quote, the backslash and the escaped
                // character, then up to the closing quote (`'\u{..}'`).
                i += 3;
                while i < src.len() && src[i] != '\'' {
                    i += 1;
                }
                i += 1;
            } else if at(i + 2) == Some('\'') {
                i += 3;
            } else {
                i += 1; // a lifetime
            }
            continue;
        }
        if c == '#'
            && src[i..]
                .iter()
                .take(CFG_TEST.len())
                .copied()
                .eq(CFG_TEST.chars())
        {
            if skip_at.is_none() {
                armed = true;
            }
            i += CFG_TEST.len();
            continue;
        }
        match c {
            '{' => {
                depth += 1;
                if armed {
                    skip_at = Some(depth);
                    armed = false;
                }
            }
            '}' => {
                if skip_at == Some(depth) {
                    skip_at = None;
                }
                depth = depth.saturating_sub(1);
            }
            ';' => armed = false,
            '\n' => line += 1,
            _ => {}
        }
        i += 1;
    }
    out
}

/// Every `.rs` file under `dir`, sorted.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A path relative to the crate root, with `/` separators on every OS.
fn shown(path: &Path) -> String {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn no_production_string_literal_shows_an_internal_id() {
    let files = rust_files(&manifest_dir().join("src"));
    let names: Vec<String> = files.iter().map(|f| shown(f)).collect();
    assert!(!files.is_empty(), "no source files found");
    for must in ["src/wasm.rs", "src/cli/mod.rs"] {
        assert!(
            names.iter().any(|n| n == must),
            "{must} not scanned: {names:?}"
        );
    }
    let mut literals = 0;
    let mut hits = Vec::new();
    for (file, name) in files.iter().zip(&names) {
        let text = std::fs::read_to_string(file).unwrap();
        for (line, literal) in production_literals(&text) {
            literals += 1;
            if has_id(&literal) {
                hits.push(format!("{name}:{line}: {literal:?}"));
            }
        }
    }
    println!(
        "literal scan: {} files, {literals} literals, {} with an id",
        files.len(),
        hits.len()
    );
    assert!(literals > 0, "the scan read no literal");
    assert!(
        hits.is_empty(),
        "{} production literals carry an id:\n{}",
        hits.len(),
        hits.join("\n")
    );
}

#[test]
fn the_literal_scanner_handles_the_hard_cases() {
    let id = concat!("SP", "EC-", "128");
    let id2 = concat!("DE", "C-", "065");
    // (name, source, literals expected to hold an id)
    let cases: Vec<(&str, String, usize)> = vec![
        (
            "an id after // inside a string",
            format!("let s = \"x // {id}\";\n"),
            1,
        ),
        (
            "a string inside a line comment",
            format!("// \"{id}\"\nfn f() {{}}\n"),
            0,
        ),
        (
            "a raw string holding // and a quote",
            format!("let s = r#\"a // {id} \"\"#;\n"),
            1,
        ),
        (
            "a quote char literal before a comment",
            format!("let q = '\"'; // {id}\nlet r = 1;\n"),
            0,
        ),
        (
            "a string inside a block comment",
            format!("/* \"{id}\" */\nfn f() {{}}\n"),
            0,
        ),
        (
            "a string inside a nested block comment",
            format!("/* /* */ \"{id}\" */\nfn f() {{}}\n"),
            0,
        ),
        ("an escaped quote", format!("let s = \"\\\"{id}\";\n"), 1),
        (
            "an id on a continuation line",
            format!("let s = \"line \\\n    ({id})\";\n"),
            1,
        ),
        (
            "lifetimes before a string",
            format!("fn f<'a>(x: &'a str) {{ \"{id}\"; }}\n"),
            1,
        ),
        (
            "an error attribute",
            format!("#[error(\"bad ({id})\")]\nstruct E;\n"),
            1,
        ),
        ("a raw byte string", format!("let b = br\"{id}\";\n"), 1),
        (
            "a byte char literal quote before a string",
            format!("let q = b'\"'; let s = \"{id}\";\n"),
            1,
        ),
    ];
    let mut wrong = Vec::new();
    for (name, src, want) in &cases {
        let got = production_literals(src)
            .iter()
            .filter(|(_, l)| has_id(l))
            .count();
        if got != *want {
            wrong.push(format!("{name}: want {want}, got {got} in {src:?}"));
        }
    }

    // A test module is skipped, and the code after it is still scanned.
    let src =
        format!("#[cfg(test)]\nmod t {{ fn g() {{ \"{id}\"; }} }}\nfn after() {{ \"{id2}\"; }}\n");
    let found: Vec<String> = production_literals(&src)
        .into_iter()
        .filter(|(_, l)| has_id(l))
        .map(|(_, l)| l)
        .collect();
    if found != [id2.to_owned()] {
        wrong.push(format!(
            "test module then code: want only {id2:?}, got {found:?}"
        ));
    }

    // A CRLF file reports the same lines as an LF file.
    let crlf = format!("fn f() {{}}\r\nlet s = \"{id}\";\r\n");
    let lines: Vec<usize> = production_literals(&crlf).iter().map(|(n, _)| *n).collect();
    if lines != [2] {
        wrong.push(format!("CRLF: want the literal on line 2, got {lines:?}"));
    }

    assert!(
        wrong.is_empty(),
        "scanner cases wrong:\n{}",
        wrong.join("\n")
    );
}

// ── The demo page and embedded text ─────────────────────────────────────────────

/// `text` with `/* … */`, `<!-- … -->` and `//`-to-end-of-line comments blanked out,
/// newlines kept so line numbers hold. `//` counts as a comment only at the start of
/// a line or after whitespace, so `https://` survives.
fn without_web_comments(text: &str) -> String {
    let src: Vec<char> = text.replace("\r\n", "\n").chars().collect();
    let starts = |i: usize, pat: &str| src[i..].iter().take(pat.len()).copied().eq(pat.chars());
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        let close = if starts(i, "/*") {
            Some("*/")
        } else if starts(i, "<!--") {
            Some("-->")
        } else {
            None
        };
        if let Some(close) = close {
            while i < src.len() && !starts(i, close) {
                if src[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            i = (i + close.len()).min(src.len());
            continue;
        }
        let after_space = i == 0 || src[i - 1].is_whitespace();
        if starts(i, "//") && after_space {
            while i < src.len() && src[i] != '\n' {
                i += 1;
            }
            continue;
        }
        out.push(src[i]);
        i += 1;
    }
    out
}

/// The files named by `include_str!("…")` in `src/**/*.rs`, resolved against the
/// naming file's folder.
fn embedded_files() -> Vec<PathBuf> {
    const MACRO: &str = "include_str!(\"";
    let mut out = BTreeSet::new();
    for file in rust_files(&manifest_dir().join("src")) {
        let text = std::fs::read_to_string(&file).unwrap();
        for (at, _) in text.match_indices(MACRO) {
            let rest = &text[at + MACRO.len()..];
            if let Some(end) = rest.find('"') {
                out.insert(file.parent().unwrap().join(&rest[..end]));
            }
        }
    }
    out.into_iter().collect()
}

#[test]
fn the_demo_page_and_embedded_text_show_no_internal_id() {
    let demo = manifest_dir().join("demo");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&demo)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "js" || e == "html"))
        .collect();
    pages.sort();
    assert!(pages.len() >= 2, "demo glob found {} files", pages.len());

    let embedded = embedded_files();
    assert!(!embedded.is_empty(), "no include_str! file found");

    let mut hits = Vec::new();
    for page in &pages {
        let text = std::fs::read_to_string(page).unwrap();
        for (line, content) in id_lines(&without_web_comments(&text)) {
            hits.push(format!("{}:{line}: {}", shown(page), content.trim()));
        }
    }
    for file in &embedded {
        let text = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("cannot read embedded {}: {e}", file.display()));
        for (line, content) in id_lines(&text) {
            hits.push(format!("{}:{line}: {}", shown(file), content.trim()));
        }
    }
    println!(
        "demo/embedded scan: {} demo files, {} embedded files, {} id lines",
        pages.len(),
        embedded.len(),
        hits.len()
    );
    assert!(hits.is_empty(), "visible ids:\n{}", hits.join("\n"));
}
