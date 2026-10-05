//! Checks that need to see the whole project at once.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use shipcheck_core::Finding;

use crate::catalog;
use crate::checks::LUCIDE_IMPORT;
use crate::fonts::{self, Places};
use crate::lines::{first_line, is_markup, line_at, re, Pattern};
use crate::SourceFile;

const MIN_LUCIDE_FILES: usize = 6;
const MIN_OFF_GRID: usize = 5;
const MIN_ARBITRARY: usize = 6;
const MIN_EM_DASHES: usize = 8;
const MIN_BUZZWORDS: usize = 5;
const EM_DASH_CHAR: char = '\u{2014}';

/// Runs every project-wide check.
pub(crate) fn project_wide(files: &[SourceFile]) -> Vec<Finding> {
    let used = fonts::used(files);
    let mut out = Vec::new();
    out.extend(inter_only(&used));
    out.extend(font_pair(&used));
    out.extend(lucide_overuse(files));
    out.extend(inconsistent_spacing(files));
    out.extend(em_dashes(files));
    out.extend(buzzwords(files));
    out
}

fn inter_only(used: &Places) -> Option<Finding> {
    if used.len() != 1 {
        return None;
    }
    let (file, line) = used.get("inter")?;
    Some(catalog::INTER_ONLY.finding(file, *line))
}

fn font_pair(used: &Places) -> Option<Finding> {
    let (file, line) = used.get("instrument serif")?;
    used.contains_key("space grotesk")
        .then(|| catalog::FONT_PAIR.finding(file, *line))
}

fn lucide_overuse(files: &[SourceFile]) -> Option<Finding> {
    let importers: Vec<(&SourceFile, u32)> = files
        .iter()
        .filter_map(|file| first_line(&file.text, &LUCIDE_IMPORT).map(|line| (file, line)))
        .collect();
    let (first, line) = importers.first()?;
    (importers.len() >= MIN_LUCIDE_FILES)
        .then(|| catalog::LUCIDE_EVERYWHERE.finding(&first.path, *line))
}

static SPACING_DECL: Pattern = LazyLock::new(|| {
    re(r"(?i)\b(?:padding|margin|gap)(?:-(?:top|right|bottom|left|inline|block))?\s*:\s*([^;}\n]+)")
});
static PX_VALUE: Pattern = LazyLock::new(|| re(r"(\d+(?:\.\d+)?)px"));
static ARBITRARY: Pattern = LazyLock::new(|| {
    re(r"\b(?:p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr|gap|space-x|space-y)-\[([0-9.]+(?:px|rem))\]")
});

/// Distinct values seen, and where the first one appeared.
#[derive(Default)]
struct Tally {
    distinct: BTreeSet<String>,
    first: Option<(String, u32)>,
}

impl Tally {
    fn add(&mut self, key: &str, file: &SourceFile, offset: usize) {
        if self.distinct.insert(key.to_owned()) && self.first.is_none() {
            self.first = Some((file.path.clone(), line_at(&file.text, offset)));
        }
    }
}

/// Whether a pixel value sits on a 4px grid, with 2px allowed for hairlines.
fn on_grid(value: &str) -> bool {
    let Ok(number) = value.parse::<f64>() else {
        return true;
    };
    (number % 4.0).abs() < f64::EPSILON || (number - 2.0).abs() < f64::EPSILON
}

fn spacing_tallies(files: &[SourceFile]) -> (Tally, Tally) {
    let mut css = Tally::default();
    let mut arbitrary = Tally::default();
    for file in files {
        for caps in SPACING_DECL.captures_iter(&file.text) {
            let (Some(whole), Some(value)) = (caps.get(0), caps.get(1)) else {
                continue;
            };
            for px in PX_VALUE.captures_iter(value.as_str()) {
                if !on_grid(&px[1]) {
                    css.add(&px[1], file, whole.start());
                }
            }
        }
        for caps in ARBITRARY.captures_iter(&file.text) {
            arbitrary.add(
                &caps[1],
                file,
                caps.get(0).as_ref().map_or(0, regex::Match::start),
            );
        }
    }
    (css, arbitrary)
}

fn inconsistent_spacing(files: &[SourceFile]) -> Option<Finding> {
    let (css, arbitrary) = spacing_tallies(files);
    let tally = if css.distinct.len() >= MIN_OFF_GRID {
        css
    } else if arbitrary.distinct.len() >= MIN_ARBITRARY {
        arbitrary
    } else {
        return None;
    };
    let (file, line) = tally.first?;
    Some(catalog::SPACING.finding(&file, line))
}

fn em_dashes(files: &[SourceFile]) -> Option<Finding> {
    let mut total = 0;
    let mut first: Option<(&SourceFile, usize)> = None;
    for file in files.iter().filter(|file| is_markup(&file.path)) {
        let count = file.text.matches(EM_DASH_CHAR).count();
        if count > 0 && first.is_none() {
            first = file.text.find(EM_DASH_CHAR).map(|offset| (file, offset));
        }
        total += count;
    }
    let (file, offset) = first?;
    (total >= MIN_EM_DASHES)
        .then(|| catalog::EM_DASHES.finding(&file.path, line_at(&file.text, offset)))
}

static BUZZ: Pattern = LazyLock::new(|| {
    re(
        r"(?i)\b(?:seamless(?:ly)?|supercharge[sd]?|unleash(?:es|ed|ing)?|elevate your|revolutioni[sz](?:e|es|ed|ing)|next-gen(?:eration)?|cutting-edge|game-chang(?:er|ing)|empower(?:s|ed|ing)?|leverage[sd]?|streamline[sd]?|effortless(?:ly)?|all-in-one|to the next level|unlock the (?:power|potential))\b",
    )
});

fn buzzwords(files: &[SourceFile]) -> Option<Finding> {
    let mut total = 0;
    let mut first: Option<(&SourceFile, usize)> = None;
    for file in files.iter().filter(|file| is_markup(&file.path)) {
        let mut hits = BUZZ.find_iter(&file.text);
        if let Some(hit) = hits.next() {
            total += 1 + hits.count();
            if first.is_none() {
                first = Some((file, hit.start()));
            }
        }
    }
    let (file, offset) = first?;
    (total >= MIN_BUZZWORDS)
        .then(|| catalog::BUZZWORDS.finding(&file.path, line_at(&file.text, offset)))
}
