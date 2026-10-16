use std::path::Path;

use shipcheck_engine::originality;

#[test]
fn the_demo_app_is_generic_and_the_dashboard_is_not() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let demo =
        originality(&base.join("examples/demo-app")).expect("the demo app has colors and fonts");
    assert!(
        demo.score < 40,
        "the demo app is built from defaults, got {}",
        demo.score
    );
    let dashboard = originality(&base.join("web")).expect("the dashboard has colors and fonts");
    assert!(
        dashboard.score >= 90,
        "the dashboard has its own look, got {}",
        dashboard.score
    );
}
