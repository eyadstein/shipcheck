//! Finds which typefaces a project actually uses.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::lines::{line_at, re, Pattern};
use crate::SourceFile;

/// Typeface name to the place it was first seen.
pub(crate) type Places = BTreeMap<String, (String, u32)>;

/// System and generic names that say nothing about a project's own typography.
const GENERIC: &[&str] = &[
    "sans-serif",
    "serif",
    "monospace",
    "system-ui",
    "ui-sans-serif",
    "ui-serif",
    "ui-monospace",
    "cursive",
    "fantasy",
    "inherit",
    "initial",
    "unset",
    "blinkmacsystemfont",
    "segoe ui",
    "helvetica neue",
    "helvetica",
    "arial",
];

static DECLARATION: Pattern = LazyLock::new(|| re(r"(?i)font-family\s*:\s*([^;}\n]+)"));
static GOOGLE_URL: Pattern = LazyLock::new(|| re(r#"(?i)fonts\.googleapis\.com[^\s"')>]*"#));
static FAMILY_PARAM: Pattern = LazyLock::new(|| re(r"(?i)family=([A-Za-z0-9+%]+)"));
static NEXT_FONT: Pattern =
    LazyLock::new(|| re(r#"import\s*\{([^}]*)\}\s*from\s*['"]next/font/google['"]"#));
static TAILWIND: Pattern =
    LazyLock::new(|| re(r#"\b(?:sans|serif|heading|display|body)\s*:\s*\[\s*['"]([^'"]+)['"]"#));

/// Collects the primary typeface of every font declaration in the project.
pub(crate) fn used(files: &[SourceFile]) -> Places {
    let mut places = Places::new();
    for file in files {
        collect(file, &mut places);
    }
    places
}

fn collect(file: &SourceFile, places: &mut Places) {
    let text = file.text.as_str();
    let mut add = |name: &str, offset: usize| {
        let key = normalize(name);
        if !key.is_empty() {
            places
                .entry(key)
                .or_insert_with(|| (file.path.clone(), line_at(text, offset)));
        }
    };
    for caps in DECLARATION.captures_iter(text) {
        let first = caps[1].split(',').next().unwrap_or_default();
        add(first, caps.get(0).as_ref().map_or(0, regex::Match::start));
    }
    for url in GOOGLE_URL.find_iter(text) {
        for param in FAMILY_PARAM.captures_iter(url.as_str()) {
            add(&param[1], url.start());
        }
    }
    for caps in NEXT_FONT.captures_iter(text) {
        let offset = caps.get(0).as_ref().map_or(0, regex::Match::start);
        for name in caps[1].split(',') {
            add(name, offset);
        }
    }
    for caps in TAILWIND.captures_iter(text) {
        add(
            &caps[1],
            caps.get(0).as_ref().map_or(0, regex::Match::start),
        );
    }
}

/// Lowercases a typeface name and returns an empty string for generic or dynamic values.
fn normalize(name: &str) -> String {
    let spaced = name.replace(['+', '_'], " ").replace("%20", " ");
    let cleaned = spaced
        .trim()
        .trim_matches(['"', '\''])
        .trim()
        .to_ascii_lowercase();
    if cleaned.contains('(') || cleaned.starts_with('-') || GENERIC.contains(&cleaned.as_str()) {
        String::new()
    } else {
        cleaned
    }
}
