//! The originality index: how much of a design is framework defaults.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;
use shipcheck_core::{Category, Finding, Severity};

use crate::checks::{SHADCN_BACKGROUND, SHADCN_PRIMARY};
use crate::fonts;
use crate::SourceFile;

const PALETTE: &str = "Palette";
const TYPOGRAPHY: &str = "Typography";
const TOKENS: &str = "Framework tokens";

const PALETTE_WEIGHT: u32 = 50;
const TYPOGRAPHY_WEIGHT: u32 = 30;
const TOKENS_WEIGHT: u32 = 20;

/// Fewer distinct colors than this say too little about a palette.
const MIN_COLORS: usize = 3;
/// How far a color channel may be from a known default and still count as that default.
const NEAR: u8 = 4;
/// Scores below this get a finding.
const LOW_SCORE: u32 = 40;
/// How many items the text report lists per group.
const SHOWN: usize = 12;

type Rgb = [u8; 3];

/// Colors that ship with popular frameworks, as six digit hex.
const DEFAULT_HEX: &[&str] = &[
    // Tailwind gray
    "f9fafb", "f3f4f6", "e5e7eb", "d1d5db", "9ca3af", "6b7280", "4b5563", "374151", "1f2937",
    "111827", // Tailwind slate
    "f8fafc", "f1f5f9", "e2e8f0", "cbd5e1", "94a3b8", "64748b", "475569", "334155", "1e293b",
    "0f172a", // Tailwind 500 shades
    "ef4444", "f97316", "f59e0b", "eab308", "84cc16", "22c55e", "10b981", "14b8a6", "06b6d4",
    "0ea5e9", "3b82f6", "6366f1", "8b5cf6", "a855f7", "d946ef", "ec4899", "f43f5e",
    // Tailwind 600 shades
    "dc2626", "ea580c", "d97706", "ca8a04", "65a30d", "16a34a", "059669", "0d9488", "0891b2",
    "0284c7", "2563eb", "4f46e5", "7c3aed", "9333ea", "c026d3", "db2777", "e11d48",
    // Tailwind neutral
    "fafafa", "f5f5f5", "e5e5e5", "d4d4d4", "a3a3a3", "737373", "525252", "404040", "262626",
    "171717", // Bootstrap
    "0d6efd", "6c757d", "198754", "dc3545", "ffc107", "0dcaf0", "212529", "f8f9fa", "007bff",
    "28a745", "17a2b8", "343a40", // Material
    "6200ee", "3700b3", "03dac6", "018786", "b00020", "3f51b5", "ff4081",
];

/// Tailwind color names that come with the framework.
const STOCK_COLORS: &[&str] = &[
    "slate", "gray", "zinc", "neutral", "stone", "red", "orange", "amber", "yellow", "lime",
    "green", "emerald", "teal", "cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia",
    "pink", "rose",
];

/// Words that look like color names in a class but are not.
const NOT_COLORS: &[&str] = &["opacity", "offset", "inset", "spacing", "width", "x", "y"];

/// Typefaces that a very large share of projects pick.
const POPULAR_FONTS: &[&str] = &[
    "inter",
    "roboto",
    "open sans",
    "lato",
    "poppins",
    "montserrat",
    "nunito",
    "source sans pro",
    "source sans 3",
    "noto sans",
    "dm sans",
    "space grotesk",
    "instrument serif",
    "playfair display",
    "geist",
    "plus jakarta sans",
    "manrope",
    "outfit",
    "work sans",
    "raleway",
    "ubuntu",
    "fira sans",
    "fira code",
    "jetbrains mono",
];

static HEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)[:=(,'"]\s*#([0-9a-f]{3,8})\b"#).expect("hex pattern is valid")
});

static RGB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\brgba?\(\s*(\d{1,3})\s*[, ]\s*(\d{1,3})\s*[, ]\s*(\d{1,3})")
        .expect("rgb pattern is valid")
});

static TAILWIND: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:bg|text|border|ring|from|via|to|fill|stroke|divide|outline|decoration|accent|caret|shadow|placeholder)-([a-z]+)-(50|100|200|300|400|500|600|700|800|900|950)\b",
    )
    .expect("tailwind pattern is valid")
});

static FONT_VARIABLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)--font[a-z0-9-]*\s*:\s*([^;}\n]+)").expect("font variable pattern is valid")
});

