/**
 * Type declarations for the celestial-js napi-rs generated module.
 *
 * Generated from bindings/js/src/lib.rs
 */

// ─── Constants ────────────────────────────────────────────────────────────────

export declare const GREG_CAL: number;
export declare const JUL_CAL: number;
export declare const SUN: number;
export declare const MOON: number;
export declare const MERCURY: number;
export declare const VENUS: number;
export declare const MARS: number;
export declare const JUPITER: number;
export declare const SATURN: number;
export declare const URANUS: number;
export declare const NEPTUNE: number;
export declare const PLUTO: number;
export declare const MEAN_NODE: number;
export declare const TRUE_NODE: number;
export declare const CHIRON: number;
export declare const EARTH: number;
export declare const FLG_BUILTIN: number;
export declare const FLG_JPL: number;
export declare const FLG_MOSHIER: number;
export declare const FLG_SPEED: number;
export declare const FLG_EQUATORIAL: number;
export declare const FLG_TOPOCTR: number;
export declare const FLG_SIDEREAL: number;
export declare const FLG_HELCTR: number;
export declare const FLG_XYZ: number;
export declare const FLG_RADIANS: number;
export declare const FLG_NONUT: number;
export declare const SIDM_FAGAN_BRADLEY: number;
export declare const SIDM_LAHIRI: number;
export declare const SIDM_RAMAN: number;
export declare const SIDM_USER: number;
export declare const ECL_TOTAL: number;
export declare const ECL_ANNULAR: number;
export declare const ECL_PARTIAL: number;
export declare const ECL_PENUMBRAL: number;
export declare const CALC_RISE: number;
export declare const CALC_SET: number;
export declare const TRUE_TO_APP: number;
export declare const APP_TO_TRUE: number;
export declare const SPLIT_DEG_ROUND_SEC: number;
export declare const SPLIT_DEG_ZODIACAL: number;

// ─── Interfaces ───────────────────────────────────────────────────────────────

export interface PlanetPos {
  /** Ecliptic longitude (degrees) */
  lon: number;
  /** Ecliptic latitude (degrees) */
  lat: number;
  /** Distance in AU */
  dist: number;
  /** Speed in longitude (deg/day) */
  speedLon: number;
  /** Speed in latitude (deg/day) */
  speedLat: number;
  /** Speed in distance (AU/day) */
  speedDist: number;
  /** Return flags from the library */
  retFlags: number;
}

export interface StarPos {
  /** 6-element position/speed array */
  xx: number[];
  /** Canonical star name (e.g. `"Sirius,alCMa"`) */
  star_name: string;
  /** Return flags */
  retFlags: number;
}

export interface HouseResult {
  /** Cusp positions (12 or 36 values depending on house system) */
  cusps: number[];
  /** Additional points: ASC, MC, ARMC, Vertex, EquatorialASC, CoASC1, CoASC2, PolarASC */
  ascmc: number[];
}

export interface HouseResultEx2 {
  cusps: number[];
  ascmc: number[];
  cuspSpeeds: number[];
  ascmcSpeeds: number[];
}

export interface EclipseResult {
  retFlags: number;
  /** Time array (10 values) */
  tret: number[];
}

export interface EclipseResultAttr {
  retFlags: number;
  tret: number[];
  /** Attribute array (20 values) */
  attr: number[];
}

export interface EclipseWhere {
  retFlags: number;
  geopos: number[];
  attr: number[];
}

export interface EclipseHow {
  retFlags: number;
  attr: number[];
}

export interface RiseTrans {
  retFlags: number;
  /** Julian day of the event */
  tret: number;
}

export interface AzAlt {
  azimuth: number;
  true_alt: number;
  apparent_alt: number;
}

export interface CalDate {
  year: number;
  month: number;
  day: number;
  hour: number;
}

export interface UtcDate {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
}

export interface JdPair {
  et: number;
  ut1: number;
}

export interface RefracExt {
  result: number;
  dret: number[];
}

export interface MoonCrossNodeResult {
  jd_cross: number;
  xlon: number;
}

// ─── Functions ────────────────────────────────────────────────────────────────

/**
 * Set the path to Swiss Ephemeris data files.
 */
export declare function setEphePath(path: string): void;

/**
 * Set the JPL ephemeris file.
 */
export declare function setJplFile(fname: string): void;

