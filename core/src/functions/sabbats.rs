//! Celtic sabbats — the Wheel of the Year.
//!
//! The eight sabbats are the major festivals of the Celtic / neopagan calendar.
//! Four are **quarter days** (solstices and equinoxes), defined as the moment
//! when the Sun crosses a specific ecliptic longitude.  Four are **cross-quarter
//! days** (fire festivals), halfway between a quarter day and the next.
//!
//! All sabbats are calculated astronomically — there are no fixed calendar
//! approximations.
//!
//! | Sabbat      | Solar longitude | Typical date |
//! |-------------|-----------------|--------------|
//! | Yule        | 270°            | Dec 21       |
//! | Imbolc      | 315°            | Feb 1        |
//! | Ostara      | 0°              | Mar 20       |
//! | Beltane     | 45°             | May 1        |
//! | Litha       | 90°             | Jun 21       |
//! | Lughnasadh  | 135°            | Aug 1        |
//! | Mabon       | 180°            | Sep 22       |
//! | Samhain     | 225°            | Oct 31       |

use crate::body::{CalcFlags, Calendar};
use crate::error::{Error, Result};

// ─── SabbatKind ──────────────────────────────────────────────────────────────

/// The eight Celtic sabbats forming the Wheel of the Year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SabbatKind {
    /// Winter Solstice — Sun at 270° — typically Dec 21.
    Yule,
    /// First cross-quarter — Sun at 315° (15° Aquarius) — typically Feb 1.
    Imbolc,
    /// Spring Equinox — Sun at 0° (0° Aries) — typically Mar 20.
    Ostara,
    /// Second cross-quarter — Sun at 45° (15° Taurus) — typically May 1.
    Beltane,
    /// Summer Solstice — Sun at 90° — typically Jun 21.
    Litha,
    /// Third cross-quarter — Sun at 135° (15° Leo) — typically Aug 1.
    Lughnasadh,
    /// Autumn Equinox — Sun at 180° — typically Sep 22.
    Mabon,
    /// Fourth cross-quarter — Sun at 225° (15° Scorpio) — typically Oct 31.
    Samhain,
}

impl SabbatKind {
    /// The ecliptic longitude (°) of the Sun at which this sabbat occurs.
    #[must_use]
    pub fn solar_longitude(self) -> f64 {
        match self {
            Self::Yule => 270.0,
            Self::Imbolc => 315.0,
            Self::Ostara => 0.0,
            Self::Beltane => 45.0,
            Self::Litha => 90.0,
            Self::Lughnasadh => 135.0,
            Self::Mabon => 180.0,
            Self::Samhain => 225.0,
        }
    }

    /// Primary name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Yule => "Yule",
            Self::Imbolc => "Imbolc",
            Self::Ostara => "Ostara",
            Self::Beltane => "Beltane",
            Self::Litha => "Litha",
            Self::Lughnasadh => "Lughnasadh",
            Self::Mabon => "Mabon",
            Self::Samhain => "Samhain",
        }
    }

    /// Alternative names from different Celtic and neopagan traditions.
    #[must_use]
    pub fn alt_names(self) -> &'static [&'static str] {
        match self {
            Self::Yule => &["Winter Solstice", "Midwinter", "Alban Arthan"],
            Self::Imbolc => &["Candlemas", "St. Brigid's Day", "Oimelc", "Brigid"],
            Self::Ostara => &["Spring Equinox", "Vernal Equinox", "Alban Eilir", "Eostre"],
            Self::Beltane => &["May Day", "Walpurgis Night", "Cétamain"],
            Self::Litha => &[
                "Summer Solstice",
                "Midsummer",
                "Alban Hefin",
                "St. John's Eve",
            ],
            Self::Lughnasadh => &["Lammas", "Lughnasa", "First Harvest"],
            Self::Mabon => &[
                "Autumn Equinox",
                "Fall Equinox",
                "Alban Elfed",
                "Second Harvest",
            ],
            Self::Samhain => &[
                "Hallowe'en",
                "All Hallows",
                "Third Harvest",
                "Celtic New Year",
            ],
        }
    }

    /// `true` for the four quarter days (solstices and equinoxes).
    #[must_use]
    pub fn is_quarter_day(self) -> bool {
        matches!(self, Self::Yule | Self::Ostara | Self::Litha | Self::Mabon)
    }

    /// `true` for the four cross-quarter days (fire festivals).
    #[must_use]
    pub fn is_cross_quarter(self) -> bool {
        !self.is_quarter_day()
    }

    /// All eight sabbats in ascending solar-longitude order
    /// (Yule at 270° → Imbolc 315° → Ostara 0° → … → Samhain 225°).
    #[must_use]
    pub fn all_by_longitude() -> [Self; 8] {
        [
            Self::Yule,
            Self::Imbolc,
            Self::Ostara,
            Self::Beltane,
            Self::Litha,
            Self::Lughnasadh,
            Self::Mabon,
            Self::Samhain,
        ]
    }

    /// The month (1 = Jan) when this sabbat typically falls.
    fn approx_month(self) -> i32 {
        match self {
            Self::Yule => 12,
            Self::Imbolc => 2,
            Self::Ostara => 3,
            Self::Beltane => 5,
            Self::Litha => 6,
            Self::Lughnasadh => 8,
            Self::Mabon => 9,
            Self::Samhain => 10,
        }
    }
}

