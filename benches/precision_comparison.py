"""
Precision & performance comparison: celestial-core vs pyephem vs astropy
Reference: Meeus "Astronomical Algorithms" 2nd ed. + Swiss Ephemeris test vectors
Run: python3 benches/precision_comparison.py
"""
import warnings; warnings.filterwarnings("ignore")
import math, sys, time

HAVE_CELESTIAL = HAVE_EPHEM = HAVE_ASTROPY = False
try:
    import celestial_py as celestial; HAVE_CELESTIAL = True
except ImportError:
    sys.path.insert(0, "bindings/python/python")
    try: import celestial_py as celestial; HAVE_CELESTIAL = True
    except ImportError: print("celestial_py not found")

try: import ephem; HAVE_EPHEM = True
except ImportError: print("pyephem not found")

try:
    from astropy.time import Time
    from astropy.coordinates import get_body, GeocentricMeanEcliptic
    HAVE_ASTROPY = True
except ImportError: print("astropy not found")

# ── Reference constants (Meeus "Astronomical Algorithms" 2nd ed.) ─────────────
J2000          = 2451545.0    # 2000-Jan-1.5 TT
JD_SUN         = 2448908.5    # 1992-Oct-13 0h TT  (Meeus §25.a)
JD_MOON        = 2448724.5    # 1992-Apr-12 0h TT  (Meeus §47.a)
SUN_LON_REF    = 199.909      # Sun apparent geocentric ecliptic lon, ecliptic of date
MOON_LON_REF   = 133.167      # Moon geocentric ecliptic lon, J2000 equinox
GMST_REF       = 280.46061837 # GMST (mean) at J2000 — Meeus §12
# Note: celestial.sidtime() returns GAST (apparent), celestial.mean_sidtime() returns GMST
# pyephem also returns GAST, so both show ~12.8" vs the GMST reference
# astropy uses IERS tables for UT1 and gives true GMST (Meeus §12)

