use std::fs;

use shipcheck_core::{Category, Finding, Severity};
use shipcheck_engine::{review, Baseline, Error, ProjectConfig};

fn finding(rule: &str, file: &str, line: u32) -> Finding {
    Finding {
        rule_id: rule.to_owned(),
        category: Category::Security,
        severity: Severity::High,
        message: format!("{rule} message"),
        file: file.to_owned(),
        line,
        fix: None,
    }
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::Builder::new()
        .prefix("sc-review-")
        .tempdir()
        .unwrap();
    for (name, body) in files {
        fs::write(dir.path().join(name), body).unwrap();
    }
    dir
}

#[test]
fn settings_switch_off_rules_and_paths() {
    let config = ProjectConfig::from_yaml(
        "ignore_rules: [LEGAL-002]\nignore_paths: [vendor/, ./examples]\n",
        "test",
    )
    .unwrap();
    let dir = project(&[]);
    let all = vec![
        finding("LEGAL-002", "(project)", 0),
        finding("SEC-001", "vendor/a.js", 1),
        finding("SEC-001", "examples/b.js", 1),
        finding("SEC-001", "src/c.js", 1),
        finding("SEC-001", "examples2/d.js", 1),
    ];
    let result = review(dir.path(), all, &config, None);
    assert_eq!(result.ignored, 3);
    let files: Vec<&str> = result
        .findings
        .iter()
        .map(|item| item.file.as_str())
        .collect();
    assert_eq!(files, vec!["src/c.js", "examples2/d.js"]);
}

#[test]
fn settings_load_from_the_project_folder() {
    let empty = project(&[]);
    assert_eq!(
        ProjectConfig::load(empty.path()).unwrap(),
        ProjectConfig::default()
    );
    let dir = project(&[(
        "shipcheck.yml",
        "fail_under: 80\nignore_rules: [LEGAL-002]\n",
    )]);
    let config = ProjectConfig::load(dir.path()).unwrap();
    assert_eq!(config.fail_under, Some(80));
    assert_eq!(config.ignore_rules, vec!["LEGAL-002".to_owned()]);
    let hidden = project(&[(".shipcheck.yml", "ignore_paths: [dist]\n")]);
    let config = ProjectConfig::load(hidden.path()).unwrap();
    assert_eq!(config.ignore_paths, vec!["dist".to_owned()]);
}

#[test]
fn bad_settings_are_rejected() {
    let typo = ProjectConfig::from_yaml("ignore_rule: [X]\n", "t");
    assert!(matches!(typo, Err(Error::Yaml { .. })));
    let range = ProjectConfig::from_yaml("fail_under: 150\n", "t");
    assert!(matches!(range, Err(Error::Invalid { .. })));
    assert_eq!(
        ProjectConfig::from_yaml("  \n", "t").unwrap(),
        ProjectConfig::default()
    );
}

#[test]
fn comments_silence_their_own_line_and_the_next() {
    let code = "eval(x); // shipcheck-ignore: SEC-001\nel.innerHTML = y;\n// shipcheck-ignore: SEC-004, SEC-001 because it is sanitized\nel.innerHTML = z;\neval(w);\n";
    let dir = project(&[("a.js", code)]);
    let all = vec![
        finding("SEC-001", "a.js", 1),
        finding("SEC-004", "a.js", 2),
        finding("SEC-004", "a.js", 4),
        finding("SEC-001", "a.js", 5),
    ];
    let result = review(dir.path(), all, &ProjectConfig::default(), None);
    assert_eq!(result.suppressed, 2);
    let left: Vec<(String, u32)> = result
        .findings
        .iter()
        .map(|item| (item.rule_id.clone(), item.line))
        .collect();
    assert_eq!(
        left,
        vec![("SEC-004".to_owned(), 2), ("SEC-001".to_owned(), 5)]
    );
}

#[test]
fn a_file_comment_covers_every_line() {
    let code = "// shipcheck-ignore-file: sec-001\nconst a = 1;\neval(x);\n";
    let dir = project(&[("a.js", code)]);
    let all = vec![
        finding("SEC-001", "a.js", 3),
        finding("SEC-001", "a.js", 9),
        finding("SEC-004", "a.js", 3),
    ];
    let result = review(dir.path(), all, &ProjectConfig::default(), None);
    assert_eq!(result.suppressed, 2);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "SEC-004");
}

