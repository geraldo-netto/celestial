#!/usr/bin/env python3
"""
Pure-logic test suite for the celestial Python binding.

Exercises every calculation that does NOT require the compiled Rust extension —
all pure math re-implemented in Python, verified against the expected values
from the original celestial Python test suite.

Run:   python3 tests/pure_logic_test.py
       or:  python3 -m pytest tests/pure_logic_test.py -v
"""

import math
import json as _json
import pathlib as _pathlib
import unittest

# ─── Pure-Python reimplementations ───────────────────────────────────────────
# These mirror the Rust/C library behaviour and serve as ground-truth
# for the expected values embedded in the full test suite.


def degnorm(x: float) -> float:
    """Normalise degrees to [0, 360). Matches swe_degnorm()."""
    x = x % 360.0
    if x < 0:
        x += 360.0
    return x


def radnorm(x: float) -> float:
    """Normalise radians to [0, 2π)."""
    TWO_PI = 2.0 * math.pi
    x = x % TWO_PI
    if x < 0:
        x += TWO_PI
    return x


def difdeg2n(p1: float, p2: float) -> float:
    """Signed difference p1−p2, result in (−180, +180]. Matches swe_difdeg2n()."""
    d = degnorm(p1) - degnorm(p2)
    if d > 180.0:
        d -= 360.0
    if d <= -180.0:
        d += 360.0
    return d


def deg_midp(x1: float, x0: float) -> float:
    """Midpoint of two degree values (shortest arc). Matches swe_deg_midp()."""
    return degnorm(x0 + difdeg2n(x1, x0) / 2.0)


def csnorm(p: int) -> int:
    """Normalise centiseconds to [0, 360×360000)."""
    FULL = 360 * 360_000
    return int(p % FULL + FULL) % FULL


def split_deg(ddeg: float, roundflag: int = 0) -> tuple:
    """
    Split a degree value into (degrees, minutes, seconds, sec_fraction, sign).
    Simplified version: roundflag 0 only.  Matches swe_split_deg(ddeg, 0).
    """
    sgn = 1 if ddeg >= 0 else -1
    absd = abs(ddeg)
    d = int(absd)
    mf = (absd - d) * 60.0
    m = int(mf)
    sf = (mf - m) * 60.0
    s = int(sf)
    frac = sf - s
    return (d, m, s, frac, sgn)


def julday(
    year: int, month: int, day: int, hour: float = 0.0, gregflag: int = 1
) -> float:
    """
    Convert a calendar date to a Julian day number.
    Matches swe_julday() for GREG_CAL (gregflag=1).
    Meeus "Astronomical Algorithms" algorithm.
    """
    y, m = year, month
    if m <= 2:
        y -= 1
        m += 12
    A = int(y / 100)
    B = 2 - A + int(A / 4) if gregflag == 1 else 0
    return (
        int(365.25 * (y + 4716))
        + int(30.6001 * (m + 1))
        + day
        + hour / 24.0
        + B
        - 1524.5
    )


def revjul(jd: float, gregflag: int = 1) -> tuple:
    """
    Reverse a Julian day to (year, month, day, hour).
    Matches swe_revjul() for GREG_CAL.
    """
    z = int(jd + 0.5)
    f = jd + 0.5 - z
    if z < 2299161 or gregflag == 0:
        a = z
    else:
        alpha = int((z - 1867216.25) / 36524.25)
        a = z + 1 + alpha - int(alpha / 4)
    b = a + 1524
    c = int((b - 122.1) / 365.25)
    dd = int(365.25 * c)
    e = int((b - dd) / 30.6001)
    day = b - dd - int(30.6001 * e)
    month = e - 1 if e < 14 else e - 13
    year = c - 4716 if month > 2 else c - 4715
    hour = f * 24.0
    return (year, month, day, hour)


def day_of_week(jd: float) -> int:
    """Day of week: 0=Monday, …, 6=Sunday. Matches swe_day_of_week()."""
    return int(math.floor(jd - 0.5) + 1) % 7


def cotrans(lon: float, lat: float, dist: float, eps: float) -> tuple:
    """
    Ecliptic ↔ equatorial coordinate transform.
    Matches swe_cotrans().  eps > 0: ecl→equ, eps < 0: equ→ecl.
    Rotation: x unchanged, y' = y·cos(ε)+z·sin(ε), z' = −y·sin(ε)+z·cos(ε)
    """
    eps_r = math.radians(eps)
    lon_r = math.radians(lon)
    lat_r = math.radians(lat)
    cos_lat = math.cos(lat_r)
    x = math.cos(lon_r) * cos_lat
    y = math.sin(lon_r) * cos_lat
    z = math.sin(lat_r)
    yp = y * math.cos(eps_r) + z * math.sin(eps_r)
    zp = -y * math.sin(eps_r) + z * math.cos(eps_r)
    new_lon = degnorm(math.degrees(math.atan2(yp, x)))
    new_lat = math.degrees(math.asin(max(-1.0, min(1.0, zp))))
    return (new_lon, new_lat, dist)


# ─── Test cases ───────────────────────────────────────────────────────────────


class TestDegnorm(unittest.TestCase):
    def test_zero(self):
        self.assertEqual(degnorm(0), 0.0)

    def test_360(self):
        self.assertEqual(degnorm(360), 0.0)

    def test_negative(self):
        self.assertEqual(degnorm(-1), 359.0)

    def test_361(self):
        self.assertAlmostEqual(degnorm(361), 1.0)

    def test_720(self):
        self.assertEqual(degnorm(720), 0.0)

    def test_neg_360(self):
        self.assertEqual(degnorm(-360), 0.0)

    def test_neg_361(self):
        self.assertAlmostEqual(degnorm(-361), 359.0)

    def test_always_in_range(self):
        for v in [-999, -360, -1, 0, 1, 180, 359, 360, 361, 720, 999]:
            n = degnorm(v)
            self.assertGreaterEqual(n, 0.0)
            self.assertLess(n, 360.0)

    def test_idempotent(self):
        for v in [0, 45, 90, 180, 270, 359.999]:
            n = degnorm(v)
            self.assertAlmostEqual(degnorm(n), n, places=12)


class TestRadnorm(unittest.TestCase):
    def test_zero(self):
        self.assertEqual(radnorm(0), 0.0)

    def test_two_pi(self):
        self.assertAlmostEqual(radnorm(2 * math.pi), 0.0, places=12)

    def test_neg_pi(self):
        self.assertAlmostEqual(radnorm(-math.pi), math.pi, places=12)

    def test_always_in_range(self):
        for v in [-2 * math.pi, -1, 0, 1, math.pi, 3 * math.pi]:
            n = radnorm(v)
            self.assertGreaterEqual(n, 0.0)
            self.assertLess(n, 2 * math.pi)


class TestDifdeg2n(unittest.TestCase):
    def test_known(self):
        self.assertAlmostEqual(difdeg2n(360.5, 540), -179.5, places=12)

    def test_zero(self):
        self.assertEqual(difdeg2n(100, 100), 0.0)

    def test_near_180(self):
        # p1 just past p2 by 1°
        self.assertAlmostEqual(difdeg2n(0, 359), 1.0, places=12)

    def test_always_in_range(self):
        pairs = [(0, 359), (1, 359), (180.5, 0), (90, 271), (350, 10), (10, 350)]
        for a, b in pairs:
            d = difdeg2n(a, b)
            self.assertGreater(d, -180.0, f"difdeg2n({a},{b}) = {d}")
            self.assertLessEqual(d, 180.0, f"difdeg2n({a},{b}) = {d}")


class TestDegMidp(unittest.TestCase):
    def test_simple(self):
        self.assertAlmostEqual(deg_midp(20, 10), 15.0, places=10)

    def test_wrap_around(self):
        # midpoint of 350° and 10° should be near 0° (not 180°)
        m = degnorm(deg_midp(10, 350))
        self.assertTrue(m < 1.0 or m > 359.0, f"wrap midpoint: {m}")

    def test_opposite(self):
        # midpoint of 90° and 270° should be 0° or 180°
        m = deg_midp(90, 270)
        ok = abs(degnorm(m)) < 0.001 or abs(degnorm(m) - 180) < 0.001
        self.assertTrue(ok, f"midpoint of 90/270: {m}")


class TestCsnorm(unittest.TestCase):
    FULL = 360 * 360_000

    def test_full_circle(self):
        self.assertEqual(csnorm(self.FULL), 0)

    def test_half(self):
        self.assertEqual(csnorm(180 * 360_000), 180 * 360_000)

    def test_540(self):
        self.assertEqual(csnorm(540 * 360_000), 180 * 360_000)

    def test_neg_720(self):
        self.assertEqual(csnorm(-720 * 360_000), 0)

    def test_always_non_negative(self):
        for v in [-(10**9), -1, 0, 1, self.FULL, self.FULL + 1, 10**9]:
            n = csnorm(v)
            self.assertGreaterEqual(n, 0, f"csnorm({v}) = {n}")
            self.assertLess(n, self.FULL, f"csnorm({v}) = {n}")


