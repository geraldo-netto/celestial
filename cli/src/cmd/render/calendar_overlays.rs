//! Compositional calendar overlays for the `--chart-type calendar` renderer.
//!
//! Each overlay function takes the requested period and returns a JSON value
//! that becomes available to templates under a fixed key (`gregorian`, `omer`,
//! `sabbats`, `moon`, `hebrew`). Templates can compose any combination.
//!
//! Day-level annotators (`annotate_gregorian_with_*`) splice tradition-specific
//! tags onto each day in the Gregorian grid, so a template loop like
//!
//! ```text
//! {{ for d in gregorian.days }}
//!   <text>{d.day}</text>
//!   {{ if d.omer_day }}<text>Day {d.omer_day} (Omer)</text>{{ endif }}
//!   {{ if d.moon_phase }}<text>{d.moon_glyph}</text>{{ endif }}
//! {{ endfor }}
//! ```
//!
//! works without the user having to compute lookups themselves.

use super::ChartContext;
use celestial_core::{
    julday, moon_illumination, moon_phases_for_month, omer_days, omer_period, revjul,
    sabbats_for_year, Calendar, PrincipalPhase,
};
use celestial_core::JulianDay;
use serde_json::{json, Value};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Gregorian month grid — the "spine" overlay
// ─────────────────────────────────────────────────────────────────────────────

const MARGIN_X: f64 = 60.0;
const HEADER_Y: f64 = 130.0;
const COL_HEADER_H: f64 = 28.0;
const CELL_W: f64 = 110.0;
const CELL_H: f64 = 90.0;

/// Build a Gregorian month-grid overlay for the month of `(year, month)`.
pub fn gregorian_overlay(year: i32, month: u32) -> Value {
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let month_idx = (month.saturating_sub(1) as usize).min(11);

    let days_in_month = days_in_gregorian_month(year, month);
    let first_jd = julday(year, month as i32, 1, 0.0, Calendar::Gregorian);
    // weekday: 0=Sunday … 6=Saturday
    let first_weekday = ((first_jd + 1.5).floor() as i64).rem_euclid(7) as u32;

    let grid_x0 = MARGIN_X;
    let grid_y0 = HEADER_Y + COL_HEADER_H;

    let mut day_objs = Vec::with_capacity(days_in_month as usize);
    let mut max_row = 0u32;
    for d in 1..=days_in_month {
        let weekday = (first_weekday + (d - 1)) % 7;
        let absolute = first_weekday + (d - 1);
        let row = absolute / 7;
        let col = weekday;
        if row > max_row {
            max_row = row;
        }
        let cell_x = grid_x0 + col as f64 * CELL_W;
        let cell_y = grid_y0 + row as f64 * CELL_H;
        let jd = julday(year, month as i32, d as i32, 12.0, Calendar::Gregorian);
        day_objs.push(json!({
            "day":          d,
            "weekday":      weekday,
            "weekday_name": weekday_short(weekday),
            "row":          row,
            "col":          col,
            "jd":           jd,
            "iso_date":     format!("{year:04}-{month:02}-{d:02}"),
            "cell_x":       cell_x,
            "cell_y":       cell_y,
            "cell_w":       CELL_W,
            "cell_h":       CELL_H,
            "label_x":      cell_x + 8.0,
            "label_y":      cell_y + 18.0,
            "center_x":     cell_x + CELL_W * 0.5,
            "center_y":     cell_y + CELL_H * 0.5,
        }));
    }

    let weeks = max_row + 1;
    let grid_w = 7.0 * CELL_W;
    let grid_h = weeks as f64 * CELL_H;

    let mut col_headers = Vec::with_capacity(7);
    for i in 0..7u32 {
        col_headers.push(json!({
            "name": weekday_short(i),
            "x":    grid_x0 + (i as f64 + 0.5) * CELL_W,
            "y":    HEADER_Y + COL_HEADER_H * 0.6,
        }));
    }

    json!({
        "year":          year,
        "month":         month,
        "month_name":    month_names[month_idx],
        "days_in_month": days_in_month,
        "first_jd":      first_jd,
        "first_weekday": first_weekday,
        "weeks":         weeks,
        "days":          day_objs,
        "col_headers":   col_headers,
        "grid_x0":       grid_x0,
        "grid_y0":       grid_y0,
        "grid_w":        grid_w,
        "grid_h":        grid_h,
        "viewbox_w":     grid_x0 + grid_w + MARGIN_X,
        "viewbox_h":     grid_y0 + grid_h + 60.0,
        "cell_w":        CELL_W,
        "cell_h":        CELL_H,
    })
}

