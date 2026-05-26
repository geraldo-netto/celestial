//! Output formatting utilities — tables, degree strings, JSON.

/// Escape a string for safe inclusion in XML/SVG text **or**
/// double-quoted attribute values (SEC-1/2): `&`, `<`, `>`, `"`, `'`.
/// User-controlled values (`--var`, `--name`, TOML `[vars]`) flow into
/// generated SVG; without this they can break out of a `<text>`
/// element or an attribute and inject markup. Strings with none of
/// these characters (e.g. the default titles, hex colours) are
/// returned unchanged, so existing output is byte-identical.
#[must_use]
pub fn xml_escape(s: &str) -> String {
    if !s.contains(['&', '<', '>', '"', '\'']) {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

// ─── Degree/longitude display ─────────────────────────────────────────────────

const SIGNS: [&str; 12] = [
    "Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis",
];

/// Format decimal degrees as a zodiac position: `DDD°MM' Ari`.
pub(crate) fn lon_zodiac(lon: f64) -> String {
    let lon = lon.rem_euclid(360.0);
    let idx = (lon / 30.0).floor() as usize;
    let deg = lon % 30.0;
    let d = deg as u32;
    let min = ((deg - d as f64) * 60.0).round() as u32;
    format!("{d:2}\u{00b0}{min:02}\' {}", SIGNS[idx])
}

/// Format decimal degrees as `±DDD°MM'SS"`.
pub(crate) fn deg_dms(deg: f64) -> String {
    let sign = if deg >= 0.0 { "" } else { "-" };
    let abs = deg.abs();
    let d = abs as u32;
    let m = ((abs - d as f64) * 60.0) as u32;
    let s = ((abs - d as f64) * 3600.0 - (m as f64 * 60.0)).round() as u32;
    format!("{sign}{d}\u{00b0}{m:02}\'{s:02}\"")
}

/// Format AU distance.
pub(crate) fn dist_au(au: f64) -> String {
    format!("{au:.6} AU")
}

/// Format degrees-per-day speed with sign.
pub(crate) fn speed_dday(s: f64) -> String {
    let sign = if s >= 0.0 { "+" } else { "" };
    format!("{sign}{s:.4}\u{00b0}/d")
}

// ─── Simple ASCII table ───────────────────────────────────────────────────────

/// Draw a horizontal rule of a given width.
pub(crate) fn rule(width: usize) -> String {
    "\u{2500}".repeat(width)
}

#[allow(dead_code)]
/// Left-pad a string to `width` characters.
pub(crate) fn lpad(s: &str, width: usize) -> String {
    format!("{s:>width$}")
}

#[allow(dead_code)]
/// Right-pad a string to `width` characters.
pub(crate) fn rpad(s: &str, width: usize) -> String {
    format!("{s:<width$}")
}

// ─── JSON helpers ─────────────────────────────────────────────────────────────

/// Produce a simple JSON object from key-value pairs where values are pre-formatted strings.
pub(crate) fn json_obj(pairs: &[(&str, String)]) -> String {
    let inner: Vec<String> = pairs
        .iter()
        .map(|(k, v)| {
            if v.parse::<f64>().is_ok() {
                format!("  \"{k}\": {v}")
            } else {
                // String-typed value: quote-escape and wrap in quotes once.
                format!("  \"{k}\": \"{}\"", v.replace('"', "\\\""))
            }
        })
        .collect();
    format!("{{\n{}\n}}", inner.join(",\n"))
}

/// Wrap a list of JSON objects in a JSON array.
pub(crate) fn json_array(items: Vec<String>) -> String {
    format!("[\n{}\n]", items.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_noop_fast_path() {
        assert_eq!(xml_escape(""), "");
        assert_eq!(xml_escape("plain text 123"), "plain text 123");
        assert_eq!(xml_escape("#ff00aa"), "#ff00aa");
    }

    #[test]
    fn xml_escape_all_special_chars() {
        assert_eq!(xml_escape("&"), "&amp;");
        assert_eq!(xml_escape("<"), "&lt;");
        assert_eq!(xml_escape(">"), "&gt;");
        assert_eq!(xml_escape("\""), "&quot;");
        assert_eq!(xml_escape("'"), "&apos;");
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
        assert_eq!(xml_escape("<a href=\"x&y\">'z'</a>"),
            "&lt;a href=&quot;x&amp;y&quot;&gt;&apos;z&apos;&lt;/a&gt;");
    }

    #[test]
    fn lon_zodiac_basic() {
        assert_eq!(lon_zodiac(0.0), " 0\u{00b0}00\' Ari");
        assert_eq!(lon_zodiac(30.0), " 0\u{00b0}00\' Tau");
        assert_eq!(lon_zodiac(359.5), "29\u{00b0}30\' Pis");
        assert_eq!(lon_zodiac(45.25), "15\u{00b0}15\' Tau");
    }

    #[test]
    fn lon_zodiac_negative_and_large_wrap() {
        // rem_euclid wraps negatives and >360 into [0,360)
        assert_eq!(lon_zodiac(-1.0), lon_zodiac(359.0));
        assert_eq!(lon_zodiac(360.0), lon_zodiac(0.0));
        assert_eq!(lon_zodiac(720.0 + 12.0), lon_zodiac(12.0));
        assert_eq!(lon_zodiac(-90.0), lon_zodiac(270.0));
    }

    #[test]
    fn deg_dms_sign_and_components() {
        assert_eq!(deg_dms(0.0), "0\u{00b0}00\'00\"");
        assert_eq!(deg_dms(1.5), "1\u{00b0}30\'00\"");
        assert_eq!(deg_dms(-1.5), "-1\u{00b0}30\'00\"");
        assert_eq!(deg_dms(23.508333), "23\u{00b0}30\'30\"");
        assert_eq!(deg_dms(-180.0), "-180\u{00b0}00\'00\"");
        assert_eq!(deg_dms(360.0), "360\u{00b0}00\'00\"");
    }

    #[test]
    fn dist_au_formats_six_decimals() {
        assert_eq!(dist_au(0.0), "0.000000 AU");
        assert_eq!(dist_au(1.0), "1.000000 AU");
        assert_eq!(dist_au(-2.5), "-2.500000 AU");
        assert_eq!(dist_au(1234.567891234), "1234.567891 AU");
    }

    #[test]
    fn speed_dday_sign_handling() {
        assert_eq!(speed_dday(0.0), "+0.0000\u{00b0}/d");
        assert_eq!(speed_dday(1.2345), "+1.2345\u{00b0}/d");
        assert_eq!(speed_dday(-0.5), "-0.5000\u{00b0}/d");
        assert_eq!(speed_dday(13.176), "+13.1760\u{00b0}/d");
    }

    #[test]
    fn rule_widths() {
        assert_eq!(rule(0), "");
        assert_eq!(rule(1), "\u{2500}");
        assert_eq!(rule(4), "\u{2500}\u{2500}\u{2500}\u{2500}");
    }

    #[test]
    fn lpad_rpad_padding() {
        assert_eq!(lpad("x", 3), "  x");
        assert_eq!(lpad("abc", 2), "abc"); // width < len: unchanged
        assert_eq!(lpad("", 2), "  ");
        assert_eq!(rpad("x", 3), "x  ");
        assert_eq!(rpad("abc", 2), "abc");
        assert_eq!(rpad("", 0), "");
    }

    #[test]
    fn json_obj_numeric_and_string_values() {
        assert_eq!(json_obj(&[]), "{\n\n}");
        assert_eq!(
            json_obj(&[("n", "1.5".to_string())]),
            "{\n  \"n\": 1.5\n}"
        );
        assert_eq!(
            json_obj(&[("s", "hello".to_string())]),
            "{\n  \"s\": \"hello\"\n}"
        );
        assert_eq!(
            json_obj(&[("q", "a\"b".to_string())]),
            "{\n  \"q\": \"a\\\"b\"\n}"
        );
        assert_eq!(
            json_obj(&[("a", "1".to_string()), ("b", "x".to_string())]),
            "{\n  \"a\": 1,\n  \"b\": \"x\"\n}"
        );
    }

    #[test]
    fn json_array_wrapping() {
        assert_eq!(json_array(vec![]), "[\n\n]");
        assert_eq!(json_array(vec!["1".to_string()]), "[\n1\n]");
        assert_eq!(
            json_array(vec!["{}".to_string(), "{}".to_string()]),
            "[\n{},\n{}\n]"
        );
    }
}