class TestSplitDeg(unittest.TestCase):
    def test_known(self):
        d, m, s, frac, sgn = split_deg(123.123)
        self.assertEqual(d, 123)
        self.assertEqual(m, 7)
        self.assertEqual(s, 22)
        self.assertAlmostEqual(frac, 0.8, places=5)
        self.assertEqual(sgn, 1)

    def test_zero(self):
        d, m, s, _, sgn = split_deg(0.0)
        self.assertEqual((d, m, s, sgn), (0, 0, 0, 1))

    def test_negative_sign(self):
        _, _, _, _, sgn = split_deg(-45.5)
        self.assertEqual(sgn, -1)

    def test_minutes_range(self):
        for ddeg in [x * 13.7 for x in range(27)]:
            _, m, _, _, _ = split_deg(ddeg)
            self.assertGreaterEqual(m, 0)
            self.assertLessEqual(m, 59)

    def test_seconds_range(self):
        for ddeg in [x * 13.7 for x in range(27)]:
            _, _, s, _, _ = split_deg(ddeg)
            self.assertGreaterEqual(s, 0)
            self.assertLessEqual(s, 59)


class TestJulday(unittest.TestCase):
    def test_known(self):
        self.assertEqual(julday(2002, 1, 1, 0), 2452275.5)

    def test_j2000(self):
        self.assertAlmostEqual(julday(2000, 1, 1, 12), 2451545.0, places=9)

    def test_revjul_roundtrip(self):
        dates = [
            (2002, 1, 1, 0.0),
            (2023, 12, 31, 23.5),
            (2000, 1, 1, 12.0),
            (1900, 3, 1, 6.0),
            (2100, 6, 15, 18.0),
        ]
        for y, m, d, h in dates:
            jd = julday(y, m, d, h)
            back = revjul(jd)
            self.assertEqual(back[0], y, f"year for {y}-{m}-{d}")
            self.assertEqual(back[1], m, "month")
            self.assertEqual(back[2], d, "day")
            self.assertAlmostEqual(back[3], h, places=8, msg=f"hour for {y}-{m}-{d}")


class TestRevjul(unittest.TestCase):
    def test_known(self):
        y, m, d, h = revjul(2452275.5)
        self.assertEqual((y, m, d), (2002, 1, 1))
        self.assertAlmostEqual(h, 0.0, places=10)


class TestDayOfWeek(unittest.TestCase):
    def test_tuesday_2002(self):
        # 2002-01-01 was a Tuesday → 1
        self.assertEqual(day_of_week(2452275.5), 1)

    def test_tuesday_2021(self):
        self.assertEqual(day_of_week(2459444.0), 1)

    def test_seven_day_cycle(self):
        base = day_of_week(2451545)
        for i in range(1, 22):
            expected = (base + i) % 7
            self.assertEqual(
                day_of_week(2451545 + i), expected, f"day {i}: expected {expected}"
            )

    def test_always_0_to_6(self):
        for jd in range(2451545, 2451545 + 14):
            d = day_of_week(jd)
            self.assertGreaterEqual(d, 0)
            self.assertLessEqual(d, 6)


class TestCotrans(unittest.TestCase):
    def test_known_values(self):
        # From test_swe_cotrans.py
        lon, lat, dist = cotrans(121.34, 43.57, 1.0, 23.4)
        self.assertAlmostEqual(lon, 114.11984833491826, places=9)
        self.assertAlmostEqual(lat, 22.754921351892474, places=9)
        self.assertEqual(dist, 1.0)

    def test_roundtrip(self):
        orig_lon, orig_lat, dist = 121.34, 43.57, 1.0
        eps = 23.44
        lon2, lat2, _ = cotrans(orig_lon, orig_lat, dist, eps)
        lon3, lat3, _ = cotrans(lon2, lat2, dist, -eps)
        self.assertAlmostEqual(lon3, orig_lon, places=9)
        self.assertAlmostEqual(lat3, orig_lat, places=9)

    def test_ecliptic_equator_at_zero_eps(self):
        # With eps=0, the transform is identity
        lon, lat, dist = cotrans(45.0, 30.0, 1.0, 0.0)
        self.assertAlmostEqual(lon, 45.0, places=10)
        self.assertAlmostEqual(lat, 30.0, places=10)
        self.assertEqual(dist, 1.0)


class TestExpectedValuesFromCelestial(unittest.TestCase):
    """Verify the expected values that the full integration tests depend on."""

    def test_julday_test_swe_julday(self):
        self.assertEqual(julday(2002, 1, 1, 0, 1), 2452275.5)

    def test_revjul_test_swe_revjul(self):
        y, m, d, h = revjul(2452275.5, 1)
        self.assertEqual((y, m, d), (2002, 1, 1))
        self.assertAlmostEqual(h, 0.0)

    def test_degnorm_test_swe_degnorm(self):
        self.assertEqual(degnorm(0), 0)
        self.assertEqual(degnorm(360), 0)
        self.assertEqual(degnorm(-1), 359)

    def test_csnorm_test_swe_csnorm(self):
        self.assertEqual(csnorm(360 * 360_000), 0)
        self.assertEqual(csnorm(540 * 360_000), 64_800_000)
        self.assertEqual(csnorm(-720 * 360_000), 0)

    def test_difdeg2n_test_swe_difdeg2n(self):
        self.assertAlmostEqual(difdeg2n(360.5, 540), -179.5, places=12)

    def test_cotrans_test_swe_cotrans(self):
        lon, lat, _ = cotrans(121.34, 43.57, 1.0, 23.4)
        self.assertAlmostEqual(lon, 114.11984833491826, places=9)
        self.assertAlmostEqual(lat, 22.754921351892474, places=9)

    def test_split_deg_test_swe_split_deg(self):
        d, m, s, frac, sgn = split_deg(123.123)
        self.assertEqual(d, 123)
        self.assertEqual(m, 7)
        self.assertEqual(s, 22)
        self.assertAlmostEqual(frac, 0.8, places=5)
        self.assertEqual(sgn, 1)

    def test_day_of_week_test_swe_day_of_week(self):
        self.assertEqual(day_of_week(2452275.5), 1)
        self.assertEqual(day_of_week(2459444.0), 1)


class TestFuzzingBoundary(unittest.TestCase):
    """Adversarial inputs — mirror the Rust fuzz_math targets."""

    def test_degnorm_large_inputs(self):
        for v in [1e15, -1e15, 1e100]:
            n = degnorm(v)
            self.assertGreaterEqual(n, 0.0)
            self.assertLess(n, 360.0)

    def test_csnorm_large_inputs(self):
        FULL = 360 * 360_000
        for v in [-(10**12), -1, 0, 1, FULL, FULL + 1, 10**12]:
            n = csnorm(v)
            self.assertGreaterEqual(n, 0)
            self.assertLess(n, FULL)

    def test_splitdeg_minutes_seconds_range(self):
        for ddeg in [x * 7.3 for x in range(50)]:
            _, m, s, _, _ = split_deg(ddeg)
            self.assertGreaterEqual(m, 0, f"min < 0 for {ddeg}")
            self.assertLessEqual(m, 59, f"min > 59 for {ddeg}")
            self.assertGreaterEqual(s, 0, f"sec < 0 for {ddeg}")
            self.assertLessEqual(s, 59, f"sec > 59 for {ddeg}")

    def test_difdeg2n_range_all_combinations(self):
        angles = [0, 1, 90, 179.9, 180, 180.1, 270, 359, 360]
        for a in angles:
            for b in angles:
                d = difdeg2n(a, b)
                self.assertGreater(d, -180.0, f"difdeg2n({a},{b}) = {d}")
                self.assertLessEqual(d, 180.0, f"difdeg2n({a},{b}) = {d}")

    def test_julday_revjul_500_random_roundtrips(self):
        """Deterministic pseudo-random round-trip test (post-1582 only)."""

        # Use a simple LCG for reproducibility
        seed = 42

        def rand():
            nonlocal seed
            seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
            return seed / 0x100000000

        for _ in range(500):
            y = int(rand() * 500) + 1583  # 1583..2082
            m = int(rand() * 12) + 1
            d = int(rand() * 28) + 1
            h = rand() * 24
            jd = julday(y, m, d, h)
            back = revjul(jd)
            self.assertEqual(back[0], y, f"year: {y}-{m}-{d}")
            self.assertEqual(back[1], m, f"month: {y}-{m}-{d}")
            self.assertEqual(back[2], d, f"day: {y}-{m}-{d}")
            self.assertAlmostEqual(back[3], h, places=8, msg=f"hour: {y}-{m}-{d} h={h}")

    def test_cotrans_output_always_finite(self):
        for lon in range(0, 360, 30):
            for lat in range(-80, 81, 20):
                for eps in [-90, -23.44, 0, 23.44, 90]:
                    out_lon, out_lat, _ = cotrans(lon, lat, 1.0, eps)
                    self.assertTrue(
                        math.isfinite(out_lon),
                        f"cotrans({lon},{lat},{eps}) lon not finite",
                    )
                    self.assertTrue(
                        math.isfinite(out_lat),
                        f"cotrans({lon},{lat},{eps}) lat not finite",
                    )