/**
 * Set sidereal mode. `sidMode` is one of the `SIDM_*` constants.
 */
export declare function setSidMode(sidMode: number, param0?: number, ayanT0?: number): void;

/**
 * Set topocentric observer position.
 */
export declare function setTopo(geolon: number, geolat: number, geoalt: number): void;

/**
 * Release all library resources.
 */
export declare function close(): void;

/**
 * Calculate planetary positions (Ephemeris Time).
 */
export declare function calc(jdet: number, planet: number, flags: number): PlanetPos;

/**
 * Calculate planetary positions (Universal Time).
 */
export declare function calcUt(jdut: number, planet: number, flags: number): PlanetPos;

/** Nutation in longitude and obliquity at a Julian Ephemeris Day (TT).
 *  Returns [dpsi_degrees, deps_degrees]. Multiply by 3600 for arcseconds.
 *  Uses IAU 2000B luni-solar series (~1 mas accuracy).
 */
export declare function nutation(jde: number): [number, number];
/** Mean obliquity of the ecliptic in degrees (IAU 2006 formula). */
export declare function meanObliquity(jde: number): number;
/** True (apparent) obliquity in degrees (includes nutation in obliquity). */
export declare function trueObliquity(jde: number): number;

/** Compute positions for multiple bodies in parallel (TT/ET input).
 *  Equivalent to calling calc() for each body, but runs concurrently.
 *  Returns results in the same order as the input planets array.
 */
export declare function calcMany(tjdet: number, planets: number[], flags: number): PlanetPos[];

/** Compute positions for multiple bodies in parallel (UT input).
 *  Equivalent to calling calcUt() for each body, but runs concurrently.
 */
export declare function calcUtMany(tjdut: number, planets: number[], flags: number): PlanetPos[];

/**
 * Planetocentric positions (ET).
 */
export declare function calcPctr(
  jdet: number,
  planet: number,
  center: number,
  flags: number,
): PlanetPos;

/**
 * Fixed star position (ET).
 */
export declare function fixstar(star: string, jdet: number, flags: number): StarPos;

/**
 * Fixed star position (UT).
 */
export declare function fixstarUt(star: string, jdut: number, flags: number): StarPos;

/**
 * Fixed star magnitude.
 */
export declare function fixstarMag(star: string): number;

/**
 * Calculate house cusps (UT). `hsys` is the ASCII code of the house letter.
 */
export declare function houses(jdut: number, lat: number, lon: number, hsys: number): HouseResult;

/**
 * Extended house cusps with optional flags.
 */
export declare function housesEx(
  jdut: number,
  lat: number,
  lon: number,
  hsys: number,
  flags?: number,
): HouseResult;

/**
 * Houses with cusp speeds.
 */
export declare function housesEx2(
  jdut: number,
  lat: number,
  lon: number,
  hsys: number,
  flags?: number,
): HouseResultEx2;

/**
 * Name of a house system.
 */
export declare function houseName(hsys: number): string;

/**
 * Next solar eclipse globally (UT).
 */
export declare function solEclipseWhenGlob(
  jdStart: number,
  flags: number,
  eclType?: number,
  backwards?: boolean,
): EclipseResult;

/**
 * Next solar eclipse from a location (UT).
 */
export declare function solEclipseWhenLoc(
  jdStart: number,
  geopos: number[],
  flags: number,
  backwards?: boolean,
): EclipseResultAttr;

/**
 * Solar eclipse attributes at a location.
 */
export declare function solEclipseHow(jdUt: number, geopos: number[], flags: number): EclipseHow;

/**
 * Where a solar eclipse is central.
 */
export declare function solEclipseWhere(jd: number, flags: number): EclipseWhere;

/**
 * Next lunar eclipse globally (UT).
 */
export declare function lunEclipseWhen(
  jdStart: number,
  flags: number,
  eclType?: number,
  backwards?: boolean,
): EclipseResult;

/**
 * Lunar eclipse attributes.
 */
export declare function lunEclipseHow(jdUt: number, flags: number, geopos?: number[]): EclipseHow;

/**
 * Rise, set, or transit calculation.
 */
export declare function riseTrans(
  jdut: number,
  planet: number,
  epheFlags: number,
  rsmi: number,
  geopos: number[],
  atpress?: number,
  attemp?: number,
): RiseTrans;

/**
 * Convert a calendar date to a Julian day number.
 */