fn days_in_gregorian_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn weekday_short(w: u32) -> &'static str {
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .get(w as usize)
        .copied()
        .unwrap_or("?")
}

// ─────────────────────────────────────────────────────────────────────────────
// Omer overlay
// ─────────────────────────────────────────────────────────────────────────────

/// Build a 12-month overlay for an entire calendar year.
///
/// Returns `{ year, months: [<month_overlay>, ...] }` where each month is
/// the same shape as [`gregorian_overlay`] (days array, headers, geometry).
/// This is what year-grid templates iterate to draw a 4×3 mini-month layout.
///
/// Each day in each month already carries `iso_date`, `jd`, `cell_x/y/w/h`
/// and (after annotation) `omer_day`, `sabbat`, `moon_phase`, `hebrew_holiday`
/// tags. Templates can compute their own per-month positioning with jinja2
/// math.
pub fn gregorian_year_overlay(year: i32) -> Value {
    let months: Vec<Value> = (1u32..=12).map(|m| gregorian_overlay(year, m)).collect();
    json!({
        "year":   year,
        "months": months,
    })
}

/// Build an Omer overlay for the period containing `jd`.
///
/// Includes both the full 49-day list (`days`) and a convenience
/// `today` field that is the matching `OmerDay` if `jd` falls inside the
/// period, or `null` otherwise. Templates can use the `today` field for
/// a single-cell tag, or iterate `days` for the full grid.
pub fn omer_overlay(jd: f64) -> Value {
    let period = omer_period(JulianDay::new(jd));
    let days = omer_days(period.hebrew_year);
    let day_objs: Vec<Value> = days
        .iter()
        .map(|d| {
            let rd = revjul(JulianDay::new(d.jd), Calendar::Gregorian);
            json!({
                "day":           d.day,
                "week":          d.week,
                "day_of_week":   d.day_of_week,
                "week_sefirah":  d.week_sefirah,
                "day_sefirah":   d.day_sefirah,
                "hebrew_text":   d.hebrew_text,
                "is_lag_baomer": d.is_lag_baomer,
                "jd":            d.jd,
                "iso_date":      format!("{:04}-{:02}-{:02}", rd.year, rd.month, rd.day),
            })
        })
        .collect();

    let today = celestial_core::omer_from_jd(JulianDay::new(jd)).map(|d| {
        let rd = revjul(JulianDay::new(d.jd), Calendar::Gregorian);
        json!({
            "day":           d.day,
            "week":          d.week,
            "day_of_week":   d.day_of_week,
            "week_sefirah":  d.week_sefirah,
            "day_sefirah":   d.day_sefirah,
            "hebrew_text":   d.hebrew_text,
            "is_lag_baomer": d.is_lag_baomer,
            "jd":            d.jd,
            "iso_date":      format!("{:04}-{:02}-{:02}", rd.year, rd.month, rd.day),
        })
    });

    json!({
        "hebrew_year": period.hebrew_year,
        "start_jd":    period.start_jd,
        "end_jd":      period.end_jd,
        "days":        day_objs,
        "today":       today,
    })
}