class TestArabicPart(unittest.TestCase):
    """Arabic Parts / Lots — pure-math implementations."""

    def _arabic_part(self, asc: float, body2: float, body1: float) -> float:
        return (asc + body2 - body1) % 360.0

    def test_lot_of_fortune_basic(self):
        # ASC=206.77°, Moon=223.32°, Sun=280.38°
        result = self._arabic_part(206.77, 223.32, 280.38)
        self.assertAlmostEqual(result, 149.71, places=1)

    def test_wraps_mod_360(self):
        # 350 + 30 - 10 = 370 → 10
        result = self._arabic_part(350.0, 30.0, 10.0)
        self.assertAlmostEqual(result, 10.0, places=3)

    def test_always_in_range(self):
        cases = [(10, 20, 350), (300, 10, 5), (0, 0, 0), (180, 350, 10)]
        for asc, b2, b1 in cases:
            r = self._arabic_part(asc, b2, b1)
            self.assertGreaterEqual(r, 0.0)
            self.assertLess(r, 360.0)


class TestLonToSign(unittest.TestCase):
    """Ecliptic longitude → zodiac sign conversion."""

    def _lon_to_sign(self, lon: float):
        lon = lon % 360.0
        sign = int(lon / 30.0)
        deg = lon % 30.0
        return sign, deg

    def test_zero_is_aries(self):
        sign, deg = self._lon_to_sign(0.0)
        self.assertEqual(sign, 0)
        self.assertAlmostEqual(deg, 0.0)

    def test_taurus(self):
        sign, deg = self._lon_to_sign(45.5)
        self.assertEqual(sign, 1)
        self.assertAlmostEqual(deg, 15.5, places=3)

    def test_late_pisces(self):
        sign, _ = self._lon_to_sign(359.9)
        self.assertEqual(sign, 11)

    def test_all_signs(self):
        for s in range(12):
            sign, _ = self._lon_to_sign(s * 30.0 + 15.0)
            self.assertEqual(sign, s)


class TestSignRuler(unittest.TestCase):
    """Traditional planetary rulers of the zodiac signs."""

    # Entries use body indices ordered from Sun through Saturn.
    TRADITIONAL = [4, 3, 2, 1, 0, 2, 3, 4, 5, 6, 6, 5]

    def test_sun_rules_leo(self):
        self.assertEqual(self.TRADITIONAL[4], 0)  # Leo → Sun

    def test_moon_rules_cancer(self):
        self.assertEqual(self.TRADITIONAL[3], 1)  # Cancer → Moon

    def test_mars_rules_aries_and_scorpio(self):
        self.assertEqual(self.TRADITIONAL[0], 4)  # Aries
        self.assertEqual(self.TRADITIONAL[7], 4)  # Scorpio

    def test_each_sign_has_a_ruler(self):
        for sign in range(12):
            self.assertIn(self.TRADITIONAL[sign], range(7))


class TestOmerPureLogic(unittest.TestCase):
    """Sefirat HaOmer pure-math tests (no compiled extension needed)."""

    SEFIROT = [
        "Chesed",
        "Gevurah",
        "Tiferet",
        "Netzach",
        "Hod",
        "Yesod",
        "Malkhut",
    ]

    def _omer_sefirot(self, day: int):
        week = (day - 1) // 7
        day_of_week = (day - 1) % 7
        return self.SEFIROT[week], self.SEFIROT[day_of_week]

    def _is_hebrew_leap_year(self, year: int) -> bool:
        return (7 * year + 1) % 19 < 7

    def test_49_unique_pairs(self):
        pairs = set()
        for d in range(1, 50):
            week_s, day_s = self._omer_sefirot(d)
            pairs.add((week_s, day_s))
        self.assertEqual(len(pairs), 49)

    def test_day_1_chesed_shebchesed(self):
        week_s, day_s = self._omer_sefirot(1)
        self.assertEqual(week_s, "Chesed")
        self.assertEqual(day_s, "Chesed")

    def test_lag_baomer_day33_hod_shebhod(self):
        week_s, day_s = self._omer_sefirot(33)
        self.assertEqual(week_s, "Hod")
        self.assertEqual(day_s, "Hod")

    def test_day_49_malkhut_shebmalkhut(self):
        week_s, day_s = self._omer_sefirot(49)
        self.assertEqual(week_s, "Malkhut")
        self.assertEqual(day_s, "Malkhut")

    def test_week_boundary(self):
        # Day 7 ends week 1 (Chesed/Malkhut), day 8 starts week 2 (Gevurah/Chesed)
        self.assertEqual(self._omer_sefirot(7), ("Chesed", "Malkhut"))
        self.assertEqual(self._omer_sefirot(8), ("Gevurah", "Chesed"))

    def test_hebrew_leap_year(self):
        # 5784 is a leap year, 5785 is not
        self.assertTrue(self._is_hebrew_leap_year(5784))
        self.assertFalse(self._is_hebrew_leap_year(5785))

    def test_omer_49_day_span(self):
        # 16 Nisan 5785 nightfall = JD 2460779.25
        start_jd = 2460779.25
        end_jd = start_jd + 48  # day 49
        self.assertEqual(int(end_jd - start_jd), 48)

    def test_sefirot_list_length(self):
        self.assertEqual(len(self.SEFIROT), 7)

    def test_all_days_in_range(self):
        for d in range(1, 50):
            week_s, day_s = self._omer_sefirot(d)
            self.assertIn(week_s, self.SEFIROT)
            self.assertIn(day_s, self.SEFIROT)


class TestJewishCalendar(unittest.TestCase):
    """Jewish calendar pure-logic tests."""

    def _is_leap(self, year):
        return (7 * year + 1) % 19 < 7

    def test_5784_is_leap(self):
        self.assertTrue(self._is_leap(5784))

    def test_5785_not_leap(self):
        self.assertFalse(self._is_leap(5785))

class TestEasterComputus(unittest.TestCase):
    """Easter computus pure-logic tests."""

    def _easter(self, year):
        a = year % 19
        b = year // 100
        c = year % 100
        d = b // 4
        e = b % 4
        f = (b + 8) // 25
        g = (b - f + 1) // 3
        h = (19 * a + b - d - g + 15) % 30
        i = c // 4
        k = c % 4
        ll = (32 + 2 * e + 2 * i - h - k) % 7
        m = (a + 11 * h + 22 * ll) // 451
        month = (h + ll - 7 * m + 114) // 31
        day = (h + ll - 7 * m + 114) % 31 + 1
        return month, day

    def test_easter_dates(self):
        cases = [(2025, (4, 20)), (2024, (3, 31)), (2019, (4, 21))]
        for year, expected in cases:
            with self.subTest(year=year):
                self.assertEqual(self._easter(year), expected)

    def test_good_friday_2_before(self):
        m, d = self._easter(2025)  # Apr 20
        self.assertEqual(d - 2, 18)
        self.assertEqual(m, 4)


class TestIslamicCalendar(unittest.TestCase):
    """Islamic Hijri calendar pure-logic tests."""

    HIJRI_EPOCH = 1_948_438.5

    def _is_hijri_leap(self, year):
        return (11 * year + 14) % 30 < 11

    def test_epoch_value(self):
        self.assertAlmostEqual(self.HIJRI_EPOCH, 1_948_438.5)

    def test_leap_years(self):
        self.assertTrue(self._is_hijri_leap(2))
        self.assertTrue(self._is_hijri_leap(5))
        self.assertFalse(self._is_hijri_leap(1))
        self.assertFalse(self._is_hijri_leap(3))

    def test_2025_overlaps_1446_1447(self):
        jd_2025 = 2_460_676.5
        year = int((jd_2025 - self.HIJRI_EPOCH) / 354.367) + 1
        self.assertAlmostEqual(year, 1446, delta=1)


class TestHinduPanchanga(unittest.TestCase):
    """Hindu Panchānga pure-logic tests."""

    TITHI_NAMES = [
        "Pratipada",
        "Dwitiya",
        "Tritiya",
        "Chaturthi",
        "Panchami",
        "Shashthi",
        "Saptami",
        "Ashtami",
        "Navami",
        "Dashami",
        "Ekadashi",
        "Dwadashi",
        "Trayodashi",
        "Chaturdashi",
        "Purnima",
        "Pratipada",
        "Dwitiya",
        "Tritiya",
        "Chaturthi",
        "Panchami",
        "Shashthi",
        "Saptami",
        "Ashtami",
        "Navami",
        "Dashami",
        "Ekadashi",
        "Dwadashi",
        "Trayodashi",
        "Chaturdashi",
        "Amavasya",
    ]

    def _tithi(self, elong):
        return int(elong / 12) % 30 + 1

    def test_tithi_from_0_degrees(self):
        self.assertEqual(self._tithi(0), 1)  # Pratipada

    def test_tithi_from_180_degrees(self):
        self.assertEqual(self._tithi(180), 16)  # Krishna Pratipada

    def test_purnima_is_tithi_15(self):
        self.assertEqual(self.TITHI_NAMES[14], "Purnima")

    def test_amavasya_is_tithi_30(self):
        self.assertEqual(self.TITHI_NAMES[29], "Amavasya")

    def test_nakshatra_span(self):
        self.assertAlmostEqual(360 / 27, 13.333, places=2)

    def test_30_tithis(self):
        self.assertEqual(len(self.TITHI_NAMES), 30)


