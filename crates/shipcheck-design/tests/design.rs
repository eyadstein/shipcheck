use std::path::Path;

use shipcheck_design::{analyze, is_design_file, SourceFile};

fn ids(files: &[(&str, &str)]) -> Vec<String> {
    let sources: Vec<SourceFile> = files
        .iter()
        .map(|(path, text)| SourceFile {
            path: (*path).to_owned(),
            text: (*text).to_owned(),
        })
        .collect();
    let mut found: Vec<String> = analyze(&sources)
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect();
    found.sort();
    found.dedup();
    found
}

fn fires(id: &str, files: &[(&str, &str)]) -> bool {
    ids(files).iter().any(|found| found == id)
}

#[test]
fn clean_markup_has_no_findings() {
    let clean = [(
        "page.tsx",
        r#"<h1 className="text-4xl font-bold">Hello</h1>"#,
    )];
    assert_eq!(ids(&clean), Vec::<String>::new());
}

#[test]
fn recognizes_design_files() {
    assert!(is_design_file(Path::new("src/app.tsx")));
    assert!(is_design_file(Path::new("styles/Main.CSS")));
    assert!(!is_design_file(Path::new("src/main.rs")));
}

#[test]
fn gradient_text_is_flagged() {
    let bad = r#"<h1 className="bg-gradient-to-r from-pink-500 to-orange-500 bg-clip-text text-transparent">Hi</h1>"#;
    assert!(fires("DES-201", &[("hero.tsx", bad)]));
    assert!(!fires(
        "DES-201",
        &[("hero.tsx", r#"<h1 className="text-4xl">Hi</h1>"#)]
    ));
}

#[test]
fn glass_cards_need_blur_and_translucency() {
    let bad = r#"<div className="bg-white/10 backdrop-blur-md rounded-xl">"#;
    assert!(fires("DES-202", &[("card.tsx", bad)]));
    let blur_only = r#"<div className="backdrop-blur-sm">"#;
    assert!(!fires("DES-202", &[("card.tsx", blur_only)]));
}

#[test]
fn accent_borders_are_flagged_in_tailwind_and_css() {
    let tailwind = r#"<div className="border-l-4 border-purple-500 p-4">"#;
    assert!(fires("DES-203", &[("card.tsx", tailwind)]));
    let css = ".card { border-left: 4px solid #7c3aed; }";
    assert!(fires("DES-203", &[("card.css", css)]));
    let plain = r#"<div className="border border-gray-200">"#;
    assert!(!fires("DES-203", &[("card.tsx", plain)]));
}

#[test]
fn inter_only_projects_are_flagged() {
    let only_inter = "body { font-family: Inter, sans-serif; }";
    assert!(fires("DES-204", &[("style.css", only_inter)]));
    let mixed = "body { font-family: Inter, sans-serif; }\nh1 { font-family: \"Playfair Display\", serif; }";
    assert!(!fires("DES-204", &[("style.css", mixed)]));
    let next_font = r#"import { Inter } from "next/font/google";"#;
    assert!(fires("DES-204", &[("layout.tsx", next_font)]));
}

#[test]
fn popular_font_pairing_is_flagged() {
    let url = r#"@import url("https://fonts.googleapis.com/css2?family=Space+Grotesk&family=Instrument+Serif");"#;
    assert!(fires("DES-217", &[("fonts.css", url)]));
    let one = r#"@import url("https://fonts.googleapis.com/css2?family=Space+Grotesk");"#;
    assert!(!fires("DES-217", &[("fonts.css", one)]));
}

#[test]
fn weak_dark_mode_contrast_is_flagged() {
    let weak = ".dark {\n  --background: #111111;\n  --foreground: #f5f5f5;\n  --muted-foreground: #5c5c5c;\n}";
    assert!(fires("DES-205", &[("theme.css", weak)]));
    let fine = ".dark {\n  --background: #111111;\n  --muted-foreground: #a3a3a3;\n}";
    assert!(!fires("DES-205", &[("theme.css", fine)]));
}

#[test]
fn three_icon_boxes_are_flagged() {
    let boxes = r#"import { Zap, Shield, Star } from "lucide-react";
export const Features = () => (
  <div className="grid md:grid-cols-3 gap-6">
    <div><Zap /><h3>Fast</h3></div>
    <div><Shield /><h3>Safe</h3></div>
    <div><Star /><h3>Nice</h3></div>
  </div>
);"#;
    assert!(fires("DES-206", &[("features.tsx", boxes)]));
    let without_icons = boxes.replace("lucide-react", "./icons");
    assert!(!fires(
        "DES-206",
        &[("features.tsx", without_icons.as_str())]
    ));
}

#[test]
fn badge_above_headline_is_flagged() {
    let bad =
        "<span className=\"rounded-full border px-3 py-1 text-xs\">New</span>\n<h1>Title</h1>";
    assert!(fires("DES-207", &[("hero.tsx", bad)]));
    assert!(!fires("DES-207", &[("hero.tsx", "<h1>Title</h1>")]));
}

#[test]
fn lucide_in_many_files_is_flagged() {
    let import = r#"import { Zap } from "lucide-react";"#;
    let names = ["a.tsx", "b.tsx", "c.tsx", "d.tsx", "e.tsx", "f.tsx"];
    let six: Vec<(&str, &str)> = names.iter().map(|name| (*name, import)).collect();
    assert!(fires("DES-208", &six));
    assert!(!fires("DES-208", &six[..5]));
}

#[test]
fn default_shadcn_theme_is_flagged() {
    let defaults = ":root {\n  --background: 0 0% 100%;\n  --primary: 222.2 47.4% 11.2%;\n}";
    assert!(fires("DES-209", &[("globals.css", defaults)]));
    let custom = ":root {\n  --background: 0 0% 100%;\n  --primary: 262 83% 58%;\n}";
    assert!(!fires("DES-209", &[("globals.css", custom)]));
}

#[test]
fn scroll_reveal_libraries_are_flagged() {
    let bad = r#"<div data-aos="fade-up">Hi</div>"#;
    assert!(fires("DES-210", &[("page.html", bad)]));
}

#[test]
fn cursor_following_beam_is_flagged() {
    let bad = "window.addEventListener(\"mousemove\", (e) => {\n  el.style.setProperty(\"--mouse-x\", e.clientX + \"px\");\n});";
    assert!(fires("DES-211", &[("beam.js", bad)]));
    let click = "window.addEventListener(\"click\", () => {});";
    assert!(!fires("DES-211", &[("beam.js", click)]));
}

#[test]
fn opacity_only_button_hover_is_flagged() {
    let bad = r#"<button className="rounded px-4 hover:opacity-80">Go</button>"#;
    assert!(fires("DES-212", &[("cta.tsx", bad)]));
    let not_a_button = r#"<div className="hover:opacity-80">Go</div>"#;
    assert!(!fires("DES-212", &[("cta.tsx", not_a_button)]));
    assert!(fires(
        "DES-212",
        &[("cta.css", ".btn:hover { opacity: 0.8; }")]
    ));
}

#[test]
fn off_grid_spacing_is_flagged() {
    let messy = ".a { padding: 13px; }\n.b { margin: 7px 22px; }\n.c { gap: 10px 14px; }\n.d { padding: 18px; }";
    assert!(fires("DES-213", &[("style.css", messy)]));
    let tidy = ".a { padding: 8px; margin: 16px; gap: 24px; }";
    assert!(!fires("DES-213", &[("style.css", tidy)]));
    let arbitrary = r#"<div className="p-[13px] m-[7px] gap-[22px] px-[9px] py-[11px] mt-[5px]">"#;
    assert!(fires("DES-213", &[("box.tsx", arbitrary)]));
}

#[test]
fn many_em_dashes_are_flagged() {
    let eight = "word \u{2014} word. ".repeat(8);
    let seven = "word \u{2014} word. ".repeat(7);
    assert!(fires("DES-214", &[("page.html", eight.as_str())]));
    assert!(!fires("DES-214", &[("page.html", seven.as_str())]));
}

#[test]
fn buzzword_copy_is_flagged() {
    let bad = "Seamless and effortless, an all-in-one cutting-edge tool. Unleash it.";
    assert!(fires("DES-215", &[("page.html", bad)]));
    assert!(!fires(
        "DES-215",
        &[("page.html", "A plain description of the product.")]
    ));
}

#[test]
fn serif_italic_accents_are_flagged() {
    let tailwind = r#"<em className="font-serif italic">fast</em>"#;
    assert!(fires("DES-216", &[("hero.tsx", tailwind)]));
    assert!(!fires(
        "DES-216",
        &[("hero.tsx", r#"<em className="italic">fast</em>"#)]
    ));
    let css = ".accent { font-style: italic; font-family: \"Instrument Serif\", serif; }";
    assert!(fires("DES-216", &[("hero.css", css)]));
}

#[test]
fn grain_over_gradient_is_flagged() {
    let bad = "<svg><filter id=\"n\"><feTurbulence baseFrequency=\"0.8\"/></filter></svg>\n<div style=\"background: linear-gradient(red, blue)\"></div>";
    assert!(fires("DES-218", &[("hero.html", bad)]));
    let gradient_only = "<div style=\"background: linear-gradient(red, blue)\"></div>";
    assert!(!fires("DES-218", &[("hero.html", gradient_only)]));
}
