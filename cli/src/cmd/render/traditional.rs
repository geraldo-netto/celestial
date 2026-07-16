use celestial_core::body::{Body, CalcFlags};
use celestial_core::{
    calc_ut, decan_ruler, diff_deg_signed, egyptian_terms_ruler, next_principal_phase,
    planet_house_number, sign_exaltation, sign_ruler, triplicity_rulers, HouseResult, JulianDay,
    Longitude, PrincipalPhase, RiseTransOptions, CALC_RISE, CALC_SET,
};
use serde::Serialize;

const FIXED_STAR_ORB: f64 = 1.0;
const HOUSE_SCORES: [i8; 12] = [12, 6, 3, 9, 7, 1, 10, 5, 4, 11, 8, 2];
const TRADITIONAL: [Body; 7] = [
    Body::SUN,
    Body::MOON,
    Body::MERCURY,
    Body::VENUS,
    Body::MARS,
    Body::JUPITER,
    Body::SATURN,
];
const WEEKDAY_LORDS: [Body; 7] = [
    Body::SUN,
    Body::MOON,
    Body::MARS,
    Body::MERCURY,
    Body::JUPITER,
    Body::VENUS,
    Body::SATURN,
];
const CHALDEAN_ORDER: [Body; 7] = [
    Body::SATURN,
    Body::JUPITER,
    Body::MARS,
    Body::SUN,
    Body::VENUS,
    Body::MERCURY,
    Body::MOON,
];

#[derive(Clone)]
pub(super) struct BodyPosition {
    pub(super) body: Body,
    pub(super) longitude: f64,
}

