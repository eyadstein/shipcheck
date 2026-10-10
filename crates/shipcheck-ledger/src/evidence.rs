//! Looks through source files for third party services, cookies and personal data collection.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::ledger::{DataKind, Evidence};
use crate::vendors::{vendor_index_for_package, DomainIndex};
use crate::SourceFile;

/// File extensions that count as project code.
pub(crate) const CODE_EXTENSIONS: &[&str] = &[
    "js", "jsx", "mjs", "cjs", "ts", "tsx", "vue", "svelte", "astro", "html", "htm", "php", "py",
    "rb", "go", "java", "kt", "swift", "cs", "dart",
];

/// Where one service was first seen, and how often.
pub(crate) struct VendorSighting {
    pub(crate) evidence: Evidence,
    pub(crate) count: usize,
}

/// Everything found in the code, before it is compared with a policy.
#[derive(Default)]
pub(crate) struct Sightings {
    pub(crate) vendors: BTreeMap<usize, VendorSighting>,
    pub(crate) data: BTreeMap<DataKind, Evidence>,
    pub(crate) cookies: Option<Evidence>,
}

impl Sightings {
    fn see_vendor(&mut self, index: usize, file: &str, line: u32) {
        self.vendors
            .entry(index)
            .and_modify(|seen| seen.count += 1)
            .or_insert_with(|| VendorSighting {
                evidence: Evidence::new(file, line),
                count: 1,
            });
    }
}

/// Dependency lists are read for service packages instead of being scanned line by line.
pub(crate) fn is_manifest_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "package.json" || lower == "requirements.txt"
}

/// Whether the file is code or a dependency list, so worth reading for evidence.
pub(crate) fn is_evidence_path(path: &str) -> bool {
    let path = Path::new(path);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if is_manifest_name(name) {
        return true;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            CODE_EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(ext))
        })
}

/// Reads every file and notes what it finds.
pub(crate) fn collect(files: &[&SourceFile]) -> Sightings {
    let domains = DomainIndex::build();
    let mut sightings = Sightings::default();
    for file in files {
        let name = Path::new(&file.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if is_manifest_name(name) {
            scan_manifest(file, name, &mut sightings);
        } else {
            scan_code(file, &domains, &mut sightings);
        }
    }
    sightings
}

static COOKIE_WRITE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bdocument\.cookie\s*=[^=]|\bset-cookie\b|\bres\.cookie\s*\(|\bresponse\.set_cookie\s*\(|\bset_cookie\s*\(|\bcookies?\.set\s*\(|\bjs-cookie\b|\breact-cookie\b|\bcookie-parser\b|\bexpress-session\b|\bcookie-session\b",
    )
    .expect("cookie pattern is valid")
});

const DATA_SOURCES: &[(DataKind, &str)] = &[
    (
        DataKind::Email,
        r#"(?i)<input[^>]*\b(?:type|name|id|autocomplete)\s*=\s*["']?email\b"#,
    ),
    (
        DataKind::Phone,
        r#"(?i)<input[^>]*\b(?:type|name|id|autocomplete)\s*=\s*["']?(?:tel|phone)\b"#,
    ),
    (
        DataKind::Address,
        r#"(?i)<input[^>]*\b(?:name|id|autocomplete)\s*=\s*["']?(?:street-address|address-line1|postal-code|zip|zipcode)\b"#,
    ),
    (
        DataKind::BirthDate,
        r#"(?i)<input[^>]*\b(?:name|id|autocomplete)\s*=\s*["']?(?:dob|birthday|birthdate|bday|date[-_]?of[-_]?birth)\b"#,
    ),
    (
        DataKind::Location,
        r"\bnavigator\.geolocation\b|\bgetCurrentPosition\s*\(|\bwatchPosition\s*\(",
    ),
    (DataKind::CameraMic, r"\bgetUserMedia\s*\("),
];

fn data_patterns() -> &'static [(DataKind, Regex)] {
    static PATTERNS: LazyLock<Vec<(DataKind, Regex)>> = LazyLock::new(|| {
        DATA_SOURCES
            .iter()
            .map(|(kind, source)| {
                (
                    *kind,
                    Regex::new(source).expect("built-in data pattern is valid"),
                )
            })
            .collect()
    });
    &PATTERNS
}

fn scan_code(file: &SourceFile, domains: &DomainIndex, sightings: &mut Sightings) {
    for (index, line) in file.text.lines().enumerate() {
        for owner in domains.owners_in(line) {
            sightings.see_vendor(owner, &file.path, line_number(index));
        }
    }
    if sightings.cookies.is_none() {
        sightings.cookies = COOKIE_WRITE
            .find(&file.text)
            .map(|found| Evidence::new(&file.path, line_at(&file.text, found.start())));
    }
    for (kind, pattern) in data_patterns() {
        if let Some(found) = pattern.find(&file.text) {
            let evidence = Evidence::new(&file.path, line_at(&file.text, found.start()));
            sightings.data.entry(*kind).or_insert(evidence);
        }
    }
}

fn scan_manifest(file: &SourceFile, name: &str, sightings: &mut Sightings) {
    let packages = if name.eq_ignore_ascii_case("package.json") {
        npm_packages(&file.text)
    } else {
        pip_packages(&file.text)
    };
    for (package, line) in packages {
        if let Some(index) = vendor_index_for_package(&package) {
            sightings.see_vendor(index, &file.path, line);
        }
    }
}

/// Runtime dependencies listed in a `package.json`, with the line each one is on.
fn npm_packages(text: &str) -> Vec<(String, u32)> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for section in ["dependencies", "optionalDependencies"] {
        let Some(table) = value.get(section).and_then(serde_json::Value::as_object) else {
            continue;
        };
        for name in table.keys() {
            found.push((name.clone(), line_of_quoted(text, name)));
        }
    }
    found
}

fn line_of_quoted(text: &str, name: &str) -> u32 {
    let needle = format!("\"{name}\"");
    text.lines()
        .position(|line| line.contains(&needle))
        .map_or(1, line_number)
}

/// Package names listed in a `requirements.txt`, with their line numbers.
fn pip_packages(text: &str) -> Vec<(String, u32)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| Some((requirement_name(line)?, line_number(index))))
        .collect()
}

fn requirement_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(['#', '-']) {
        return None;
    }
    let end = trimmed
        .find(['=', '<', '>', '!', '~', '[', ';', ' ', '@'])
        .unwrap_or(trimmed.len());
    let name = trimmed[..end].to_ascii_lowercase().replace('_', "-");
    (!name.is_empty()).then_some(name)
}

/// One based line number for a zero based line index.
fn line_number(index: usize) -> u32 {
    u32::try_from(index + 1).unwrap_or(u32::MAX)
}

/// One based line number of a byte offset.
fn line_at(text: &str, offset: usize) -> u32 {
    let newlines = text
        .get(..offset)
        .map_or(0, |head| head.matches('\n').count());
    line_number(newlines)
}