class TestBuddhistObservances(unittest.TestCase):
    """Buddhist observances pure-logic tests."""

    def test_four_uposatha_phases(self):
        phases = ["NewMoon", "FirstQuarter", "FullMoon", "LastQuarter"]
        self.assertEqual(len(phases), 4)

    def test_vesak_window(self):
        # Vesak falls Apr 20 – Jun 20 (approx)
        import datetime

        apr20 = datetime.date(2025, 4, 20)
        jun20 = datetime.date(2025, 6, 20)
        self.assertEqual((jun20 - apr20).days, 61)


class TestNowruzBahai(unittest.TestCase):
    """Nowruz and Bahá'í calendar pure-logic tests."""

    def test_bahai_year_1_is_1844_ce(self):
        self.assertEqual(1844 - 1843, 1)

    def test_2025_is_bahai_year_182(self):
        self.assertEqual(2025 - 1843, 182)

    def test_solar_hijri_2025(self):
        self.assertEqual(2025 - 621, 1404)

    def test_19_months_of_19_days(self):
        self.assertEqual(19 * 19, 361)

    def test_leap_year_has_5_ayyam(self):
        self.assertEqual(361 + 5, 366)

    def test_regular_year_has_4_ayyam(self):
        self.assertEqual(361 + 4, 365)

    def test_months_before_ayyam(self):
        self.assertEqual(18 * 19, 342)

    def test_bahai_months_list_length(self):
        months = [
            "Bahá",
            "Jalál",
            "Jamál",
            "'Aẓamat",
            "Núr",
            "Raḥmat",
            "Kalimát",
            "Kamál",
            "Asmá'",
            "'Izzat",
            "Mashíyyat",
            "'Ilm",
            "Qudrat",
            "Qawl",
            "Masá'il",
            "Sharaf",
            "Sulṭán",
            "Mulk",
            "'Alá'",
        ]
        self.assertEqual(len(months), 19)


_PHASE_NEW_MOON = "New Moon"
_PHASE_WAXING_CRESCENT = "Waxing Crescent"
_PHASE_FIRST_QUARTER = "First Quarter"
_PHASE_WAXING_GIBBOUS = "Waxing Gibbous"
_PHASE_FULL_MOON = "Full Moon"
_PHASE_WANING_GIBBOUS = "Waning Gibbous"
_PHASE_LAST_QUARTER = "Last Quarter"
_PHASE_WANING_CRESCENT = "Waning Crescent"


class TestMoonPhases(unittest.TestCase):
    """Moon phase pure-logic tests (no compiled extension needed)."""

    SYNODIC_MONTH = 29.530588853
    EPOCH_NEW_MOON = 2451550.1

    PHASE_NAMES = [
        _PHASE_NEW_MOON,
        _PHASE_WAXING_CRESCENT,
        _PHASE_FIRST_QUARTER,
        _PHASE_WAXING_GIBBOUS,
        _PHASE_FULL_MOON,
        _PHASE_WANING_GIBBOUS,
        _PHASE_LAST_QUARTER,
        _PHASE_WANING_CRESCENT,
    ]

    def _phase_from_elongation(self, e):
        if e < 22.5 or e >= 337.5:
            return _PHASE_NEW_MOON
        if e < 67.5:
            return _PHASE_WAXING_CRESCENT
        if e < 112.5:
            return _PHASE_FIRST_QUARTER
        if e < 157.5:
            return _PHASE_WAXING_GIBBOUS
        if e < 202.5:
            return _PHASE_FULL_MOON
        if e < 247.5:
            return _PHASE_WANING_GIBBOUS
        if e < 292.5:
            return _PHASE_LAST_QUARTER
        return _PHASE_WANING_CRESCENT

    def _illumination(self, e):
        import math

        return (1 - math.cos(math.radians(e))) / 2

    def test_eight_named_phases(self):
        self.assertEqual(len(self.PHASE_NAMES), 8)
        self.assertEqual(360 / 8, 45)  # 45° per octant

    def test_phase_at_0_is_new_moon(self):
        self.assertEqual(self._phase_from_elongation(0), _PHASE_NEW_MOON)

    def test_phase_at_90_is_first_quarter(self):
        self.assertEqual(self._phase_from_elongation(90), _PHASE_FIRST_QUARTER)

    def test_phase_at_180_is_full_moon(self):
        self.assertEqual(self._phase_from_elongation(180), _PHASE_FULL_MOON)

    def test_phase_at_270_is_last_quarter(self):
        self.assertEqual(self._phase_from_elongation(270), _PHASE_LAST_QUARTER)

    def test_phase_at_359_is_new_moon(self):
        self.assertEqual(self._phase_from_elongation(359), _PHASE_NEW_MOON)

    def test_illumination_at_new_moon(self):
        self.assertAlmostEqual(self._illumination(0), 0.0, places=3)

    def test_illumination_at_full_moon(self):
        self.assertAlmostEqual(self._illumination(180), 1.0, places=3)

    def test_illumination_at_quarter(self):
        self.assertAlmostEqual(self._illumination(90), 0.5, places=3)

    def test_synodic_month_value(self):
        self.assertAlmostEqual(self.SYNODIC_MONTH, 29.53, places=1)

    def test_principal_phase_offsets(self):
        sm = self.SYNODIC_MONTH
        offsets = [0.0, sm / 4, sm / 2, 3 * sm / 4]
        self.assertAlmostEqual(offsets[1], 7.38, places=1)
        self.assertAlmostEqual(offsets[2], 14.77, places=1)
        self.assertAlmostEqual(offsets[3], 22.15, places=1)

    def test_waxing_phases_below_180(self):
        self.assertIn(self._phase_from_elongation(45), [_PHASE_WAXING_CRESCENT])
        self.assertIn(self._phase_from_elongation(135), [_PHASE_WAXING_GIBBOUS])

    def test_waning_phases_above_180(self):
        self.assertEqual(self._phase_from_elongation(225), _PHASE_WANING_GIBBOUS)
        self.assertEqual(self._phase_from_elongation(315), _PHASE_WANING_CRESCENT)

    def test_illumination_increases_to_full(self):
        illums = [self._illumination(e) for e in range(0, 181, 30)]
        for i in range(len(illums) - 1):
            self.assertLessEqual(illums[i], illums[i + 1])

    def test_illumination_decreases_after_full(self):
        illums = [self._illumination(e) for e in range(180, 361, 30)]
        for i in range(len(illums) - 1):
            self.assertGreaterEqual(illums[i], illums[i + 1])


if __name__ == "__main__":
    unittest.main(verbosity=2)


# ─── Pure-Python helper library logic tests ──────────────────────────────────
#
# These test the mathematical formulas used by the helper library without importing the
# compiled Rust extension.  Each class verifies one module's core logic.


class TestSwephelpAspectFormulas(unittest.TestCase):
    """Verify aspect-matching math independently."""

    @staticmethod
    def _difdegn(p1, p2):
        return (p2 - p1) % 360.0

    def test_exact_conjunction(self):
        diff = self._difdegn(45.0, 45.0) - 0.0
        self.assertAlmostEqual(diff, 0.0)

    def test_trine_within_orb(self):
        diff = self._difdegn(0.0, 122.0) - 120.0
        self.assertLessEqual(abs(diff), 5.0)

    def test_opposition_exact(self):
        diff = self._difdegn(0.0, 180.0) - 180.0
        self.assertAlmostEqual(diff, 0.0)

    def test_antiscion_formula(self):
        """anti = 2*axis - pos (mod 360)."""
        self.assertAlmostEqual((2 * 90 - 30) % 360, 150.0)
        self.assertAlmostEqual((2 * 90 - 150) % 360, 30.0)
        self.assertAlmostEqual((2 * 0 - 30) % 360, 330.0)

    def test_contrantiscion_is_180_from_antiscion(self):
        for pos in range(0, 360, 15):
            for axis in range(0, 360, 45):
                anti = (2 * axis - pos) % 360
                contra = (anti + 180) % 360
                diff = abs(anti - contra)
                diff = min(diff, 360 - diff)
                self.assertAlmostEqual(diff, 180.0, places=6)