#[test]
fn project_level_findings_and_missing_files_are_left_alone() {
    let dir = project(&[]);
    let all = vec![
        finding("LEGAL-001", "(project)", 0),
        finding("SEC-001", "missing.js", 1),
    ];
    let result = review(dir.path(), all, &ProjectConfig::default(), None);
    assert_eq!(result.suppressed, 0);
    assert_eq!(result.findings.len(), 2);
}

#[test]
fn the_baseline_hides_known_findings_even_when_lines_move() {
    let dir = project(&[("a.js", "eval(x);\n")]);
    let baseline = Baseline::from_findings(dir.path(), &[finding("SEC-001", "a.js", 1)]);
    assert_eq!(baseline.accepted(), 1);

    fs::write(dir.path().join("a.js"), "\n\neval(x);\n").unwrap();
    let (kept, hidden) = baseline.filter(dir.path(), vec![finding("SEC-001", "a.js", 3)]);
    assert_eq!(kept, Vec::<Finding>::new());
    assert_eq!(hidden, 1);

    fs::write(dir.path().join("a.js"), "\n\neval(x);\neval(y);\n").unwrap();
    let both = vec![finding("SEC-001", "a.js", 3), finding("SEC-001", "a.js", 4)];
    let (kept, hidden) = baseline.filter(dir.path(), both);
    assert_eq!(hidden, 1);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].line, 4);
}

#[test]
fn identical_lines_are_counted_one_by_one() {
    let dir = project(&[("a.js", "eval(x);\neval(x);\n")]);
    let known = vec![finding("SEC-001", "a.js", 1), finding("SEC-001", "a.js", 2)];
    let baseline = Baseline::from_findings(dir.path(), &known);
    assert_eq!(baseline.accepted(), 2);

    fs::write(dir.path().join("a.js"), "eval(x);\neval(x);\neval(x);\n").unwrap();
    let now = vec![
        finding("SEC-001", "a.js", 1),
        finding("SEC-001", "a.js", 2),
        finding("SEC-001", "a.js", 3),
    ];
    let (kept, hidden) = baseline.filter(dir.path(), now);
    assert_eq!(hidden, 2);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].line, 3);
}

#[test]
fn baselines_round_trip_through_files() {
    let dir = project(&[("a.js", "eval(x);\n")]);
    let findings = [
        finding("SEC-001", "a.js", 1),
        finding("LEGAL-001", "(project)", 0),
    ];
    let baseline = Baseline::from_findings(dir.path(), &findings);
    let path = dir.path().join("baseline.json");
    baseline.save(&path).unwrap();
    let loaded = Baseline::load(&path).unwrap();
    assert_eq!(loaded, baseline);
    assert_eq!(loaded.accepted(), 2);
}

#[test]
fn broken_baseline_files_are_reported() {
    let dir = project(&[
        ("future.json", "{\"version\":99,\"entries\":{}}"),
        ("broken.json", "not json"),
    ]);
    assert!(matches!(
        Baseline::load(&dir.path().join("future.json")),
        Err(Error::Invalid { .. })
    ));
    assert!(matches!(
        Baseline::load(&dir.path().join("broken.json")),
        Err(Error::Json { .. })
    ));
    assert!(matches!(
        Baseline::load(&dir.path().join("missing.json")),
        Err(Error::Io { .. })
    ));
}

#[test]
fn the_review_applies_settings_then_comments_then_the_baseline() {
    let code = "eval(x); // shipcheck-ignore: SEC-001\nel.innerHTML = y;\nfoo(z);\n";
    let dir = project(&[("a.js", code)]);
    let config = ProjectConfig::from_yaml("ignore_rules: [LEGAL-002]\n", "t").unwrap();
    let baseline = Baseline::from_findings(dir.path(), &[finding("SEC-004", "a.js", 2)]);
    let all = vec![
        finding("LEGAL-002", "(project)", 0),
        finding("SEC-001", "a.js", 1),
        finding("SEC-004", "a.js", 2),
        finding("SEC-002", "a.js", 3),
    ];
    let result = review(dir.path(), all, &config, Some(&baseline));
    assert_eq!(
        (result.ignored, result.suppressed, result.baselined),
        (1, 1, 1)
    );
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].rule_id, "SEC-002");
}
