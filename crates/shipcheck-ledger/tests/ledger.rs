use std::path::Path;

use shipcheck_core::Severity;
use shipcheck_ledger::{analyze, is_ledger_file, render_text, SourceFile};

const GTM: &str = r#"<script src="https://www.googletagmanager.com/gtag/js"></script>"#;
const HOTJAR: &str = r#"<script src="https://static.hotjar.com/c/hotjar-1.js"></script>"#;
const COOKIE_JS: &str = r#"document.cookie = "a=1";"#;
const EMAIL_INPUT: &str = r#"<input type="email" name="email">"#;
const GEO: &str = "navigator.geolocation.getCurrentPosition(show);";

fn file(path: &str, text: &str) -> SourceFile {
    SourceFile {
        path: path.to_owned(),
        text: text.to_owned(),
    }
}

fn ids(files: &[SourceFile]) -> Vec<String> {
    let mut ids: Vec<String> = analyze(files)
        .findings
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect();
    ids.sort();
    ids
}

fn expect(list: &[&str]) -> Vec<String> {
    list.iter().map(|id| (*id).to_owned()).collect()
}

#[test]
fn without_a_policy_there_is_nothing_to_compare() {
    let files = [file("index.html", GTM)];
    assert_eq!(ids(&files), expect(&[]));
    let analysis = analyze(&files);
    let vendor = analysis
        .ledger
        .vendors
        .first()
        .expect("a service was found");
    assert_eq!(vendor.vendor, "Google Analytics");
    assert_eq!(vendor.disclosed, None);
}

#[test]
fn an_unnamed_service_is_flagged_where_it_is_used() {
    let page = format!("<html>\n<body>\n{HOTJAR}");
    let analysis = analyze(&[file("index.html", &page), file("privacy.md", "Hello.")]);
    let finding = analysis.findings.first().expect("one finding");
    assert_eq!(finding.rule_id, "DRIFT-001");
    assert_eq!(finding.severity, Severity::Medium);
    assert_eq!(finding.file, "index.html");
    assert_eq!(finding.line, 3);
}

#[test]
fn a_named_service_is_not_flagged() {
    let files = [
        file("index.html", GTM),
        file("privacy.md", "We use Google Analytics to count visits."),
    ];
    assert_eq!(ids(&files), expect(&[]));
    let analysis = analyze(&files);
    assert_eq!(analysis.ledger.vendors[0].disclosed, Some(true));
}