/// One part of the index, such as the palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Dimension {
    pub name: &'static str,
    /// How many distinct things were found.
    pub total: usize,
    /// How many of them are framework or popular defaults.
    pub defaults: usize,
    /// How much this part counts towards the score.
    pub weight: u32,
    pub default_items: Vec<String>,
    pub own_items: Vec<String>,
}

impl Dimension {
    /// Share of the things found that are the project's own, from 0 to 100.
    #[must_use]
    pub fn own_percent(&self) -> u32 {
        if self.total == 0 {
            return 100;
        }
        let own = self.total.saturating_sub(self.defaults);
        u32::try_from(own * 100 / self.total).unwrap_or(100)
    }
}

/// How much of a design is the project's own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Originality {
    /// From 0 (all defaults) to 100 (all your own).
    pub score: u32,
    pub dimensions: Vec<Dimension>,
}

/// Measures the originality of a design. `None` when there is too little to judge.
#[must_use]
pub fn originality(files: &[SourceFile]) -> Option<Originality> {
    let dimensions: Vec<Dimension> = [
        palette_dimension(files),
        font_dimension(files),
        token_dimension(files),
    ]
    .into_iter()
    .flatten()
    .collect();
    if dimensions.is_empty() {
        return None;
    }
    let weight: u32 = dimensions.iter().map(|dimension| dimension.weight).sum();
    let earned: u32 = dimensions
        .iter()
        .map(|dimension| dimension.weight * dimension.own_percent())
        .sum();
    Some(Originality {
        score: earned / weight,
        dimensions,
    })
}

/// A finding when the palette was judged and the design is mostly defaults.
pub(crate) fn finding(files: &[SourceFile]) -> Option<Finding> {
    let index = originality(files)?;
    let judged_palette = index
        .dimensions
        .iter()
        .any(|dimension| dimension.name == PALETTE);
    if !judged_palette || index.score >= LOW_SCORE {
        return None;
    }
    Some(Finding {
        rule_id: "DES-301".to_owned(),
        category: Category::Design,
        severity: Severity::Low,
        message: format!(
            "Originality index is {}/100: most of the palette and typography are framework or popular defaults.",
            index.score
        ),
        file: "(project)".to_owned(),
        line: 0,
        fix: Some(
            "Choose your own palette and typefaces, and rename Tailwind colors after your brand, for example brand-500."
                .to_owned(),
        ),
    })
}

#[derive(Default)]
struct Palette {
    own: BTreeSet<String>,
    defaults: BTreeSet<String>,
}

impl Palette {
    fn add_color(&mut self, rgb: Rgb) {
        if rgb == [255, 255, 255] || rgb == [0, 0, 0] {
            return;
        }
        let label = hex_label(rgb);
        if is_default_color(rgb) {
            self.defaults.insert(label);
        } else {
            self.own.insert(label);
        }
    }

    fn add_class(&mut self, name: &str, shade: &str) {
        if NOT_COLORS.contains(&name) {
            return;
        }
        let label = format!("{name}-{shade}");
        if STOCK_COLORS.contains(&name) {
            self.defaults.insert(label);
        } else {
            self.own.insert(label);
        }
    }
}

fn collect_palette(files: &[SourceFile]) -> Palette {
    let mut palette = Palette::default();
    for file in files {
        // Links such as href="#add" look like short hex colors, so those lines are skipped.
        for line in file.text.lines().filter(|line| !line.contains("href")) {
            for caps in HEX.captures_iter(line) {
                if let Some(rgb) = parse_hex(&caps[1]) {
                    palette.add_color(rgb);
                }
            }
        }
        for caps in RGB.captures_iter(&file.text) {
            if let Some(rgb) = rgb_from(&caps) {
                palette.add_color(rgb);
            }
        }
        for caps in TAILWIND.captures_iter(&file.text) {
            palette.add_class(&caps[1], &caps[2]);
        }
    }
    palette
}

fn rgb_from(caps: &regex::Captures) -> Option<Rgb> {
    let channel = |index: usize| caps[index].parse::<u8>().ok();
    Some([channel(1)?, channel(2)?, channel(3)?])
}