#[derive(Clone)]
pub(super) struct ChartPoint {
    pub(super) name: String,
    pub(super) glyph: String,
    pub(super) longitude: f64,
    pub(super) kind: PointKind,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum PointKind {
    Planet,
    Angle,
}

#[derive(Clone)]
pub(super) struct FixedStarPosition {
    pub(super) name: String,
    pub(super) constellation: String,
    pub(super) longitude: f64,
}

#[derive(Serialize)]
pub(super) struct FixedStarConjunction {
    pub(super) star: String,
    pub(super) constellation: String,
    pub(super) star_lon: f64,
    pub(super) point: String,
    pub(super) point_glyph: String,
    pub(super) point_lon: f64,
    pub(super) point_kind: PointKind,
    pub(super) orb: f64,
}

#[derive(Serialize)]
pub(super) struct AlmutenScore {
    pub(super) body: i32,
    pub(super) essential_score: i16,
    pub(super) house: u8,
    pub(super) house_score: i16,
    pub(super) day_bonus: i16,
    pub(super) hour_bonus: i16,
    pub(super) total_score: i16,
}

#[derive(Serialize)]
pub(super) struct PrenatalSyzygy {
    pub(super) name: &'static str,
    pub(super) jd: f64,
    pub(super) longitude: f64,
}

#[derive(Serialize)]
pub(super) struct AlmutenFiguris {
    pub(super) winner: i32,
    pub(super) essential_score: i16,
    pub(super) total_score: i16,
    pub(super) day_lord: Option<i32>,
    pub(super) hour_lord: Option<i32>,
    pub(super) prenatal_syzygy: PrenatalSyzygy,
    pub(super) scores: Vec<AlmutenScore>,
}

pub(super) fn fixed_star_conjunctions(
    planets: &[ChartPoint],
    fixed_stars: &[FixedStarPosition],
    angles: &[ChartPoint],
) -> Vec<FixedStarConjunction> {
    let mut hits = Vec::new();
    for star in fixed_stars {
        append_star_hits(&mut hits, star, planets);
        append_star_hits(&mut hits, star, angles);
    }
    hits.sort_by(|a, b| a.orb.total_cmp(&b.orb));
    hits
}

fn append_star_hits(
    hits: &mut Vec<FixedStarConjunction>,
    star: &FixedStarPosition,
    points: &[ChartPoint],
) {
    for point in points {
        if let Some(hit) = fixed_star_hit(star, point) {
            hits.push(hit);
        }
    }
}

fn fixed_star_hit(star: &FixedStarPosition, point: &ChartPoint) -> Option<FixedStarConjunction> {
    let orb = diff_deg_signed(star.longitude, point.longitude).abs();
    if orb > FIXED_STAR_ORB {
        return None;
    }
    Some(FixedStarConjunction {
        star: star.name.clone(),
        constellation: star.constellation.clone(),
        star_lon: star.longitude,
        point: point.name.clone(),
        point_glyph: point.glyph.clone(),
        point_lon: point.longitude,
        point_kind: point.kind,
        orb,
    })
}

pub(super) fn almuten_figuris(
    jd: f64,
    lat: f64,
    lon: f64,
    houses: &HouseResult,
    planets: &[BodyPosition],
    fortune_lon: f64,
) -> Option<AlmutenFiguris> {
    let Some((syzygy_jd, syzygy_lon, syzygy_name)) = prenatal_syzygy(jd) else {
        return None;
    };
    let Some(sun_lon) = planet_lon(planets, Body::SUN) else {
        return None;
    };
    let Some(moon_lon) = planet_lon(planets, Body::MOON) else {
        return None;
    };
    let points = [sun_lon, moon_lon, houses.ascmc[0], fortune_lon, syzygy_lon];
    let lords = planetary_lords(jd, lat, lon);
    let mut scores = score_planets(planets, houses, &points, lords);
    sort_scores(&mut scores);
    let winner = scores.first()?;
    Some(AlmutenFiguris {
        winner: winner.body,
        essential_score: winner.essential_score,
        total_score: winner.total_score,
        day_lord: lords.map(|v| v.0.as_raw()),
        hour_lord: lords.map(|v| v.1.as_raw()),
        prenatal_syzygy: PrenatalSyzygy {
            name: syzygy_name,
            jd: syzygy_jd,
            longitude: syzygy_lon,
        },
        scores,
    })
}

fn score_planets(
    planets: &[BodyPosition],
    houses: &HouseResult,
    points: &[f64; 5],
    lords: Option<(Body, Body)>,
) -> Vec<AlmutenScore> {
    TRADITIONAL
        .iter()
        .filter_map(|&body| score_planet(body, planets, houses, points, lords))
        .collect()
}

fn score_planet(
    body: Body,
    planets: &[BodyPosition],
    houses: &HouseResult,
    points: &[f64; 5],
    lords: Option<(Body, Body)>,
) -> Option<AlmutenScore> {
    let lon = planet_lon(planets, body)?;
    let essential: i16 = points
        .iter()
        .map(|&point| i16::from(dignity_claim(body, point)))
        .sum();
    let house = planet_house_number(lon, &houses.cusps);
    let house_score = i16::from(HOUSE_SCORES[usize::from(house - 1)]);
    let day_bonus = bonus_for(lords.map(|v| v.0), body, 7);
    let hour_bonus = bonus_for(lords.map(|v| v.1), body, 6);
    Some(AlmutenScore {
        body: body.as_raw(),
        essential_score: essential,
        house,
        house_score,
        day_bonus,
        hour_bonus,
        total_score: essential + house_score + day_bonus + hour_bonus,
    })
}

fn dignity_claim(body: Body, lon: f64) -> i8 {
    let sign = ((lon.rem_euclid(360.0) / 30.0) as u8) % 12;
    let mut score = 0;
    if sign_ruler(sign) == body {
        score += 5;
    }
    if sign_exaltation(body) == sign as i8 {
        score += 4;
    }
    let (day_ruler, night_ruler, participating_ruler) = triplicity_rulers(Longitude::new(lon));
    if [day_ruler, night_ruler, participating_ruler].contains(&body) {
        score += 3;
    }
    if egyptian_terms_ruler(Longitude::new(lon)) == body {
        score += 2;
    }
    if decan_ruler(Longitude::new(lon)) == body {
        score += 1;
    }
    score
}

fn bonus_for(lord: Option<Body>, body: Body, points: i16) -> i16 {
    if lord == Some(body) {
        points
    } else {
        0
    }
}

fn sort_scores(scores: &mut [AlmutenScore]) {
    scores.sort_by(|a, b| {
        b.total_score
            .cmp(&a.total_score)
            .then_with(|| b.essential_score.cmp(&a.essential_score))
            .then_with(|| a.body.cmp(&b.body))
    });
}

fn prenatal_syzygy(jd: f64) -> Option<(f64, f64, &'static str)> {
    let new_moon = previous_phase(jd, PrincipalPhase::NewMoon)?;
    let full_moon = previous_phase(jd, PrincipalPhase::FullMoon)?;
    let (phase_jd, body, name) = if full_moon > new_moon {
        (full_moon, Body::MOON, "Full Moon")
    } else {
        (new_moon, Body::SUN, "New Moon")
    };
    let pos = calc_ut(JulianDay::new(phase_jd), body, CalcFlags::BUILTIN).ok()?;
    Some((phase_jd, pos.lon, name))
}

fn previous_phase(jd: f64, phase: PrincipalPhase) -> Option<f64> {
    let mut cursor = jd - 40.0;
    let mut previous = None;
    for _ in 0..3 {
        let event = next_principal_phase(JulianDay::new(cursor), phase).ok()?;
        if event.jd >= jd {
            break;
        }
        previous = Some(event.jd);
        cursor = event.jd + 0.01;
    }
    previous
}

fn planetary_lords(jd: f64, lat: f64, lon: f64) -> Option<(Body, Body)> {
    let rises = solar_events(jd, lat, lon, CALC_RISE);
    let sets = solar_events(jd, lat, lon, CALC_SET);
    let past_rise = event_before(&rises, jd)?;
    let future_rise = event_after(&rises, jd)?;
    let past_set = event_before(&sets, jd)?;
    let future_set = event_after(&sets, jd)?;
    let day_lord = weekday_lord(past_rise, lon);
    let hour_index = planetary_hour_index(jd, past_rise, future_rise, past_set, future_set);
    Some((day_lord, advance_chaldean(day_lord, hour_index)))
}

fn solar_events(jd: f64, lat: f64, lon: f64, event: i32) -> Vec<f64> {
    let mut events = [-1.5, -0.5, 0.5, 1.5]
        .iter()
        .filter_map(|offset| {
            RiseTransOptions::new(JulianDay::new(jd + offset), Body::SUN, [lon, lat, 0.0])
                .event(event)
                .search()
                .ok()
                .map(|result| result.tret)
        })
        .collect::<Vec<_>>();
    events.sort_by(f64::total_cmp);
    events.dedup_by(|a, b| (*a - *b).abs() < 1.0e-6);
    events
}

fn planetary_hour_index(
    jd: f64,
    past_rise: f64,
    future_rise: f64,
    past_set: f64,
    future_set: f64,
) -> usize {
    let (start, end, offset) = if past_rise > past_set {
        (past_rise, future_set, 0)
    } else {
        (past_set, future_rise, 12)
    };
    let hour = ((jd - start) / ((end - start) / 12.0)).floor();
    offset + hour.clamp(0.0, 11.0) as usize
}

fn weekday_lord(sunrise_jd: f64, lon: f64) -> Body {
    let local_solar_jd = sunrise_jd + lon / 360.0;
    let weekday = ((local_solar_jd + 1.5).floor() as i64).rem_euclid(7) as usize;
    WEEKDAY_LORDS[weekday]
}

fn advance_chaldean(lord: Body, steps: usize) -> Body {
    let start = CHALDEAN_ORDER
        .iter()
        .position(|&body| body == lord)
        .unwrap_or(0);
    CHALDEAN_ORDER[(start + steps) % CHALDEAN_ORDER.len()]
}

fn event_before(events: &[f64], jd: f64) -> Option<f64> {
    events
        .iter()
        .copied()
        .filter(|event| *event <= jd)
        .max_by(f64::total_cmp)
}

fn event_after(events: &[f64], jd: f64) -> Option<f64> {
    events
        .iter()
        .copied()
        .filter(|event| *event > jd)
        .min_by(f64::total_cmp)
}

fn planet_lon(planets: &[BodyPosition], body: Body) -> Option<f64> {
    planets
        .iter()
        .find(|planet| planet.body == body)
        .map(|planet| planet.longitude)
}

#[cfg(test)]
fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use celestial_core::{julday, Calendar};
    use serde_json::Value;

