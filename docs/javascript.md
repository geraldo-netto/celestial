# celestial — JavaScript / TypeScript binding

**Package:** `celestial-js` (napi-rs)  
**Platforms:** Node.js ≥ 18, pre-built `.node` addon  
**Types:** Full TypeScript declarations in `bindings/js/index.d.ts`

## Quick start

```bash
cd bindings/js && npm install && npm run build
```

This guide covers TypeScript interfaces, all function signatures, and tradition-specific
examples.

---

> **Note on Rust builder structs:** `CalcOptions`, `RiseTransOptions`,
> `SearchOptions`, and `AspectOrbs` are Rust-only builder structs. The JS
> binding exposes the underlying functions directly: `calc_ut`, `calc_many`,
> `rise_trans`, `next_aspect_cusp`, `match_aspect3`, `match_aspect4`, etc.

## Installation

```bash
cd bindings/js
npm install
npm run build        # compiles the native addon via napi-rs
```

Or from a pre-built release:

```bash
npm install celestial-js
```

---

## Import

```typescript
import * as celestial from "celestial-js";
// or
const celestial = require("celestial-js");
```

---

## Constants

### Body indices

```typescript
const SUN = 0, MOON = 1, MERCURY = 2, VENUS = 3, MARS = 4;
const JUPITER = 5, SATURN = 6, URANUS = 7, NEPTUNE = 8, PLUTO = 9;
const MEAN_NODE = 10, TRUE_NODE = 11, CHIRON = 15;
```

### Calculation flags

```typescript
const FLG_BUILTIN    = 2;    // use built-in ephemeris (always include)
const FLG_SPEED      = 256;  // include daily speed
const FLG_SIDEREAL   = 65536; // sidereal positions
const FLG_EQUATORIAL = 2048; // equatorial coordinates
const FLG_HELCTR     = 8;    // heliocentric
```

### Sidereal modes

```typescript
const SIDM_FAGAN_BRADLEY = 0;
const SIDM_LAHIRI        = 1;
const SIDM_RAMAN         = 3;
const SIDM_KRISHNAMURTI  = 5;
```

---

## TypeScript types

```typescript
interface PlanetPos {
  lon:       number;  // ecliptic longitude (degrees)
  lat:       number;  // ecliptic latitude
  dist:      number;  // distance (AU)
  speed_lon: number;  // daily speed in longitude (°/day)
  speed_lat: number;
  speed_dist: number;
}

interface HouseResult {
  cusps:  number[];   // [0..12], cusps[1..12] are the house cusps
  ascmc:  number[];   // [0]=ASC [1]=MC [2]=ARMC [3]=Vertex
}

// nutation returns a plain [dpsi, deps] array (degrees), not an object.
```

---

## Core functions

### Time

```typescript
// Calendar → Julian Day
const jd = celestial.julday(2025, 3, 20, 9.0, 1);  // 1 = GREG_CAL

// Julian Day → calendar date
const date = celestial.revjul(jd, 1);
console.log(`${date.year}-${date.month}-${date.day}`);

// Current JD
const now = celestial.jdnow();
```

### Planetary positions

```typescript
// Single body
const sun: PlanetPos = celestial.calc_ut(jd, 0, 2 | 256);  // FLG_BUILTIN | FLG_SPEED
console.log(`Sun lon=${sun.lon.toFixed(4)}°  dist=${sun.dist.toFixed(6)} AU`);

// Multiple bodies in parallel
const planets = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15];
const results: PlanetPos[] = celestial.calc_many(jd, planets, 2 | 256);
// results[i] corresponds to planets[i]

// Nutation (IAU 2000B, 77 terms) — one arg, returns [dpsi, deps] in degrees
const [dpsi, deps]: number[] = celestial.nutation(jd);
console.log(`dpsi=${dpsi.toFixed(6)}°  deps=${deps.toFixed(6)}°`);

// Mean sidereal time
const gmst: number = celestial.mean_sidtime(jd);  // degrees
```

### Houses

```typescript
// Placidus
const h: HouseResult = celestial.houses_ex(jd, 0, 48.85, 2.35, "P".charCodeAt(0));
console.log(`ASC=${h.ascmc[0].toFixed(2)}°  MC=${h.ascmc[1].toFixed(2)}°`);
// h.cusps[1..12] are the twelve house cusps

// With sidereal flag
const hSid = celestial.houses_ex(jd, 65536, 48.85, 2.35, "P".charCodeAt(0));
```

**House system codes:** `"P".charCodeAt(0)` Placidus · `"K"` Koch · `"E"` Equal · `"W"` Whole-Sign · `"O"` Porphyry · `"R"` Regiomontanus

### Sidereal positions

```typescript
celestial.set_sid_mode(1, 0, 0);  // SIDM_LAHIRI
const moonSid = celestial.calc_ut(jd, 1, 2 | 65536);  // FLG_BUILTIN | FLG_SIDEREAL
console.log(`Moon (Lahiri) = ${moonSid.lon.toFixed(4)}°`);

const ayan: number = celestial.ayanamsa_ut(jd);
```

