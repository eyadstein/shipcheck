//! Walks a syntax tree and tracks which variables hold user input.

use std::collections::HashSet;

use shipcheck_core::{Category, Finding};
use tree_sitter::Node;

use crate::catalog::{
    Class, JS_REFLECT_CALLS, JS_SANITIZERS, JS_SINKS, JS_SOURCES, PY_SANITIZERS, PY_SHELL_CALLS,
    PY_SINKS, PY_SOURCES,
};
use crate::lang::Lang;

/// Deepest syntax nesting the analyzer follows. Guards against stack overflow.
const MAX_DEPTH: usize = 200;

pub(crate) struct Analyzer<'a> {
    lang: Lang,
    src: &'a str,
    file: &'a str,
    tainted: HashSet<String>,
    findings: Vec<Finding>,
}

impl<'a> Analyzer<'a> {
    pub(crate) fn new(lang: Lang, src: &'a str, file: &'a str) -> Self {
        Self {
            lang,
            src,
            file,
            tainted: HashSet::new(),
            findings: Vec::new(),
        }
    }

    pub(crate) fn run(mut self, root: Node<'_>) -> Vec<Finding> {
        self.walk(root, 0);
        self.findings
    }

    fn text(&self, node: Node<'_>) -> &'a str {
        node.utf8_text(self.src.as_bytes()).unwrap_or_default()
    }

    fn walk(&mut self, node: Node<'_>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        self.check_sinks(node);
        // Taint created inside a function does not leak out of it.
        let saved = is_function(node.kind()).then(|| self.tainted.clone());
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.walk(child, depth + 1);
        }
        if let Some(previous) = saved {
            self.tainted = previous;
        }
        self.update_env(node);
    }

    fn update_env(&mut self, node: Node<'_>) {
        let (target, value) = match node.kind() {
            "variable_declarator" => (
                node.child_by_field_name("name"),
                node.child_by_field_name("value"),
            ),
            "assignment_expression"
            | "augmented_assignment_expression"
            | "assignment"
            | "augmented_assignment"
            | "for_in_statement"
            | "for_statement" => (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ),
            _ => return,
        };
        let (Some(target), Some(value)) = (target, value) else {
            return;
        };
        let dirty = self.is_tainted(value, 0);
        let augmented = node.kind().starts_with("augmented");
        let mut names = Vec::new();
        self.collect_names(target, &mut names);
        for name in names {
            if dirty {
                self.tainted.insert(name);
            } else if !augmented {
                self.tainted.remove(&name);
            }
        }
    }

    fn collect_names(&self, node: Node<'_>, out: &mut Vec<String>) {
        match node.kind() {
            "identifier" | "shorthand_property_identifier_pattern" => {
                out.push(self.text(node).to_owned());
            }
            "member_expression" | "subscript_expression" | "attribute" | "subscript" => {
                if let Some(object) = node.named_child(0) {
                    self.collect_names(object, out);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    self.collect_names(child, out);
                }
            }
        }
    }