    use super::*;
    use crate::cmd::render::context::build_context;

    fn chart_context(year: i32, month: i32) -> Value {
        let jd = julday(year, month, 15, 6.0, Calendar::Gregorian);
        build_context(jd, 0.0, 0.0, "test", 'P', BTreeMap::new()).unwrap()
    }

    fn star(name: &str, lon: f64) -> FixedStarPosition {
        FixedStarPosition {
            name: name.to_string(),
            constellation: "Test".to_string(),
            longitude: lon,
        }
    }

    fn point(name: &str, lon: f64, kind: PointKind) -> ChartPoint {
        ChartPoint {
            name: name.to_string(),
            glyph: String::new(),
            longitude: lon,
            kind,
        }
    }

    #[test]
    fn dignity_claim_sums_all_five_levels() {
        assert_eq!(dignity_claim(Body::MERCURY, 151.0), 11);
    }

    #[test]
    fn thursday_last_night_hour_is_sun() {
        let day_lord = weekday_lord(2_446_579.902_8, 0.0);
        assert_eq!(day_lord, Body::JUPITER);
        assert_eq!(advance_chaldean(day_lord, 23), Body::SUN);
    }

    #[test]
    fn planetary_day_uses_local_solar_date() {
        let jd = julday(2024, 1, 15, 3.0, Calendar::Gregorian);
        let lords = planetary_lords(jd, 35.6762, 139.6503).unwrap();
        assert_eq!(lords.0, Body::MOON);
    }