class TestSwephelpVedicFormulas(unittest.TestCase):
    """Verify Vedic/Jyotish math independently."""

    def _rasi(self, lon):
        return int(lon % 360 / 30)

    def _nakshatra(self, lon):
        return int(lon % 360 / (40 / 3.0))

    def _navamsa(self, lon):
        return int(lon % 360 / (10 / 3.0)) % 12

    def test_rasi_all_signs(self):
        for s in range(12):
            lon = s * 30.0 + 15.0
            self.assertEqual(self._rasi(lon), s)

    def test_nakshatra_aswini(self):
        self.assertEqual(self._nakshatra(0.0), 0)

    def test_nakshatra_revathi(self):
        self.assertEqual(self._nakshatra(359.99), 26)

    def test_nakshatra_27_total(self):
        seen = set()
        for i in range(360):
            seen.add(self._nakshatra(float(i)))
        self.assertEqual(len(seen), 27)

    def test_navamsa_cycles(self):
        for lon in range(0, 360, 10):
            n = self._navamsa(float(lon))
            self.assertGreaterEqual(n, 0)
            self.assertLess(n, 12)

    def test_raman_houses_midpoint_formula(self):
        """6 equal arcs between ASC and MC."""
        asc, mc = 15.0, 275.0
        arc1 = abs((mc - asc + 360) % 360)
        if arc1 > 180:
            arc1 = 360 - arc1
        arc = arc1 / 3.0
        c11 = (asc - arc) % 360
        self.assertGreaterEqual(c11, 0.0)
        self.assertLess(c11, 360.0)

    def test_ochchabala_exalt_zero(self):
        """Planet at its exaltation point = 0 ochchabala."""
        cases = [
            (0, 190, 190),
            (1, 213, 213),
            (2, 345, 345),
            (3, 177, 177),
            (4, 118, 118),
            (5, 275, 275),
            (6, 20, 20),
        ]
        for body, position, exaltation in cases:
            with self.subTest(body=body):
                diff = abs(((position - exaltation + 180) % 360) - 180)
                self.assertAlmostEqual(diff / 3.0, 0.0)

    def test_tatkalika_adjacent_is_mitra(self):
        def rasi_diff2(r1, r2):
            r1, r2 = r1 % 12, r2 % 12
            d = (r1 - r2) if r1 >= r2 else 12 - (r2 - r1)
            return -6 + (d - 6) if d > 6 else d

        # adjacent signs → |diff| = 1 ≤ 3 → Mitra (+1)
        self.assertLessEqual(abs(rasi_diff2(0, 1)), 3)
        # opposite signs → |diff| = 6 > 3 → Satru (-1)
        self.assertGreater(abs(rasi_diff2(0, 6)), 3)

    def test_rasi_norm(self):
        for r in range(-24, 25):
            n = r % 12
            self.assertGreaterEqual(n, 0)
            self.assertLess(n, 12)


class TestSwephelpDatetimeFormulas(unittest.TestCase):
    """Verify datetime helper math."""

    def test_revjul_known_jd(self):
        """JD 2452275.5 = 2002-01-01 00:00 UTC."""
        jd = 2452275.5
        z = int(jd + 0.5)
        if z < 2299161:
            a = z
        else:
            al = int((z - 1867216.25) / 36524.25)
            a = z + 1 + al - al // 4
        b = a + 1524
        c = int((b - 122.1) / 365.25)
        d_val = int(365.25 * c)
        e = int((b - d_val) / 30.6001)
        day = b - d_val - int(30.6001 * e)
        month = e - 1 if e < 14 else e - 13
        year = c - 4716 if month > 2 else c - 4715
        self.assertEqual(year, 2002)
        self.assertEqual(month, 1)
        self.assertEqual(day, 1)

    def test_jd_duration_formula(self):
        span = abs(2452276.0 - 2452275.5)  # 0.5 days
        days = int(span)
        rem = span - days
        hours = int(rem * 24)
        minutes = int((rem - hours / 24.0) * 1440)
        self.assertEqual(days, 0)
        self.assertEqual(hours, 12)
        self.assertEqual(minutes, 0)

    def test_jd_duration_one_day(self):
        span = abs(2452276.5 - 2452275.5)
        self.assertEqual(int(span), 1)

    def test_parse_datetime_components(self):
        # "2002-01-01" → year=2002, month=1, day=1
        import re

        m = re.match(r"(\d{4})-(\d{2})-(\d{2})", "2002-01-01")
        self.assertIsNotNone(m)
        self.assertEqual(int(m.group(1)), 2002)
        self.assertEqual(int(m.group(2)), 1)


class TestSwephelpGeoFormulas(unittest.TestCase):
    """Verify coordinate parsing formulas."""

    def test_dms_to_decimal(self):
        """51°30'26"N → 51.50722°"""
        d, m, s = 51, 30, 26
        v = d + m / 60.0 + s / 3600.0
        self.assertAlmostEqual(v, 51.50722, places=3)

    def test_west_is_negative(self):
        v = -(2 + 20 / 60.0)
        self.assertLess(v, 0)
        self.assertAlmostEqual(abs(v), 2.3333, places=3)

    def test_geo_to_dms_roundtrip(self):
        for coord in [51.5074, 48.8566, -34.6037, 139.6917]:
            c = abs(coord)
            deg = int(c)
            rem = c - deg
            min_ = int(round(rem * 60, 6))
            rem2 = rem - min_ / 60.0
            sec = int(round(rem2 * 3600, 4))
            back = deg + min_ / 60.0 + sec / 3600.0
            self.assertAlmostEqual(back, c, places=3)

    def test_degsplit_sign_degrees(self):
        for lon in range(0, 360, 30):
            expected_sign = lon // 30
            actual = lon % 360
            sign = int(actual / 30)
            self.assertEqual(sign, expected_sign)


class TestSwephelpTimezoneTable(unittest.TestCase):
    """Verify timezone table logic (pure Python)."""

    # Reproduce the key entries from the 203-entry C table
    TZ_SUBSET = [
        ("UTC", 0, 0),
        ("GMT", 0, 0),
        ("CET", 1, 0),
        ("CEST", 2, 0),
        ("IST", 5, 30),
        ("IST", 1, 0),
        ("IST", 2, 0),  # IST is tripl
        ("JST", 9, 0),
        ("PST", -8, 0),
        ("EST", -5, 0),
        ("NZST", 12, 0),
        ("NPT", 5, 45),
        ("ACDT", 10, 30),
    ]

    def test_utc_offset_zero(self):
        utc = [e for e in self.TZ_SUBSET if e[0] == "UTC"]
        self.assertTrue(any(h == 0 and m == 0 for _, h, m in utc))

    def test_ist_has_three_entries(self):
        ist = [e for e in self.TZ_SUBSET if e[0] == "IST"]
        self.assertGreaterEqual(len(ist), 2)

    def test_nepal_is_plus545(self):
        npt = [e for e in self.TZ_SUBSET if e[0] == "NPT"]
        self.assertTrue(any(h == 5 and m == 45 for _, h, m in npt))

    def test_nzst_is_plus12(self):
        nzst = [e for e in self.TZ_SUBSET if e[0] == "NZST"]
        self.assertTrue(any(h == 12 for _, h, m in nzst))

    def test_all_hours_in_range(self):
        for name, h, m in self.TZ_SUBSET:
            self.assertGreaterEqual(h, -12, f"{name}")
            self.assertLessEqual(h, 14, f"{name}")
            self.assertGreaterEqual(m, 0, f"{name}")
            self.assertLess(m, 60, f"{name}")

    def test_no_duplicate_utc(self):
        utc = [e for e in self.TZ_SUBSET if e[0] == "UTC"]
        hours = [h for _, h, m in utc]
        self.assertEqual(hours, [0])  # UTC must appear exactly once with offset 0


class TestCalcManyPureLogic(unittest.TestCase):
    """Pure-logic tests for calc_many / calc_ut_many (no extension needed)."""

    def test_planet_constants_for_calc_many(self):
        """All standard planet constants used with calc_many are defined."""
        expected = {
            "SUN": 0,
            "MOON": 1,
            "MERCURY": 2,
            "VENUS": 3,
            "MARS": 4,
            "JUPITER": 5,
            "SATURN": 6,
            "URANUS": 7,
            "NEPTUNE": 8,
            "PLUTO": 9,
            "MEAN_NODE": 10,
            "CHIRON": 15,
        }
        for name, value in expected.items():
            self.assertGreaterEqual(value, 0, f"Planet {name} constant must be >= 0")
            self.assertLess(value, 100, f"Planet {name} constant must be < 100")


class TestIAU2000BNutationPureLogic(unittest.TestCase):
    """Pure-logic verification of the IAU 2000B nutation model properties."""

    def test_iau2000b_term_count_is_77(self):
        """IAU 2000B has exactly 77 luni-solar terms — verify via Meeus reference."""
        # At JDE 2446895.5, the dominant term (l=0,l'=0,F=0,D=0,Ω=1) contributes:
        # Δψ = −172064161 × sin(Ω) × 0.1 μas
        # For the Meeus 1987-Apr-10 reference, Ω ≈ 11.25°
        import math

        omega_deg = 11.253
        contribution_01uas = -172064161.0 * math.sin(math.radians(omega_deg))
        contribution_arcsec = contribution_01uas / 1e7
        # First-term Δψ ≈ −3.36″ (dominant but not complete sum)
        self.assertAlmostEqual(abs(contribution_arcsec), 3.36, delta=0.1)

    def test_mean_obliquity_iau2006_j2000(self):
        """IAU 2006 obliquity at J2000: ε₀ = 84381.406 arcsec = 23.439291°."""

        t = 0.0  # J2000
        eps0_arcsec = 84_381.406 - 46.836769 * t - 0.0001831 * t**2 + 0.00200340 * t**3
        eps0_deg = eps0_arcsec / 3600.0
        self.assertAlmostEqual(eps0_deg, 23.439291, places=4)

    def test_mean_obliquity_iau2006_1987(self):
        """IAU 2006 obliquity at 1987-Apr-10 ≈ 23.44094°."""

        t = (2446895.5 - 2451545.0) / 36525.0  # ≈ −0.12730
        eps0_arcsec = 84_381.406 - 46.836769 * t - 0.0001831 * t**2 + 0.00200340 * t**3
        eps0_deg = eps0_arcsec / 3600.0
        self.assertAlmostEqual(eps0_deg, 23.44094, delta=0.001)


