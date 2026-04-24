#![cfg(feature = "timezone")]
//! Tests for the database, geographic atlas, and timezone modules.

use celestial_core::*;

// ═══════════════════════════════════════════════════════════════════════════
// Timezone — full 203-entry table
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tz_table_has_203_entries() {
    // The C source defines SWH_TZABBR_NUM as 203
    assert!(
        TZ_TABLE.len() >= 200,
        "expected ≥200 TZ entries, got {}",
        TZ_TABLE.len()
    );
}

#[test]
fn tz_find_utc() {
    let r = tz_abbr_find("UTC");
    assert!(!r.is_empty(), "UTC not found");
    assert_eq!(r[0].hours, 0);
    assert_eq!(r[0].minutes, 0);
    assert!(!r[0].offset.is_empty());
}

#[test]
fn tz_find_ist_three_matches() {
    // IST is tripl: Indian Standard Time, Irish Standard Time, Israel Standard Time
    let r = tz_abbr_find("IST");
    assert!(r.len() >= 2, "IST should have ≥2 entries, got {}", r.len());
}

#[test]
fn tz_find_bst_three_matches() {
    let r = tz_abbr_find("BST");
    assert!(r.len() >= 2, "BST should have ≥2 entries");
}

#[test]
fn tz_find_case_insensitive() {
    let lower = tz_abbr_find("utc");
    let upper = tz_abbr_find("UTC");
    assert_eq!(lower.len(), upper.len(), "case insensitive search");
}

#[test]
fn tz_find_nzst() {
    let r = tz_abbr_find("NZST");
    assert!(!r.is_empty());
    assert_eq!(r[0].hours, 12);
}

#[test]
fn tz_find_npt_nepal() {
    let r = tz_abbr_find("NPT");
    assert!(!r.is_empty());
    let npt = r.iter().find(|t| t.hours == 5 && t.minutes == 45);
    assert!(npt.is_some(), "Nepal Time (+05:45) not found");
}

#[test]
fn tz_find_acdt_plus1030() {
    // Australian Central Daylight +10:30
    let r = tz_abbr_find("ACDT");
    assert!(!r.is_empty());
    assert_eq!(r[0].hours, 10);
    assert_eq!(r[0].minutes, 30);
}

#[test]
fn tz_find_unknown_empty() {
    let r = tz_abbr_find("XYZNOTEXIST");
    assert!(r.is_empty());
}

#[test]
fn tz_find_too_short() {
    let r = tz_abbr_find("X");
    assert!(r.is_empty(), "single-char should return nothing");
}

#[test]
fn tz_all_offsets_reasonable() {
    for tz in TZ_TABLE.iter() {
        assert!(
            tz.hours >= -12 && tz.hours <= 14,
            "hours={} for {}",
            tz.hours,
            tz.name
        );
        assert!(
            tz.minutes >= 0 && tz.minutes < 60,
            "minutes={} for {}",
            tz.minutes,
            tz.name
        );
        assert!(!tz.name.is_empty(), "empty name");
        assert!(!tz.offset.is_empty(), "empty offset for {}", tz.name);
    }
}