    #[test]
    fn varied_charts_can_select_every_traditional_planet() {
        let cases = [
            (1900, 1, "Mars"),
            (1900, 4, "Sun"),
            (1900, 7, "Jupiter"),
            (1900, 10, "Venus"),
            (1901, 1, "Moon"),
            (1902, 10, "Mercury"),
            (1903, 1, "Saturn"),
        ];
        for (year, month, expected) in cases {
            let context = chart_context(year, month);
            assert_eq!(context["almuten_figuris"]["name"], expected);
        }
    }

    #[test]
    fn chart_star_conjunctions_can_be_empty_or_select_other_stars() {
        let j2000 = julday(2000, 1, 1, 12.0, Calendar::Gregorian);
        let paris = build_context(j2000, 48.8566, 2.3522, "test", 'P', BTreeMap::new()).unwrap();
        assert!(paris["fixed_star_conjunctions"]
            .as_array()
            .unwrap()
            .is_empty());

        let historical = chart_context(1902, 7);
        let hits = historical["fixed_star_conjunctions"].as_array().unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0]["star"], "Pollux");
        assert_eq!(hits[1]["star"], "Capella");
    }

    #[test]
    fn fixed_star_orb_includes_boundary_wraps_and_sorts() {
        let stars = [
            star("Wrap", 359.8),
            star("Boundary", 100.0),
            star("Outside", 200.0),
        ];
        let planets = [point("Planet", 0.2, PointKind::Planet)];
        let angles = [
            point("Angle", 101.0, PointKind::Angle),
            point("Too far", 201.000_1, PointKind::Angle),
        ];
        let hits = fixed_star_conjunctions(&planets, &stars, &angles);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].star, "Wrap");
        assert_eq!(round4(hits[0].orb), 0.4);
        assert_eq!(hits[1].star, "Boundary");
        assert_eq!(round4(hits[1].orb), 1.0);
    }

    #[test]
    fn prenatal_syzygy_selects_new_and_full_moons() {
        let new_moon_chart = julday(1902, 7, 15, 6.0, Calendar::Gregorian);
        let full_moon_chart = julday(1986, 5, 30, 9.0, Calendar::Gregorian);
        assert_eq!(prenatal_syzygy(new_moon_chart).unwrap().2, "New Moon");
        assert_eq!(prenatal_syzygy(full_moon_chart).unwrap().2, "Full Moon");
    }

    #[test]
    fn planetary_day_changes_at_sunrise() {
        let jd = julday(2024, 1, 15, 6.0, Calendar::Gregorian);
        let rises = solar_events(jd, 0.0, 0.0, CALC_RISE);
        let sunrise = event_after(&rises, jd - 0.5).unwrap();
        let before = planetary_lords(sunrise - 1.0 / 1_440.0, 0.0, 0.0).unwrap();
        let after = planetary_lords(sunrise + 1.0 / 1_440.0, 0.0, 0.0).unwrap();
        assert_eq!(before.0, Body::SUN);
        assert_eq!(after, (Body::MOON, Body::MOON));
    }
}
