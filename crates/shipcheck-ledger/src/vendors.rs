//! Third party services: where they show up in code, and how a policy would name them.

use std::collections::HashMap;

use regex::Regex;
use serde::Serialize;
use shipcheck_core::Severity;

use crate::policy::Policy;

/// What a third party service does with the data it receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorKind {
    Analytics,
    Advertising,
    Monitoring,
    Ai,
    Payments,
    Email,
    Support,
    Maps,
    Fonts,
    Captcha,
    Backend,
    Auth,
}

impl VendorKind {
    /// A short label for messages and tables.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Analytics => "analytics",
            Self::Advertising => "advertising",
            Self::Monitoring => "monitoring",
            Self::Ai => "AI",
            Self::Payments => "payments",
            Self::Email => "email",
            Self::Support => "support",
            Self::Maps => "maps",
            Self::Fonts => "fonts",
            Self::Captcha => "captcha",
            Self::Backend => "backend",
            Self::Auth => "login",
        }
    }

    /// How serious it is when the policy leaves this kind of service out.
    pub(crate) const fn severity(self) -> Severity {
        match self {
            Self::Advertising => Severity::High,
            Self::Analytics | Self::Monitoring | Self::Ai => Severity::Medium,
            Self::Payments
            | Self::Email
            | Self::Support
            | Self::Maps
            | Self::Fonts
            | Self::Captcha
            | Self::Backend
            | Self::Auth => Severity::Low,
        }
    }

    /// Whether the service receives data about visitors or what they write.
    pub(crate) const fn shares_user_data(self) -> bool {
        matches!(
            self,
            Self::Analytics
                | Self::Advertising
                | Self::Monitoring
                | Self::Ai
                | Self::Payments
                | Self::Email
                | Self::Support
        )
    }

    /// Whether the service observes what visitors do.
    pub(crate) const fn is_tracking(self) -> bool {
        matches!(self, Self::Analytics | Self::Advertising | Self::Monitoring)
    }
}

/// One known service.
pub(crate) struct Vendor {
    pub(crate) name: &'static str,
    pub(crate) kind: VendorKind,
    /// Host names that appear in code, matched on word boundaries.
    pub(crate) domains: &'static [&'static str],
    /// Package names. A trailing slash means every package in that scope.
    pub(crate) packages: &'static [&'static str],
    /// How a privacy policy might name the service, in lowercase.
    pub(crate) aliases: &'static [&'static str],
}

impl Vendor {
    /// Whether the policy names this service in any of the usual ways.
    pub(crate) fn is_named_in(&self, policy: &Policy) -> bool {
        policy.mentions(&self.name.to_ascii_lowercase())
            || self.aliases.iter().any(|alias| policy.mentions(alias))
    }
}

const fn vendor(
    name: &'static str,
    kind: VendorKind,
    domains: &'static [&'static str],
    packages: &'static [&'static str],
    aliases: &'static [&'static str],
) -> Vendor {
    Vendor {
        name,
        kind,
        domains,
        packages,
        aliases,
    }
}

