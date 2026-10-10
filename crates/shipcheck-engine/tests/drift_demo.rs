use std::path::Path;

use shipcheck_engine::{ledger, scan, Catalog};

#[test]
fn the_drift_demo_shows_where_the_policy_and_the_code_disagree() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let demo = base.join("examples/drift-demo");
    let catalog = Catalog::load(&base.join("rules")).expect("shipped rules load");
    let found: Vec<String> = scan(&demo, &catalog)
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect();
    for id in ["DRIFT-001", "DRIFT-003", "DRIFT-004"] {
        assert!(
            found.iter().any(|found| found == id),
            "the demo should trigger {id}"
        );
    }

    let analysis = ledger(&demo);
    assert_eq!(analysis.ledger.policy_files, vec!["privacy.md".to_owned()]);
    let names: Vec<&str> = analysis
        .ledger
        .vendors
        .iter()
        .map(|vendor| vendor.vendor.as_str())
        .collect();
    for expected in ["Google Analytics", "Hotjar", "Stripe", "Sentry", "OpenAI"] {
        assert!(
            names.contains(&expected),
            "the ledger should list {expected}"
        );
    }
}
