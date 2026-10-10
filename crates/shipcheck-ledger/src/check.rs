//! Turns the ledger and the policy into findings.

use shipcheck_core::{Category, Finding, Severity};

use crate::claims;
use crate::ledger::{Evidence, Ledger};
use crate::policy::Policy;
use crate::vendors::VENDORS;

/// Findings from comparing the ledger with the policies. Without a policy there is nothing to compare.
pub(crate) fn findings(ledger: &Ledger, policies: &[Policy], has_code: bool) -> Vec<Finding> {
    if policies.is_empty() {
        return Vec::new();
    }
    let mut found = undisclosed_vendors(ledger);
    found.extend(undisclosed_cookies(ledger));
    found.extend(undisclosed_data(ledger));
    for policy in policies {
        found.extend(claims::contradictions(ledger, policy));
    }
    if has_code {
        found.extend(stale_mentions(ledger, policies));
    }
    found
}

fn make(
    id: &str,
    severity: Severity,
    message: String,
    evidence: &Evidence,
    fix: String,
) -> Finding {
    Finding {
        rule_id: id.to_owned(),
        category: Category::Legal,
        severity,
        message,
        file: evidence.file.clone(),
        line: evidence.line,
        fix: Some(fix),
    }
}

fn undisclosed_vendors(ledger: &Ledger) -> Vec<Finding> {
    ledger
        .vendors
        .iter()
        .filter(|item| item.disclosed == Some(false))
        .map(|item| {
            let kind = item.kind.label();
            make(
                "DRIFT-001",
                item.kind.severity(),
                format!(
                    "{} ({kind}) is used here but the privacy policy never names it.",
                    item.vendor
                ),
                &item.evidence,
                format!(
                    "Name {} in the privacy policy and say what data it receives and why.",
                    item.vendor
                ),
            )
        })
        .collect()
}

fn undisclosed_cookies(ledger: &Ledger) -> Option<Finding> {
    let cookies = ledger.cookies.as_ref()?;
    if cookies.disclosed != Some(false) {
        return None;
    }
    Some(make(
        "DRIFT-002",
        Severity::Medium,
        "Cookies are set here but the privacy policy never mentions cookies.".to_owned(),
        &cookies.evidence,
        "Add a cookies section: which cookies, what for, and how long they last.".to_owned(),
    ))
}

fn undisclosed_data(ledger: &Ledger) -> Vec<Finding> {
    ledger
        .data
        .iter()
        .filter(|item| item.disclosed == Some(false))
        .map(|item| {
            let label = item.kind.label();
            make(
                "DRIFT-003",
                item.kind.severity(),
                format!("The code collects {label} but the privacy policy never mentions it."),
                &item.evidence,
                format!("Say in the privacy policy that you collect {label}, why, and how long you keep it."),
            )
        })
        .collect()
}

/// Services the policy names that no code uses. Informational: the code may live elsewhere.
fn stale_mentions(ledger: &Ledger, policies: &[Policy]) -> Vec<Finding> {
    let mut found = Vec::new();
    for vendor in VENDORS {
        if ledger.vendors.iter().any(|used| used.vendor == vendor.name) {
            continue;
        }
        let Some(alias) = vendor.aliases.first() else {
            continue;
        };
        for policy in policies {
            if let Some(offset) = policy.find_word(alias) {
                found.push(Finding {
                    rule_id: "DRIFT-005".to_owned(),
                    category: Category::Legal,
                    severity: Severity::Info,
                    message: format!(
                        "The privacy policy names {} but no code here uses it.",
                        vendor.name
                    ),
                    file: policy.path.clone(),
                    line: policy.line_of(offset),
                    fix: Some(
                        "Remove it from the policy if you stopped using it, or ignore this if it is used elsewhere."
                            .to_owned(),
                    ),
                });
                break;
            }
        }
    }
    found
}
