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
pub fn lon_zodiac(lon: f64) -> String {
    let lon = lon.rem_euclid(360.0);
    let idx = (lon / 30.0).floor() as usize;
    let deg = lon % 30.0;
    let d = deg as u32;
    let min = ((deg - d as f64) * 60.0).round() as u32;
    format!("{d:2}\u{00b0}{min:02}\' {}", SIGNS[idx])
}

/// Format decimal degrees as `±DDD°MM'SS"`.
pub fn deg_dms(deg: f64) -> String {
    let sign = if deg >= 0.0 { "" } else { "-" };
    let abs = deg.abs();
    let d = abs as u32;
    let m = ((abs - d as f64) * 60.0) as u32;
    let s = ((abs - d as f64) * 3600.0 - (m as f64 * 60.0)).round() as u32;
    format!("{sign}{d}\u{00b0}{m:02}\'{s:02}\"")
}

/// Format AU distance.
pub fn dist_au(au: f64) -> String {
    format!("{au:.6} AU")
}

/// Format degrees-per-day speed with sign.
pub fn speed_dday(s: f64) -> String {
    let sign = if s >= 0.0 { "+" } else { "" };
    format!("{sign}{s:.4}\u{00b0}/d")
}

// ─── Simple ASCII table ───────────────────────────────────────────────────────

/// Draw a horizontal rule of a given width.
pub fn rule(width: usize) -> String {
    "\u{2500}".repeat(width)
}

#[allow(dead_code)]
/// Left-pad a string to `width` characters.
pub fn lpad(s: &str, width: usize) -> String {
    format!("{s:>width$}")
}

#[allow(dead_code)]
/// Right-pad a string to `width` characters.
pub fn rpad(s: &str, width: usize) -> String {
    format!("{s:<width$}")
}

// ─── JSON helpers ─────────────────────────────────────────────────────────────

/// Produce a simple JSON object from key-value pairs where values are pre-formatted strings.
pub fn json_obj(pairs: &[(&str, String)]) -> String {
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
pub fn json_array(items: Vec<String>) -> String {
    format!("[\n{}\n]", items.join(",\n"))
}