/// Reads 3, 4, 6 or 8 hex digits. Alpha digits are ignored.
fn parse_hex(digits: &str) -> Option<Rgb> {
    let six: String = match digits.len() {
        3 | 4 => digits.chars().take(3).flat_map(|ch| [ch, ch]).collect(),
        6 | 8 => digits.chars().take(6).collect(),
        _ => return None,
    };
    let channel = |index: usize| u8::from_str_radix(&six[index..index + 2], 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn hex_label(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

fn default_colors() -> &'static [Rgb] {
    static COLORS: LazyLock<Vec<Rgb>> = LazyLock::new(|| {
        DEFAULT_HEX
            .iter()
            .filter_map(|hex| parse_hex(hex))
            .collect()
    });
    &COLORS
}

fn is_default_color(rgb: Rgb) -> bool {
    default_colors().iter().any(|known| {
        known
            .iter()
            .zip(rgb.iter())
            .all(|(a, b)| a.abs_diff(*b) <= NEAR)
    })
}

fn palette_dimension(files: &[SourceFile]) -> Option<Dimension> {
    let palette = collect_palette(files);
    let total = palette.own.len() + palette.defaults.len();
    (total >= MIN_COLORS).then(|| Dimension {
        name: PALETTE,
        total,
        defaults: palette.defaults.len(),
        weight: PALETTE_WEIGHT,
        default_items: palette.defaults.into_iter().collect(),
        own_items: palette.own.into_iter().collect(),
    })
}

/// Typefaces from font declarations, font links and `--font-*` custom properties.
fn collect_fonts(files: &[SourceFile]) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = fonts::used(files).into_keys().collect();
    for file in files {
        for caps in FONT_VARIABLE.captures_iter(&file.text) {
            let first = caps[1].split(',').next().unwrap_or_default();
            let name = fonts::normalize(first);
            if !name.is_empty() {
                names.insert(name);
            }
        }
    }
    names
}

fn font_dimension(files: &[SourceFile]) -> Option<Dimension> {
    let names = collect_fonts(files);
    if names.is_empty() {
        return None;
    }
    let (popular, own): (Vec<String>, Vec<String>) = names
        .into_iter()
        .partition(|name| POPULAR_FONTS.contains(&name.as_str()));
    Some(Dimension {
        name: TYPOGRAPHY,
        total: popular.len() + own.len(),
        defaults: popular.len(),
        weight: TYPOGRAPHY_WEIGHT,
        default_items: popular,
        own_items: own,
    })
}

fn token_dimension(files: &[SourceFile]) -> Option<Dimension> {
    let untouched = files
        .iter()
        .any(|file| SHADCN_BACKGROUND.is_match(&file.text) && SHADCN_PRIMARY.is_match(&file.text));
    untouched.then(|| Dimension {
        name: TOKENS,
        total: 1,
        defaults: 1,
        weight: TOKENS_WEIGHT,
        default_items: vec!["shadcn/ui default theme".to_owned()],
        own_items: Vec::new(),
    })
}

/// A readable report of the index.
#[must_use]
pub fn render_originality(index: &Originality) -> String {
    let mut lines = vec![
        format!("Originality index: {}/100", index.score),
        String::new(),
    ];
    for dimension in &index.dimensions {
        lines.push(format!(
            "{} (weight {}): {}% your own, {} of {} are defaults",
            dimension.name,
            dimension.weight,
            dimension.own_percent(),
            dimension.defaults,
            dimension.total
        ));
        lines.push(format!("    defaults: {}", list(&dimension.default_items)));
        lines.push(format!("    your own: {}", list(&dimension.own_items)));
    }
    lines.join("\n")
}

fn list(items: &[String]) -> String {
    if items.is_empty() {
        return "none".to_owned();
    }
    let shown = items
        .iter()
        .take(SHOWN)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let rest = items.len().saturating_sub(SHOWN);
    if rest > 0 {
        format!("{shown} and {rest} more")
    } else {
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_digits_are_read_in_every_length() {
        assert_eq!(parse_hex("fff"), Some([255, 255, 255]));
        assert_eq!(parse_hex("3b82f6"), Some([59, 130, 246]));
        assert_eq!(parse_hex("3b82f6cc"), Some([59, 130, 246]));
        assert_eq!(parse_hex("1234"), Some([17, 34, 51]));
        assert_eq!(parse_hex("12"), None);
        assert_eq!(parse_hex("ggg"), None);
    }

    #[test]
    fn labels_are_lowercase_six_digit_hex() {
        assert_eq!(hex_label([59, 130, 246]), "#3b82f6");
        assert_eq!(hex_label([0, 10, 255]), "#000aff");
    }

    #[test]
    fn every_default_color_parses() {
        assert_eq!(default_colors().len(), DEFAULT_HEX.len());
    }

    #[test]
    fn long_lists_are_shortened() {
        let items: Vec<String> = (0..15).map(|number| format!("c{number}")).collect();
        let text = list(&items);
        assert!(text.ends_with("and 3 more"));
        assert_eq!(list(&[]), "none");
    }
}
