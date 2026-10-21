//! Number formatting for the dashboard, in the profile's language:
//! English `48,213` and `212.4`, Italian `48 213` and `212,4`.

use crate::i18n::Language;

fn separators(lang: Language) -> (char, char) {
    match lang {
        Language::En => (',', '.'),
        Language::It => (' ', ','),
    }
}

pub fn int(lang: Language, n: u64) -> String {
    let (group, _) = separators(lang);
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(group);
        }
        out.push(c);
    }
    out
}

pub fn dec(lang: Language, x: f64, decimals: usize) -> String {
    let (_, point) = separators(lang);
    let neg = x < 0.0;
    let s = format!("{:.*}", decimals, x.abs());
    let (i, f) = s.split_once('.').unwrap_or((&s, ""));
    let mut out = String::new();
    if neg {
        out.push('−');
    }
    out.push_str(&int(lang, i.parse().unwrap_or(0)));
    if decimals > 0 {
        out.push(point);
        out.push_str(f);
    }
    out
}

/// Placeholder shown instead of a number with `decimals` decimals, `--.-`.
pub fn blank(lang: Language, int_digits: usize, decimals: usize) -> String {
    let (_, point) = separators(lang);
    let mut out = "-".repeat(int_digits);
    if decimals > 0 {
        out.push(point);
        out.push_str(&"-".repeat(decimals));
    }
    out
}

/// `m:ss` below one hour, `h:mm` above.
pub fn duration(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, s) = (s / 3600, (s / 60) % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Language::{En, It};

    #[test]
    fn english_numbers() {
        assert_eq!(int(En, 48213), "48,213");
        assert_eq!(int(En, 999), "999");
        assert_eq!(int(En, 1_000_000), "1,000,000");
        assert_eq!(dec(En, 212.44, 1), "212.4");
        assert_eq!(dec(En, 1834.96, 1), "1,835.0");
        assert_eq!(dec(En, -0.5, 1), "−0.5");
        assert_eq!(blank(En, 3, 1), "---.-");
    }

    #[test]
    fn italian_numbers() {
        assert_eq!(int(It, 48213), "48 213");
        assert_eq!(int(It, 1_000_000), "1 000 000");
        assert_eq!(dec(It, 212.44, 1), "212,4");
        assert_eq!(dec(It, 1834.96, 1), "1 835,0");
        assert_eq!(dec(It, -0.5, 1), "−0,5");
        assert_eq!(blank(It, 2, 1), "--,-");
    }

    #[test]
    fn durations() {
        assert_eq!(duration(65.0), "1:05");
        assert_eq!(duration(3725.0), "1:02");
    }
}