    fn is_tainted(&self, node: Node<'_>, depth: usize) -> bool {
        let kind = node.kind();
        if depth > MAX_DEPTH || is_function(kind) || kind == "comment" {
            return false;
        }
        if self.is_source(node) {
            return true;
        }
        match kind {
            "identifier" | "shorthand_property_identifier" => {
                return self.tainted.contains(self.text(node));
            }
            "attribute" => {
                return node
                    .child_by_field_name("object")
                    .is_some_and(|object| self.is_tainted(object, depth + 1));
            }
            "keyword_argument" => {
                return node
                    .child_by_field_name("value")
                    .is_some_and(|value| self.is_tainted(value, depth + 1));
            }
            _ => {}
        }
        if is_call(kind) && self.is_sanitizer_call(node) {
            return false;
        }
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if self.is_tainted(child, depth + 1) {
                return true;
            }
        }
        false
    }

    fn is_source(&self, node: Node<'_>) -> bool {
        let kind = node.kind();
        if is_call(kind) {
            let callee = self.callee(node);
            return if self.lang.is_python() {
                callee == "input"
            } else {
                callee == "prompt"
            };
        }
        if !matches!(
            kind,
            "member_expression" | "subscript_expression" | "attribute" | "subscript"
        ) {
            return false;
        }
        let text = normalize(self.text(node));
        let prefixes = if self.lang.is_python() {
            PY_SOURCES
        } else {
            JS_SOURCES
        };
        prefixes.iter().any(|prefix| {
            text.strip_prefix(*prefix)
                .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
        })
    }

    fn is_sanitizer_call(&self, node: Node<'_>) -> bool {
        let list = if self.lang.is_python() {
            PY_SANITIZERS
        } else {
            JS_SANITIZERS
        };
        name_matches(&self.callee(node), list, false)
    }

    fn callee(&self, node: Node<'_>) -> String {
        let field = if node.kind() == "new_expression" {
            "constructor"
        } else {
            "function"
        };
        node.child_by_field_name(field)
            .map_or_else(String::new, |callee| normalize(self.text(callee)))
    }

    fn check_sinks(&mut self, node: Node<'_>) {
        match node.kind() {
            "call_expression" | "call" | "new_expression" => self.check_call(node),
            "assignment_expression" => self.check_html_assignment(node),
            "jsx_attribute" => self.check_jsx(node),
            _ => {}
        }
    }

    fn check_call(&mut self, node: Node<'_>) {
        let callee = self.callee(node);
        if callee.is_empty() {
            return;
        }
        let args = arguments(node);
        let table = if self.lang.is_python() {
            PY_SINKS
        } else {
            JS_SINKS
        };
        for sink in table {
            if !name_matches(&callee, sink.names, sink.exact) {
                continue;
            }
            let hit = if sink.any_arg {
                args.iter().any(|arg| self.is_tainted(*arg, 0))
            } else {
                args.first().is_some_and(|arg| self.is_tainted(*arg, 0))
            };
            if hit {
                self.report(node, sink.class);
            }
        }
        if self.lang.is_python() {
            self.check_shell_call(node, &callee, &args);
        } else {
            self.check_reflected(node, &callee, &args);
        }
    }

    fn check_shell_call(&mut self, node: Node<'_>, callee: &str, args: &[Node<'_>]) {
        if !name_matches(callee, PY_SHELL_CALLS, true) {
            return;
        }
        let shell = args.iter().any(|arg| {
            arg.kind() == "keyword_argument"
                && arg
                    .child_by_field_name("name")
                    .is_some_and(|name| self.text(name) == "shell")
                && arg
                    .child_by_field_name("value")
                    .is_some_and(|value| self.text(value) == "True")
        });
        let first_tainted = args.first().is_some_and(|arg| self.is_tainted(*arg, 0));
        if shell && first_tainted {
            self.report(node, Class::CommandInjection);
        }
    }

    fn check_reflected(&mut self, node: Node<'_>, callee: &str, args: &[Node<'_>]) {
        if !name_matches(callee, JS_REFLECT_CALLS, true) {
            return;
        }
        let Some(first) = args.first() else {
            return;
        };
        let html_like = matches!(first.kind(), "template_string" | "binary_expression")
            && self.text(*first).contains('<');
        if html_like && self.is_tainted(*first, 0) {
            self.report(node, Class::Xss);
        }
    }

    fn check_html_assignment(&mut self, node: Node<'_>) {
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else {
            return;
        };
        let target = self.text(left);
        let writes_html = target.ends_with(".innerHTML") || target.ends_with(".outerHTML");
        if writes_html && self.is_tainted(right, 0) {
            self.report(node, Class::Xss);
        }
    }

    fn check_jsx(&mut self, node: Node<'_>) {
        if self.text(node).starts_with("dangerouslySetInnerHTML") && self.is_tainted(node, 0) {
            self.report(node, Class::Xss);
        }
    }

    fn report(&mut self, node: Node<'_>, class: Class) {
        let info = class.info();
        let line = u32::try_from(node.start_position().row + 1).unwrap_or(u32::MAX);
        if self
            .findings
            .iter()
            .any(|found| found.rule_id == info.id && found.line == line)
        {
            return;
        }
        self.findings.push(Finding {
            rule_id: info.id.to_owned(),
            category: Category::Security,
            severity: info.severity,
            message: info.message.to_owned(),
            file: self.file.to_owned(),
            line,
            fix: Some(info.fix.to_owned()),
        });
    }
}

fn arguments(call: Node<'_>) -> Vec<Node<'_>> {
    let Some(list) = call.child_by_field_name("arguments") else {
        return Vec::new();
    };
    let mut cursor = list.walk();
    let mut out = Vec::new();
    for child in list.named_children(&mut cursor) {
        if child.kind() != "comment" {
            out.push(child);
        }
    }
    out
}

fn normalize(text: &str) -> String {
    text.replace("?.", ".")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn name_matches(callee: &str, names: &[&str], exact: bool) -> bool {
    names.iter().any(|name| {
        callee == *name
            || (!exact
                && callee
                    .strip_suffix(*name)
                    .is_some_and(|rest| rest.ends_with('.')))
    })
}

fn is_call(kind: &str) -> bool {
    matches!(kind, "call_expression" | "call" | "new_expression")
}

fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_expression"
            | "function"
            | "generator_function"
            | "generator_function_declaration"
            | "arrow_function"
            | "method_definition"
            | "function_definition"
            | "lambda"
    )
}
