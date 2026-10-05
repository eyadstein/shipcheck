//! Small helpers for working with source text line by line.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

/// A lazily compiled built-in pattern.
pub(crate) type Pattern = LazyLock<Regex>;

/// Compiles a built-in pattern. Patterns are fixed in the source and covered by tests.
pub(crate) fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("built-in pattern is valid")
}

/// Converts a zero-based line index into a one-based line number.
pub(crate) fn number(index: usize) -> u32 {
    u32::try_from(index + 1).unwrap_or(u32::MAX)
}

/// One-based line number of a byte offset.
pub(crate) fn line_at(text: &str, offset: usize) -> u32 {
    let before = text
        .get(..offset)
        .map_or(0, |head| head.matches('\n').count());
    number(before)
}

/// First line matching the pattern.
pub(crate) fn first_line(text: &str, pattern: &Pattern) -> Option<u32> {
    text.lines()
        .position(|line| pattern.is_match(line))
        .map(number)
}

/// First line where every pattern matches.
pub(crate) fn first_line_all(text: &str, patterns: &[&Pattern]) -> Option<u32> {
    text.lines()
        .position(|line| patterns.iter().all(|pattern| pattern.is_match(line)))
        .map(number)
}

/// First line matching `anchor` that also has every partner pattern
/// matching somewhere within `window` lines on either side.
pub(crate) fn first_near(
    text: &str,
    anchor: &Pattern,
    partners: &[&Pattern],
    window: usize,
) -> Option<u32> {
    let lines: Vec<&str> = text.lines().collect();
    let hit = lines.iter().enumerate().find(|&(index, line)| {
        anchor.is_match(line) && {
            let from = index.saturating_sub(window);
            let to = (index + window + 1).min(lines.len());
            partners
                .iter()
                .all(|partner| lines[from..to].iter().any(|near| partner.is_match(near)))
        }
    });
    hit.map(|(index, _)| number(index))
}

/// Lowercase file extension, or an empty string.
pub(crate) fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

/// Whether the file holds page markup or components rather than plain scripts.
pub(crate) fn is_markup(path: &str) -> bool {
    matches!(
        extension(path).as_str(),
        "html" | "htm" | "jsx" | "tsx" | "vue" | "svelte" | "astro" | "mdx"
    )
}
