//! Plain text view of the ledger.

use crate::ledger::{Evidence, Ledger};

/// Renders the ledger as a readable report.
#[must_use]
pub fn render_text(ledger: &Ledger) -> String {
    let mut lines = vec![
        "Disclosure ledger".to_owned(),
        String::new(),
        policy_line(ledger),
        String::new(),
    ];
    lines.extend(vendor_section(ledger));
    lines.extend(cookie_section(ledger));
    lines.extend(data_section(ledger));
    lines.join("\n")
}

fn policy_line(ledger: &Ledger) -> String {
    if ledger.policy_files.is_empty() {
        "Privacy policy: none found".to_owned()
    } else {
        format!("Privacy policy: {}", ledger.policy_files.join(", "))
    }
}

fn place(evidence: &Evidence) -> String {
    format!("{}:{}", evidence.file, evidence.line)
}

fn status(disclosed: Option<bool>) -> &'static str {
    match disclosed {
        Some(true) => "named in policy",
        Some(false) => "NOT in policy",
        None => "no policy to compare",
    }
}

fn vendor_section(ledger: &Ledger) -> Vec<String> {
    let mut lines = vec![format!("Third parties ({})", ledger.vendors.len())];
    if ledger.vendors.is_empty() {
        lines.push("  none found".to_owned());
    }
    for item in &ledger.vendors {
        lines.push(format!(
            "  {:<22} {:<12} {:<28} {}",
            item.vendor,
            item.kind.label(),
            place(&item.evidence),
            status(item.disclosed)
        ));
    }
    lines.push(String::new());
    lines
}

fn cookie_section(ledger: &Ledger) -> Vec<String> {
    let mut lines = vec!["Cookies".to_owned()];
    match &ledger.cookies {
        Some(cookie) => lines.push(format!(
            "  set at {:<24} {}",
            place(&cookie.evidence),
            status(cookie.disclosed)
        )),
        None => lines.push("  none set".to_owned()),
    }
    lines.push(String::new());
    lines
}

fn data_section(ledger: &Ledger) -> Vec<String> {
    let mut lines = vec!["Personal data".to_owned()];
    if ledger.data.is_empty() {
        lines.push("  none found".to_owned());
    }
    for item in &ledger.data {
        lines.push(format!(
            "  {:<22} {:<28} {}",
            item.kind.label(),
            place(&item.evidence),
            status(item.disclosed)
        ));
    }
    lines
}