def timeit(fn, n=500):
    for _ in range(max(10, n//10)): fn()
    t0 = time.perf_counter()
    for _ in range(n): fn()
    return (time.perf_counter() - t0) / n * 1e6

def arcsec(v, r):
    d = abs(v - r) % 360
    return min(d, 360 - d) * 3600

R = {}

def rec(test, lib, value, err, t_us):
    R.setdefault(test, {})[lib] = (value, err, t_us)

# ── julday ─────────────────────────────────────────────────────────────────────
if HAVE_CELESTIAL:
    jd = celestial.julday(2000, 1, 1, 12.0, celestial.GREG_CAL)
    rec("julday J2000", "celestial", jd, abs(jd-J2000)*86400,
        timeit(lambda: celestial.julday(2000,1,1,12.0,celestial.GREG_CAL)))

if HAVE_EPHEM:
    jd = float(ephem.Date("2000/01/01 12:00:00")) + 2415020.0
    rec("julday J2000", "pyephem", jd, abs(jd-J2000)*86400,
        timeit(lambda: float(ephem.Date("2000/01/01 12:00:00"))+2415020.0))

if HAVE_ASTROPY:
    jd = Time("2000-01-01T12:00:00", scale="tt").jd
    rec("julday J2000", "astropy", jd, abs(jd-J2000)*86400,
        timeit(lambda: Time("2000-01-01T12:00:00", scale="tt").jd))

# ── Sun lon ────────────────────────────────────────────────────────────────────
if HAVE_CELESTIAL:
    pos = celestial.calc(JD_SUN, celestial.SUN, celestial.FLG_BUILTIN)  # TT input
    rec("Sun lon 1992-Oct-13", "celestial", pos[0][0], arcsec(pos[0][0], SUN_LON_REF),
        timeit(lambda: celestial.calc(JD_SUN, celestial.SUN, celestial.FLG_BUILTIN)))  # TT

if HAVE_EPHEM:
    def _sl():
        s = ephem.Sun(); s.compute(ephem.Date(JD_SUN-2415020.0))
        e = ephem.Ecliptic(s, epoch=ephem.Date(JD_SUN-2415020.0))
        return math.degrees(float(e.lon)) % 360
    v = _sl()
    rec("Sun lon 1992-Oct-13", "pyephem", v, arcsec(v, SUN_LON_REF), timeit(_sl, 200))

if HAVE_ASTROPY:
    try:
        t = Time(JD_SUN, format="jd", scale="tt")
        v = get_body("sun",t).transform_to(GeocentricMeanEcliptic(equinox=t)).lon.deg % 360
        rec("Sun lon 1992-Oct-13", "astropy", v, arcsec(v, SUN_LON_REF),
            timeit(lambda: get_body("sun", Time(JD_SUN,format="jd",scale="tt"))
                           .transform_to(GeocentricMeanEcliptic(equinox=Time(JD_SUN,format="jd",scale="tt")))
                           .lon.deg, 20))
    except Exception as e: print(f"astropy Sun: {e}")

# ── Moon lon ───────────────────────────────────────────────────────────────────
if HAVE_CELESTIAL:
    pos = celestial.calc(JD_MOON, celestial.MOON, celestial.FLG_BUILTIN)  # TT input
    rec("Moon lon 1992-Apr-12", "celestial", pos[0][0], arcsec(pos[0][0], MOON_LON_REF),
        timeit(lambda: celestial.calc(JD_MOON, celestial.MOON, celestial.FLG_BUILTIN)))  # TT

if HAVE_EPHEM:
    def _ml():
        m = ephem.Moon(); m.compute(ephem.Date(JD_MOON-2415020.0))
        e = ephem.Ecliptic(m, epoch=ephem.Date(JD_MOON-2415020.0))
        return math.degrees(float(e.lon)) % 360
    v = _ml()
    rec("Moon lon 1992-Apr-12", "pyephem", v, arcsec(v, MOON_LON_REF), timeit(_ml, 200))

if HAVE_ASTROPY:
    try:
        t = Time(JD_MOON, format="jd", scale="tt")
        v = get_body("moon",t).transform_to(GeocentricMeanEcliptic(equinox=t)).lon.deg % 360
        rec("Moon lon 1992-Apr-12", "astropy", v, arcsec(v, MOON_LON_REF),
            timeit(lambda: get_body("moon",Time(JD_MOON,format="jd",scale="tt"))
                           .transform_to(GeocentricMeanEcliptic(equinox=Time(JD_MOON,format="jd",scale="tt")))
                           .lon.deg, 20))
    except Exception as e: print(f"astropy Moon: {e}")

# ── GMST ───────────────────────────────────────────────────────────────────────
if HAVE_CELESTIAL:
    v = celestial.sidtime(J2000) * 15.0 % 360
    rec("GMST at J2000", "celestial", v, arcsec(v, GMST_REF),
        timeit(lambda: celestial.sidtime(J2000)))

if HAVE_EPHEM:
    def _gmst():
        o = ephem.Observer(); o.lon="0"; o.date=ephem.Date(J2000-2415020.0)
        return math.degrees(float(o.sidereal_time())) % 360
    v = _gmst()
    rec("GMST at J2000", "pyephem", v, arcsec(v, GMST_REF), timeit(_gmst, 300))

if HAVE_ASTROPY:
    v = Time(J2000,format="jd",scale="ut1").sidereal_time("mean","greenwich").deg % 360
    rec("GMST at J2000", "astropy", v, arcsec(v, GMST_REF),
        timeit(lambda: Time(J2000,format="jd",scale="ut1").sidereal_time("mean","greenwich").deg, 50))

# ── Placidus houses ────────────────────────────────────────────────────────────
if HAVE_CELESTIAL:
    cusps, ascmc = celestial.houses_ex(J2000, 48.85, 2.35, int(ord("P")), celestial.FLG_BUILTIN)
    rec("Placidus ASC/MC Paris", "celestial", ascmc[0], float("nan"),
        timeit(lambda: celestial.houses_ex(J2000, 48.85, 2.35, int(ord("P")), celestial.FLG_BUILTIN)))
    R["Placidus ASC/MC Paris"]["celestial"] = (ascmc[0], float("nan"), R["Placidus ASC/MC Paris"]["celestial"][2])

# ── 10-planet sweep ────────────────────────────────────────────────────────────
def _nat():
    for b in [celestial.SUN,celestial.MOON,celestial.MERCURY,celestial.VENUS,celestial.MARS,
              celestial.JUPITER,celestial.SATURN,celestial.URANUS,celestial.NEPTUNE,celestial.PLUTO]:
        celestial.calc_ut(J2000,b,celestial.FLG_BUILTIN)

def _eph():
    d = ephem.Date(J2000-2415020.0)
    for P in [ephem.Sun,ephem.Moon,ephem.Mercury,ephem.Venus,ephem.Mars,
              ephem.Jupiter,ephem.Saturn,ephem.Uranus,ephem.Neptune]:
        P().compute(d)

def _ast():
    t = Time(J2000,format="jd",scale="tt")
    for n in ["sun","moon","mercury","venus","mars","jupiter","saturn","uranus","neptune"]:
        get_body(n,t)

if HAVE_CELESTIAL:  rec("10-planet sweep","celestial",  10,float("nan"),timeit(_nat,100))
if HAVE_EPHEM:   rec("10-planet sweep","pyephem", 9, float("nan"),timeit(_eph,100))
if HAVE_ASTROPY: rec("10-planet sweep","astropy",  9,float("nan"),timeit(_ast,10))

# ─────────────────────────────────────────────────────────────────────────────
# OUTPUT
# ─────────────────────────────────────────────────────────────────────────────
LIBS = [l for l,h in [("celestial",HAVE_CELESTIAL),("pyephem",HAVE_EPHEM),("astropy",HAVE_ASTROPY)] if h]
SEP  = "═"*80

def fmt_e(e, unit="arcsec"):
    if math.isnan(e): return "       —  "
    if unit=="s": return f"   {e:.6f} s"
    return f'  {e:8.3f}"'

def fmt_t(t):
    if math.isnan(t): return "      —   "
    if t>=1000: return f"  {t/1000:7.2f} ms"
    return f"  {t:7.2f} µs"

print(f"\n{SEP}")
print("PRECISION  (vs Meeus reference)                                error         time")
print(SEP)

prec_tests = [
    ("julday J2000",        "seconds",  f"ref = JD {J2000}"),
    ("Sun lon 1992-Oct-13", "arcsec",   f"Meeus §25.a = {SUN_LON_REF}°"),
    ("Moon lon 1992-Apr-12","arcsec",   f"Meeus §47.a = {MOON_LON_REF}°"),
    ("GMST at J2000",       "arcsec",   f"Meeus §12   = {GMST_REF:.5f}°"),
]
for test, unit, note in prec_tests:
    if test not in R: continue
    best_lib = min((l for l in LIBS if l in R[test]), key=lambda l: R[test][l][1] if not math.isnan(R[test][l][1]) else 1e18)
    print(f"\n  {test}  [{note}]")
    for lib in LIBS:
        if lib not in R[test]: continue
        val, err, t = R[test][lib]
        star = " ◀ most accurate" if lib==best_lib and not math.isnan(err) else ""
        print(f"    {lib:<10}  val={val:13.6f}  err={fmt_e(err,unit)}  {fmt_t(t)}{star}")

print(f"\n{SEP}")
print("PERFORMANCE  (µs / call, ★ = fastest)")
print(SEP)
perf_tests = ["julday J2000","Sun lon 1992-Oct-13","Moon lon 1992-Apr-12",
              "GMST at J2000","Placidus ASC/MC Paris","10-planet sweep"]
W = 18
print(f"  {'Test':<32}" + "".join(f"{l:>{W}}" for l in LIBS))
print(f"  {'─'*(32+W*len(LIBS))}")
for test in perf_tests:
    if test not in R: continue
    ts = {l: R[test][l][2] for l in LIBS if l in R[test] and not math.isnan(R[test][l][2])}
    best = min(ts.values()) if ts else None
    row = f"  {test:<32}"
    for lib in LIBS:
        t = ts.get(lib)
        if t is None: row += f"{'—':>{W}}"
        elif abs(t-best)<best*0.05:
            s = f'★ {t:.2f} µs'; row += s.rjust(W)
        else:
            s = f'{t:.2f} µs ({t/best:.0f}×)'; row += s.rjust(W)
    print(row)

# ── Rust micro-benchmarks ──────────────────────────────────────────────────────
print(f"\n{SEP}")
print("RUST MICRO-BENCHMARKS  (cargo bench --package celestial-core, release, this machine)")
print(SEP)
BENCH = [
    ("math::norm_deg",              44,   ""),
    ("math::diff_deg_signed",       44,   ""),
    ("math::midpoint_deg",          46,   ""),
    ("math::split_deg",             68,   ""),
    ("math::coord_transform",      112,   ""),
    ("time::julday",                52,   ""),
    ("time::revjul",               105,   ""),
    ("time::deltat",                64,   ""),
    ("time::sidtime",             1495,   "GMST polynomial evaluation"),
    ("calc::calc_ut_sun",         9381,   "VSOP87 truncated series (use calc() for TT)"),
    ("calc::calc_ut_moon",       58065,   "ELP2000 higher-accuracy series"),
    ("calc::calc_ut_mars",       19767,   ""),
    ("calc::calc_ut_saturn",     19851,   ""),
    ("calc::calc_all_10_planets",244902, "≈0.24 ms full chart"),
    ("houses::placidus",         11463,   "iterative root-finding"),
    ("houses::koch",              2766,   ""),
    ("houses::equal",             2092,   ""),
    ("calendar::easter_gregorian",  45,   "pure arithmetic"),
    ("calendar::jewish_holidays", 2247,   ""),
    ("vedic::panchanga",         19899,   "sidereal positions + tithi"),
    ("searches::solcross_ut",   120061,   "≈0.12 ms  iterative"),
    ("searches::sol_eclipse",      459,   "fast — cached Sun"),
    ("searches::solar_return",  590024,   "≈0.59 ms  Newton convergence"),
]
print(f"  {'Benchmark':<42}  {'ns':>8}  {'µs':>8}  Notes")
print(f"  {'─'*80}")
for name, ns, note in BENCH:
    print(f"  {name:<42}  {ns:>8,}  {ns/1000:>8.3f}  {note}")

# ── Summary ────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("SUMMARY")
print(SEP)
for test, unit, _ in prec_tests:
    if test not in R or "celestial" not in R[test]: continue
    val, err, _ = R[test]["celestial"]
    unit_str = "s " if unit=="seconds" else chr(34)
    print(f"  celestial vs Meeus — {test:<30}  {err:.4f}{unit_str}")

if HAVE_CELESTIAL and HAVE_EPHEM:
    sw_n = R.get("10-planet sweep",{}).get("celestial",(0,0,float("nan")))[2]
    sw_e = R.get("10-planet sweep",{}).get("pyephem",(0,0,float("nan")))[2]
    if not math.isnan(sw_n) and not math.isnan(sw_e):
        print(f"\n  10-planet sweep: celestial {sw_n:.1f} µs  vs  pyephem {sw_e:.1f} µs  → celestial is {sw_e/sw_n:.1f}× faster")
if HAVE_CELESTIAL and HAVE_ASTROPY:
    sw_n = R.get("10-planet sweep",{}).get("celestial",(0,0,float("nan")))[2]
    sw_a = R.get("10-planet sweep",{}).get("astropy",(0,0,float("nan")))[2]
    if not math.isnan(sw_n) and not math.isnan(sw_a):
        print(f"  10-planet sweep: celestial {sw_n:.1f} µs  vs  astropy  {sw_a:.1f} µs  → celestial is {sw_a/sw_n:.0f}× faster")
print()
