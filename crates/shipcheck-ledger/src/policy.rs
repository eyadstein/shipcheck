//! Privacy policy text, cleaned up so it can be searched.

use std::sync::LazyLock;

use regex::Regex;

use crate::SourceFile;

/// File extensions a policy page can have.
pub(crate) const POLICY_EXTENSIONS: &[&str] = &[
    "md", "mdx", "txt", "html", "htm", "tsx", "jsx", "vue", "svelte", "astro",
];

/// File names, without extension, that mark a policy document.
const POLICY_NAMES: &[&str] = &[
    "privacy",
    "privacy-policy",
    "privacypolicy",
    "privacy-notice",
    "privacy-statement",
    "data-policy",
    "cookie-policy",
    "cookie-notice",
];

/// File names that count as the policy when their folder carries a policy name.
const INDEX_NAMES: &[&str] = &["index", "page", "readme"];

static TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<[^>]*>").expect("tag pattern is valid"));

/// Whether a project relative path looks like a privacy policy.
pub(crate) fn is_policy_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase().replace('\\', "/");
    let mut parts = lower.rsplit('/');
    let Some(file) = parts.next() else {
        return false;
    };
    let Some((stem, extension)) = file.rsplit_once('.') else {
        return false;
    };
    if !POLICY_EXTENSIONS.contains(&extension) {
        return false;
    }
    let stem = stem.replace('_', "-");
    if POLICY_NAMES.contains(&stem.as_str()) {
        return true;
    }
    let parent = parts.next().map(|dir| dir.replace('_', "-"));
    INDEX_NAMES.contains(&stem.as_str())
        && parent.is_some_and(|dir| POLICY_NAMES.contains(&dir.as_str()))
}

/// A policy document as lowercase plain text, with a map back to its line numbers.
pub(crate) struct Policy {
    pub(crate) path: String,
    text: String,
    starts: Vec<usize>,
}

impl Policy {
    pub(crate) fn from_file(file: &SourceFile) -> Self {
        let cleaned = clean(&file.text);
        let mut text = String::new();
        let mut starts = Vec::new();
        for line in cleaned.lines() {
            starts.push(text.len());
            text.push_str(&collapse(line));
            text.push(' ');
        }
        Self {
            path: file.path.clone(),
            text,
            starts,
        }
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// The one based line a byte offset of the cleaned text came from.
    pub(crate) fn line_of(&self, offset: usize) -> u32 {
        let count = self.starts.partition_point(|&start| start <= offset);
        u32::try_from(count.max(1)).unwrap_or(u32::MAX)
    }

    /// Offset of the first whole word match. A trailing "s" counts as a plural.
    pub(crate) fn find_word(&self, term: &str) -> Option<usize> {
        if term.is_empty() {
            return None;
        }
        self.text
            .match_indices(term)
            .map(|(start, _)| start)
            .find(|&start| {
                let before = self.text[..start].chars().next_back();
                let rest = &self.text[start + term.len()..];
                !before.is_some_and(char::is_alphanumeric) && ends_word(rest)
            })
    }

    pub(crate) fn mentions(&self, term: &str) -> bool {
        self.find_word(term).is_some()
    }

    pub(crate) fn mentions_any(&self, terms: &[&str]) -> bool {
        terms.iter().any(|term| self.mentions(term))
    }
}

fn ends_word(rest: &str) -> bool {
    let mut chars = rest.chars();
    match chars.next() {
        None => true,
        Some(first) if !first.is_alphanumeric() => true,
        Some('s') => chars.next().is_none_or(|next| !next.is_alphanumeric()),
        Some(_) => false,
    }
}

/// Removes markup and normalizes quotes, keeping every line break.
fn clean(raw: &str) -> String {
    let without_tags = TAG.replace_all(raw, |caps: &regex::Captures| {
        let breaks = caps[0].matches('\n').count();
        format!(" {}", "\n".repeat(breaks))
    });
    without_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&rsquo;", "'")
        .replace("&lsquo;", "'")
        .replace(['\u{2019}', '\u{2018}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"")
}

fn collapse(line: &str) -> String {
    line.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(text: &str) -> Policy {
        Policy::from_file(&SourceFile {
            path: "privacy.md".to_owned(),
            text: text.to_owned(),
        })
    }

    #[test]
    fn words_match_on_boundaries_and_plurals() {
        let found = policy("We use cookies and the Stripe's checkout. Emails are sent.");
        assert!(found.mentions("cookie"));
        assert!(found.mentions("stripe"));
        assert!(found.mentions("email"));
        assert!(!found.mentions("strip"));
        assert!(!found.mentions("check"));
    }

    #[test]
    fn html_tags_and_entities_are_removed() {
        let page = policy("<p>We&nbsp;use <b>Stripe</b>&rsquo;s tools.</p>");
        assert!(page.mentions("stripe"));
        assert!(page.mentions("we use"));
    }

    #[test]
    fn line_numbers_survive_cleaning() {
        let page = policy("# Title\n\n<div\n class=\"x\">Hello</div>\nWe use cookies.\n");
        let offset = page.find_word("cookies").unwrap();
        assert_eq!(page.line_of(offset), 5);
    }

    #[test]
    fn policy_paths_are_recognized() {
        for path in [
            "PRIVACY.md",
            "docs/privacy-policy.md",
            "legal\\privacy_policy.html",
            "app/privacy/page.tsx",
            "cookie-policy.mdx",
        ] {
            assert!(is_policy_path(path), "{path}");
        }
        for path in [
            "src/PrivacyBanner.tsx",
            "docs/notes.md",
            "privacy.rs",
            "lib/privacy.ts",
            "README.md",
        ] {
            assert!(!is_policy_path(path), "{path}");
        }
    }
}
