//! Metadata for every design rule the analyzer can report.

use shipcheck_core::{Category, Finding, Severity};

/// One design rule: what it says and how to fix it.
pub(crate) struct Tell {
    pub id: &'static str,
    pub severity: Severity,
    pub message: &'static str,
    pub fix: &'static str,
}

impl Tell {
    /// Builds a finding for this rule at a location.
    pub(crate) fn finding(&self, file: &str, line: u32) -> Finding {
        Finding {
            rule_id: self.id.to_owned(),
            category: Category::Design,
            severity: self.severity,
            message: self.message.to_owned(),
            file: file.to_owned(),
            line,
            fix: Some(self.fix.to_owned()),
        }
    }
}

const fn tell(
    id: &'static str,
    severity: Severity,
    message: &'static str,
    fix: &'static str,
) -> Tell {
    Tell {
        id,
        severity,
        message,
        fix,
    }
}

pub(crate) const GRADIENT_TEXT: Tell = tell(
    "DES-201",
    Severity::Low,
    "Gradient text on a heading is a common vibecoded tell.",
    "Use a solid text color with strong contrast and keep gradients for non-text decoration.",
);

pub(crate) const GLASS_CARD: Tell = tell(
    "DES-202",
    Severity::Low,
    "Glassmorphism card (blur plus translucent background).",
    "Use solid surfaces with a clear border or shadow, and keep blur for overlays that need it.",
);

pub(crate) const ACCENT_BORDER: Tell = tell(
    "DES-203",
    Severity::Low,
    "Card with a thick colored accent border.",
    "Use a neutral border and show hierarchy with spacing and type instead.",
);

pub(crate) const INTER_ONLY: Tell = tell(
    "DES-204",
    Severity::Low,
    "Inter is the only typeface used across the project.",
    "Add a distinctive heading typeface, or choose a different primary typeface.",
);

pub(crate) const LOW_CONTRAST_DARK: Tell = tell(
    "DES-205",
    Severity::Medium,
    "Dark mode text has low contrast against its background (WCAG 1.4.3).",
    "Raise the lightness of the foreground colors until contrast reaches at least 4.5:1.",
);

pub(crate) const ICON_BOXES: Tell = tell(
    "DES-206",
    Severity::Low,
    "Three feature boxes with icons in a row.",
    "Vary the layout of feature sections, or show the product instead of icon cards.",
);

pub(crate) const BADGE_ABOVE_HEADLINE: Tell = tell(
    "DES-207",
    Severity::Low,
    "Small pill badge above the main headline.",
    "Remove the badge, or let the headline carry the message on its own.",
);

pub(crate) const LUCIDE_EVERYWHERE: Tell = tell(
    "DES-208",
    Severity::Low,
    "Lucide icons are used in many files.",
    "Use icons only where they add meaning, and consider a custom or different icon set.",
);

pub(crate) const SHADCN_DEFAULT: Tell = tell(
    "DES-209",
    Severity::Low,
    "shadcn/ui theme variables are still the defaults.",
    "Customize the color, radius and typography tokens so the app has its own look.",
);

pub(crate) const FADE_ON_SCROLL: Tell = tell(
    "DES-210",
    Severity::Low,
    "Fade-in on scroll animation pattern.",
    "Drop the reveal animations, or use motion only where it explains something.",
);

pub(crate) const CURSOR_BEAM: Tell = tell(
    "DES-211",
    Severity::Low,
    "Cursor-following glow or beam effect.",
    "Remove the pointer tracking effect, or limit it to one focused element.",
);

pub(crate) const HOVER_FADE: Tell = tell(
    "DES-212",
    Severity::Low,
    "Button only changes opacity on hover.",
    "Give buttons a real hover state such as a color, shadow or position change.",
);

pub(crate) const SPACING: Tell = tell(
    "DES-213",
    Severity::Low,
    "Spacing values do not follow a consistent scale.",
    "Pick a spacing scale, for example multiples of 4px, and use only those values.",
);

pub(crate) const EM_DASHES: Tell = tell(
    "DES-214",
    Severity::Low,
    "Em dashes appear many times in page copy.",
    "Rewrite with commas, colons or shorter sentences.",
);

pub(crate) const BUZZWORDS: Tell = tell(
    "DES-215",
    Severity::Low,
    "Generic buzzword copy found.",
    "Say concretely what the product does and who it is for.",
);

pub(crate) const SERIF_ITALIC: Tell = tell(
    "DES-216",
    Severity::Low,
    "Serif italic accent words inside headings or copy.",
    "Use one typographic voice per heading unless the contrast has a clear purpose.",
);

pub(crate) const FONT_PAIR: Tell = tell(
    "DES-217",
    Severity::Low,
    "Space Grotesk paired with Instrument Serif.",
    "Choose a pairing that fits your brand instead of a popular default.",
);

pub(crate) const GRAIN_GRADIENT: Tell = tell(
    "DES-218",
    Severity::Low,
    "Grain or noise overlay on top of a gradient.",
    "Use flat color or a clean image and drop the texture overlay.",
);
