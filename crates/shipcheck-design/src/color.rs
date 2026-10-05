//! Minimal CSS color parsing and WCAG contrast ratios.

/// A color in linear light, each channel from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Linear {
    red: f64,
    green: f64,
    blue: f64,
}

impl Linear {
    fn from_srgb(red: f64, green: f64, blue: f64) -> Self {
        Self {
            red: decode(red),
            green: decode(green),
            blue: decode(blue),
        }
    }

    fn luminance(self) -> f64 {
        0.2126 * self.red + 0.7152 * self.green + 0.0722 * self.blue
    }
}

fn decode(value: f64) -> f64 {
    let clamped = value.clamp(0.0, 1.0);
    if clamped <= 0.04045 {
        clamped / 12.92
    } else {
        ((clamped + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG contrast ratio between two colors, from 1 to 21.
pub(crate) fn contrast(first: Linear, second: Linear) -> f64 {
    let lum_first = first.luminance();
    let lum_second = second.luminance();
    let (light, dark) = if lum_first >= lum_second {
        (lum_first, lum_second)
    } else {
        (lum_second, lum_first)
    };
    (light + 0.05) / (dark + 0.05)
}

/// Parses hex, rgb, hsl, oklch, or a bare shadcn style `H S% L%` triple.
pub(crate) fn parse(value: &str) -> Option<Linear> {
    let text = value
        .trim()
        .trim_end_matches("!important")
        .trim()
        .to_ascii_lowercase();
    if let Some(hex) = text.strip_prefix('#') {
        return from_hex(hex);
    }
    if let Some(args) = function_args(&text, "rgba").or_else(|| function_args(&text, "rgb")) {
        return from_rgb(&args);
    }
    if let Some(args) = function_args(&text, "hsla").or_else(|| function_args(&text, "hsl")) {
        return from_hsl(&args);
    }
    if let Some(args) = function_args(&text, "oklch") {
        return from_oklch(&args);
    }
    let bare = split_args(&text);
    let percent_pair = bare.len() == 3 && bare[1].ends_with('%') && bare[2].ends_with('%');
    if percent_pair {
        from_hsl(&bare)
    } else {
        None
    }
}

fn function_args(text: &str, name: &str) -> Option<Vec<String>> {
    let inner = text.strip_prefix(name)?.trim_start().strip_prefix('(')?;
    let body = inner.split(')').next()?;
    Some(split_args(body))
}

fn split_args(text: &str) -> Vec<String> {
    text.split(|ch: char| ch == ',' || ch == '/' || ch.is_whitespace())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn from_hex(hex: &str) -> Option<Linear> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|ch| ch.to_digit(16).and_then(|digit| u8::try_from(digit).ok()))
        .collect::<Option<Vec<u8>>>()?;
    let (red, green, blue) = match digits.len() {
        3 | 4 => (digits[0] * 17, digits[1] * 17, digits[2] * 17),
        6 | 8 => (
            digits[0] * 16 + digits[1],
            digits[2] * 16 + digits[3],
            digits[4] * 16 + digits[5],
        ),
        _ => return None,
    };
    Some(Linear::from_srgb(
        f64::from(red) / 255.0,
        f64::from(green) / 255.0,
        f64::from(blue) / 255.0,
    ))
}

fn channel(token: &str) -> Option<f64> {
    if let Some(percent) = token.strip_suffix('%') {
        Some(percent.parse::<f64>().ok()? / 100.0)
    } else {
        Some(token.parse::<f64>().ok()? / 255.0)
    }
}

fn from_rgb(args: &[String]) -> Option<Linear> {
    let [red, green, blue, ..] = args else {
        return None;
    };
    Some(Linear::from_srgb(
        channel(red)?,
        channel(green)?,
        channel(blue)?,
    ))
}

fn share(token: &str) -> Option<f64> {
    token
        .trim_end_matches('%')
        .parse::<f64>()
        .ok()
        .map(|value| value / 100.0)
}

fn from_hsl(args: &[String]) -> Option<Linear> {
    let [hue_token, sat_token, light_token, ..] = args else {
        return None;
    };
    let hue = hue_token.trim_end_matches("deg").parse::<f64>().ok()?;
    let sat = share(sat_token)?;
    let light = share(light_token)?;
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * sat;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (red, green, blue) = if sector < 1.0 {
        (chroma, second, 0.0)
    } else if sector < 2.0 {
        (second, chroma, 0.0)
    } else if sector < 3.0 {
        (0.0, chroma, second)
    } else if sector < 4.0 {
        (0.0, second, chroma)
    } else if sector < 5.0 {
        (second, 0.0, chroma)
    } else {
        (chroma, 0.0, second)
    };
    let offset = light - chroma / 2.0;
    Some(Linear::from_srgb(
        red + offset,
        green + offset,
        blue + offset,
    ))
}

fn from_oklch(args: &[String]) -> Option<Linear> {
    let [light_token, chroma_token, hue_token, ..] = args else {
        return None;
    };
    let light = if let Some(percent) = light_token.strip_suffix('%') {
        percent.parse::<f64>().ok()? / 100.0
    } else {
        light_token.parse::<f64>().ok()?
    };
    let chroma = chroma_token.parse::<f64>().ok()?;
    let hue = hue_token.trim_end_matches("deg").parse::<f64>().ok()?;
    Some(oklch_to_linear(light, chroma, hue))
}

fn oklch_to_linear(light: f64, chroma: f64, hue_degrees: f64) -> Linear {
    let radians = hue_degrees.to_radians();
    let axis_a = chroma * radians.cos();
    let axis_b = chroma * radians.sin();
    let long = (light + 0.3964 * axis_a + 0.2158 * axis_b).powi(3);
    let medium = (light - 0.1056 * axis_a - 0.0639 * axis_b).powi(3);
    let short = (light - 0.0895 * axis_a - 1.2915 * axis_b).powi(3);
    Linear {
        red: (4.0767 * long - 3.3077 * medium + 0.2310 * short).clamp(0.0, 1.0),
        green: (-1.2684 * long + 2.6098 * medium - 0.3413 * short).clamp(0.0, 1.0),
        blue: (-0.0042 * long - 0.7034 * medium + 1.7076 * short).clamp(0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(first: &str, second: &str) -> f64 {
        contrast(parse(first).unwrap(), parse(second).unwrap())
    }

    #[test]
    fn black_on_white_is_21() {
        assert!((ratio("#000", "#ffffff") - 21.0).abs() < 0.01);
    }

    #[test]
    fn shadcn_triples_parse() {
        assert!((ratio("0 0% 0%", "0 0% 100%") - 21.0).abs() < 0.01);
    }

    #[test]
    fn oklch_extremes_parse() {
        assert!(ratio("oklch(0 0 0)", "oklch(1 0 0)") > 20.9);
    }

    #[test]
    fn rgb_with_alpha_parses() {
        assert!((ratio("rgb(0 0 0 / 50%)", "rgb(255, 255, 255)") - 21.0).abs() < 0.01);
    }

    #[test]
    fn mid_gray_on_near_black_is_weak() {
        assert!(ratio("#5c5c5c", "#111111") < 4.5);
    }

    #[test]
    fn non_colors_are_rejected() {
        assert!(parse("red").is_none());
        assert!(parse("transparent").is_none());
        assert!(parse("var(--x)").is_none());
    }
}
