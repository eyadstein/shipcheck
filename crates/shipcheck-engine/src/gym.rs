//! The rule gym: attacks every rule with rewritten versions of its own examples.

use crate::rule::{Catalog, Compiled};

/// Languages where a space may sit between a function name and its parenthesis.
const CODE_LANGUAGES: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "cjs", "py", "rb", "php", "go", "java", "kt", "swift", "cs",
    "dart",
];

/// Languages where upper and lower case mean the same thing.
const CASE_INSENSITIVE: &[&str] = &["html", "htm", "css", "scss"];

const INDENT: &str = "        ";

/// A way of rewriting a line without changing what it means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutation {
    DoubleSpaces,
    Tabs,
    SpaceBeforeParen,
    SwapQuotes,
    UpperCase,
    BreakAfterParen,
    BreakAfterEquals,
    Indent,
    TrailingComment,
}

impl Mutation {
    pub const ALL: [Self; 9] = [
        Self::DoubleSpaces,
        Self::Tabs,
        Self::SpaceBeforeParen,
        Self::SwapQuotes,
        Self::UpperCase,
        Self::BreakAfterParen,
        Self::BreakAfterEquals,
        Self::Indent,
        Self::TrailingComment,
    ];

    /// A short name for reports.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DoubleSpaces => "double_spaces",
            Self::Tabs => "tabs",
            Self::SpaceBeforeParen => "space_before_paren",
            Self::SwapQuotes => "swap_quotes",
            Self::UpperCase => "upper_case",
            Self::BreakAfterParen => "break_after_paren",
            Self::BreakAfterEquals => "break_after_equals",
            Self::Indent => "indent",
            Self::TrailingComment => "trailing_comment",
        }
    }

    /// The rewritten line, or `None` when the mutation does not apply or changes nothing.
    #[must_use]
    pub fn apply(self, line: &str, extensions: &[String]) -> Option<String> {
        let changed = match self {
            Self::DoubleSpaces => line.contains(' ').then(|| line.replace(' ', "  ")),
            Self::Tabs => line.contains(' ').then(|| line.replace(' ', "\t")),
            Self::SpaceBeforeParen => {
                if all_in(extensions, CODE_LANGUAGES) {
                    space_before_paren(line)
                } else {
                    None
                }
            }
            Self::SwapQuotes => Some(line.chars().map(swap_quote).collect()),
            Self::UpperCase => all_in(extensions, CASE_INSENSITIVE).then(|| line.to_uppercase()),
            Self::BreakAfterParen => break_after(line, '('),
            Self::BreakAfterEquals => break_after(line, '='),
            Self::Indent => Some(format!("{INDENT}{line}")),
            Self::TrailingComment => Some(format!("{line} // ok")),
        };
        changed.filter(|variant| variant != line)
    }
}

fn all_in(extensions: &[String], group: &[&str]) -> bool {
    !extensions.is_empty()
        && extensions
            .iter()
            .all(|ext| group.iter().any(|known| known.eq_ignore_ascii_case(ext)))
}

fn swap_quote(ch: char) -> char {
    match ch {
        '"' => '\'',
        '\'' => '"',
        other => other,
    }
}

fn space_before_paren(line: &str) -> Option<String> {
    let index = line.find('(')?;
    if index == 0 || line[..index].ends_with(' ') {
        return None;
    }
    Some(format!("{} {}", &line[..index], &line[index..]))
}

fn break_after(line: &str, mark: char) -> Option<String> {
    let (index, _) = line.char_indices().find(|&(_, ch)| ch == mark)?;
    let split = index + mark.len_utf8();
    if split >= line.len() {
        return None;
    }
    Some(format!("{}\n{}", &line[..split], &line[split..]))
}

/// Whether an attack was handled badly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlipKind {
    /// A line the rule should flag was not flagged.
    Missed,
    /// A line the rule should leave alone was flagged.
    FalseAlarm,
}

impl SlipKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Missed => "missed",
            Self::FalseAlarm => "false alarm",
        }
    }
}

/// One attack that got through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slip {
    pub mutation: Mutation,
    pub variant: String,
    pub kind: SlipKind,
}