#[test]
fn services_are_found_in_package_manifests() {
    let manifest = r#"{"dependencies":{"@sentry/browser":"^8","stripe":"^14","left-pad":"1"}}"#;
    let files = [
        file("package.json", manifest),
        file("privacy.md", "Nothing here."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-001", "DRIFT-001"]));
}

#[test]
fn python_requirements_are_read() {
    let requirements = "openai==1.2\nflask>=2\n# comment\nsentry_sdk[flask]==2";
    let files = [
        file("requirements.txt", requirements),
        file("privacy.md", "Nothing here."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-001", "DRIFT-001"]));
}

#[test]
fn lookalike_domains_are_ignored() {
    let code = r#"fetch("https://mysentry.io/x"); fetch("https://notmixpanel.com");"#;
    let analysis = analyze(&[file("a.js", code), file("privacy.md", "Hello.")]);
    assert_eq!(analysis.ledger.vendors, Vec::new());
}

#[test]
fn cookies_need_a_mention() {
    let silent = [
        file("app.js", COOKIE_JS),
        file("privacy.md", "We respect your privacy."),
    ];
    assert_eq!(ids(&silent), expect(&["DRIFT-002"]));
    let covered = [
        file("app.js", COOKIE_JS),
        file("privacy.md", "We use cookies to keep you signed in."),
    ];
    assert_eq!(ids(&covered), expect(&[]));
}

#[test]
fn personal_data_needs_a_mention() {
    let form = format!("{EMAIL_INPUT}\n<input name=\"dob\">");
    let files = [
        file("form.html", &form),
        file("app.js", GEO),
        file("privacy.md", "Hello."),
    ];
    assert_eq!(
        ids(&files),
        expect(&["DRIFT-003", "DRIFT-003", "DRIFT-003"])
    );
}

#[test]
fn a_policy_that_covers_the_data_passes() {
    let form = format!("{EMAIL_INPUT}\n<input name=\"dob\">");
    let policy = "We collect your email address, your date of birth and your location.";
    let files = [
        file("form.html", &form),
        file("app.js", GEO),
        file("privacy.md", policy),
    ];
    assert_eq!(ids(&files), expect(&[]));
}

#[test]
fn a_no_cookies_promise_is_checked_against_the_code() {
    let files = [
        file("app.js", COOKIE_JS),
        file("privacy.md", "We do not use cookies."),
    ];
    let analysis = analyze(&files);
    let finding = analysis.findings.first().expect("one finding");
    assert_eq!(analysis.findings.len(), 1);
    assert_eq!(finding.rule_id, "DRIFT-004");
    assert_eq!(finding.severity, Severity::High);
    assert_eq!(finding.file, "privacy.md");
    assert_eq!(finding.line, 1);
    assert!(finding.message.contains("app.js:1"));
}

#[test]
fn a_conditional_promise_is_not_a_contradiction() {
    let policy = "We do not use cookies except to keep you signed in.";
    let files = [file("app.js", COOKIE_JS), file("privacy.md", policy)];
    assert_eq!(ids(&files), expect(&[]));
}

#[test]
fn a_no_sharing_promise_is_checked_against_services() {
    let manifest = r#"{"dependencies":{"mixpanel-browser":"^2"}}"#;
    let files = [
        file("package.json", manifest),
        file("privacy.md", "We never share your data with third parties."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-001", "DRIFT-004"]));
}

#[test]
fn a_no_tracking_promise_is_checked_against_trackers() {
    let files = [
        file("index.html", HOTJAR),
        file("privacy.md", "We do not track you."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-001", "DRIFT-004"]));
}

#[test]
fn a_narrow_tracking_promise_is_left_alone() {
    let files = [
        file("index.html", HOTJAR),
        file("privacy.md", "We do not track your location."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-001"]));
}

#[test]
fn a_no_collection_promise_is_checked_against_forms() {
    let files = [
        file("form.html", EMAIL_INPUT),
        file("privacy.md", "We do not collect any personal information."),
    ];
    assert_eq!(ids(&files), expect(&["DRIFT-003", "DRIFT-004"]));
}

#[test]
fn promises_about_children_are_left_alone() {
    let policy = "We do not collect personal information from children.";
    let files = [file("form.html", EMAIL_INPUT), file("privacy.md", policy)];
    assert_eq!(ids(&files), expect(&["DRIFT-003"]));
}

#[test]
fn stale_mentions_are_informational() {
    let files = [
        file("app.js", "const a = 1;"),
        file("privacy.md", "We use Mixpanel for analytics."),
    ];
    let analysis = analyze(&files);
    assert_eq!(ids(&files), expect(&["DRIFT-005"]));
    assert_eq!(analysis.findings[0].severity, Severity::Info);
}

#[test]
fn policy_pages_are_not_evidence() {
    let page = r#"<a href="https://www.hotjar.com/privacy">x</a> we use Hotjar."#;
    let analysis = analyze(&[file("app.js", "const a = 1;"), file("privacy.html", page)]);
    assert_eq!(analysis.ledger.vendors, Vec::new());
}

#[test]
fn policy_files_are_recognized_by_name() {
    let analysis = analyze(&[
        file("PRIVACY.md", "x"),
        file("app/privacy/page.tsx", "x"),
        file("src/PrivacyBanner.tsx", "x"),
        file("docs/notes.md", "x"),
    ]);
    assert_eq!(
        analysis.ledger.policy_files,
        expect(&["PRIVACY.md", "app/privacy/page.tsx"])
    );
}

#[test]
fn html_policies_are_read_as_text() {
    let manifest = r#"{"dependencies":{"stripe":"1"}}"#;
    let files = [
        file("package.json", manifest),
        file(
            "privacy.html",
            "<p>We use <strong>Stripe</strong> to take payments.</p>",
        ),
    ];
    assert_eq!(ids(&files), expect(&[]));
}

#[test]
fn the_text_view_lists_everything() {
    let analysis = analyze(&[file("index.html", HOTJAR), file("privacy.md", "Hello.")]);
    let text = render_text(&analysis.ledger);
    assert!(text.contains("Privacy policy: privacy.md"));
    assert!(text.contains("Third parties (1)"));
    assert!(text.contains("Hotjar"));
    assert!(text.contains("index.html:1"));
    assert!(text.contains("NOT in policy"));
    assert!(text.contains("none set"));
}

#[test]
fn the_text_view_says_when_there_is_no_policy() {
    let text = render_text(&analyze(&[file("index.html", GTM)]).ledger);
    assert!(text.contains("Privacy policy: none found"));
    assert!(text.contains("no policy to compare"));
}

#[test]
fn the_ledger_serializes_to_json() {
    let analysis = analyze(&[file("index.html", HOTJAR), file("privacy.md", "Hello.")]);
    let json = serde_json::to_string(&analysis.ledger).unwrap();
    assert!(json.contains("\"vendor\":\"Hotjar\""));
    assert!(json.contains("\"kind\":\"analytics\""));
    assert!(json.contains("\"disclosed\":false"));
}

#[test]
fn odd_input_does_not_panic() {
    let long_line = "a".repeat(200_000);
    let policy = "\u{130}stanbul \u{130} \u{df} \u{fb01} We do not use cookies. <<>> &#39; \u{0}";
    let analysis = analyze(&[
        file("min.js", &long_line),
        file("empty.html", ""),
        file("privacy.md", policy),
        file("odd.html", "<<<>>> &#39; \u{0}"),
    ]);
    assert_eq!(analysis.ledger.policy_files.len(), 1);
}

#[test]
fn the_analyzer_reads_the_right_files() {
    for path in [
        "app/package.json",
        "src/App.TSX",
        "PRIVACY.md",
        "server/main.py",
        "requirements.txt",
    ] {
        assert!(is_ledger_file(Path::new(path)), "{path}");
    }
    for path in ["src/lib.rs", "styles/site.css", "image.png", "Cargo.toml"] {
        assert!(!is_ledger_file(Path::new(path)), "{path}");
    }
}