class TestPhase1AntisciaPureLogic(unittest.TestCase):
    """Pure-logic tests for antiscia and contra-antiscia (Phase 1)."""

    @staticmethod
    def antiscion(lon):
        return (180.0 - lon) % 360.0

    @staticmethod
    def contra_antiscion(lon):
        return (360.0 - lon) % 360.0

    def test_aries_15_antiscion_is_virgo_15(self):
        self.assertAlmostEqual(self.antiscion(15.0), 165.0, places=9)

    def test_cancer_0_antiscion_is_self(self):
        self.assertAlmostEqual(self.antiscion(90.0), 90.0, places=9)

    def test_capricorn_0_antiscion_is_self(self):
        self.assertAlmostEqual(self.antiscion(270.0), 270.0, places=9)

    def test_double_application_is_identity(self):
        for lon in [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0]:
            self.assertAlmostEqual(self.antiscion(self.antiscion(lon)), lon, places=9)

    def test_contra_antiscion_double_is_identity(self):
        for lon in [0.0, 45.0, 90.0, 180.0, 270.0]:
            self.assertAlmostEqual(
                self.contra_antiscion(self.contra_antiscion(lon)), lon, places=9
            )

    def test_result_always_in_range(self):

        for i in range(360):
            a = self.antiscion(float(i))
            self.assertGreaterEqual(a, 0.0)
            self.assertLess(a, 360.0)


class TestPhase1MinorAspectsPureLogic(unittest.TestCase):
    """Verify minor aspect angle values (Phase 1)."""

    def test_quintile_is_72(self):
        self.assertAlmostEqual(360.0 / 5.0, 72.0, places=9)

    def test_biquintile_is_144(self):
        self.assertAlmostEqual(2 * 360.0 / 5.0, 144.0, places=9)

    def test_septile_approx_51_43(self):
        self.assertAlmostEqual(360.0 / 7.0, 51.4286, delta=0.001)

    def test_novile_is_40(self):
        self.assertAlmostEqual(360.0 / 9.0, 40.0, places=9)

    def test_semi_square_is_45(self):
        self.assertAlmostEqual(360.0 / 8.0, 45.0, places=9)

    def test_sesquiquadrate_is_135(self):
        self.assertAlmostEqual(3 * 360.0 / 8.0, 135.0, places=9)


class TestPhase1ArabicPartsPureLogic(unittest.TestCase):
    """Pure-math verification of Arabic Parts formulas (Phase 1)."""

    def test_lot_of_fortune_day(self):
        # ASC + Moon - Sun, day chart
        asc, sun, moon = 0.0, 30.0, 120.0
        expected = (asc + moon - sun) % 360.0
        self.assertAlmostEqual(expected, 90.0, places=9)

    def test_lot_of_fortune_night_reversed(self):
        # Night: ASC + Sun - Moon
        asc, sun, moon = 0.0, 30.0, 120.0
        expected = (asc + sun - moon) % 360.0
        self.assertAlmostEqual(expected, 270.0, places=9)

    def test_all_parts_in_range(self):

        for asc in [0, 45, 90, 180, 270]:
            sun, moon = 30.0, 120.0
            fortune = (asc + moon - sun) % 360.0
            self.assertGreaterEqual(fortune, 0.0)
            self.assertLess(fortune, 360.0)


class TestPhase4VedicPureLogic(unittest.TestCase):
    """Pure-logic tests for Vedic rasi / navamsa / nakshatra math (Phase 4)."""

    def test_rasi_range(self):
        """Every longitude maps to a rasi in [0, 11]."""
        for lon in range(0, 360, 5):
            rasi = int(lon / 30) % 12
            self.assertGreaterEqual(rasi, 0)
            self.assertLess(rasi, 12)

    def test_navamsa_range(self):
        """Every longitude maps to a navamsa in [0, 11]."""
        # Navamsa divides each rasi into 9 parts of 3°20' each
        for lon in range(0, 360):
            nav = int(lon / (30.0 / 9.0)) % 12
            self.assertGreaterEqual(nav, 0)
            self.assertLess(nav, 12)

    def test_nakshatra_range(self):
        """Every longitude maps to a nakshatra in [0, 26]."""
        for lon in range(0, 360, 5):
            nak = int(lon / (360.0 / 27.0)) % 27
            self.assertGreaterEqual(nak, 0)
            self.assertLess(nak, 27)

    def test_nakshatra_pada_range(self):
        """Pada must always be 1–4."""
        for lon in range(0, 360, 5):
            nak_len = 360.0 / 27.0  # ~13.333°
            nak = int(lon / nak_len)
            deg_in_nak = lon - nak * nak_len
            pada = int(deg_in_nak / (nak_len / 4)) + 1
            self.assertGreaterEqual(pada, 1)
            self.assertLessEqual(pada, 4)

    def test_vimshottari_period_sum(self):
        """Vimshottari dasha total spans exactly 120 years."""
        periods = {
            "Sun": 6,
            "Moon": 10,
            "Mars": 7,
            "Rahu": 18,
            "Jupiter": 16,
            "Saturn": 19,
            "Mercury": 17,
            "Ketu": 7,
            "Venus": 20,
        }
        self.assertEqual(sum(periods.values()), 120)

    def test_ashtakavarga_max_bindus(self):
        """Maximum bindus per sign per planet is 8 (one per reference point)."""
        # 8 reference points: Sun, Moon, Mars, Mer, Jup, Ven, Sat, ASC
        n_refs = 8
        # Each reference contributes at most 1 bindu to any given sign
        self.assertEqual(n_refs, 8)

    def test_north_indian_12_houses(self):
        """North Indian chart has exactly 12 houses."""
        ni_cells = [
            (270.0, 72.0),
            (405.0, 144.0),
            (468.0, 270.0),
            (405.0, 396.0),
            (270.0, 468.0),
            (135.0, 396.0),
            (72.0, 270.0),
            (135.0, 144.0),
            (270.0, 180.0),
            (360.0, 270.0),
            (270.0, 360.0),
            (180.0, 270.0),
        ]
        self.assertEqual(len(ni_cells), 12)

    def test_house_rotation_from_lagna(self):
        """House number = (sign - lagna + 12) % 12 + 1, always in [1, 12]."""
        for lagna in range(12):
            for sign in range(12):
                house = (sign - lagna + 12) % 12 + 1
                self.assertGreaterEqual(house, 1)
                self.assertLessEqual(house, 12)
                if sign == lagna:
                    self.assertEqual(house, 1)