/// How one rule held up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGrade {
    pub rule_id: String,
    /// Attacks tried.
    pub checks: usize,
    pub slips: Vec<Slip>,
}

impl RuleGrade {
    #[must_use]
    pub fn missed(&self) -> usize {
        self.slips
            .iter()
            .filter(|slip| slip.kind == SlipKind::Missed)
            .count()
    }

    #[must_use]
    pub fn false_alarms(&self) -> usize {
        self.slips
            .iter()
            .filter(|slip| slip.kind == SlipKind::FalseAlarm)
            .count()
    }

    /// Share of attacks the rule handled correctly, from 0 to 100.
    #[must_use]
    pub fn robustness(&self) -> u32 {
        percent(self.checks, self.slips.len())
    }

    /// A for 90 and above, B for 75, C for 50, otherwise D.
    #[must_use]
    pub fn letter(&self) -> char {
        match self.robustness() {
            90..=100 => 'A',
            75..=89 => 'B',
            50..=74 => 'C',
            _ => 'D',
        }
    }
}

fn percent(checks: usize, slips: usize) -> u32 {
    if checks == 0 {
        return 100;
    }
    let good = checks.saturating_sub(slips);
    u32::try_from(good * 100 / checks).unwrap_or(100)
}

/// Attacks every rule that has examples and a line pattern.
#[must_use]
pub fn grade_rules(catalog: &Catalog) -> Vec<RuleGrade> {
    catalog.rules().iter().filter_map(grade_rule).collect()
}

fn grade_rule(compiled: &Compiled) -> Option<RuleGrade> {
    compiled.regex.as_ref()?;
    let rule = &compiled.rule;
    let mut checks = 0;
    let mut slips = Vec::new();
    for (lines, kind) in [
        (&rule.examples.flag, SlipKind::Missed),
        (&rule.examples.pass, SlipKind::FalseAlarm),
    ] {
        for line in lines {
            for mutation in Mutation::ALL {
                let Some(variant) = mutation.apply(line, &rule.extensions) else {
                    continue;
                };
                checks += 1;
                let flagged = flags_variant(compiled, &variant);
                let wrong = match kind {
                    SlipKind::Missed => !flagged,
                    SlipKind::FalseAlarm => flagged,
                };
                if wrong {
                    slips.push(Slip {
                        mutation,
                        variant,
                        kind,
                    });
                }
            }
        }
    }
    (checks > 0).then_some(RuleGrade {
        rule_id: rule.id.clone(),
        checks,
        slips,
    })
}

/// A variant is flagged when any of its lines is, because the scanner reads one line at a time.
fn flags_variant(compiled: &Compiled, variant: &str) -> bool {
    variant.lines().any(|line| compiled.flags_line(line))
}

/// Share of all attacks, over all rules, that were handled correctly.
#[must_use]
pub fn overall_robustness(grades: &[RuleGrade]) -> u32 {
    let checks: usize = grades.iter().map(|grade| grade.checks).sum();
    let slips: usize = grades.iter().map(|grade| grade.slips.len()).sum();
    percent(checks, slips)
}