// ─── Sabbat ───────────────────────────────────────────────────────────────────

/// A Celtic sabbat occurrence with its exact Julian day.
#[derive(Debug, Clone, PartialEq)]
pub struct Sabbat {
    /// Which sabbat this is.
    pub kind: SabbatKind,
    /// Primary name (e.g. `"Yule"`).
    pub name: &'static str,
    /// Julian day (UT) of the exact astronomical moment.
    pub jd: f64,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Find the exact Julian day (UT) when the Sun reaches a sabbat's ecliptic
/// longitude in a given Gregorian year.
///
/// # Example
/// ```rust
/// use celestial_core::{sabbat_jd, SabbatKind, revjul};
/// use celestial_core::body::Calendar;
/// let jd = sabbat_jd(2024, SabbatKind::Ostara).unwrap();
/// let d  = revjul(jd, Calendar::Gregorian);
/// assert_eq!(d.month, 3);  // March
/// assert_eq!(d.year,  2024);
/// ```
#[inline]
pub fn sabbat_jd(year: i32, kind: SabbatKind) -> Result<f64> {
    let lon = kind.solar_longitude();
    // Start the search ~20 days before the typical calendar date so that
    // `solcross` (which moves forward in time) reliably finds the right crossing.
    let month = kind.approx_month();
    let jd_start = crate::julday(year, month, 1, 0.0, Calendar::Gregorian) - 20.0;

    crate::functions::motion::solcross_ut(lon, jd_start, CalcFlags::BUILTIN)
        .map_err(|_| Error::Calc(format!("could not find {} for year {year}", kind.name())))
}

/// All eight sabbats for a given Gregorian year, sorted chronologically.
///
/// Returns the moment when the Sun crosses each of the eight defining ecliptic
/// longitudes during the calendar year.  The year is in the Gregorian sense
/// (Jan 1 – Dec 31); Yule always falls in December of the given year.
pub fn sabbats_for_year(year: i32) -> Result<Vec<Sabbat>> {
    let mut result = Vec::with_capacity(8);
    for kind in SabbatKind::all_by_longitude() {
        let jd = sabbat_jd(year, kind)?;
        result.push(Sabbat {
            kind,
            name: kind.name(),
            jd,
        });
    }
    result.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    Ok(result)
}

/// The next sabbat at or after the given Julian day.
pub fn next_sabbat(jd_from: f64) -> Result<Sabbat> {
    let d = crate::revjul(jd_from, Calendar::Gregorian);
    // Collect from this year and next to guarantee a result.
    let mut candidates: Vec<Sabbat> = Vec::with_capacity(16);
    for year in [d.year, d.year + 1] {
        if let Ok(sabbats) = sabbats_for_year(year) {
            candidates.extend(sabbats);
        }
    }
    candidates
        .into_iter()
        .filter(|s| s.jd >= jd_from)
        .min_by(|a, b| a.jd.total_cmp(&b.jd))
        .ok_or_else(|| Error::Calc("could not find next sabbat".into()))
}