export declare function julday(
  year: number,
  onth: number,
  day: number,
  hour?: number,
  gregflag?: number,
): number;

/**
 * Reverse a Julian day to a calendar date.
 */
export declare function revjul(jd: number, gregflag?: number): CalDate;

/**
 * Day of week (0 = Monday, …, 6 = Sunday).
 */
export declare function dayOfWeek(jd: number): number;

/**
 * Delta-T (TT − UT).
 */
export declare function deltat(jd: number): number;

/**
 * Sidereal time.
 */
export declare function sidtime(jdUt: number): number;

/**
 * Convert UTC to Julian day numbers.
 */
export declare function utcToJd(date: UtcDate, gregflag?: number): JdPair;

/**
 * Ayanamsa value (ET).
 */
export declare function getAyanamsa(jdEt: number): number;

/**
 * Ayanamsa value (UT).
 */
export declare function getAyanamsaUt(jdUt: number): number;

/**
 * Name of a sidereal mode.
 */
export declare function getAyanamsaName(sidMode: number): string;

/**
 * Normalise degrees to 0…360.
 */
export declare function normDeg(x: number): number;

/**
 * Degree midpoint (360° wrap aware).
 */
export declare function degMidp(x1: number, x0: number): number;

/**
 * Signed difference between two degree values.
 */
export declare function diffDegSigned(p1: number, p2: number): number;

/**
 * Normalise centiseconds.
 */
export declare function normCs(p: number): number;

/**
 * Split a degree value into components.
 * Returns `[deg, min, sec, secFraction, sign]`.
 */
export declare function splitDeg(ddeg: number, roundflag: number): number[];

/**
 * Coordinate transform (ecliptic ↔ equatorial).
 */
export declare function coordTransform(coord: number[], eps: number): number[];

/**
 * Azimuth and altitude from ecliptic/equatorial coordinates.
 */
export declare function azalt(
  jdut: number,
  calc_flag: number,
  geopos: number[],
  atpress: number,
  attemp: number,
  xin: number[],
): AzAlt;

/**
 * Atmospheric refraction.
 */
export declare function refrac(
  inalt: number,
  atpress: number,
  attemp: number,
  calc_flag: number,
): number;

/**
 * Extended atmospheric refraction.
 */
export declare function refracExtended(
  inalt: number,
  geoalt: number,
  atpress: number,
  attemp: number,
  lapse_rate: number,
  calc_flag: number,
): RefracExt;

/**
 * Library version string.
 */
export declare function version(): string;

/**
 * Name of a planet / body.
 */
export declare function getPlanetName(planet: number): string;

/**
 * Name of a house system.
 */
export declare function houseNameStr(hsys: number): string;

/**
 * Duration between two JDs → [days, hours, minutes, seconds].
 */
export declare function jdDuration(jdStart: number, jdEnd: number): number[];

/**
 * Format Julian day as ISO string "YYYY-MM-DD HH:MM:SS UTC".
 */
export declare function jdToIsoString(jd: number, gregflag: number): string;

/**
 * Moon node crossing (ET).
 */
export declare function mooncrossNode(jdEt: number, flags: number): MoonCrossNodeResult;

/**
 * Moon node crossing (UT).
 */
export declare function mooncrossNodeUt(jdUt: number, flags: number): MoonCrossNodeResult;

/** Normalise degrees to [0, 360). */

/** Name of zodiac sign 0-11, or null if out of range. */
export declare function signName(sign: number): string | null;

/** Current Julian Day (UTC). */
export declare function jdnow(): number;

/** Parse a coordinate string (e.g. "51N30", "-12.5") to decimal degrees. */
export declare function parseCoord(s: string): number | null;

/** Format decimal degrees as a coordinate string. */
export declare function formatCoord(coord: number, isLatitude: boolean): string | null;

/** Rasi (sign) number 0-11 for ecliptic longitude. */
export declare function longToRasi(lon: number): number;

/** Navamsa number 0-11 for ecliptic longitude. */
export declare function longToNavamsa(lon: number): number;

/** [nakshatra 0-26, pada 1-4] for ecliptic longitude. */
export declare function longToNakshatra(lon: number): number[];

/** Name of nakshatra 0-26, or null if out of range. */
export declare function nakshatraName(nak: number): string | null;

/** Raman house cusps — returns 12 house cusp longitudes. */
export declare function ramanHouses(asc: number, mc: number, sandhi: boolean): number[];