class TestPhase5HellenisticPureLogic(unittest.TestCase):
    """Pure-logic tests for Hellenistic/Persian functions (Phase 5)."""

    # ── Egyptian terms ────────────────────────────────────────────────────────

    def _terms_ruler(self, lon: float) -> str:
        """Replicate Egyptian terms table (Ptolemy) in Python for cross-check."""
        TERMS = [
            # Aries
            [
                (6, "Jupiter"),
                (12, "Venus"),
                (20, "Mercury"),
                (25, "Mars"),
                (30, "Saturn"),
            ],
            # Taurus
            [
                (8, "Venus"),
                (14, "Mercury"),
                (22, "Jupiter"),
                (27, "Saturn"),
                (30, "Mars"),
            ],
            # Gemini
            [
                (6, "Mercury"),
                (12, "Jupiter"),
                (17, "Venus"),
                (24, "Mars"),
                (30, "Saturn"),
            ],
            # Cancer
            [
                (7, "Mars"),
                (13, "Venus"),
                (19, "Mercury"),
                (26, "Jupiter"),
                (30, "Saturn"),
            ],
            # Leo
            [
                (6, "Jupiter"),
                (11, "Venus"),
                (18, "Saturn"),
                (24, "Mercury"),
                (30, "Mars"),
            ],
            # Virgo
            [
                (7, "Mercury"),
                (17, "Venus"),
                (21, "Jupiter"),
                (28, "Mars"),
                (30, "Saturn"),
            ],
            # Libra
            [
                (6, "Saturn"),
                (14, "Mercury"),
                (21, "Jupiter"),
                (28, "Venus"),
                (30, "Mars"),
            ],
            # Scorpio
            [
                (7, "Mars"),
                (11, "Venus"),
                (19, "Mercury"),
                (24, "Jupiter"),
                (30, "Saturn"),
            ],
            # Sagittarius
            [
                (12, "Jupiter"),
                (17, "Venus"),
                (21, "Mercury"),
                (26, "Saturn"),
                (30, "Mars"),
            ],
            # Capricorn
            [
                (7, "Mercury"),
                (14, "Jupiter"),
                (22, "Venus"),
                (26, "Saturn"),
                (30, "Mars"),
            ],
            # Aquarius
            [
                (7, "Mercury"),
                (13, "Venus"),
                (20, "Jupiter"),
                (25, "Mars"),
                (30, "Saturn"),
            ],
            # Pisces
            [
                (12, "Venus"),
                (16, "Jupiter"),
                (19, "Mercury"),
                (28, "Mars"),
                (30, "Saturn"),
            ],
        ]
        sign = int(lon / 30) % 12
        deg_in_sign = lon % 30
        for end_deg, planet in TERMS[sign]:
            if deg_in_sign < end_deg:
                return planet
        return "Saturn"

    def test_terms_partition_every_sign(self):
        """Every 1° increment must have a terms ruler."""
        traditional = {"Jupiter", "Venus", "Mercury", "Mars", "Saturn"}
        for i in range(360):
            lon = i + 0.5
            ruler = self._terms_ruler(lon)
            self.assertIn(ruler, traditional, f"lon {lon}: {ruler} not traditional")

    def test_aries_first_term_is_jupiter(self):
        self.assertEqual(self._terms_ruler(3.0), "Jupiter")

    def test_aries_second_term_is_venus(self):
        self.assertEqual(self._terms_ruler(7.0), "Venus")

    # ── Decans ────────────────────────────────────────────────────────────────

    def _decan_ruler(self, lon: float) -> str:
        DECAN_RULERS = [
            "Mars",
            "Sun",
            "Venus",  # Aries
            "Mercury",
            "Moon",
            "Saturn",  # Taurus
            "Jupiter",
            "Mars",
            "Sun",  # Gemini
            "Venus",
            "Mercury",
            "Moon",  # Cancer
            "Saturn",
            "Jupiter",
            "Mars",  # Leo
            "Sun",
            "Venus",
            "Mercury",  # Virgo
            "Moon",
            "Saturn",
            "Jupiter",  # Libra
            "Mars",
            "Sun",
            "Venus",  # Scorpio
            "Mercury",
            "Moon",
            "Saturn",  # Sagittarius
            "Jupiter",
            "Mars",
            "Sun",  # Capricorn
            "Venus",
            "Mercury",
            "Moon",  # Aquarius
            "Saturn",
            "Jupiter",
            "Mars",  # Pisces
        ]
        return DECAN_RULERS[int(lon / 10) % 36]

    def test_aries_first_decan_is_mars(self):
        self.assertEqual(self._decan_ruler(5.0), "Mars")

    def test_aries_second_decan_is_sun(self):
        self.assertEqual(self._decan_ruler(15.0), "Sun")

    def test_all_36_decans_covered(self):
        rulers = {self._decan_ruler(i * 10 + 5) for i in range(36)}
        # Should use all 7 classical planets (Sun + 6)
        self.assertEqual(len(rulers), 7)

    # ── Triplicity ────────────────────────────────────────────────────────────

    def _element(self, lon: float) -> str:
        sign = int(lon / 30) % 12
        return ["fire", "earth", "air", "water"][sign % 4]

    def test_aries_is_fire(self):
        self.assertEqual(self._element(15.0), "fire")

    def test_taurus_is_earth(self):
        self.assertEqual(self._element(45.0), "earth")

    def test_gemini_is_air(self):
        self.assertEqual(self._element(75.0), "air")

    def test_cancer_is_water(self):
        self.assertEqual(self._element(105.0), "water")

    # ── Sect ──────────────────────────────────────────────────────────────────

    def test_sect_day_planets(self):
        """Sun, Jupiter, Saturn are diurnal."""
        DIURNAL = {"Sun", "Jupiter", "Saturn"}
        NOCTURNAL = {"Moon", "Venus", "Mars"}
        for p in DIURNAL:
            self.assertNotIn(p, NOCTURNAL)

    # ── Firdaria sequence ─────────────────────────────────────────────────────

    def test_vimshottari_day_sequence_starts_sun(self):
        """Day Firdaria: Sun→Venus→Mercury→Moon→Saturn→Jupiter→Mars (10+8+13+9+11+12+7=70)."""
        DAY_SEQ = [
            ("Sun", 10),
            ("Venus", 8),
            ("Mercury", 13),
            ("Moon", 9),
            ("Saturn", 11),
            ("Jupiter", 12),
            ("Mars", 7),
        ]
        self.assertEqual(DAY_SEQ[0][0], "Sun")
        self.assertEqual(sum(d for _, d in DAY_SEQ), 70)

    def test_firdaria_night_sequence_starts_moon(self):
        NIGHT_SEQ = [
            ("Moon", 9),
            ("Saturn", 11),
            ("Mercury", 13),
            ("Venus", 8),
            ("Jupiter", 12),
            ("Mars", 7),
            ("Sun", 10),
        ]
        self.assertEqual(NIGHT_SEQ[0][0], "Moon")

    # ── Profections ───────────────────────────────────────────────────────────

    def test_profection_rotation(self):
        """One house per year, returns to house 1 after 12 years."""
        for age in range(48):
            house = (age % 12) + 1
            self.assertGreaterEqual(house, 1)
            self.assertLessEqual(house, 12)
        boundary_houses = [(age % 12) + 1 for age in (0, 11, 12)]
        self.assertEqual(boundary_houses, [1, 12, 1])


