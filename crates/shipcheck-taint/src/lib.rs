//! Syntax-tree taint analysis for Shipcheck security rules.

mod analyzer;
mod catalog;
mod lang;

use shipcheck_core::Finding;
use tree_sitter::Parser;

pub use lang::Lang;

/// Parses `source` and reports places where user input reaches a dangerous call.
#[must_use]
pub fn analyze(lang: Lang, source: &str, file: &str) -> Vec<Finding> {
    let mut parser = Parser::new();
    if parser.set_language(&lang.grammar()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    analyzer::Analyzer::new(lang, source, file).run(tree.root_node())
}