/** Residential strength of a graha in its bhava (0.0–1.0). */
export declare function residentialStrength(graha: number, bm: number[]): number;

/** Uccha bala (exaltation strength) for a graha. */
export declare function ochchabala(graha: number, sputha: number): number;

/** Naisargika (natural) relationship between two grahas: +1 friend, -1 enemy. */
export declare function naisargikaRelation(gr1: number, gr2: number): number;

/** Saturn 4-Stars index — returns [saturn_lon, aldebaran, regulus, antares, fomalhaut, index]. */
export declare function saturnFourStars(jd: number, flags: number): number[];
export function ayanamsa(tjdEt: number): number;
export function ayanamsaUt(tjdUt: number): number;
export function ayanamsaName(sidMode: number): string;
export function planetName(planet: number): string;

// ─── Chart functions ──────────────────────────────────────────────────────────

export interface IngressResult {
  /** Julian Day of the ingress. */
  jd: number;
  /** Sign number 0–11 (0=Aries … 11=Pisces). */
  sign: number;
}

export interface Stations {
  /** Julian Day of the retrograde station (planet turns retrograde). */
  retrograde: number;
  /** Julian Day of the direct station (planet turns direct). */
  direct: number;
}

/** Next time a planet enters a new zodiac sign (or the previous ingress if backward=true). */
export declare function signIngressUt(
  planet: number,
  jd: number,
  flags: number,
  backward?: boolean,
): IngressResult;

/** Next retrograde and direct station dates for a planet. */
export declare function retrogradeStationUt(planet: number, jd: number, flags: number): Stations;

/** Arabic Part: `(asc + body2 − body1) mod 360`. */
export declare function arabicPart(asc: number, body2: number, body1: number): number;

/** Next time a planet reaches a fixed ecliptic degree. */
export declare function transitToDegree(
  planet: number,
  targetLon: number,
  jd: number,
  flags: number,
  backward?: boolean,
): number;

/** Next time a planet transits the natal Midheaven (MC). */
export declare function mcTransitUt(
  planet: number,
  jdNatal: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  flags: number,
  backward?: boolean,
): number;

/** Julian Day of the next time `planet` transits the IC of a natal chart. */
export declare function icTransitUt(
  planet: number,
  jdNatal: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  flags: number,
  backward?: boolean,
): number;

/** Julian Day of the next time `planet` transits the Ascendant of a natal chart. */
export declare function ascTransitUt(
  planet: number,
  jdNatal: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  flags: number,
  backward?: boolean,
): number;

/** Julian Day of the next time `planet` transits the Descendant of a natal chart. */
export declare function dscTransitUt(
  planet: number,
  jdNatal: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  flags: number,
  backward?: boolean,
): number;

/** Next time `planet` makes `aspect` to house cusp `cusp` (1–12).
 * Returns `[jd]` or `null` if no event found. */
export declare function nextAspectCusp(
  body: number,
  aspect: number,
  cusp: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  backward: boolean,
  flags: number,
): number[] | null;

/** Same as `nextAspectCusp` but uses the secondary cusp algorithm. */
export declare function nextAspectCusp2(
  body: number,
  aspect: number,
  cusp: number,
  jdStart: number,
  lat: number,
  lon: number,
  hsys: number,
  backward: boolean,
  flags: number,
): number[] | null;

/** Julian Day of the solar return for a given year. */
export declare function solarReturnJd(jdNatal: number, returnYear: number, flags: number): number;

/** Julian Day of the next lunar return after jdStart. */
export declare function lunarReturnJd(jdNatal: number, jdStart: number, flags: number): number;

/** Midpoint between two ecliptic longitudes (shorter arc). */
export declare function midpoint(lon1: number, lon2: number): number;

/** Traditional planetary ruler of a sign (0–11). */
export declare function signRuler(sign: number): number;

/** Modern planetary ruler (Uranus/Neptune/Pluto for Aquarius/Pisces/Scorpio). */
export declare function signRulerModern(sign: number): number;

/** Name of a zodiac sign 0–11 (e.g. "Aries"). */
export declare function zodiacSignName(sign: number): string;

/** `[sign 0–11, degrees_in_sign]` for an ecliptic longitude. */
export declare function lonToSign(lon: number): number[];

/** Local Apparent Solar Time (sundial time) in decimal hours. */
export declare function localApparentSolarTime(jdUt: number, geolonDeg: number): number;