/// A readable report, weakest rules first.
#[must_use]
pub fn render_gym(grades: &[RuleGrade]) -> String {
    let attacks: usize = grades.iter().map(|grade| grade.checks).sum();
    let mut lines = vec![
        format!(
            "Rule gym: {} rules graded, {}% of {attacks} attacks handled",
            grades.len(),
            overall_robustness(grades)
        ),
        String::new(),
    ];
    let mut ordered: Vec<&RuleGrade> = grades.iter().collect();
    ordered.sort_by(|a, b| {
        a.robustness()
            .cmp(&b.robustness())
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });
    for grade in ordered {
        lines.push(format!(
            "{:<10} {} {:>3}%  {} of {} attacks slipped through",
            grade.rule_id,
            grade.letter(),
            grade.robustness(),
            grade.slips.len(),
            grade.checks
        ));
        for slip in &grade.slips {
            lines.push(format!(
                "    {:<12} {:<20} {:?}",
                slip.kind.label(),
                slip.mutation.name(),
                slip.variant
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn exts(list: &[&str]) -> Vec<String> {
        list.iter().map(|ext| (*ext).to_owned()).collect()
    }

    fn grade_with(checks: usize, slips: usize) -> RuleGrade {
        let slip = Slip {
            mutation: Mutation::Tabs,
            variant: "a\tb".to_owned(),
            kind: SlipKind::Missed,
        };
        RuleGrade {
            rule_id: "X".to_owned(),
            checks,
            slips: vec![slip; slips],
        }
    }

    #[test]
    fn whitespace_mutations_need_a_space() {
        assert_eq!(
            Mutation::DoubleSpaces.apply("a b", &[]),
            Some("a  b".to_owned())
        );
        assert_eq!(Mutation::Tabs.apply("a b", &[]), Some("a\tb".to_owned()));
        assert_eq!(Mutation::DoubleSpaces.apply("ab", &[]), None);
        assert_eq!(Mutation::Tabs.apply("ab", &[]), None);
    }

    #[test]
    fn a_space_before_a_call_only_applies_to_programming_languages() {
        let js = exts(&["js"]);
        assert_eq!(
            Mutation::SpaceBeforeParen.apply("eval(x)", &js),
            Some("eval (x)".to_owned())
        );
        assert_eq!(
            Mutation::SpaceBeforeParen.apply("eval(x)", &exts(&["css"])),
            None
        );
        assert_eq!(Mutation::SpaceBeforeParen.apply("eval(x)", &[]), None);
        assert_eq!(Mutation::SpaceBeforeParen.apply("eval (x)", &js), None);
        assert_eq!(Mutation::SpaceBeforeParen.apply("(x)", &js), None);
    }

    #[test]
    fn quotes_swap_in_both_directions() {
        assert_eq!(
            Mutation::SwapQuotes.apply("a=\"b\"", &[]),
            Some("a='b'".to_owned())
        );
        assert_eq!(
            Mutation::SwapQuotes.apply("a='b'", &[]),
            Some("a=\"b\"".to_owned())
        );
        assert_eq!(Mutation::SwapQuotes.apply("ab", &[]), None);
    }

    #[test]
    fn upper_case_only_applies_where_case_does_not_matter() {
        assert_eq!(
            Mutation::UpperCase.apply("<p>x</p>", &exts(&["html"])),
            Some("<P>X</P>".to_owned())
        );
        assert_eq!(Mutation::UpperCase.apply("<p>x</p>", &exts(&["js"])), None);
        assert_eq!(Mutation::UpperCase.apply("<p>x</p>", &[]), None);
    }

    #[test]
    fn breaks_split_after_the_first_mark() {
        assert_eq!(
            Mutation::BreakAfterParen.apply("f(x)", &[]),
            Some("f(\nx)".to_owned())
        );
        assert_eq!(Mutation::BreakAfterParen.apply("f(", &[]), None);
        assert_eq!(Mutation::BreakAfterParen.apply("fx", &[]), None);
        assert_eq!(
            Mutation::BreakAfterEquals.apply("a=b", &[]),
            Some("a=\nb".to_owned())
        );
        assert_eq!(Mutation::BreakAfterEquals.apply("ab", &[]), None);
    }

    #[test]
    fn indent_and_comment_always_apply() {
        assert_eq!(
            Mutation::Indent.apply("x", &[]),
            Some("        x".to_owned())
        );
        assert_eq!(
            Mutation::TrailingComment.apply("x", &[]),
            Some("x // ok".to_owned())
        );
    }

    #[test]
    fn mutation_names_are_unique() {
        let names: HashSet<&str> = Mutation::ALL
            .iter()
            .map(|mutation| mutation.name())
            .collect();
        assert_eq!(names.len(), Mutation::ALL.len());
    }

    #[test]
    fn letters_follow_the_percentage() {
        for (slips, letter) in [(0, 'A'), (1, 'A'), (2, 'B'), (3, 'C'), (5, 'C'), (6, 'D')] {
            assert_eq!(grade_with(10, slips).letter(), letter, "{slips} slips");
        }
        assert_eq!(grade_with(4, 1).letter(), 'B');
    }

    #[test]
    fn no_grades_means_full_marks() {
        assert_eq!(overall_robustness(&[]), 100);
    }
}
