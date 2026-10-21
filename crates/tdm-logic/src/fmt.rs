//! Italian number formatting for the dashboard (decimal comma, space as
//! thousands separator).

pub fn int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

pub fn dec(x: f64, decimals: usize) -> String {
    let neg = x < 0.0;
    let s = format!("{:.*}", decimals, x.abs());
    let (i, f) = s.split_once('.').unwrap_or((&s, ""));
    let mut out = String::new();
    if neg {
        out.push('−');
    }
    out.push_str(&int(i.parse().unwrap_or(0)));
    if decimals > 0 {
        out.push(',');
        out.push_str(f);
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

    #[test]
    fn italian_numbers() {
        assert_eq!(int(48213), "48 213");
        assert_eq!(int(999), "999");
        assert_eq!(int(1_000_000), "1 000 000");
        assert_eq!(dec(212.44, 1), "212,4");
        assert_eq!(dec(1834.96, 1), "1 835,0");
        assert_eq!(dec(-0.5, 1), "−0,5");
        assert_eq!(duration(65.0), "1:05");
        assert_eq!(duration(3725.0), "1:02");
    }
}