---

## Moon phases

```typescript
const phase  = celestial.moon_phase(jd);           // number (phase variant)
const illum  = celestial.moon_illumination(jd);    // 0.0–1.0
const elong  = celestial.moon_elongation(jd);      // 0°–360°
const info   = celestial.moon_phase_info(jd);

console.log(`${info.phase_name}  ${(info.illumination * 100).toFixed(1)}%`);

const newMoon  = celestial.next_new_moon(jd);
const fullMoon = celestial.next_full_moon_phase(jd);
```

---

## Hellenistic / Persian

```typescript
const sun = celestial.calc_ut(jd, 0, 2 | 256);
const h   = celestial.houses_ex(jd, 0, 48.85, 2.35, "P".charCodeAt(0));

// Day or night chart
const isDay: boolean = celestial.is_day_chart(sun.lon, h.cusps);

// Egyptian terms ruler (returns planet index 0–6)
const ruler: number = celestial.egyptian_terms_ruler(sun.lon);

// Chaldean decan ruler
const decan: number = celestial.decan_ruler(sun.lon);

// Triplicity rulers [day, night, participating]
const [dayR, nightR, partR]: number[] = celestial.triplicity_rulers(sun.lon);

// Full dignity [dignityName, score]
const [dignityName, score]: [string, number] = celestial.full_dignity(0, sun.lon, isDay);

// Almuten [bodyRaw, score]
const [almutenBody, almutenScore]: [number, number] = celestial.almuten(sun.lon, isDay);

// Firdaria periods → array of [majorRaw, minorRaw, startJd, endJd, years]
const periods: number[][] = celestial.firdaria(jd, isDay, 75.0);
for (const [major, minor, start, end, years] of periods.slice(0, 3)) {
  console.log(`Major: ${major}  Minor: ${minor}  Start JD: ${start.toFixed(1)}`);
}

// Annual profection → [houseNumber, profectedLon]
const [houseNum, profLon]: [number, number] = celestial.annual_profection(h.cusps, 35);
console.log(`Age 35 → House ${houseNum} (${profLon.toFixed(2)}°)`);
```

---

## Chinese astrology (Ba Zi)

```typescript
const sun = celestial.calc_ut(jd, 0, 2);

// Four Pillars → [[stemName, branchName, animal, stemElement, branchElement, polarity], ×4]
const pillars: string[][] = celestial.four_pillars(jd, 9.0, sun.lon);
const [year, month, day2, hour] = pillars;
console.log(`Year pillar: ${year[0]} ${year[1]} (${year[2]})`);

// Solar term position → [currentIdx, degInto, nextIdx, degToNext]
const [curIdx, degInto, nextIdx, degToNext]: number[] = celestial.solar_term_position(sun.lon);
```

---

## Mesoamerican calendars

```typescript
// Aztec Tonalpohualli → [trecena, signIdx, nahuatlName, english]
const [trecena, signIdx, nahuatl, english]: [number, number, string, string] =
  celestial.tonalpohualli(jd);
console.log(`Tonalpohualli: ${trecena} ${nahuatl} (${english})`);

// Aztec Xiuhpohualli → [monthIdx, day, name, english]
const [monthIdx, dayNum, monthName, monthEn]: [number, number, string, string] =
  celestial.xiuhpohualli(jd);

// Maya Tzolkin → [trecena, signIdx, mayanName, english]
const tzolkin: [number, number, string, string] = celestial.tzolkin(jd);

// Maya Haab → [monthIdx, day, name]
const haab: [number, number, string] = celestial.haab(jd);

// Calendar Round → [tzTrecena, tzSign, haabDay, haabMonth]
const cr: [number, number, number, number] = celestial.calendar_round(jd);
```

---

## Solar (Schwabe) cycle

```typescript
import { solarCycle, grandSolarEpoch, cycleNickname } from "celestial-js";

// Inside the numbered Schwabe cycles (1755 → ~2030):
const info = solarCycle(2_451_545.0);
if (info === null) {
  console.log("outside numbered cycles");
} else {
  console.log(`Cycle ${info.cycleNum} — ${info.phaseName} ` +
              `(${info.yearsSinceMin.toFixed(1)}y in)`);
  if (info.nickname) console.log(`  aka: ${info.nickname}`);
  if (info.grandEpoch) console.log(`  grand epoch: ${info.grandEpoch}`);
}

// Centuries-scale label, callable for any JD (returns null when normal):
const epoch = grandSolarEpoch(2_341_973.0); // 1700 → "Maunder Minimum"

// Informal cycle nicknames (null for cycles without one):
console.log(cycleNickname(19)); // "the Great Cycle"
console.log(cycleNickname(20)); // null
```

**`SolarCycleInfo` fields:** `cycleNum`, `phase`, `phaseName`, `minJd`,
`maxJd`, `nextMinJd`, `yearsSinceMin`, `nickname?`, `grandEpoch?`.