pub(crate) static VENDORS: &[Vendor] = &[
    vendor(
        "Google Analytics",
        VendorKind::Analytics,
        &[
            "google-analytics.com",
            "googletagmanager.com",
            "analytics.google.com",
        ],
        &[
            "react-ga",
            "react-ga4",
            "ga-gtag",
            "@analytics/google-analytics",
        ],
        &["google analytics", "google tag manager", "gtag"],
    ),
    vendor(
        "Google Ads",
        VendorKind::Advertising,
        &[
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
        ],
        &["react-adsense"],
        &["google ads", "doubleclick", "adsense", "google advertising"],
    ),
    vendor(
        "Meta Pixel",
        VendorKind::Advertising,
        &["connect.facebook.net", "facebook.com/tr"],
        &["react-facebook-pixel"],
        &["facebook", "meta pixel", "meta platforms"],
    ),
    vendor(
        "TikTok Pixel",
        VendorKind::Advertising,
        &["analytics.tiktok.com"],
        &["react-tiktok-pixel", "tiktok-pixel"],
        &["tiktok"],
    ),
    vendor(
        "X (Twitter) Ads",
        VendorKind::Advertising,
        &["ads-twitter.com"],
        &[],
        &["twitter", "x corp"],
    ),
    vendor(
        "LinkedIn Insight",
        VendorKind::Advertising,
        &["snap.licdn.com", "px.ads.linkedin.com"],
        &[],
        &["linkedin"],
    ),
    vendor(
        "Hotjar",
        VendorKind::Analytics,
        &["hotjar.com"],
        &["@hotjar/browser", "react-hotjar"],
        &["hotjar"],
    ),
    vendor(
        "Mixpanel",
        VendorKind::Analytics,
        &["mixpanel.com", "mxpnl.com"],
        &["mixpanel-browser", "mixpanel"],
        &["mixpanel"],
    ),
    vendor(
        "Segment",
        VendorKind::Analytics,
        &["segment.com", "segment.io"],
        &["@segment/", "analytics-node"],
        &["segment"],
    ),
    vendor(
        "PostHog",
        VendorKind::Analytics,
        &["posthog.com"],
        &["posthog-js", "posthog-node", "posthog"],
        &["posthog"],
    ),
    vendor(
        "Amplitude",
        VendorKind::Analytics,
        &["amplitude.com"],
        &["@amplitude/", "amplitude-js"],
        &["amplitude"],
    ),
    vendor(
        "Plausible",
        VendorKind::Analytics,
        &["plausible.io"],
        &["plausible-tracker", "next-plausible"],
        &["plausible"],
    ),
    vendor(
        "Vercel Analytics",
        VendorKind::Analytics,
        &["vitals.vercel-insights.com", "va.vercel-scripts.com"],
        &["@vercel/analytics", "@vercel/speed-insights"],
        &["vercel"],
    ),
    vendor(
        "Microsoft Clarity",
        VendorKind::Analytics,
        &["clarity.ms"],
        &["@microsoft/clarity"],
        &["microsoft clarity", "clarity"],
    ),
    vendor(
        "Sentry",
        VendorKind::Monitoring,
        &["sentry.io", "sentry-cdn.com"],
        &["@sentry/", "sentry-sdk", "raven"],
        &["sentry"],
    ),
    vendor(
        "Datadog",
        VendorKind::Monitoring,
        &["datadoghq.com", "datadoghq.eu"],
        &["@datadog/", "ddtrace"],
        &["datadog"],
    ),
    vendor(
        "LogRocket",
        VendorKind::Monitoring,
        &["logrocket.com", "lr-ingest.io"],
        &["logrocket"],
        &["logrocket"],
    ),
    vendor(
        "FullStory",
        VendorKind::Monitoring,
        &["fullstory.com"],
        &["@fullstory/browser"],
        &["fullstory"],
    ),
    vendor(
        "OpenAI",
        VendorKind::Ai,
        &["api.openai.com"],
        &["openai"],
        &["openai", "chatgpt"],
    ),
    vendor(
        "Anthropic",
        VendorKind::Ai,
        &["api.anthropic.com"],
        &["@anthropic-ai/sdk", "anthropic"],
        &["anthropic", "claude"],
    ),
    vendor(
        "Google Gemini",
        VendorKind::Ai,
        &["generativelanguage.googleapis.com"],
        &["@google/generative-ai", "google-generativeai"],
        &["gemini", "google ai"],
    ),
    vendor(
        "Stripe",
        VendorKind::Payments,
        &["stripe.com"],
        &["stripe", "@stripe/"],
        &["stripe"],
    ),
    vendor(
        "PayPal",
        VendorKind::Payments,
        &["paypal.com", "paypalobjects.com"],
        &["@paypal/"],
        &["paypal"],
    ),
    vendor(
        "Paddle",
        VendorKind::Payments,
        &["paddle.com"],
        &["@paddle/"],
        &["paddle"],
    ),
    vendor(
        "Lemon Squeezy",
        VendorKind::Payments,
        &["lemonsqueezy.com"],
        &["@lemonsqueezy/lemonsqueezy.js"],
        &["lemon squeezy", "lemonsqueezy"],
    ),
    vendor(
        "SendGrid",
        VendorKind::Email,
        &["sendgrid.com", "sendgrid.net"],
        &["@sendgrid/", "sendgrid"],
        &["sendgrid", "twilio"],
    ),
    vendor(
        "Mailchimp",
        VendorKind::Email,
        &["mailchimp.com", "list-manage.com"],
        &["@mailchimp/", "mailchimp-api-v3"],
        &["mailchimp"],
    ),
    vendor(
        "Mailgun",
        VendorKind::Email,
        &["mailgun.net", "mailgun.com"],
        &["mailgun.js", "mailgun-js"],
        &["mailgun"],
    ),
    vendor(
        "Postmark",
        VendorKind::Email,
        &["postmarkapp.com"],
        &["postmark"],
        &["postmark"],
    ),
    vendor(
        "Resend",
        VendorKind::Email,
        &["resend.com"],
        &["resend"],
        &["resend"],
    ),
    vendor(
        "Intercom",
        VendorKind::Support,
        &["intercom.io", "intercomcdn.com"],
        &["@intercom/", "react-use-intercom"],
        &["intercom"],
    ),
    vendor(
        "Zendesk",
        VendorKind::Support,
        &["zendesk.com", "zdassets.com"],
        &["@zendesk/"],
        &["zendesk"],
    ),
    vendor(
        "Crisp",
        VendorKind::Support,
        &["crisp.chat"],
        &["crisp-sdk-web"],
        &["crisp"],
    ),
    vendor(
        "Google Maps",
        VendorKind::Maps,
        &["maps.googleapis.com", "maps.google.com"],
        &["@googlemaps/", "@react-google-maps/api"],
        &["google maps"],
    ),
    vendor(
        "Google Fonts",
        VendorKind::Fonts,
        &["fonts.googleapis.com", "fonts.gstatic.com"],
        &[],
        &["google fonts"],
    ),
    vendor(
        "reCAPTCHA",
        VendorKind::Captcha,
        &["google.com/recaptcha", "recaptcha.net"],
        &["react-google-recaptcha", "react-google-recaptcha-v3"],
        &["recaptcha", "google recaptcha"],
    ),
    vendor(
        "hCaptcha",
        VendorKind::Captcha,
        &["hcaptcha.com"],
        &["@hcaptcha/", "react-hcaptcha"],
        &["hcaptcha"],
    ),
    vendor(
        "Firebase",
        VendorKind::Backend,
        &["firebaseio.com", "firebaseapp.com"],
        &["firebase", "@firebase/", "firebase-admin"],
        &["firebase"],
    ),
    vendor(
        "Supabase",
        VendorKind::Backend,
        &["supabase.co", "supabase.com"],
        &["@supabase/"],
        &["supabase"],
    ),
    vendor(
        "Auth0",
        VendorKind::Auth,
        &["auth0.com"],
        &["@auth0/", "auth0"],
        &["auth0"],
    ),
    vendor(
        "Clerk",
        VendorKind::Auth,
        &["clerk.com", "clerk.dev", "clerk.accounts.dev"],
        &["@clerk/"],
        &["clerk"],
    ),
];