/** Annual profection — returns `[house 1–12, degree]` for a given age. */
export declare function annualProfection(cusps: number[], age: number): number[];

/**
 * Vimshottari dasha periods from birth.
 * Returns an array of `[planet, start_jd, end_jd, years]` for each period.
 */
export declare function vimshottariDasha(
  jdBirth: number,
  moonLonSidereal: number,
  yearsAhead?: number,
): number[][];

// ─── Sefirat HaOmer ──────────────────────────────────────────────────────────

export interface OmerDay {
  /** Day number in the Omer (1–49). */
  day: number;
  /** Week number (1–7). */
  week: number;
  /** Day within the week (1–7). */
  dayOfWeek: number;
  /** Sefirah of the week (e.g. "Chesed"). */
  weekSefirah: string;
  /** Sefirah of the day (e.g. "Tiferet"). */
  daySefirah: string;
  /** Full Hebrew declaration text. */
  hebrewText: string;
  /** True if this is Lag Ba'Omer (day 33). */
  isLagBaomer: boolean;
  /** Julian day of the start of this Omer day (at nightfall). */
  jd: number;
}

export interface OmerPeriod {
  /** Julian day of the first Omer day (16 Nisan, nightfall). */
  startJd: number;
  /** Julian day of the 49th Omer day (5 Sivan, nightfall). */
  endJd: number;
  /** Hebrew year. */
  hebrewYear: number;
}

/** Return the Omer day for a given Julian day, or null if outside the Omer period. */
export declare function omerFromJd(jd: number): OmerDay | null;

/** Julian day of a specific Omer day (1–49) in the given Hebrew year. */
export declare function omerDayJd(hebrewYear: number, day: number): number | null;

/** Julian day of the first Omer day (16 Nisan nightfall) for the given Hebrew year. */
export declare function omerStartJd(hebrewYear: number): number;

/** All 49 Omer days for the given Hebrew year. */
export declare function omerDays(hebrewYear: number): OmerDay[];

/** The Omer period (start/end JD, Hebrew year) that contains or follows the given JD. */
export declare function omerPeriod(jd: number): OmerPeriod;

/** Full Hebrew declaration and Sefirot annotation for the given Omer day (1–49). */
export declare function omerDeclaration(day: number): string;

// ─── Jewish Holidays ─────────────────────────────────────────────────────────

export interface JewishHoliday {
  name: string;
  hebrewName: string;
  hebrewMonth: number;
  hebrewDay: number;
  jd: number;
  jdEnd: number;
  days: number;
  /** "MajorFestival" | "RabbinicFestival" | "Fast" | "Minor" | "SpecialShabbat" */
  category: string;
}

export interface HebrewDate {
  year: number;
  month: number;
  day: number;
}

/** All major Jewish holidays for the given Hebrew year, sorted chronologically. */
export declare function jewishHolidays(hebrewYear: number): JewishHoliday[];
/** JD of a named Jewish holiday (e.g. "Passover (Pesach)") in the Hebrew year. */
export declare function jewishHolidayJd(hebrewYear: number, name: string): number | null;
/** Hebrew year number for a given Julian day. */
export declare function hebrewYearFromJd(jd: number): number;
/** Convert a Julian day to a Hebrew date. */
export declare function jdToHebrewDate(jd: number): HebrewDate;

// ─── Easter & Christian Calendar ─────────────────────────────────────────────

export interface CalendarDate {
  year: number;
  month: number;
  day: number;
}

export interface ChristianFeast {
  name: string;
  /** Days offset from Easter Sunday (negative = before Easter). */
  easterOffset: number;
  jd: number;
  year: number;
  month: number;
  day: number;
}

/** Western (Gregorian) Easter date. */
export declare function easterGregorian(year: number): CalendarDate;
/** Eastern Orthodox Easter date (converted to Gregorian). */
export declare function easterOrthodox(year: number): CalendarDate;
/** JD of Western Easter. */
export declare function easterJd(year: number): number;
/** JD of Orthodox Easter. */
export declare function easterOrthodoxJd(year: number): number;
/** All Western moveable Christian feasts (Ash Wednesday → Corpus Christi). */
export declare function christianFeasts(year: number): ChristianFeast[];
/** Fixed (non-moveable) Christian feasts (Christmas, Epiphany, etc.). */
export declare function christianFixedFeasts(year: number): ChristianFeast[];

