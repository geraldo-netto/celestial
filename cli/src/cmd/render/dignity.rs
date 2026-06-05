//! Planet table, display colours, aspect defs, essential dignities,
//! antiscia. Extracted from the render god-file (ARCH-8); re-exported
//! by `mod.rs` so `super::planet_dignity` call sites are unchanged.

use celestial_core::body::Body;
use celestial_core::{sign_exaltation, sign_ruler};

/// Map a planet key string to a Body.
pub(crate) fn key_to_body(key: &str) -> Option<Body> {
    match key {
        "sun" => Some(Body::SUN),
        "moon" => Some(Body::MOON),
        "mercury" => Some(Body::MERCURY),
        "venus" => Some(Body::VENUS),
        "mars" => Some(Body::MARS),
        "jupiter" => Some(Body::JUPITER),
        "saturn" => Some(Body::SATURN),
        "uranus" => Some(Body::URANUS),
        "neptune" => Some(Body::NEPTUNE),
        "pluto" => Some(Body::PLUTO),
        "mean_node" => Some(Body::MEAN_NODE),
        "chiron" => Some(Body::CHIRON),
        _ => None,
    }
}

/// Planet display glyphs. Each Unicode symbol is followed by the
/// text-presentation variation selector (`U+FE0E`) so renderers don't
/// substitute the chunky colour-emoji form for the astrological symbol
/// — same fix as the zodiac glyphs in `SIGN_GLYPHS`.
pub(crate) const BODIES: &[(Body, &str, &str, &str)] = &[
    (Body::SUN, "sun", "Sun", "\u{2609}\u{FE0E}"),
    (Body::MOON, "moon", "Moon", "\u{263D}\u{FE0E}"),
    (Body::MERCURY, "mercury", "Mercury", "\u{263F}\u{FE0E}"),
    (Body::VENUS, "venus", "Venus", "\u{2640}\u{FE0E}"),
    (Body::MARS, "mars", "Mars", "\u{2642}\u{FE0E}"),
    (Body::JUPITER, "jupiter", "Jupiter", "\u{2643}\u{FE0E}"),
    (Body::SATURN, "saturn", "Saturn", "\u{2644}\u{FE0E}"),
    (Body::URANUS, "uranus", "Uranus", "\u{2645}\u{FE0E}"),
    (Body::NEPTUNE, "neptune", "Neptune", "\u{2646}\u{FE0E}"),
    (Body::PLUTO, "pluto", "Pluto", "\u{2647}\u{FE0E}"),
    (
        Body::MEAN_NODE,
        "mean_node",
        "Node (North)",
        "\u{260A}\u{FE0E}",
    ),
    (Body::CHIRON, "chiron", "Chiron", "\u{26B7}\u{FE0E}"),
];

/// Per-body display colour (key → hex). Picked to approximate the
/// traditional astrological palette used by the World-of-Wisdom PDF
/// reference: warm metals for the luminaries, mode-coloured outer
/// planets, mercury/venus in green/pink for their domiciles.
pub(crate) const BODY_COLORS: &[(&str, &str)] = &[
    ("sun", "#d4a017"),
    ("moon", "#6b7888"),
    ("mercury", "#2c9c4f"),
    ("venus", "#d65a9e"),
    ("mars", "#c1272d"),
    ("jupiter", "#5d3f8e"),
    ("saturn", "#4a4036"),
    ("uranus", "#0085c7"),
    ("neptune", "#1ba89d"),
    ("pluto", "#7c1a1a"),
    ("mean_node", "#6a4f8a"),
    ("true_node", "#6a4f8a"),
    ("south_node", "#6a4f8a"),
    ("chiron", "#6c3a1a"),
];

#[must_use]
pub(crate) fn body_color(key: &str) -> &'static str {
    BODY_COLORS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("#0d0d1e", |(_, c)| *c)
}

/// (angle°, name, orb_limit, is_minor)
pub(crate) const ASPECT_DEFS: &[(f64, &str, f64, bool)] = &[
    // ── Major aspects ─────────────────────────────────────────────────────────
    (0.0, "conjunction", 8.0, false),
    (60.0, "sextile", 6.0, false),
    (90.0, "square", 7.0, false),
    (120.0, "trine", 8.0, false),
    (150.0, "quincunx", 3.0, false),
    (180.0, "opposition", 8.0, false),
    // ── Minor aspects ─────────────────────────────────────────────────────────
    (30.0, "semi-sextile", 2.0, true),
    (45.0, "semi-square", 2.0, true),
    (72.0, "quintile", 1.5, true),
    (135.0, "sesquiquadrate", 2.0, true),
    (144.0, "biquintile", 1.5, true),
    (51.43, "septile", 1.0, true),
    (40.0, "novile", 1.0, true),
];

/// Secondary (pre-outer-planet) domicile for the 5 classical rulers.
///
/// Returns `Some(second_sign)` when `body` is one of Mercury/Venus/Mars/Jupiter/Saturn,
/// otherwise `None`. Each classical planet rules two signs of opposite polarity;
/// `sign_ruler` only reports one of them, so this fills in the other.
fn second_domicile(body: Body) -> Option<u8> {
    match body.as_raw() {
        2 => Some(8),  // Mercury: Gemini + Virgo
        3 => Some(6),  // Venus:   Taurus + Libra
        4 => Some(7),  // Mars:    Aries  + Scorpio
        5 => Some(11), // Jupiter: Sagittarius + Pisces
        6 => Some(9),  // Saturn:  Capricorn + Aquarius
        _ => None,
    }
}

/// Classical essential dignity of `body` in `sign`.
///
/// Checks, in order: domicile → detriment → exaltation → fall → peregrine.
/// Returns the dignity name as a static string.
pub(crate) fn planet_dignity(body: Body, sign: u8) -> &'static str {
    let s = sign % 12;
    let opp = (s + 6) % 12;

    // A body rules `sign` if it is the primary ruler or the secondary domicile
    let rules = |sign: u8| -> bool {
        sign_ruler(sign) == body || second_domicile(body).is_some_and(|s2| s2 == sign)
    };

    if rules(s) {
        return "domicile";
    }
    if rules(opp) {
        return "detriment";
    }

    // Exaltation & fall
    let ex = sign_exaltation(body);
    if ex >= 0 && ex as u8 == s {
        return "exaltation";
    }
    if ex >= 0 && (ex as u8 + 6) % 12 == s {
        return "fall";
    }

    "peregrine"
}

/// Antiscion longitude: reflection over the 0°Cancer / 0°Capricorn (solstice) axis.
/// Formula: antiscion = (180° - lon) mod 360°
#[inline]
pub(crate) fn antiscion_lon(lon: f64) -> f64 {
    (180.0 - lon).rem_euclid(360.0)
}

/// Contra-antiscion longitude: reflection over the 0°Aries / 0°Libra (equinox) axis.
/// Formula: contra = (360° - lon) mod 360°
#[inline]
pub(crate) fn contra_antiscion_lon(lon: f64) -> f64 {
    (360.0 - lon).rem_euclid(360.0)
}
