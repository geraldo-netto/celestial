//! Auto-split from chart.rs — do not edit section headers.

use crate::units::Longitude;

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
    sun_lon: Longitude,
) -> (&'static str, &'static str, &'static str, &'static str) {
    let sun_lon: f64 = sun_lon.into();
    if !sun_lon.is_finite() {
        return ("Snow Goose", "Earth", "Turtle", "Winter");
    }
    const TOTEMS: &[(&str, &str, &str, &str)] = &[
        ("Red Hawk", "Fire", "Thunderbird", "Spring"),
        ("Beaver", "Earth", "Turtle", "Spring"),
        ("Deer", "Air", "Butterfly", "Spring"),
        ("Flicker", "Water", "Frog", "Summer"),
        ("Sturgeon", "Fire", "Thunderbird", "Summer"),
        ("Brown Bear", "Earth", "Turtle", "Summer"),
        ("Raven", "Air", "Butterfly", "Autumn"),
        ("Snake", "Water", "Frog", "Autumn"),
        ("Elk", "Fire", "Thunderbird", "Autumn"),
        ("Snow Goose", "Earth", "Turtle", "Winter"),
        ("Otter", "Air", "Butterfly", "Winter"),
        ("Cougar", "Water", "Frog", "Winter"),
    ];
    let index = (sun_lon.rem_euclid(360.0) / 30.0) as usize;
    TOTEMS[index]
}

/// Egyptian decans: the 36 ten-degree sectors of the zodiac, each with a
/// traditional decan deity name and the rising star association.
///
/// Returns `(decan_index 0–35, decan_name, associated_star)`.
#[must_use]
pub fn egyptian_decan(lon: Longitude) -> (usize, &'static str, &'static str) {
    let lon: f64 = lon.into();
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
    let idx = (lon.rem_euclid(360.0) / 10.0) as usize;
    (idx, DECANS[idx].0, DECANS[idx].1)
}

#[cfg(test)]
mod tests {
    use super::{egyptian_decan, medicine_wheel_totem};
    use crate::units::Longitude;

    #[test]
    fn medicine_wheel_matches_birth_sectors() {
        let expected = [
            ("Red Hawk", "Fire", "Thunderbird", "Spring"),
            ("Beaver", "Earth", "Turtle", "Spring"),
            ("Deer", "Air", "Butterfly", "Spring"),
            ("Flicker", "Water", "Frog", "Summer"),
            ("Sturgeon", "Fire", "Thunderbird", "Summer"),
            ("Brown Bear", "Earth", "Turtle", "Summer"),
            ("Raven", "Air", "Butterfly", "Autumn"),
            ("Snake", "Water", "Frog", "Autumn"),
            ("Elk", "Fire", "Thunderbird", "Autumn"),
            ("Snow Goose", "Earth", "Turtle", "Winter"),
            ("Otter", "Air", "Butterfly", "Winter"),
            ("Cougar", "Water", "Frog", "Winter"),
        ];

        for (sector, totem) in expected.into_iter().enumerate() {
            let longitude = sector as f64 * 30.0;
            assert_eq!(medicine_wheel_totem(Longitude::new(longitude)), totem);
        }
    }

    #[test]
    fn medicine_wheel_wraps_both_directions() {
        let red_hawk = ("Red Hawk", "Fire", "Thunderbird", "Spring");
        let cougar = ("Cougar", "Water", "Frog", "Winter");
        let snow_goose = ("Snow Goose", "Earth", "Turtle", "Winter");

        assert_eq!(medicine_wheel_totem(Longitude::new(360.0)), red_hawk);
        assert_eq!(medicine_wheel_totem(Longitude::new(-0.1)), cougar);
        assert_eq!(medicine_wheel_totem(Longitude::new(f64::NAN)), snow_goose);
    }

    #[test]
    fn egyptian_decan_wraps_both_directions() {
        assert_eq!(egyptian_decan(Longitude::new(-10.0)).0, 35);
        assert_eq!(egyptian_decan(Longitude::new(360.0)).0, 0);
        assert_eq!(egyptian_decan(Longitude::new(370.0)).0, 1);
    }
}
