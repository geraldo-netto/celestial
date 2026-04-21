//! Canonical data types shared by the astronomy computation layer and the
//! public API layer.  Defined once here; both layers import from this module.

// ─── Planetary position ───────────────────────────────────────────────────────

/// Geocentric apparent position and velocity of a body.
///
/// Coordinates are ecliptic (degrees) unless [`FLG_EQUATORIAL`] is set, in
/// which case they are equatorial (RA / Dec).  Distance is in AU.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanetPos {
    /// Ecliptic longitude or right ascension (degrees).
    pub lon: f64,
    /// Ecliptic latitude or declination (degrees).
    pub lat: f64,
    /// Distance from Earth (AU).
    pub dist: f64,
    /// Daily rate of change of `lon` (degrees/day).
    pub speed_lon: f64,
    /// Daily rate of change of `lat` (degrees/day).
    pub speed_lat: f64,
    /// Daily rate of change of `dist` (AU/day).
    pub speed_dist: f64,
    /// Flags echoed back by the calculation (bitfield).
    pub ret_flags: i32,
}

// ─── Fixed star ───────────────────────────────────────────────────────────────

/// Position (and optionally speed) of a fixed star.
#[derive(Debug, Clone, PartialEq)]
pub struct FixStarPos {
    /// Six-element position/speed vector `[lon, lat, dist, speed_lon, speed_lat, speed_dist]`.
    pub xx: [f64; 6],
    /// Canonical name of the matched star (`"Name,BayerDesignation"`).
    pub star_name: String,
    /// Flags echoed back by the calculation.
    pub ret_flags: i32,
}

// ─── Nodes and apsides ────────────────────────────────────────────────────────

/// Nodes and apsides for a planet's orbit.
///
/// Each array is a six-element `[lon, lat, dist, speed_lon, speed_lat, speed_dist]`.
#[derive(Debug, Clone, PartialEq)]
pub struct NodAps {
    /// Ascending node.
    pub nasc: [f64; 6],
    /// Descending node.
    pub ndsc: [f64; 6],
    /// Perihelion.
    pub peri: [f64; 6],
    /// Aphelion.
    pub aphe: [f64; 6],
    /// Flags echoed back.
    pub ret_flags: i32,
}

// ─── Orbital elements / distances ─────────────────────────────────────────────

/// Full set of orbital elements for a planet.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitalElements {
    /// Semi-major axis (AU).
    pub semi_major_axis: f64,
    /// Numerical eccentricity (0 = circular, <1 = elliptical).
    pub eccentricity: f64,
    /// Orbital inclination (degrees).
    pub inclination: f64,
    /// Longitude of ascending node (degrees).
    pub ascending_node: f64,
    /// Argument of perihelion (degrees).
    pub arg_perihelion: f64,
    /// Mean anomaly at epoch (degrees).
    pub mean_anomaly: f64,
    /// Reference epoch (Julian day).
    pub epoch: f64,
    /// Mean daily motion (degrees/day).
    pub mean_daily_motion: f64,
    /// Additional implementation-specific values.
    pub extra: Vec<f64>,
}

/// Maximum, minimum, and current true distance of a planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitalDistances {
    /// Aphelion distance (AU).
    pub dmax: f64,
    /// Perihelion distance (AU).
    pub dmin: f64,
    /// Current true distance (AU).
    pub dtrue: f64,
}