// ─── Islamic Calendar ─────────────────────────────────────────────────────────

export interface HijriDate {
  year: number;
  month: number;
  day: number;
}

export interface IslamicObservance {
  name: string;
  arabicName: string;
  hijriMonth: number;
  hijriDay: number;
  jd: number;
  days: number;
}

export interface HijriYears {
  year1: number;
  year2: number;
}

/** Convert a Julian day to a Hijri (Islamic) date. */
export declare function hijriFromJd(jd: number): HijriDate;
/** Convert a Hijri date to a Julian day. */
export declare function hijriToJd(year: number, month: number, day: number): number;
/** English name of a Hijri month (1–12). */
export declare function hijriMonthName(month: number): string;
/** All major Islamic observances for the given Hijri year. */
export declare function islamicObservances(hijriYear: number): IslamicObservance[];
/** Hijri years that overlap with the given Gregorian year (usually two). */
export declare function gregorianToHijriYears(gregorianYear: number): HijriYears;

// ─── Hindu Panchānga ──────────────────────────────────────────────────────────

export interface Panchanga {
  /** Lunar day (1–30). */
  tithi: number;
  tithi_name: string;
  /** "Shukla" (waxing) | "Krishna" (waning). */
  paksha: string;
  /** Weekday (0=Sunday … 6=Saturday). */
  vara: number;
  vara_name: string;
  /** Lunar mansion (0–26). */
  nakshatra: number;
  nakshatra_name: string;
  /** Pada within Nakshatra (1–4). */
  nakshatra_pada: number;
  /** Yoga number (0–26). */
  yoga: number;
  yoga_name: string;
  /** Half-tithi (1–60). */
  karana: number;
  karana_name: string;
  sun_lon: number;
  moon_lon: number;
  elongation: number;
}

export interface HinduFestival {
  name: string;
  description: string;
  jd: number;
}

/** Complete Panchānga (five limbs) for the given Julian day. */
export declare function panchanga(jd: number): Panchanga;
/** Major Hindu festivals in the given Gregorian year. */
export declare function hinduFestivals(gregorianYear: number): HinduFestival[];

// ─── Buddhist Observances ─────────────────────────────────────────────────────

export interface Uposatha {
  /** "NewMoon" | "FirstQuarter" | "FullMoon" | "LastQuarter" */
  phase: string;
  jd: number;
  elongation: number;
}

/** Julian day of Vesak (Buddha Day) for the given Gregorian year. */
export declare function vesakJd(year: number): number;
/** All four-phase Uposatha (lunar observance) days for the given Gregorian year. */
export declare function uposathaDays(year: number): Uposatha[];

// ─── Nowruz & Bahá'í Calendar ─────────────────────────────────────────────────

export interface BahaiDate {
  /** Bahá'í Era year (1 BE = 1844 CE). */
  year: number;
  /** Month 1–19, or 0 for Ayyám-i-Há. */
  month: number;
  day: number;
  monthName: string;
}

export interface BahaiHolyDay {
  name: string;
  description: string;
  bahaiMonth: number;
  bahaiDay: number;
  jd: number;
}

/** JD of Nowruz (Persian New Year / vernal equinox) for the given Gregorian year. */
export declare function nowruzJd(year: number): number;
/** Convert a Gregorian year to the Solar Hijri (Persian) year. */
export declare function gregorianToSolarHijri(year: number): number;
/** JD of Naw-Rúz (Bahá'í New Year) for the given Bahá'í year. */
export declare function nawRuzJd(bahaiYear: number): number;
/** Convert a Julian day to a Bahá'í date. */
export declare function jdToBahai(jd: number): BahaiDate;
/** All Bahá'í holy days for the given Bahá'í year. */
export declare function bahaiHolyDays(bahaiYear: number): BahaiHolyDay[];

// ─── Moon Phases ──────────────────────────────────────────────────────────────

export interface PhaseEvent {
  /** "New Moon" | "First Quarter" | "Full Moon" | "Last Quarter" */
  phase: string;
  /** Julian day of the exact phase moment. */
  jd: number;
  /** Moon–Sun elongation at this moment (degrees). */
  elongation: number;
}