/// Finds the service that owns a package name from npm or pip.
pub(crate) fn vendor_index_for_package(package: &str) -> Option<usize> {
    let name = package.to_ascii_lowercase();
    VENDORS.iter().position(|vendor| {
        vendor
            .packages
            .iter()
            .any(|pattern| package_matches(pattern, &name))
    })
}

fn package_matches(pattern: &str, name: &str) -> bool {
    if pattern.ends_with('/') {
        name.starts_with(pattern)
    } else {
        name == pattern
    }
}

/// Finds known domains in a line of code with one combined pattern.
pub(crate) struct DomainIndex {
    pattern: Regex,
    owners: HashMap<String, usize>,
}

impl DomainIndex {
    pub(crate) fn build() -> Self {
        let mut owners = HashMap::new();
        let mut alternatives = Vec::new();
        for (index, vendor) in VENDORS.iter().enumerate() {
            for domain in vendor.domains {
                owners.insert((*domain).to_owned(), index);
                alternatives.push(regex::escape(domain));
            }
        }
        let source = format!(r"(?i)\b(?:{})\b", alternatives.join("|"));
        let pattern = Regex::new(&source).expect("vendor domains form a valid pattern");
        Self { pattern, owners }
    }

    /// Indexes into `VENDORS` for every known domain on the line.
    pub(crate) fn owners_in(&self, line: &str) -> Vec<usize> {
        self.pattern
            .find_iter(line)
            .filter_map(|found| {
                self.owners
                    .get(&found.as_str().to_ascii_lowercase())
                    .copied()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn table_is_consistent() {
        let mut domains = HashSet::new();
        let mut names = HashSet::new();
        for vendor in VENDORS {
            assert!(
                names.insert(vendor.name),
                "duplicate vendor {}",
                vendor.name
            );
            assert!(!vendor.aliases.is_empty(), "{} needs an alias", vendor.name);
            assert!(
                !vendor.domains.is_empty() || !vendor.packages.is_empty(),
                "{} has no way to be found",
                vendor.name
            );
            for alias in vendor.aliases {
                assert_eq!(*alias, alias.to_lowercase(), "aliases must be lowercase");
            }
            for domain in vendor.domains {
                assert_eq!(*domain, domain.to_lowercase(), "domains must be lowercase");
                assert!(domains.insert(*domain), "duplicate domain {domain}");
            }
        }
    }

    #[test]
    fn domains_match_whole_names_only() {
        let index = DomainIndex::build();
        assert_eq!(
            index
                .owners_in("https://static.hotjar.com/c/hotjar.js")
                .len(),
            1
        );
        assert_eq!(
            index.owners_in("https://mysentry.io/x"),
            Vec::<usize>::new()
        );
        let captcha = "<script src=\"https://www.google.com/recaptcha/api.js\">";
        assert_eq!(index.owners_in(captcha).len(), 1);
    }

    #[test]
    fn packages_match_exactly_or_by_scope() {
        assert!(vendor_index_for_package("stripe").is_some());
        assert!(vendor_index_for_package("@sentry/browser").is_some());
        assert!(vendor_index_for_package("@Sentry/Node").is_some());
        assert!(vendor_index_for_package("stripe-mock").is_none());
        assert!(vendor_index_for_package("left-pad").is_none());
    }
}
