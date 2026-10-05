//! Checks that look at one file at a time.

use std::collections::HashMap;
use std::sync::LazyLock;

use shipcheck_core::Finding;

use crate::catalog;
use crate::color::{self, Linear};
use crate::lines::{first_line, first_line_all, first_near, line_at, number, re, Pattern};
use crate::SourceFile;

/// Lowest acceptable contrast ratio for body text (WCAG AA).
const MIN_CONTRAST: f64 = 4.5;
/// Feature headings must appear within this many lines of a three column grid.
const ICON_WINDOW: usize = 40;
/// A badge counts as above the headline within this many lines.
const BADGE_WINDOW: usize = 6;

type Check = fn(&SourceFile) -> Option<Finding>;

const CHECKS: &[Check] = &[
    gradient_text,
    glass_card,
    accent_border,
    low_contrast,
    icon_boxes,
    headline_badge,
    shadcn_defaults,
    scroll_reveal,
    cursor_beam,
    hover_fade,
    serif_italic,
    grain_gradient,
];

/// Runs every per-file check on every file, at most one finding per check and file.
pub(crate) fn per_file(files: &[SourceFile]) -> Vec<Finding> {
    files
        .iter()
        .flat_map(|file| CHECKS.iter().filter_map(move |check| check(file)))
        .collect()
}

static GRADIENT_CLIP: Pattern =
    LazyLock::new(|| re(r"(?i)bg-clip-text|background-clip\s*:\s*text"));
static TRANSPARENT_TEXT: Pattern = LazyLock::new(|| {
    re(r"(?i)text-transparent|text-fill-color\s*:\s*transparent|[^-]color\s*:\s*transparent")
});
static GRADIENT_WORD: Pattern = LazyLock::new(|| re(r"(?i)gradient"));

fn gradient_text(file: &SourceFile) -> Option<Finding> {
    let line = first_near(
        &file.text,
        &GRADIENT_CLIP,
        &[&TRANSPARENT_TEXT, &GRADIENT_WORD],
        4,
    )?;
    Some(catalog::GRADIENT_TEXT.finding(&file.path, line))
}

static BLUR: Pattern = LazyLock::new(|| re(r"(?i)backdrop-blur|backdrop-filter\s*:\s*blur"));
static TRANSLUCENT: Pattern = LazyLock::new(|| {
    re(
        r"(?i)bg-(?:white|black|slate-\d+|gray-\d+|zinc-\d+|neutral-\d+)/\d+|rgba\(\s*(?:255\s*,\s*255\s*,\s*255|0\s*,\s*0\s*,\s*0)\s*,\s*0?\.\d+",
    )
});

fn glass_card(file: &SourceFile) -> Option<Finding> {
    let line = first_near(&file.text, &BLUR, &[&TRANSLUCENT], 5)?;
    Some(catalog::GLASS_CARD.finding(&file.path, line))
}

static BORDER_WIDTH: Pattern = LazyLock::new(|| re(r"\bborder-[ltrb]-[2-8]\b"));
static BORDER_COLOR: Pattern = LazyLock::new(|| {
    re(
        r"\bborder-(?:purple|violet|indigo|blue|cyan|teal|emerald|green|pink|rose|orange|amber|red|sky|fuchsia)-\d{3}\b",
    )
});
static CSS_ACCENT: Pattern =
    LazyLock::new(|| re(r"(?i)border-(?:left|top)\s*:\s*[3-9]px\s+solid\s+(?:#|rgb|hsl|var)"));

fn accent_border(file: &SourceFile) -> Option<Finding> {
    let line = first_line_all(&file.text, &[&BORDER_WIDTH, &BORDER_COLOR])
        .or_else(|| first_line(&file.text, &CSS_ACCENT))?;
    Some(catalog::ACCENT_BORDER.finding(&file.path, line))
}

static DARK_BLOCK: Pattern = LazyLock::new(|| re(r"(?i)\.dark[^{}]*\{([^{}]*)\}"));
static VARIABLE: Pattern = LazyLock::new(|| re(r"(?i)--([a-z0-9-]+)\s*:\s*([^;}]+)"));

fn low_contrast(file: &SourceFile) -> Option<Finding> {
    DARK_BLOCK.captures_iter(&file.text).find_map(|caps| {
        let body = caps.get(1)?;
        has_weak_text(body.as_str()).then(|| {
            catalog::LOW_CONTRAST_DARK.finding(&file.path, line_at(&file.text, body.start()))
        })
    })
}

/// Whether the foreground variables of a theme block are too close to its background.
fn has_weak_text(body: &str) -> bool {
    let colors: HashMap<String, Linear> = VARIABLE
        .captures_iter(body)
        .filter_map(|caps| Some((caps[1].to_owned(), color::parse(&caps[2])?)))
        .collect();
    let Some(background) = colors.get("background") else {
        return false;
    };
    ["foreground", "muted-foreground"]
        .iter()
        .filter_map(|name| colors.get(*name))
        .any(|text| color::contrast(*text, *background) < MIN_CONTRAST)
}

