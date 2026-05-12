//! Hebrew calendar arithmetic — re-exports the canonical implementation
//! from [`crate::functions::omer`] under shorter, calendar-tradition names.
//!
//! Historically this module duplicated the postponement-rule (dechiyot)
//! algorithm. Both copies are mathematically identical, so the duplicate
//! was removed in favour of these aliases. Tests live in `omer.rs`.

pub use crate::functions::omer::{
    approx_hebrew_year as approx_year, days_in_hebrew_year as days_in_year, elapsed_days,
    hebrew_month_days as month_days, hebrew_month_start_jd as month_start_jd,
    hebrew_new_year_jd as new_year_jd, is_hebrew_leap_year as is_leap_year,
    months_in_hebrew_year as months_in_year,
};
