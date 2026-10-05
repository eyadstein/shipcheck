use std::collections::BTreeSet;
use std::path::Path;

use shipcheck_engine::{scan, Catalog};

#[test]
fn demo_app_trips_every_kind_of_check() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let catalog = Catalog::load(&base.join("rules")).expect("shipped rules load");
    let found: BTreeSet<String> = scan(&base.join("examples/demo-app"), &catalog)
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect();
    for id in [
        "LEGAL-001",
        "SEC-101",
        "DES-201",
        "DES-202",
        "DES-203",
        "DES-204",
        "DES-205",
        "DES-207",
        "DES-209",
        "DES-210",
        "DES-212",
        "DES-213",
        "DES-214",
        "DES-215",
    ] {
        assert!(found.contains(id), "demo app should trigger {id}");
    }
}
