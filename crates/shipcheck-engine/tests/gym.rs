use std::path::Path;

use shipcheck_engine::{
    grade_rules, overall_robustness, render_gym, Catalog, Mutation, RuleGrade, SlipKind,
};

const BRITTLE: &str = r"
- id: T-BRITTLE
  category: security
  severity: high
  message: brittle
  extensions: [js]
  match:
    kind: line_regex
    pattern: 'eval\(userInput\)'
  examples:
    flag:
      - 'eval(userInput);'
";

const ROBUST: &str = r"
- id: T-ROBUST
  category: security
  severity: high
  message: robust
  extensions: [js]
  match:
    kind: line_regex
    pattern: '\beval\s*\('
  examples:
    flag:
      - 'eval(userInput);'
";

const NOISY: &str = r#"
- id: T-NOISY
  category: legal
  severity: medium
  message: image without alt text
  extensions: [html]
  match:
    kind: line_regex
    pattern: '(?i)<img\b'
    unless: '(?i)\balt\s*='
  examples:
    flag:
      - '<img src="a.png">'
    pass:
      - '<img src="a.png" alt="a">'
"#;

fn grades(yaml: &str) -> Vec<RuleGrade> {
    grade_rules(&Catalog::from_yaml(yaml).unwrap())
}

#[test]
fn a_brittle_rule_is_caught_out() {
    let all = grades(BRITTLE);
    let grade = &all[0];
    assert_eq!(grade.rule_id, "T-BRITTLE");
    assert_eq!(grade.checks, 4);
    assert_eq!(grade.missed(), 2);
    assert_eq!(grade.false_alarms(), 0);
    assert_eq!(grade.robustness(), 50);
    assert_eq!(grade.letter(), 'C');
    let names: Vec<&str> = grade
        .slips
        .iter()
        .map(|slip| slip.mutation.name())
        .collect();
    assert_eq!(names, vec!["space_before_paren", "break_after_paren"]);
}

#[test]
fn a_robust_rule_earns_an_a() {
    let all = grades(ROBUST);
    let grade = &all[0];
    assert_eq!(grade.checks, 4);
    assert_eq!(grade.slips, Vec::new());
    assert_eq!(grade.robustness(), 100);
    assert_eq!(grade.letter(), 'A');
}

#[test]
fn false_alarms_are_counted_too() {
    let all = grades(NOISY);
    let grade = &all[0];
    assert_eq!(grade.missed(), 0);
    assert_eq!(grade.false_alarms(), 1);
    assert_eq!(grade.slips[0].mutation, Mutation::BreakAfterEquals);
    assert_eq!(grade.slips[0].kind, SlipKind::FalseAlarm);
}

#[test]
fn rules_without_examples_or_line_patterns_are_skipped() {
    let yaml = r"
- id: T-FILE
  category: legal
  severity: high
  message: missing file
  match:
    kind: missing_file
    any_of: [PRIVACY.md]
- id: T-BARE
  category: legal
  severity: low
  message: no examples
  match:
    kind: line_regex
    pattern: 'x'
";
    assert_eq!(grades(yaml), Vec::new());
}

#[test]
fn the_report_lists_the_weakest_rule_first() {
    let all = grades(&format!("{BRITTLE}\n{ROBUST}"));
    assert_eq!(overall_robustness(&all), 75);
    let text = render_gym(&all);
    assert!(text.contains("Rule gym: 2 rules graded, 75% of 8 attacks handled"));
    assert!(text.contains("space_before_paren"));
    assert!(text.contains("eval(\\nuserInput);"));
    let brittle = text.find("T-BRITTLE").unwrap();
    let robust = text.find("T-ROBUST").unwrap();
    assert!(brittle < robust);
}

#[test]
fn the_shipped_rules_hold_up() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules");
    let all = grade_rules(&Catalog::load(&dir).expect("shipped rules load"));
    assert!(
        all.len() >= 15,
        "most shipped rules have examples to attack"
    );
    for grade in &all {
        assert!(
            grade.robustness() >= 75,
            "{} is too easy to fool: {}%",
            grade.rule_id,
            grade.robustness()
        );
    }
    assert!(overall_robustness(&all) >= 90);
    for id in ["LEGAL-008", "LEGAL-009"] {
        let grade = all
            .iter()
            .find(|grade| grade.rule_id == id)
            .expect("rule is graded");
        assert_eq!(grade.letter(), 'A', "{id}");
    }
}
