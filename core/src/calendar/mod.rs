//! Multi-tradition religious and spiritual calendars.
//!
//! | Submodule | Tradition |
//! |---|---|
//! | [`hebrew`] | Hebrew calendar, Sefirat HaOmer, Jewish holidays |
//! | [`christian`] | Easter (Gregorian + Orthodox), liturgical feasts |
//! | [`islamic`] | Hijri calendar, Ramadan, Eid |
//! | [`hindu`] | Panchānga, festivals |
//! | [`buddhist`] | Vesak, Uposatha days |
//! | [`persian`] | Nowruz, Solar Hijri, Bahá'í |
//! | [`celtic`] | Wheel of the Year, named full moons |

#[cfg(feature = "calendar-traditions")]
pub mod buddhist;
#[cfg(feature = "calendar-traditions")]
pub mod celtic;
#[cfg(feature = "calendar-traditions")]
pub mod christian;
#[cfg(feature = "calendar-traditions")]
pub mod hebrew;
#[cfg(feature = "calendar-traditions")]
pub mod hindu;
#[cfg(feature = "calendar-traditions")]
pub mod islamic;
#[cfg(feature = "calendar-traditions")]
pub mod persian;
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::easter::{
    christian_feasts, christian_fixed_feasts, easter_gregorian, easter_jd, easter_julian,
    easter_orthodox, easter_orthodox_jd, ChristianFeast,
};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::islamic::{
    gregorian_to_hijri_years, hijri_from_jd, hijri_month_days, hijri_month_name,
    hijri_month_start_jd, hijri_new_year_jd, hijri_to_jd, is_hijri_leap_year, islamic_observances,
    islamic_observances_for_jd, IslamicObservance,
};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::jewish::{
    hebrew_year_from_jd, jd_to_hebrew_date, jewish_holiday_jd, jewish_holidays, JewishHoliday,
};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::nowruz::{
    bahai_holy_days, gregorian_to_solar_hijri, is_bahai_leap_year, jd_to_bahai, naw_ruz_jd,
    nowruz_jd, solar_hijri_to_gregorian, BahaiDate, BahaiHolyDay,
};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::omer::{
    approx_hebrew_year, days_in_hebrew_year, elapsed_days, hebrew_month_days,
    hebrew_month_start_jd, hebrew_new_year_jd, is_hebrew_leap_year, months_in_hebrew_year,
    omer_day_jd, omer_days, omer_declaration, omer_from_jd, omer_period, omer_start_jd, OmerDay,
    OmerPeriod,
};

#[cfg(feature = "calendar-traditions")]
pub use crate::functions::coptic::{
    coptic_month_days, coptic_to_jd, ethiopic_to_jd, is_coptic_leap_year, jd_to_coptic,
    jd_to_ethiopic, COPTIC_EPOCH_JD, COPTIC_MONTHS, ETHIOPIC_EPOCH_JD, ETHIOPIC_MONTHS,
};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::tibetan::{losar_jd, tibetan_year_name, LHASA_TZ_OFFSET_HOURS};
#[cfg(feature = "calendar-traditions")]
pub use crate::functions::zoroastrian::{fasli_nowruz_jd, jd_to_fasli, FASLI_MONTHS, GATHA_DAYS};
