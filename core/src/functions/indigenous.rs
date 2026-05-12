//! Auto-split from chart.rs — do not edit section headers.

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 8 — Indigenous / other calendar systems
// ═══════════════════════════════════════════════════════════════════════════════

/// Native American Medicine Wheel birth totem (modern Sun Bear synthesis, 1970s).
///
/// Returns `(totem_animal, element, clan, season)` for a given calendar date.
/// This is a modern (New Age) synthesis, not a traditional indigenous system.
/// The system divides the solar year into 12 birth totems by Sun longitude.
///
/// # Note
/// This reflects the Sun Bear / Wabun Wind system from "The Medicine Wheel" (1980).
/// It is NOT representative of any single traditional indigenous nation's teachings.
#[must_use]
pub fn medicine_wheel_totem(
    sun_lon: f64,
) -> (&'static str, &'static str, &'static str, &'static str) {
    // (min_lon, max_lon, animal, element, clan, season)
    // Aligned to approximate Sun longitude ranges (tropical)
    const TOTEMS: &[(f64, f64, &str, &str, &str, &str)] = &[
        (300.0, 330.0, "Snow Goose", "Earth", "Turtle", "Winter"),
        (330.0, 360.0, "Otter", "Air", "Butterfly", "Winter"),
        (0.0, 30.0, "Cougar", "Air", "Butterfly", "Spring"),
        (30.0, 60.0, "Red Hawk", "Fire", "Thunderbird", "Spring"),
        (60.0, 90.0, "Beaver", "Earth", "Turtle", "Spring"),
        (90.0, 120.0, "Deer", "Air", "Butterfly", "Summer"),
        (120.0, 150.0, "Flicker", "Water", "Frog", "Summer"),
        (150.0, 180.0, "Sturgeon", "Fire", "Thunderbird", "Summer"),
        (180.0, 210.0, "Brown Bear", "Earth", "Turtle", "Autumn"),
        (210.0, 240.0, "Raven", "Air", "Butterfly", "Autumn"),
        (240.0, 270.0, "Snake", "Water", "Frog", "Autumn"),
        (270.0, 300.0, "Elk", "Fire", "Thunderbird", "Winter"),
    ];
    let lon = sun_lon.rem_euclid(360.0);
    for &(lo, hi, animal, element, clan, season) in TOTEMS {
        let in_range = if lo < hi {
            lon >= lo && lon < hi
        } else {
            lon >= lo || lon < hi
        };
        if in_range {
            return (animal, element, clan, season);
        }
    }
    ("Snow Goose", "Earth", "Turtle", "Winter") // fallback
}

/// Egyptian decans: the 36 ten-degree sectors of the zodiac, each with a
/// traditional decan deity name and the rising star association.
///
/// Returns `(decan_index 0–35, decan_name, associated_star)`.
#[must_use]
pub fn egyptian_decan(lon: f64) -> (usize, &'static str, &'static str) {
    const DECANS: &[(&str, &str)] = &[
        ("Khontarty", "Alphard"),
        ("Khontarty II", "Alphard"),
        ("Sirius", "Sirius"),
        ("Arcturus", "Arcturus"),
        ("Kemthor", "Algol"),
        ("Sothis", "Sirius"),
        ("Sag.", "Antares"),
        ("Knmt", "Fomalhaut"),
        ("Hnt-hrw", "Aldebaran"),
        ("Hnt-hrw II", "Aldebaran"),
        ("Tpy-'", "Regulus"),
        ("'rt", "Spica"),
        ("Hry-ib wiꜣ", "Spica"),
        ("Stwy", "Arcturus"),
        ("Ipds", "Vega"),
        ("Sbšsn", "Vega"),
        ("Srt", "Fomalhaut"),
        ("Tpy-'", "Capella"),
        ("'rt II", "Capella"),
        ("Rmn Hry", "Pollux"),
        ("Hspy", "Regulus"),
        ("Qd", "Regulus"),
        ("Hȝtȝ", "Sirius"),
        ("Bȝwy", "Sirius"),
        ("Knmt II", "Antares"),
        ("Sdy", "Antares"),
        ("Wȝḫȝ", "Fomalhaut"),
        ("Bktȝ", "Aldebaran"),
        ("Iry Ḥr", "Aldebaran"),
        ("Remennut", "Spica"),
        ("Sothis B", "Sirius"),
        ("Sȝḥ", "Orion's Belt"),
        ("Hkt", "Regulus"),
        ("Dsr-Ḏsr", "Vega"),
        ("Rmn Ḥry II", "Arcturus"),
        ("Wȝḫ", "Fomalhaut"),
    ];
    let idx = (lon / 10.0) as usize % 36;
    (idx, DECANS[idx].0, DECANS[idx].1)
}
