use std::collections::HashSet;
use std::path::Path;

use shipcheck_engine::Catalog;

fn shipped() -> Catalog {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules");
    Catalog::load(&dir).expect("shipped rules load")
}

#[test]
fn shipped_rules_pass_their_own_examples() {
    let catalog = shipped();
    assert!(catalog.rules().len() >= 20);
    assert_eq!(catalog.check_examples(), Vec::<String>::new());
}

#[test]
fn shipped_rule_ids_are_unique() {
    let catalog = shipped();
    let mut seen = HashSet::new();
    for compiled in catalog.rules() {
        assert!(
            seen.insert(compiled.rule.id.clone()),
            "duplicate rule id {}",
            compiled.rule.id
        );
    }
}
