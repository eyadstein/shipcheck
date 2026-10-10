//! The ledger itself: what the code does, and whether the policy admits it.

use serde::Serialize;
use shipcheck_core::Severity;

use crate::evidence::Sightings;
use crate::policy::Policy;
use crate::vendors::{VendorKind, VENDORS};

/// A place in the code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Evidence {
    pub file: String,
    pub line: u32,
}

impl Evidence {
    pub(crate) fn new(file: &str, line: u32) -> Self {
        Self {
            file: file.to_owned(),
            line,
        }
    }
}

/// A kind of personal data the code collects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataKind {
    Email,
    Phone,
    Address,
    BirthDate,
    Location,
    CameraMic,
}

impl DataKind {
    /// A short label for messages and tables.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Email => "email addresses",
            Self::Phone => "phone numbers",
            Self::Address => "postal addresses",
            Self::BirthDate => "dates of birth",
            Self::Location => "location",
            Self::CameraMic => "camera or microphone data",
        }
    }

    /// How serious it is when the policy leaves this kind of data out.
    pub(crate) const fn severity(self) -> Severity {
        match self {
            Self::Location | Self::CameraMic => Severity::High,
            Self::Phone | Self::Address | Self::BirthDate => Severity::Medium,
            Self::Email => Severity::Low,
        }
    }

    /// Words a policy would use when it covers this kind of data.
    pub(crate) const fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Email => &["email", "e-mail"],
            Self::Phone => &["phone", "telephone", "mobile number", "sms", "text message"],
            Self::Address => &[
                "postal address",
                "mailing address",
                "home address",
                "shipping address",
                "billing address",
                "street address",
                "physical address",
                "delivery address",
                "your address",
                "zip code",
                "postcode",
            ],
            Self::BirthDate => &[
                "date of birth",
                "birth date",
                "birthday",
                "birthdate",
                "your age",
                "age range",
            ],
            Self::Location => &["location", "geolocation", "gps"],
            Self::CameraMic => &["camera", "microphone", "audio", "video", "voice", "webcam"],
        }
    }
}

/// A third party service the code talks to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VendorUse {
    pub vendor: String,
    pub kind: VendorKind,
    /// The first place in the code that points at the service.
    pub evidence: Evidence,
    /// How many places in the code point at the service.
    pub sightings: usize,
    /// Whether the policy names it. `None` when there is no policy to compare with.
    pub disclosed: Option<bool>,
}

/// A kind of personal data the code collects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DataUse {
    pub kind: DataKind,
    pub evidence: Evidence,
    pub disclosed: Option<bool>,
}

/// The code sets cookies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CookieUse {
    pub evidence: Evidence,
    pub disclosed: Option<bool>,
}

/// Everything the code does that a privacy policy should cover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ledger {
    /// The files treated as the privacy policy.
    pub policy_files: Vec<String>,
    pub vendors: Vec<VendorUse>,
    pub data: Vec<DataUse>,
    pub cookies: Option<CookieUse>,
}

impl Ledger {
    pub(crate) fn from_sightings(sightings: Sightings, policies: &[Policy]) -> Self {
        let vendors = sightings
            .vendors
            .into_iter()
            .map(|(index, seen)| {
                let vendor = &VENDORS[index];
                VendorUse {
                    vendor: vendor.name.to_owned(),
                    kind: vendor.kind,
                    evidence: seen.evidence,
                    sightings: seen.count,
                    disclosed: disclosure(policies, |policy| vendor.is_named_in(policy)),
                }
            })
            .collect();
        let data = sightings
            .data
            .into_iter()
            .map(|(kind, evidence)| DataUse {
                kind,
                evidence,
                disclosed: disclosure(policies, |policy| policy.mentions_any(kind.keywords())),
            })
            .collect();
        let cookies = sightings.cookies.map(|evidence| CookieUse {
            evidence,
            disclosed: disclosure(policies, |policy| policy.mentions("cookie")),
        });
        Self {
            policy_files: policies.iter().map(|policy| policy.path.clone()).collect(),
            vendors,
            data,
            cookies,
        }
    }
}

/// `None` without any policy, otherwise whether some policy covers the item.
fn disclosure(policies: &[Policy], covered: impl Fn(&Policy) -> bool) -> Option<bool> {
    if policies.is_empty() {
        None
    } else {
        Some(policies.iter().any(covered))
    }
}
