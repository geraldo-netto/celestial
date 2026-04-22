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

pub mod buddhist;
pub mod celtic;
pub mod christian;
pub mod hebrew;
pub mod hindu;
pub mod islamic;
pub mod persian;
pub use crate::functions::easter::{
    christian_feasts, christian_fixed_feasts, easter_gregorian, easter_jd, easter_orthodox,
    easter_orthodox_jd, ChristianFeast,
};
pub use crate::functions::islamic::{
    gregorian_to_hijri_years, hijri_from_jd, hijri_month_name, hijri_to_jd, islamic_observances,
    IslamicObservance,
};
pub use crate::functions::jewish::{
    hebrew_year_from_jd, jd_to_hebrew_date, jewish_holiday_jd, jewish_holidays, JewishHoliday,
};
pub use crate::functions::nowruz::{
    bahai_holy_days, gregorian_to_solar_hijri, jd_to_bahai, naw_ruz_jd, nowruz_jd, BahaiDate,
    BahaiHolyDay,
};
pub use crate::functions::omer::{
    omer_day_jd, omer_days, omer_declaration, omer_from_jd, omer_period, omer_start_jd, OmerDay,
    OmerPeriod,
};