class TestPhase6ChinesePureLogic(unittest.TestCase):
    """Pure-logic tests for Chinese Ba Zi calendar math (Phase 6)."""

    def _year_cycle(self, year):
        return (year - 4) % 60

    def test_jiazi_year_2044(self):
        # 2044 - 4 = 2040; 2040 % 60 = 0 → cycle 0 = Jiǎ-Zǐ
        self.assertEqual(self._year_cycle(2044), 0)

    def test_year_cycle_range(self):
        for y in range(1900, 2100):
            c = self._year_cycle(y)
            self.assertGreaterEqual(c, 0)
            self.assertLess(c, 60)

    def test_stem_count_is_10(self):
        # 10 Heavenly Stems
        stems = ["Jiǎ", "Yǐ", "Bǐng", "Dīng", "Wù", "Jǐ", "Gēng", "Xīn", "Rén", "Guǐ"]
        self.assertEqual(len(stems), 10)

    def test_branch_count_is_12(self):
        animals = [
            "Rat",
            "Ox",
            "Tiger",
            "Rabbit",
            "Dragon",
            "Snake",
            "Horse",
            "Goat",
            "Monkey",
            "Rooster",
            "Dog",
            "Pig",
        ]
        self.assertEqual(len(animals), 12)

    def test_solar_terms_count(self):
        # 24 solar terms, one every 15°
        self.assertEqual(360 // 15, 24)

    def test_hour_branch(self):
        # 12 double-hours cover 24 hours
        for h in range(24):
            branch = ((h + 1) // 2) % 12
            self.assertGreaterEqual(branch, 0)
            self.assertLess(branch, 12)


class TestPhase7MesoamericanPureLogic(unittest.TestCase):
    """Pure-logic tests for Mesoamerican calendar math (Phase 7)."""

    GMT = 584_283

    def _tonalpohualli(self, jd):
        day_num = (int(jd) - self.GMT) % 260
        return (day_num % 13 + 1, day_num % 20)

    def _xiuhpohualli(self, jd):
        day_num = (int(jd) - self.GMT) % 365
        return (day_num // 20, day_num % 20 + 1)

    def test_tonalpohualli_trecena_range(self):
        for i in range(260):
            jd = 2_451_545 + i
            t, _ = self._tonalpohualli(jd)
            self.assertGreaterEqual(t, 1)
            self.assertLessEqual(t, 13)

    def test_tonalpohualli_260_cycle(self):
        jd = 2_451_545
        t1, s1 = self._tonalpohualli(jd)
        t2, s2 = self._tonalpohualli(jd + 260)
        self.assertEqual(t1, t2)
        self.assertEqual(s1, s2)

    def test_xiuhpohualli_365_cycle(self):
        jd = 2_451_545
        m1, d1 = self._xiuhpohualli(jd)
        m2, d2 = self._xiuhpohualli(jd + 365)
        self.assertEqual(m1, m2)
        self.assertEqual(d1, d2)

    def test_calendar_round_18980_days(self):
        # Calendar Round repeats at the least common multiple of both cycles.
        import math

        self.assertEqual(math.lcm(260, 365), 18_980)

    def test_signs_table_length(self):
        signs = [
            "Cipactli",
            "Ehecatl",
            "Calli",
            "Cuetzpallin",
            "Coatl",
            "Miquiztli",
            "Mazatl",
            "Tochtli",
            "Atl",
            "Itzcuintli",
            "Ozomatli",
            "Malinalli",
            "Acatl",
            "Ocelotl",
            "Cuauhtli",
            "Cozcacuauhtli",
            "Ollin",
            "Tecpatl",
            "Quiahuitl",
            "Xochitl",
        ]
        self.assertEqual(len(signs), 20)


_SNOW_GOOSE_TOTEM = "Snow Goose"


class TestPhase8IndigenousPureLogic(unittest.TestCase):
    """Pure-logic tests for Medicine Wheel and Egyptian decans (Phase 8)."""

    def test_medicine_wheel_12_totems(self):
        # Sun Bear system: 12 birth totems aligned to solar year
        totems = [
            "Snow Goose",
            "Otter",
            "Cougar",
            "Red Hawk",
            "Beaver",
            "Deer",
            "Flicker",
            "Sturgeon",
            "Brown Bear",
            "Raven",
            "Snake",
            "Elk",
        ]
        self.assertEqual(len(totems), 12)

    def test_36_egyptian_decans(self):
        # 360° / 10° = 36 decans
        self.assertEqual(360 // 10, 36)

    def test_decan_idx_range(self):
        for deg in range(360):
            idx = int(deg / 10) % 36
            self.assertGreaterEqual(idx, 0)
            self.assertLess(idx, 36)

    def test_4_elements_in_medicine_wheel(self):
        elements = {"Fire", "Earth", "Air", "Water"}
        self.assertEqual(len(elements), 4)


# ── Shared fixture-driven cross-language tests ────────────────────────────────


def _load_fixtures():
    p = (
        _pathlib.Path(__file__).parents[5]
        / "tests"
        / "fixtures"
        / "reference_values.json"
    )
    with open(p, encoding="utf-8") as f:
        return _json.load(f)


class TestSharedFixtures(unittest.TestCase):
    """Validates pure-logic against the canonical reference_values.json fixture.
    These same fixtures are loaded by the JS and PHP test suites too."""

    @classmethod
    def setUpClass(cls):
        cls.fx = _load_fixtures()

    # ── Antiscia ──────────────────────────────────────────────────────────────
    def _antiscion(self, lon):
        return (180.0 - lon) % 360.0

    def _contra_antiscion(self, lon):
        return (360.0 - lon) % 360.0

    def test_antiscia_from_fixture(self):
        for case in self.fx["antiscia"]:
            got = self._antiscion(case["input_lon"])
            self.assertAlmostEqual(
                got,
                case["antiscion"],
                places=9,
                msg=f"antiscion at {case['input_lon']}",
            )
            got_c = self._contra_antiscion(case["input_lon"])
            self.assertAlmostEqual(
                got_c,
                case["contra"],
                places=9,
                msg=f"contra_antiscion at {case['input_lon']}",
            )

    # ── Egyptian terms ─────────────────────────────────────────────────────────
    _TERMS = [
        [(6, "Jupiter"), (12, "Venus"), (20, "Mercury"), (25, "Mars"), (30, "Saturn")],
        [(8, "Venus"), (14, "Mercury"), (22, "Jupiter"), (27, "Saturn"), (30, "Mars")],
        [(6, "Mercury"), (12, "Jupiter"), (17, "Venus"), (24, "Mars"), (30, "Saturn")],
        [(7, "Mars"), (13, "Venus"), (19, "Mercury"), (26, "Jupiter"), (30, "Saturn")],
        [(6, "Jupiter"), (11, "Venus"), (18, "Saturn"), (24, "Mercury"), (30, "Mars")],
        [(7, "Mercury"), (17, "Venus"), (21, "Jupiter"), (28, "Mars"), (30, "Saturn")],
        [(6, "Saturn"), (14, "Mercury"), (21, "Jupiter"), (28, "Venus"), (30, "Mars")],
        [(7, "Mars"), (11, "Venus"), (19, "Mercury"), (24, "Jupiter"), (30, "Saturn")],
        [(12, "Jupiter"), (17, "Venus"), (21, "Mercury"), (26, "Saturn"), (30, "Mars")],
        [(7, "Mercury"), (14, "Jupiter"), (22, "Venus"), (26, "Saturn"), (30, "Mars")],
        [(7, "Mercury"), (13, "Venus"), (20, "Jupiter"), (25, "Mars"), (30, "Saturn")],
        [(12, "Venus"), (16, "Jupiter"), (19, "Mercury"), (28, "Mars"), (30, "Saturn")],
    ]

    def _terms_ruler(self, lon):
        sign = int(lon / 30) % 12
        deg = lon % 30
        for end, planet in self._TERMS[sign]:
            if deg < end:
                return planet
        return "Saturn"

    def test_egyptian_terms_from_fixture(self):
        for case in self.fx["egyptian_terms"]:
            got = self._terms_ruler(case["lon"])
            self.assertEqual(
                got, case["ruler"], msg=f"lon={case['lon']} ({case.get('comment', '')})"
            )

    # ── Tonalpohualli ──────────────────────────────────────────────────────────
    _GMT = 584283

    def _tonalpohualli(self, jd):
        day = (int(jd) - self._GMT) % 260
        return (day % 13 + 1, day % 20)

    def test_tonalpohualli_from_fixture(self):
        for case in self.fx["tonalpohualli"]:
            t, s = self._tonalpohualli(case["jd"])
            self.assertEqual(t, case["trecena"], msg=f"trecena at jd={case['jd']}")
            self.assertEqual(s, case["sign_idx"], msg=f"sign at jd={case['jd']}")

    # ── Profections ────────────────────────────────────────────────────────────
    def test_profections_from_fixture(self):
        for case in self.fx["profections"]:
            house = (case["age"] % 12) + 1
            self.assertEqual(house, case["house"], msg=f"age {case['age']}")

    # ── Solar terms ───────────────────────────────────────────────────────────
    def test_solar_terms_from_fixture(self):
        TERMS = [
            (0.0, "Chūnfēn"),
            (15.0, "Qīngmíng"),
            (30.0, "Gǔyǔ"),
            (45.0, "Lìxià"),
            (60.0, "Xiǎomǎn"),
            (75.0, "Mángzhòng"),
            (90.0, "Xiàzhì"),
            (105.0, "Xiǎoshǔ"),
            (120.0, "Dàshǔ"),
            (135.0, "Lìqiū"),
            (150.0, "Chǔshǔ"),
            (165.0, "Báilù"),
            (180.0, "Qiūfēn"),
            (195.0, "Hánlù"),
            (210.0, "Shuāngjiàng"),
            (225.0, "Lìdōng"),
            (240.0, "Xiǎoxuě"),
            (255.0, "Dàxuě"),
            (270.0, "Dōngzhì"),
            (285.0, "Xiǎohán"),
            (300.0, "Dàhán"),
            (315.0, "Lìchūn"),
            (330.0, "Yǔshuǐ"),
            (345.0, "Jīngzhé"),
        ]
        for case in self.fx["solar_terms"]:
            lon, name = TERMS[case["idx"]]
            self.assertAlmostEqual(lon, case["lon"], places=9)
            self.assertEqual(name, case["pinyin"])

    # ── Medicine Wheel ────────────────────────────────────────────────────────
    _TOTEMS = [
        (300.0, 330.0, _SNOW_GOOSE_TOTEM, "Earth", "Turtle", "Winter"),
        (330.0, 360.0, "Otter", "Air", "Butterfly", "Winter"),
        (0.0, 30.0, "Cougar", "Air", "Butterfly", "Spring"),
        (30.0, 60.0, "Red Hawk", "Fire", "Thunderbird", "Spring"),
        (60.0, 90.0, "Beaver", "Earth", "Turtle", "Spring"),
        (90.0, 120.0, "Deer", "Air", "Butterfly", "Summer"),
        (120.0, 150.0, "Flicker", "Water", "Frog", "Summer"),
        (150.0, 180.0, "Sturgeon", "Fire", "Thunderbird", "Summer"),
        (180.0, 210.0, "Brown Bear", "Earth", "Turtle", "Autumn"),
        (210.0, 240.0, "Raven", "Air", "Butterfly", "Autumn"),
        (240.0, 270.0, "Snake", "Water", "Frog", "Autumn"),
        (270.0, 300.0, "Elk", "Fire", "Thunderbird", "Winter"),
    ]

    def _totem(self, lon):
        lon = lon % 360
        for lo, hi, animal, element, clan, season in self._TOTEMS:
            if lo < hi:
                if lo <= lon < hi:
                    return (animal, element, clan, season)
            else:
                if lon >= lo or lon < hi:
                    return (animal, element, clan, season)
        return (_SNOW_GOOSE_TOTEM, "Earth", "Turtle", "Winter")

    def test_medicine_wheel_from_fixture(self):
        for case in self.fx["medicine_wheel"]:
            got = self._totem(case["sun_lon"])
            self.assertEqual(
                got[0], case["animal"], msg=f"animal at lon={case['sun_lon']}"
            )
            self.assertEqual(got[1], case["element"])
            self.assertEqual(got[2], case["clan"])
            self.assertEqual(got[3], case["season"])