Returns `null` for non-finite input or dates outside cycles 1..=25.
Phase classification follows the Waldmeier effect (asymmetric rise / decline).

---

## Indigenous / Egyptian

```typescript
const sun = celestial.calc_ut(jd, 0, 2);

// Medicine Wheel → [animal, element, clan, season]
const [animal, element, clan, season]: string[] = celestial.medicine_wheel_totem(sun.lon);
console.log(`Totem: ${animal} — ${element} element, ${clan} clan, ${season}`);

// Egyptian decan → [idx, decanName, risingStar]
const [decanIdx, decanName, risingStar]: [number, string, string] =
  celestial.egyptian_decan(sun.lon);
console.log(`Decan ${decanIdx + 1}: ${decanName} (${risingStar})`);
```


---

## Angle transits

```typescript
// Natal angle transits (ic / asc / dsc added alongside existing mcTransitUt)
const jdMc  = celestial.mcTransitUt(planet, jdNatal, jdStart, lat, lon, hsys, flags);
const jdIc  = celestial.icTransitUt(planet, jdNatal, jdStart, lat, lon, hsys, flags);
const jdAsc = celestial.ascTransitUt(planet, jdNatal, jdStart, lat, lon, hsys, flags);
const jdDsc = celestial.dscTransitUt(planet, jdNatal, jdStart, lat, lon, hsys, flags);

// Aspect to a house cusp — returns [jd] or null
const hit = celestial.nextAspectCusp(planet, 90.0, 10, jdStart,
                                      lat, lon, hsys, false, flags);
```


---

## Error handling

Functions throw `Error` with a descriptive message on failure:

```typescript
try {
  const pos = celestial.calc_ut(jd, 0, 2);
} catch (e) {
  console.error("Calculation failed:", (e as Error).message);
}
```

---

## TypeScript strict mode

All exported functions have full TypeScript declarations in `index.d.ts`. The
binding was built with napi-rs; function names use `snake_case` to match the
Rust/Python APIs.

```typescript
import type { PlanetPos, HouseResult } from "celestial-js";
```

---

## Legacy / compatibility aliases

```javascript
import {
  degnorm,          // → normDeg()
  difdeg2n,         // → difDegSigned()
  getAyanamsa,      // → ayanamsa()
  getAyanamsaName,  // → ayanamsaName()
  nextSabbatName,   // → nextSabbat() — name only
  nextFullMoon,     // → nextFullMoonAfter()
  solcrossUt,       // finds when Sun crosses a given degree
} from "celestial-js";
```

## Recent additions — 19 new functions

### ISO 8601 week

```javascript
import { isoWeek, dayOfYear, weeksInIsoYear } from "celestial-js";

const [isoYear, week] = isoWeek(2456293.0);    // [2012, 52] for 2012-12-31
const dow              = dayOfYear(2024, 3, 15);
const wks              = weeksInIsoYear(2020); // 53
```

### Maya Long Count

```javascript
import { mayaLongCount, mayaLongCountStr } from "celestial-js";

const [b, k, t, u, ki] = mayaLongCount(2456283.0);  // 2012-12-21 → [13,0,0,0,0]
const s                = mayaLongCountStr(2451545.0); // "12.19.6.15.2"
```

### Yallop crescent visibility

```javascript
import { yallopQ, bestTimeMethod } from "celestial-js";

const jdBest = bestTimeMethod(jdSunset, jdMoonset);
const [q, classCode] = yallopQ(arcvDeg, arclDeg, sdArcmin);
// classCode is the ASCII code of 'A'..'F' (65..70)
const cls = String.fromCharCode(classCode);
```

### Coptic / Ethiopic (feature: `calendar-traditions`)

```javascript
import {
  copticToJd, jdToCoptic, ethiopicToJd, jdToEthiopic,
  isCopticLeapYear, copticMonthDays,
} from "celestial-js";

const jd            = copticToJd(1740, 1, 1);
const [y, m, d]     = jdToCoptic(jd);
const isLeap        = isCopticLeapYear(1739);     // true
const days          = copticMonthDays(1739, 13);  // 6
```

### Zoroastrian Fasli (feature: `calendar-traditions`)

```javascript
import { fasliNowruzJd, jdToFasli } from "celestial-js";

const jdNowruz = fasliNowruzJd(2024);   // number | null
const date     = jdToFasli(2460400.0);  // [fasliYear, monthIndex, day] | null
```

### Tibetan Phugpa (feature: `calendar-traditions`)

```javascript
import { losarJd, tibetanYearName } from "celestial-js";

const jdLosar                         = losarJd(2024);
const [cycle, yic, el, gender, animal] = tibetanYearName(2024);
// ["17", "38", "Wood", "Male", "Dragon"]
```

### Vietnamese Âm Lịch

```javascript
import { vietnameseMonthStartJd, vietnameseChineseBoundaryDiffers } from "celestial-js";

const jdMonthStart = vietnameseMonthStartJd(2460000.0);
const diverges     = vietnameseChineseBoundaryDiffers(2460000.0);
```