/// Annotate Gregorian days that fall inside the Omer period.
pub fn annotate_gregorian_with_omer(gregorian: &mut Value, omer: &Value) {
    let Some(omer_days) = omer["days"].as_array() else {
        return;
    };
    let mut by_iso: HashMap<String, &Value> = HashMap::new();
    for od in omer_days {
        if let Some(iso) = od["iso_date"].as_str() {
            by_iso.insert(iso.to_string(), od);
        }
    }
    let Some(days) = gregorian["days"].as_array_mut() else {
        return;
    };
    for day in days {
        let Some(iso) = day["iso_date"].as_str().map(str::to_string) else {
            continue;
        };
        if let Some(omer_day) = by_iso.get(&iso) {
            day["omer_day"] = omer_day["day"].clone();
            day["omer_week_sefirah"] = omer_day["week_sefirah"].clone();
            day["omer_day_sefirah"] = omer_day["day_sefirah"].clone();
            day["is_lag_baomer"] = omer_day["is_lag_baomer"].clone();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sabbats overlay
// ─────────────────────────────────────────────────────────────────────────────

/// Build a Wheel-of-the-Year overlay for the given Gregorian year.
pub fn sabbats_overlay(year: i32) -> Value {
    let sabbats = sabbats_for_year(year).unwrap_or_default();
    let entries: Vec<Value> = sabbats
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let rd = revjul(JulianDay::new(s.jd), Calendar::Gregorian);
            json!({
                "index":            i,
                // Pre-computed pixel y-offset for stacked list rendering at
                // 14-px line height (templates can multiply this themselves
                // if they want different spacing).
                "list_y":           14 * (i as i32),
                "name":             s.name,
                "alt_names":        s.kind.alt_names(),
                "lon":              s.kind.solar_longitude(),
                "jd":               s.jd,
                "iso_date":         format!("{:04}-{:02}-{:02}", rd.year, rd.month, rd.day),
                "is_quarter_day":   s.kind.is_quarter_day(),
                "is_cross_quarter": s.kind.is_cross_quarter(),
            })
        })
        .collect();
    json!({
        "year":    year,
        "sabbats": entries,
    })
}

/// Annotate Gregorian days that match a sabbat date.
pub fn annotate_gregorian_with_sabbats(gregorian: &mut Value, sabbats: &Value) {
    let Some(sabbats_list) = sabbats["sabbats"].as_array() else {
        return;
    };
    let mut by_iso: HashMap<String, &Value> = HashMap::new();
    for s in sabbats_list {
        if let Some(iso) = s["iso_date"].as_str() {
            by_iso.insert(iso.to_string(), s);
        }
    }
    let Some(days) = gregorian["days"].as_array_mut() else {
        return;
    };
    for day in days {
        let Some(iso) = day["iso_date"].as_str().map(str::to_string) else {
            continue;
        };
        if let Some(s) = by_iso.get(&iso) {
            day["sabbat_name"] = s["name"].clone();
            day["is_sabbat"] = json!(true);
            day["is_quarter_day"] = s["is_quarter_day"].clone();
            day["is_cross_quarter"] = s["is_cross_quarter"].clone();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Moon overlay
// ─────────────────────────────────────────────────────────────────────────────

/// Build a moon overlay: principal phase events for the month.
pub fn moon_overlay(year: i32, month: u32) -> Value {
    let events = moon_phases_for_month(year, month as u8).unwrap_or_default();
    let event_objs: Vec<Value> = events
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let rd = revjul(JulianDay::new(e.jd), Calendar::Gregorian);
            json!({
                "index":       i,
                "list_y":      14 * (i as i32),
                "phase":       phase_name(e.phase),
                "phase_short": phase_short_name(e.phase),
                "glyph":       phase_glyph(e.phase),
                "jd":          e.jd,
                "iso_date":    format!("{:04}-{:02}-{:02}", rd.year, rd.month, rd.day),
            })
        })
        .collect();

    json!({
        "year":         year,
        "month":        month,
        "phase_events": event_objs,
    })
}

/// Annotate Gregorian days with phase events + daily illumination.
pub fn annotate_gregorian_with_moon(gregorian: &mut Value, moon: &Value) {
    let Some(events) = moon["phase_events"].as_array() else {
        return;
    };
    let mut by_iso: HashMap<String, &Value> = HashMap::new();
    for e in events {
        if let Some(iso) = e["iso_date"].as_str() {
            by_iso.insert(iso.to_string(), e);
        }
    }
    let Some(days) = gregorian["days"].as_array_mut() else {
        return;
    };
    for day in days {
        let Some(iso) = day["iso_date"].as_str().map(str::to_string) else {
            continue;
        };
        if let Some(e) = by_iso.get(&iso) {
            day["moon_phase"] = e["phase"].clone();
            day["moon_phase_short"] = e["phase_short"].clone();
            day["moon_glyph"] = e["glyph"].clone();
        }
        let Some(jd) = day["jd"].as_f64() else {
            continue;
        };
        if let Ok(illum) = moon_illumination(JulianDay::new(jd)) {
            day["moon_illumination"] = json!(illum);
            day["moon_illumination_pct"] = json!((illum * 100.0).round() as i32);
        }
    }
}

fn phase_name(p: PrincipalPhase) -> &'static str {
    match p {
        PrincipalPhase::NewMoon => "New Moon",
        PrincipalPhase::FirstQuarter => "First Quarter",
        PrincipalPhase::FullMoon => "Full Moon",
        PrincipalPhase::LastQuarter => "Last Quarter",
    }
}
fn phase_short_name(p: PrincipalPhase) -> &'static str {
    match p {
        PrincipalPhase::NewMoon => "New",
        PrincipalPhase::FirstQuarter => "Q1",
        PrincipalPhase::FullMoon => "Full",
        PrincipalPhase::LastQuarter => "Q3",
    }
}
fn phase_glyph(p: PrincipalPhase) -> &'static str {
    match p {
        PrincipalPhase::NewMoon => "\u{1F311}",      // 🌑
        PrincipalPhase::FirstQuarter => "\u{1F313}", // 🌓
        PrincipalPhase::FullMoon => "\u{1F315}",     // 🌕
        PrincipalPhase::LastQuarter => "\u{1F317}",  // 🌗
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Hebrew overlay
// ─────────────────────────────────────────────────────────────────────────────

const HEBREW_MONTH_NAMES: [&str; 13] = [
    "Tishrei", "Cheshvan", "Kislev", "Tevet", "Shvat", "Adar", "Adar II", "Nisan", "Iyar", "Sivan",
    "Tammuz", "Av", "Elul",
];

/// Hebrew calendar overlay covering Gregorian JDs in `[jd_start, jd_end]`.
pub fn hebrew_overlay(jd_start: f64, jd_end: f64) -> Value {
    use celestial_core::{
        approx_hebrew_year, days_in_hebrew_year, hebrew_month_days, hebrew_month_start_jd,
        is_hebrew_leap_year, months_in_hebrew_year,
    };

    let h_start = approx_hebrew_year(JulianDay::new(jd_start));
    let h_end = approx_hebrew_year(JulianDay::new(jd_end));
    let mut years_covered = vec![h_start];
    if h_end != h_start {
        years_covered.push(h_end);
    }

    let years_json: Vec<Value> = years_covered
        .iter()
        .map(|&y| {
            json!({
                "year":    y,
                "is_leap": is_hebrew_leap_year(y),
                "months":  months_in_hebrew_year(y),
                "days":    days_in_hebrew_year(y),
            })
        })
        .collect();

    // Day-by-day mapping: Gregorian JD floor → (hebrew_year, hebrew_month, hebrew_day)
    let mut day_lookup = Vec::new();
    let mut jd_cur = jd_start.floor();
    while jd_cur <= jd_end {
        let h_year = approx_hebrew_year(JulianDay::new(jd_cur));
        let n_months = months_in_hebrew_year(h_year);
        let found = (1..=n_months).find_map(|m| {
            let m_start = hebrew_month_start_jd(h_year, m) as f64;
            let m_days = hebrew_month_days(h_year, m) as f64;
            if jd_cur >= m_start && jd_cur < m_start + m_days {
                Some((m, (jd_cur - m_start).floor() as i32 + 1))
            } else {
                None
            }
        });
        if let Some((m, d)) = found {
            let month_name = HEBREW_MONTH_NAMES
                .get((m as usize).saturating_sub(1))
                .copied()
                .unwrap_or("?");
            day_lookup.push(json!({
                "jd":                jd_cur,
                "hebrew_year":       h_year,
                "hebrew_month":      m,
                "hebrew_month_name": month_name,
                "hebrew_day":        d,
            }));
        }
        jd_cur += 1.0;
    }

    json!({
        "years": years_json,
        "days":  day_lookup,
    })
}

/// Annotate Gregorian days with Hebrew-calendar tags.
pub fn annotate_gregorian_with_hebrew(gregorian: &mut Value, hebrew: &Value) {
    let Some(h_days) = hebrew["days"].as_array() else {
        return;
    };
    let mut by_jd: HashMap<i64, &Value> = HashMap::new();
    for hd in h_days {
        if let Some(jd) = hd["jd"].as_f64() {
            by_jd.insert(jd.floor() as i64, hd);
        }
    }

    let Some(days) = gregorian["days"].as_array_mut() else {
        return;
    };
    for day in days {
        let Some(jd) = day["jd"].as_f64() else {
            continue;
        };
        let key = jd.floor() as i64;
        if let Some(hd) = by_jd.get(&key) {
            day["hebrew_year"] = hd["hebrew_year"].clone();
            day["hebrew_month"] = hd["hebrew_month"].clone();
            day["hebrew_month_name"] = hd["hebrew_month_name"].clone();
            day["hebrew_day"] = hd["hebrew_day"].clone();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default built-in renderer for `--chart-type calendar`
// ─────────────────────────────────────────────────────────────────────────────

/// Render one day cell into the calendar SVG buffer.
fn render_day_cell(s: &mut String, d: &Value, palette: &CalendarPalette) {
    use std::fmt::Write;

    let x = d["cell_x"].as_f64().unwrap_or(0.0);
    let y = d["cell_y"].as_f64().unwrap_or(0.0);
    let w = d["cell_w"].as_f64().unwrap_or(110.0);
    let h = d["cell_h"].as_f64().unwrap_or(90.0);
    let day_num = d["day"].as_u64().unwrap_or(0);
    let is_lag = d["is_lag_baomer"].as_bool().unwrap_or(false);
    let is_sabbat = d["is_sabbat"].as_bool().unwrap_or(false);

    let cell_fill = if is_lag {
        format!(r#"fill="{}" fill-opacity=".15""#, palette.lag)
    } else if is_sabbat {
        format!(r#"fill="{}" fill-opacity=".12""#, palette.sabbat)
    } else {
        "fill=\"none\"".to_string()
    };
    let _ = writeln!(
        s,
        r#"  <rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" {cell_fill} stroke="{ring}" stroke-width=".5"/>"#,
        ring = palette.ring
    );

    // Day number
    let lx = d["label_x"].as_f64().unwrap_or(x + 8.0);
    let ly = d["label_y"].as_f64().unwrap_or(y + 18.0);
    let _ = writeln!(
        s,
        r#"  <text x="{lx:.1}" y="{ly:.1}" font-size="13" font-weight="700" font-family="system-ui,sans-serif" fill="{txt}">{day_num}</text>"#,
        txt = palette.txt
    );

    // Moon glyph (top-right) when there's a phase event
    if let Some(glyph) = d["moon_glyph"].as_str() {
        let mx = x + w - 16.0;
        let my = y + 22.0;
        let _ = writeln!(
            s,
            r#"  <text x="{mx:.1}" y="{my:.1}" text-anchor="middle" font-size="14">{glyph}</text>"#
        );
        if let Some(short) = d["moon_phase_short"].as_str() {
            let _ = writeln!(
                s,
                r#"  <text x="{mx:.1}" y="{my2:.1}" text-anchor="middle" font-size="7" font-family="system-ui,sans-serif" fill="{moon}">{short}</text>"#,
                my2 = my + 9.0,
                moon = palette.moon
            );
        }
    }

    // Omer day badge (centre-bottom)
    if let Some(om) = d["omer_day"].as_u64() {
        let cx = x + w * 0.5;
        let cy = y + h - 22.0;
        let (badge, omer_color) = if is_lag {
            (format!("Day {om} ★"), &palette.lag)
        } else {
            (format!("Day {om}"), &palette.accent)
        };
        let _ = writeln!(
            s,
            r#"  <text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" font-size="9" font-weight="600" font-family="Georgia,serif" fill="{omer_color}">{badge}</text>"#
        );
        if let Some(week_sef) = d["omer_day_sefirah"].as_str() {
            let _ = writeln!(
                s,
                r#"  <text x="{cx:.1}" y="{cy2:.1}" text-anchor="middle" font-size="8" font-family="Georgia,serif" fill="{ring}">{week_sef}</text>"#,
                cy2 = cy + 11.0,
                ring = palette.ring
            );
        }
    }

    // Sabbat badge (centre)
    if let Some(name) = d["sabbat_name"].as_str() {
        let cx = x + w * 0.5;
        let cy = y + h * 0.5 + 4.0;
        let _ = writeln!(
            s,
            r#"  <text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" font-size="11" font-weight="600" font-family="Georgia,serif" fill="{sabbat}">{name}</text>"#,
            sabbat = palette.sabbat
        );
    }

    // Hebrew date (bottom)
    if let Some(hd) = d["hebrew_day"].as_u64() {
        let hm = d["hebrew_month_name"].as_str().unwrap_or("?");
        let cx = x + w * 0.5;
        let cy = y + h - 6.0;
        let _ = writeln!(
            s,
            r#"  <text x="{cx:.1}" y="{cy:.1}" text-anchor="middle" font-size="8" font-family="system-ui,sans-serif" fill="{ring}" opacity=".75">{hd} {hm}</text>"#,
            ring = palette.ring
        );
    }
}

/// Helper for [`render_default_calendar_svg`]: render the bottom legend.
fn render_legend(s: &mut String, ctx: &Value, palette: &CalendarPalette, gx: f64, vh: f64) {
    use std::fmt::Write;
    let mut legend_parts = Vec::new();
    if !ctx["omer"].is_null() {
        legend_parts.push(format!(
            r#"<rect x="0" y="0" width="14" height="10" fill="{}" fill-opacity=".15" stroke="{}" stroke-width=".6"/>
            <text x="20" y="9">Lag Ba'Omer (Day 33)</text>"#,
            palette.lag, palette.lag
        ));
    }
    if !ctx["sabbats"].is_null() {
        legend_parts.push(format!(
            r#"<rect x="0" y="0" width="14" height="10" fill="{}" fill-opacity=".12" stroke="{}" stroke-width=".6"/>
            <text x="20" y="9">Wheel-of-Year sabbat</text>"#,
            palette.sabbat, palette.sabbat
        ));
    }
    if !ctx["moon"].is_null() {
        legend_parts
            .push(r#"<text x="0" y="9">🌑 🌓 🌕 🌗 principal moon phases</text>"#.to_string());
    }
    let _ = writeln!(
        s,
        r#"  <g font-family="system-ui,sans-serif" font-size="10" fill="{}">"#,
        palette.txt
    );
    let mut legend_x = gx;
    let legend_y = vh - 30.0;
    for part in &legend_parts {
        let _ = writeln!(
            s,
            r#"    <g transform="translate({legend_x:.0} {legend_y:.0})">{part}</g>"#
        );
        legend_x += 230.0;
    }
    let _ = writeln!(s, r#"  </g>"#);
}

/// Bundles the colour palette extracted from `ctx["vars"]` so we don't need to
/// re-look-up each colour in every helper.
struct CalendarPalette {
    bg: String,
    txt: String,
    ring: String,
    accent: String,
    lag: String,
    sabbat: String,
    moon: String,
    title: String,
}

impl CalendarPalette {
    fn from_ctx(ctx: &Value) -> Self {
        let v = &ctx["vars"];
        Self {
            bg: super::svg_common::esc_var(v, "bg_color", "#ffffff"),
            txt: super::svg_common::esc_var(v, "text_color", "#222"),
            ring: super::svg_common::esc_var(v, "ring_color", "#888"),
            accent: super::svg_common::esc_var(v, "accent_color", "#5c4a8a"),
            lag: super::svg_common::esc_var(v, "lag_color", "#c87f32"),
            sabbat: super::svg_common::esc_var(v, "sabbat_color", "#3d6b35"),
            moon: super::svg_common::esc_var(v, "moon_color", "#3a4a6a"),
            title: super::svg_common::esc_var(v, "title", "Calendar"),
        }
    }
}

pub fn render_default_calendar_svg(ctx: &ChartContext) -> String {
    use std::fmt::Write;

    let palette = CalendarPalette::from_ctx(ctx);
    let g = &ctx["gregorian"];
    let vw = g["viewbox_w"].as_f64().unwrap_or(900.0);
    let vh = g["viewbox_h"].as_f64().unwrap_or(800.0);
    let gx = g["grid_x0"].as_f64().unwrap_or(60.0);
    let gy = g["grid_y0"].as_f64().unwrap_or(158.0);
    let gw = g["grid_w"].as_f64().unwrap_or(770.0);
    let gh = g["grid_h"].as_f64().unwrap_or(540.0);

    let mut s = String::with_capacity(8192);
    s.push_str(&super::svg_common::svg_doc_open(vw, vh, &palette.bg));
    let _ = write!(
        s,
        r#"
  <text x="{cx:.0}" y="42" text-anchor="middle" font-size="22" font-weight="600"
        font-family="Georgia,serif" fill="{txt}">{title}</text>
"#,
        cx = vw / 2.0,
        txt = palette.txt,
        title = palette.title,
    );

    // Active overlays subtitle
    if let Some(cals) = ctx["calendars"].as_array() {
        let names: Vec<&str> = cals.iter().filter_map(|v| v.as_str()).collect();
        let _ = writeln!(
            s,
            r#"  <text x="{cx:.0}" y="68" text-anchor="middle" font-size="11" font-family="system-ui,sans-serif" fill="{ring}" opacity=".75">overlays: {}</text>"#,
            names.join(" · "),
            cx = vw / 2.0,
            ring = palette.ring
        );
    }

    // Column headers (Sun..Sat)
    if let Some(headers) = g["col_headers"].as_array() {
        for h in headers {
            let name = h["name"].as_str().unwrap_or("?");
            let x = h["x"].as_f64().unwrap_or(0.0);
            let y = h["y"].as_f64().unwrap_or(0.0);
            let _ = writeln!(
                s,
                r#"  <text x="{x:.1}" y="{y:.1}" text-anchor="middle" dominant-baseline="central" font-size="12" font-weight="600" font-family="Georgia,serif" fill="{accent}">{name}</text>"#,
                accent = palette.accent
            );
        }
    }

    // Outer grid rect
    let _ = writeln!(
        s,
        r#"  <rect x="{gx:.1}" y="{gy:.1}" width="{gw:.1}" height="{gh:.1}" fill="none" stroke="{ring}" stroke-width="1.0"/>"#,
        ring = palette.ring
    );

    // Day cells — delegated to render_day_cell
    if let Some(days) = g["days"].as_array() {
        for d in days {
            render_day_cell(&mut s, d, &palette);
        }
    }

    // Legend — delegated to render_legend
    render_legend(&mut s, ctx, &palette, gx, vh);

    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_overlay_may_2024() {
        let g = gregorian_overlay(2024, 5);
        assert_eq!(g["days_in_month"].as_u64(), Some(31));
        assert_eq!(g["days"].as_array().unwrap().len(), 31);
        // May 2024 starts on Wednesday
        assert_eq!(g["first_weekday"].as_u64(), Some(3));
    }

    #[test]
    fn gregorian_overlay_feb_leap_year() {
        let g_2024 = gregorian_overlay(2024, 2);
        assert_eq!(g_2024["days_in_month"].as_u64(), Some(29), "2024 is leap");
        let g_2023 = gregorian_overlay(2023, 2);
        assert_eq!(
            g_2023["days_in_month"].as_u64(),
            Some(28),
            "2023 is non-leap"
        );
        let g_1900 = gregorian_overlay(1900, 2);
        assert_eq!(
            g_1900["days_in_month"].as_u64(),
            Some(28),
            "1900 not leap (divisible by 100 but not 400)"
        );
        let g_2000 = gregorian_overlay(2000, 2);
        assert_eq!(
            g_2000["days_in_month"].as_u64(),
            Some(29),
            "2000 is leap (divisible by 400)"
        );
    }

    #[test]
    fn gregorian_with_omer_annotations() {
        let mut g = gregorian_overlay(2024, 5);
        let jd_may1 = celestial_core::julday(2024, 5, 1, 12.0, Calendar::Gregorian);
        let omer = omer_overlay(jd_may1);
        annotate_gregorian_with_omer(&mut g, &omer);

        let day25 = g["days"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["day"].as_u64() == Some(25))
            .unwrap();
        assert_eq!(
            day25["omer_day"].as_u64(),
            Some(33),
            "May 25, 2024 should be Omer Day 33 (Lag Ba'Omer)"
        );
        assert_eq!(day25["is_lag_baomer"].as_bool(), Some(true));
    }

    #[test]
    fn gregorian_with_moon_annotations() {
        let mut g = gregorian_overlay(2024, 5);
        let moon = moon_overlay(2024, 5);
        annotate_gregorian_with_moon(&mut g, &moon);

        for d in g["days"].as_array().unwrap() {
            let illum = d["moon_illumination"]
                .as_f64()
                .expect("illumination missing");
            assert!((0.0..=1.0).contains(&illum));
        }
        let tagged_count = g["days"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| !d["moon_phase"].is_null())
            .count();
        assert!(
            tagged_count >= 1,
            "expected at least one phase event in May 2024"
        );
    }

    #[test]
    fn sabbats_overlay_2024_has_8() {
        let s = sabbats_overlay(2024);
        assert_eq!(s["sabbats"].as_array().unwrap().len(), 8);
    }

    #[test]
    fn hebrew_overlay_for_may_2024() {
        let jd_start = celestial_core::julday(2024, 5, 1, 0.0, Calendar::Gregorian);
        let jd_end = celestial_core::julday(2024, 5, 31, 23.0, Calendar::Gregorian);
        let h = hebrew_overlay(jd_start, jd_end);
        let years = h["years"].as_array().unwrap();
        assert!(years.iter().any(|y| y["year"].as_i64() == Some(5784)));
        assert!(h["days"].as_array().unwrap().len() >= 30);
    }
}
