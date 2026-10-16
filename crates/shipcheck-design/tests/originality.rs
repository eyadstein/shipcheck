use shipcheck_core::Severity;
use shipcheck_design::{analyze, originality, render_originality, SourceFile};

fn file(path: &str, text: &str) -> SourceFile {
    SourceFile {
        path: path.to_owned(),
        text: text.to_owned(),
    }
}

const STOCK: &str = r#"<div className="bg-blue-500 text-gray-700 border-indigo-600 hover:bg-purple-600 from-pink-500">"#;
const OWN_COLORS: &str = ":root{--brand:#0b5c63;--paper:#f6f3ec;--ink:#1d1b16;}";

#[test]
fn too_little_evidence_means_no_judgement() {
    assert_eq!(originality(&[file("a.tsx", "<h1>Hello</h1>")]), None);
    let two = "a{color:#111111;background:#222222}";
    assert_eq!(originality(&[file("a.css", two)]), None);
}

#[test]
fn stock_tailwind_colors_score_zero() {
    let index = originality(&[file("a.tsx", STOCK)]).expect("a palette was found");
    assert_eq!(index.score, 0);
    assert_eq!(index.dimensions.len(), 1);
    let palette = &index.dimensions[0];
    assert_eq!(palette.name, "Palette");
    assert_eq!((palette.total, palette.defaults), (5, 5));
    assert_eq!(
        palette.default_items,
        vec![
            "blue-500",
            "gray-700",
            "indigo-600",
            "pink-500",
            "purple-600"
        ]
    );
}

#[test]
fn an_original_palette_scores_full_marks() {
    let index = originality(&[file("a.css", OWN_COLORS)]).expect("a palette was found");
    assert_eq!(index.score, 100);
    assert_eq!(index.dimensions[0].own_items.len(), 3);
}

#[test]
fn renamed_tailwind_colors_count_as_your_own() {
    let page = r#"<div className="bg-brand-500 text-ink-900 border-paper-200 bg-blue-500">"#;
    let index = originality(&[file("a.tsx", page)]).expect("a palette was found");
    let palette = &index.dimensions[0];
    assert_eq!((palette.total, palette.defaults), (4, 1));
    assert_eq!(palette.own_items, vec!["brand-500", "ink-900", "paper-200"]);
    assert_eq!(index.score, 75);
}

#[test]
fn colors_close_to_a_default_count_as_that_default() {
    let css = "a{color:#3b82f6;background:#3c83f5;border-color:#111827}";
    let index = originality(&[file("a.css", css)]).expect("a palette was found");
    assert_eq!(index.dimensions[0].defaults, 3);
    assert_eq!(index.score, 0);
}

#[test]
fn black_and_white_say_nothing_about_a_palette() {
    let css = "a{color:#fff;background:#000;border:1px solid #ffffff;--a:#123456;--b:#654321;--c:#0a0b0c}";
    let index = originality(&[file("a.css", css)]).expect("a palette was found");
    assert_eq!(index.dimensions[0].total, 3);
    assert_eq!(index.score, 100);
}

#[test]
fn rgb_colors_are_read() {
    let css =
        "a{color:rgb(11, 92, 99);background:rgba(246, 243, 236, 0.5);border-color:rgb(59 130 246)}";
    let index = originality(&[file("a.css", css)]).expect("a palette was found");
    let palette = &index.dimensions[0];
    assert_eq!((palette.total, palette.defaults), (3, 1));
    assert_eq!(index.score, 66);
}

#[test]
fn page_links_are_not_colors() {
    let page = r##"<a href="#add">x</a> <a href="#bad">y</a> <a href="#fade">z</a>"##;
    assert_eq!(originality(&[file("a.html", page)]), None);
}

#[test]
fn popular_fonts_are_defaults_and_custom_ones_are_not() {
    let css = "body { font-family: Inter, sans-serif; }\nh1 { font-family: \"GT Walsheim\", serif; }\n:root { --font-display: \"Recoleta\", serif; }\n";
    let index = originality(&[file("a.css", css)]).expect("fonts were found");
    assert_eq!(index.dimensions.len(), 1);
    let fonts = &index.dimensions[0];
    assert_eq!(fonts.name, "Typography");
    assert_eq!(fonts.default_items, vec!["inter"]);
    assert_eq!(fonts.own_items, vec!["gt walsheim", "recoleta"]);
    assert_eq!(index.score, 66);
}

#[test]
fn untouched_shadcn_tokens_count_as_defaults() {
    let css = ":root { --background: 0 0% 100%; --primary: 222.2 47.4% 11.2%; }";
    let index = originality(&[file("a.css", css)]).expect("tokens were found");
    assert_eq!(index.dimensions[0].name, "Framework tokens");
    assert_eq!(index.score, 0);
}

#[test]
fn weights_adapt_to_what_is_present() {
    let css = format!("{OWN_COLORS} body{{font-family: Inter, sans-serif;}}");
    let index = originality(&[file("a.css", &css)]).expect("something was found");
    assert_eq!(index.dimensions.len(), 2);
    assert_eq!(index.score, 62);
}

#[test]
fn a_generic_design_gets_a_finding_and_an_original_one_does_not() {
    let generic = analyze(&[file("a.tsx", STOCK)]);
    let finding = generic
        .iter()
        .find(|finding| finding.rule_id == "DES-301")
        .expect("DES-301 is reported");
    assert_eq!(finding.file, "(project)");
    assert_eq!(finding.severity, Severity::Low);

    let original = analyze(&[file("a.css", OWN_COLORS)]);
    assert!(original.iter().all(|finding| finding.rule_id != "DES-301"));
}

#[test]
fn a_font_only_judgement_does_not_raise_a_finding() {
    let css = "body { font-family: Inter, sans-serif; }";
    let findings = analyze(&[file("a.css", css)]);
    assert!(findings.iter().all(|finding| finding.rule_id != "DES-301"));
}

#[test]
fn the_report_lists_defaults_and_your_own() {
    let index = originality(&[file("a.tsx", STOCK)]).expect("a palette was found");
    let text = render_originality(&index);
    assert!(text.contains("Originality index: 0/100"));
    assert!(text.contains("Palette (weight 50): 0% your own, 5 of 5 are defaults"));
    assert!(text.contains("defaults: blue-500, gray-700"));
    assert!(text.contains("your own: none"));
}
