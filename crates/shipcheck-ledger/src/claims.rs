//! Statements a privacy policy can make that the code may contradict.

use std::sync::LazyLock;

use regex::Regex;
use shipcheck_core::{Category, Finding, Severity};

use crate::ledger::Ledger;
use crate::policy::Policy;
use crate::vendors::VendorKind;

/// How far past a claim to look for words that make it conditional.
const MAX_TAIL: usize = 160;

const SUBJECT: &str = r"(?:we|this (?:site|website|app|service)|our (?:site|website|app|service))";
const NEGATION: &str = r"(?:do not|don't|never|will not|won't|does not|doesn't)";

/// Words after a claim that make it conditional, so it is not a flat promise.
const QUALIFIERS: &[&str] = &[
    "except",
    "other than",
    "unless",
    "apart from",
    "besides",
    "beyond",
    "without your consent",
    "without consent",
    "without your permission",
    "for advertising",
    "for marketing",
    "for tracking",
    "for analytics",
    "for third",
    "from children",
    "from minors",
    "from kids",
    "under the age",
    "under 13",
    "under 16",
    "under 18",
];

/// What a claim is checked against.
#[derive(Clone, Copy)]
enum Against {
    Cookies,
    SharedData,
    Tracking,
    PersonalData,
}

struct Claim {
    pattern: Regex,
    against: Against,
    /// The claim must end its clause, so "we do not track your location" is not a claim about tracking.
    whole_clause: bool,
}

impl Claim {
    fn new(body: &str, against: Against, whole_clause: bool) -> Self {
        let source = format!(r"\b{SUBJECT} {NEGATION} {body}");
        let pattern = Regex::new(&source).expect("built-in claim pattern is valid");
        Self {
            pattern,
            against,
            whole_clause,
        }
    }

    /// Start and end of the first match that is a flat promise.
    fn first_match(&self, text: &str) -> Option<(usize, usize)> {
        self.pattern
            .find_iter(text)
            .map(|found| (found.start(), found.end()))
            .find(|&(_, end)| self.applies(&text[end..]))
    }

    fn applies(&self, tail: &str) -> bool {
        if self.whole_clause && !ends_clause(tail) {
            return false;
        }
        let sentence = rest_of_sentence(tail);
        !QUALIFIERS
            .iter()
            .any(|qualifier| sentence.contains(*qualifier))
    }
}

fn claims() -> &'static [Claim] {
    static CLAIMS: LazyLock<Vec<Claim>> = LazyLock::new(|| {
        vec![
            Claim::new(
                r"(?:use|set|store|place|drop|serve|collect) (?:any )?cookies",
                Against::Cookies,
                false,
            ),
            Claim::new(
                r"(?:(?:sell|rent|trade) (?:or|and) )?(?:share|disclose)[^.,;]{0,40}?(?:third[- ]part(?:y|ies)|anyone)",
                Against::SharedData,
                false,
            ),
            Claim::new(
                r"(?:track|profile|monitor)(?: you| your activity| users| visitors| anyone)?",
                Against::Tracking,
                true,
            ),
            Claim::new(
                r"(?:use|run|employ|include|utili[sz]e) (?:any )?(?:analytics|tracking|trackers|tracking technologies|advertising|ad trackers)",
                Against::Tracking,
                false,
            ),
            Claim::new(
                r"(?:collect|gather|store|keep|process|record)(?: any| your| user| users'| personal| or)* (?:personal )?(?:data|information)",
                Against::PersonalData,
                false,
            ),
        ]
    });
    &CLAIMS
}

fn ends_clause(tail: &str) -> bool {
    let rest = tail.trim_start();
    rest.is_empty()
        || rest.starts_with(['.', ',', ';', '!', ':'])
        || rest.starts_with("or ")
        || rest.starts_with("and ")
}

fn rest_of_sentence(tail: &str) -> String {
    tail.chars()
        .take(MAX_TAIL)
        .take_while(|ch| *ch != '.')
        .collect()
}

fn evidence_against(against: Against, ledger: &Ledger) -> Option<String> {
    match against {
        Against::Cookies => ledger.cookies.as_ref().map(|cookie| {
            format!(
                "the code sets cookies at {}:{}",
                cookie.evidence.file, cookie.evidence.line
            )
        }),
        Against::SharedData => vendor_evidence(ledger, VendorKind::shares_user_data),
        Against::Tracking => vendor_evidence(ledger, VendorKind::is_tracking),
        Against::PersonalData => ledger.data.first().map(|item| {
            format!(
                "the code collects {} at {}:{}",
                item.kind.label(),
                item.evidence.file,
                item.evidence.line
            )
        }),
    }
}

fn vendor_evidence(ledger: &Ledger, wanted: fn(VendorKind) -> bool) -> Option<String> {
    ledger
        .vendors
        .iter()
        .find(|item| wanted(item.kind))
        .map(|item| {
            format!(
                "the code uses {} ({}) at {}:{}",
                item.vendor,
                item.kind.label(),
                item.evidence.file,
                item.evidence.line
            )
        })
}

/// Promises in the policy that the code contradicts.
pub(crate) fn contradictions(ledger: &Ledger, policy: &Policy) -> Vec<Finding> {
    let text = policy.text();
    let mut found = Vec::new();
    for claim in claims() {
        let Some((start, end)) = claim.first_match(text) else {
            continue;
        };
        let Some(reason) = evidence_against(claim.against, ledger) else {
            continue;
        };
        let promise: String = text[start..end].chars().take(80).collect();
        found.push(Finding {
            rule_id: "DRIFT-004".to_owned(),
            category: Category::Legal,
            severity: Severity::High,
            message: format!("The policy says \"{promise}\" but {reason}."),
            file: policy.path.clone(),
            line: policy.line_of(start),
            fix: Some(
                "Change the policy to match what the code does, or remove what contradicts it."
                    .to_owned(),
            ),
        });
    }
    found
}