pub(crate) static LUCIDE_IMPORT: Pattern = LazyLock::new(|| re(r#"from\s+['"]lucide-react['"]"#));
static GRID_THREE: Pattern = LazyLock::new(|| re(r"\bgrid-cols-3\b"));
static HEADING_THREE: Pattern = LazyLock::new(|| re(r"<h3\b"));

fn icon_boxes(file: &SourceFile) -> Option<Finding> {
    if !LUCIDE_IMPORT.is_match(&file.text) {
        return None;
    }
    let lines: Vec<&str> = file.text.lines().collect();
    let start = lines.iter().position(|line| GRID_THREE.is_match(line))?;
    let end = (start + ICON_WINDOW + 1).min(lines.len());
    let headings = lines[start..end]
        .iter()
        .filter(|line| HEADING_THREE.is_match(line))
        .count();
    (headings >= 3).then(|| catalog::ICON_BOXES.finding(&file.path, number(start)))
}

static HEADING_ONE: Pattern = LazyLock::new(|| re(r"<h1\b"));
static PILL: Pattern = LazyLock::new(|| re(r"\brounded-full\b"));
static PILL_TRAIT: Pattern =
    LazyLock::new(|| re(r"\b(?:px-[23](?:\.5)?|uppercase|tracking-wider?|text-xs)\b"));
static BADGE_TAG: Pattern = LazyLock::new(|| re(r"<Badge\b"));

fn is_badge_line(line: &str) -> bool {
    BADGE_TAG.is_match(line) || (PILL.is_match(line) && PILL_TRAIT.is_match(line))
}

fn headline_badge(file: &SourceFile) -> Option<Finding> {
    let lines: Vec<&str> = file.text.lines().collect();
    let heading = lines.iter().position(|line| HEADING_ONE.is_match(line))?;
    let from = heading.saturating_sub(BADGE_WINDOW);
    lines[from..heading]
        .iter()
        .copied()
        .any(is_badge_line)
        .then(|| catalog::BADGE_ABOVE_HEADLINE.finding(&file.path, number(heading)))
}

static SHADCN_BACKGROUND: Pattern =
    LazyLock::new(|| re(r"(?i)--background\s*:\s*(?:0\s+0%\s+100%|oklch\(\s*1\s+0\s+0\s*\))"));
static SHADCN_PRIMARY: Pattern = LazyLock::new(|| {
    re(
        r"(?i)--primary\s*:\s*(?:222\.2\s+47\.4%\s+11\.2%|0\s+0%\s+9%|240\s+5\.9%\s+10%|oklch\(\s*0\.205\s+0\s+0\s*\))",
    )
});

fn shadcn_defaults(file: &SourceFile) -> Option<Finding> {
    if !SHADCN_BACKGROUND.is_match(&file.text) {
        return None;
    }
    let line = first_line(&file.text, &SHADCN_PRIMARY)?;
    Some(catalog::SHADCN_DEFAULT.finding(&file.path, line))
}

static SCROLL_REVEAL: Pattern = LazyLock::new(|| {
    re(
        r#"(?i)\bdata-aos\b|whileInView|ScrollReveal|from\s+['"]aos['"]|animate-on-scroll|fade-in-up|reveal-on-scroll"#,
    )
});

fn scroll_reveal(file: &SourceFile) -> Option<Finding> {
    let line = first_line(&file.text, &SCROLL_REVEAL)?;
    Some(catalog::FADE_ON_SCROLL.finding(&file.path, line))
}

static MOUSE_MOVE: Pattern = LazyLock::new(|| re(r"(?i)mousemove|pointermove"));
static SET_VARIABLE: Pattern =
    LazyLock::new(|| re(r#"(?i)setProperty\(\s*['"]--(?:mouse|cursor|pointer|x|y|spotlight)"#));

fn cursor_beam(file: &SourceFile) -> Option<Finding> {
    let line = first_near(&file.text, &MOUSE_MOVE, &[&SET_VARIABLE], 15)?;
    Some(catalog::CURSOR_BEAM.finding(&file.path, line))
}

static HOVER_OPACITY: Pattern = LazyLock::new(|| re(r"\bhover:opacity-[5-9]\d?\b"));
static BUTTONISH: Pattern = LazyLock::new(|| re(r"(?i)button|\bbtn\b"));
static CSS_BUTTON_HOVER: Pattern = LazyLock::new(|| re(r"(?i)(?:button|\.btn)[^{]*:hover"));
static CSS_OPACITY: Pattern = LazyLock::new(|| re(r"opacity\s*:\s*0?\.[5-9]"));

fn hover_fade(file: &SourceFile) -> Option<Finding> {
    let line = first_line_all(&file.text, &[&HOVER_OPACITY, &BUTTONISH])
        .or_else(|| first_near(&file.text, &CSS_BUTTON_HOVER, &[&CSS_OPACITY], 3))?;
    Some(catalog::HOVER_FADE.finding(&file.path, line))
}

static SERIF_CLASS: Pattern = LazyLock::new(|| re(r"\bfont-serif\b"));
static ITALIC_CLASS: Pattern = LazyLock::new(|| re(r"\bitalic\b"));
static ITALIC_CSS: Pattern = LazyLock::new(|| re(r"font-style\s*:\s*italic"));
static SERIF_FAMILY: Pattern = LazyLock::new(|| {
    re(
        r"(?i)font-family\s*:[^;]*\b(?:instrument serif|playfair display|fraunces|cormorant|dm serif|newsreader)",
    )
});

fn serif_italic(file: &SourceFile) -> Option<Finding> {
    let line = first_line_all(&file.text, &[&SERIF_CLASS, &ITALIC_CLASS])
        .or_else(|| first_near(&file.text, &ITALIC_CSS, &[&SERIF_FAMILY], 3))?;
    Some(catalog::SERIF_ITALIC.finding(&file.path, line))
}

static GRAIN: Pattern = LazyLock::new(|| {
    re(
        r"(?i)feTurbulence|(?:noise|grain)\.(?:png|svg|webp|jpe?g)|\bbg-noise\b|\bgrain-overlay\b|\bnoise-overlay\b",
    )
});

fn grain_gradient(file: &SourceFile) -> Option<Finding> {
    let line = first_near(&file.text, &GRAIN, &[&GRADIENT_WORD], 20)?;
    Some(catalog::GRAIN_GRADIENT.finding(&file.path, line))
}
