//! Auto-split from chart.rs — do not edit section headers.

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 7 — Mesoamerican calendars (Aztec/Nahuatl + Mayan)
// ═══════════════════════════════════════════════════════════════════════════════

/// GMT correlation constant (Goodman–Martínez–Thompson).
/// Converts Julian Day to the Maya/Aztec Long Count.
/// JD 584283 = Maya Day 0 (correlation adopted by most scholars).
pub const GMT_CORRELATION: i64 = 584_283;

/// The 20 Nahuatl day signs (Tonalpohualli) in traditional order.
pub const TONALPOHUALLI_SIGNS: &[(&str, &str)] = &[
    ("Cipactli", "Crocodile"),
    ("Ehecatl", "Wind"),
    ("Calli", "House"),
    ("Cuetzpallin", "Lizard"),
    ("Coatl", "Serpent"),
    ("Miquiztli", "Death"),
    ("Mazatl", "Deer"),
    ("Tochtli", "Rabbit"),
    ("Atl", "Water"),
    ("Itzcuintli", "Dog"),
    ("Ozomatli", "Monkey"),
    ("Malinalli", "Grass"),
    ("Acatl", "Reed"),
    ("Ocelotl", "Ocelot"),
    ("Cuauhtli", "Eagle"),
    ("Cozcacuauhtli", "Vulture"),
    ("Ollin", "Earthquake"),
    ("Tecpatl", "Flint"),
    ("Quiahuitl", "Rain"),
    ("Xochitl", "Flower"),
];

/// The 20 Maya day signs (Tzolkin) in traditional order.
pub const TZOLKIN_SIGNS: &[(&str, &str)] = &[
    ("Imix", "Water Lily"),
    ("Ik", "Wind"),
    ("Akbal", "Night"),
    ("Kan", "Corn"),
    ("Chicchan", "Serpent"),
    ("Cimi", "Death"),
    ("Manik", "Deer"),
    ("Lamat", "Star"),
    ("Muluc", "Rain"),
    ("Oc", "Dog"),
    ("Chuen", "Monkey"),
    ("Eb", "Road"),
    ("Ben", "Corn Stalk"),
    ("Ix", "Jaguar"),
    ("Men", "Eagle"),
    ("Cib", "Vulture"),
    ("Caban", "Earth"),
    ("Etznab", "Flint"),
    ("Cauac", "Storm"),
    ("Ahau", "Sun"),
];

/// The 18 Aztec months of the Xiuhpohualli (365-day solar year).
pub const XIUHPOHUALLI_MONTHS: &[(&str, &str)] = &[
    ("Izcalli", "Sprouting"),
    ("Atlcahualo", "Ceasing of Water"),
    ("Tlacaxipehualiztli", "Flaying of Men"),
    ("Tozoztontli", "Small Vigil"),
    ("Huei Tozoztli", "Great Vigil"),
    ("Toxcatl", "Drought"),
    ("Etzalqualiztli", "Eating of Maize"),
    ("Tecuilhuitontli", "Small Feast of Lords"),
    ("Huei Tecuilhuitl", "Great Feast of Lords"),
    ("Tlaxochimaco", "Offering of Flowers"),
    ("Xocotl Huetzi", "Falling Fruit"),
    ("Ochpaniztli", "Sweeping"),
    ("Teotleco", "Return of the Gods"),
    ("Tepeilhuitl", "Mountain Feast"),
    ("Quecholli", "Flamingo"),
    ("Panquetzaliztli", "Raising of Banners"),
    ("Atemoztli", "Falling Water"),
    ("Tititl", "Stretching"),
];

/// Aztec Tonalpohualli position for a given Julian Day.
///
/// Returns `(trecena_number, day_sign_index, day_sign_name, day_sign_english)`.
/// The trecena (13-day week) number is 1–13; the day sign index is 0–19.
///
/// Uses the GMT correlation. The Aztec calendar system is in continuous
/// synchrony with the Maya Tzolkin.
pub fn tonalpohualli(jd: f64) -> (u8, usize, &'static str, &'static str) {
    // Aztec day number from the base correlation
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(260) as usize;
    let trecena = (day_num % 13 + 1) as u8; // 1–13
    let sign_idx = day_num % 20; // 0–19
    (
        trecena,
        sign_idx,
        TONALPOHUALLI_SIGNS[sign_idx].0,
        TONALPOHUALLI_SIGNS[sign_idx].1,
    )
}

/// Aztec Xiuhpohualli (365-day solar year) position for a Julian Day.
///
/// Returns `(month_index, day_in_month, month_name, month_english)`.
/// Month 18 (index 18) is the 5-day "Nemontemi" (unlucky days).
pub fn xiuhpohualli(jd: f64) -> (usize, u8, &'static str, &'static str) {
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(365) as usize;
    let month_idx = (day_num / 20).min(18);
    let day_in_month = (day_num % 20 + 1) as u8;
    if month_idx < 18 {
        (
            month_idx,
            day_in_month,
            XIUHPOHUALLI_MONTHS[month_idx].0,
            XIUHPOHUALLI_MONTHS[month_idx].1,
        )
    } else {
        // Nemontemi: 5 unlucky days after the 18 months
        (18, (day_num - 360 + 1) as u8, "Nemontemi", "Unlucky Days")
    }
}

/// Maya Tzolkin (260-day sacred calendar) position for a Julian Day.
///
/// Returns `(trecena_number, day_sign_index, day_sign_name, day_sign_english)`.
pub fn tzolkin(jd: f64) -> (u8, usize, &'static str, &'static str) {
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(260) as usize;
    let trecena = (day_num % 13 + 1) as u8;
    let sign_idx = day_num % 20;
    (
        trecena,
        sign_idx,
        TZOLKIN_SIGNS[sign_idx].0,
        TZOLKIN_SIGNS[sign_idx].1,
    )
}

/// Maya Haab (365-day vague year) position for a Julian Day.
///
/// Returns `(month_index, day_in_month, month_name)`.
/// Months 0–17 each have 20 days; month 18 (Wayeb) has 5.
pub fn haab(jd: f64) -> (usize, u8, &'static str) {
    const HAAB_MONTHS: &[&str] = &[
        "Pop", "Wo", "Sip", "Sotz", "Sek", "Xul", "Yaxkin", "Mol", "Ch'en", "Yax", "Sak", "Keh",
        "Mak", "Kankin", "Muwan", "Pax", "Kayab", "Kumku", "Wayeb",
    ];
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(365) as usize;
    let month_idx = (day_num / 20).min(18);
    let day = (day_num % 20) as u8;
    let name = if month_idx < 19 {
        HAAB_MONTHS[month_idx]
    } else {
        "Wayeb"
    };
    (month_idx, day, name)
}

/// Maya Calendar Round: the 52-year cycle combining Tzolkin + Haab.
/// Returns `(tzolkin_trecena, tzolkin_sign, haab_day, haab_month)`.
pub fn calendar_round(jd: f64) -> (u8, &'static str, u8, &'static str) {
    let (trecena, _, sign_name, _) = tzolkin(jd);
    let (_, haab_day, haab_month) = haab(jd);
    (trecena, sign_name, haab_day, haab_month)
}