export interface MoonPhaseInfo {
  /** One of the eight named phases (e.g. "Waxing Gibbous"). */
  phaseName: string;
  /** Moon–Sun elongation (0°–360°). */
  elongation: number;
  /** Fraction of disk illuminated (0.0–1.0). */
  illumination: number;
  /** Name of the preceding principal phase. */
  prevPhaseName: string;
  /** JD of the preceding principal phase. */
  prevPhaseJd: number;
  /** Name of the upcoming principal phase. */
  nextPhaseName: string;
  /** JD of the upcoming principal phase. */
  nextPhaseJd: number;
  /** Days since the preceding principal phase. */
  ageDays: number;
}

/**
 * Current Moon phase name at the given Julian day.
 * Returns one of: "New Moon", "Waxing Crescent", "First Quarter",
 * "Waxing Gibbous", "Full Moon", "Waning Gibbous", "Last Quarter", "Waning Crescent".
 */
export declare function moonPhase(jd: number): string;
/** Fraction of the Moon's disk that is illuminated (0.0 = new, 1.0 = full). */
export declare function moonIllumination(jd: number): number;
/** Moon–Sun elongation in degrees (0°–360°). */
export declare function moonElongation(jd: number): number;
/** Moon phase angle in degrees (0° = new moon, 180° = full moon). */
export declare function moonPhaseAngle(jd: number): number;
/** Julian day of the next new moon at or after jdFrom. */
export declare function nextNewMoon(jdFrom: number): number;
/** Julian day of the next first-quarter moon at or after jdFrom. */
export declare function nextFirstQuarter(jdFrom: number): number;
/** Julian day of the next full moon at or after jdFrom. */
export declare function nextFullMoonPhase(jdFrom: number): number;
/** Julian day of the next last-quarter moon at or after jdFrom. */
export declare function nextLastQuarter(jdFrom: number): number;
/** All principal phase events in the given calendar month (4–5 events). */
export declare function moonPhasesForMonth(year: number, month: number): PhaseEvent[];
/** Detailed Moon phase info including illumination, age, and adjacent phases. */
export declare function moonPhaseInfo(jd: number): MoonPhaseInfo;

// ── Phase 5–8 functions ──────────────────────────────────────────────────────
export declare function egyptian_terms_ruler(lon: number): number;
export declare function decan_ruler(lon: number): number;
export declare function full_dignity(body: number, lon: number, is_day: boolean): [string, number];
export declare function almuten(lon: number, is_day: boolean): [number, number];
export declare function firdaria(jd_birth: number, is_day: boolean, span_years: number): number[][];
export declare function four_pillars(jd_ut: number, hour_ut: number, sun_lon: number): string[][];
export declare function solar_term_position(sun_lon: number): [number, number, number, number];
export declare function tonalpohualli(jd: number): [number, number, string, string];
export declare function xiuhpohualli(jd: number): [string, string, string, string];
export declare function tzolkin(jd: number): [number, number, string, string];
export declare function haab(jd: number): [string, string, string];
export declare function calendar_round(jd: number): [string, string, string, string];
export declare function medicine_wheel_totem(sun_lon: number): [string, string, string, string];
export declare function egyptian_decan(lon: number): [string, string, string];
export declare function is_day_chart(sun_lon: number, cusps: number[]): boolean;
export declare function mean_sidtime(jd: number): number;
export declare function triplicity_rulers(lon: number): [number, number, number];

// ── Legacy / compatibility aliases ───────────────────────────────────────────

/** Normalise degrees to [0°, 360°). Legacy alias for normDeg(). */
export declare function degnorm(d: number): number;

/** Signed degree difference in (-180°, +180°]. Legacy alias for difDegSigned(). */
export declare function difdeg2n(p1: number, p2: number): number;

/** Ayanamsa at Julian Day ET. Legacy alias for ayanamsa(). */
export declare function getAyanamsa(jdEt: number): number;

/** Name of a sidereal mode by SIDM_* constant. Legacy alias for ayanamsaName(). */
export declare function getAyanamsaName(sidMode: number): string;

/** Name of the next sabbat after jdFrom. Legacy alias for nextSabbat(). */
export declare function nextSabbatName(jdFrom: number): string;

/** JD of the next full moon after jdStart. Legacy alias for nextFullMoonAfter(). */
export declare function nextFullMoon(jdStart: number): number;

/** JD when the Sun next crosses ecliptic longitude x2cross (UT). */
export declare function solcrossUt(x2cross: number, jdUt: number, flags: number): number;
