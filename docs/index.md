# celestial — Documentation

**celestial** is a pure-Rust Swiss Ephemeris port with CLI, Python, JavaScript and PHP bindings,
supporting eight astrological traditions across 24 chart types.

## Documents

| File | Contents |
|---|---|
| [api_reference.md](api_reference.md) | Complete function reference — all 8 phases, all return types |
| [rust.md](rust.md) | Rust crate API with extended code examples |
| [python.md](python.md) | Python (PyO3) binding — installation, all functions, constants |
| [javascript.md](javascript.md) | JavaScript / TypeScript (napi-rs) — typed API, full guide |
| [php.md](php.md) | PHP (ext-php-rs) — installation, phpstan stubs, all functions |

The main [README.md](../README.md) contains a full Table of Contents, CLI reference, and
quick-start examples for all languages.

---

## Quick start

### Rust

```rust
use celestial_core::*;

let jd  = julday(2025, 3, 20, 9.0, GREG_CAL);
let pos = calc_ut(jd, SUN, FLG_SPEED).unwrap();
println!("Sun longitude: {:.4}°", pos.lon);
```

### Python

```python
import celestial_py as celestial
jd  = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL)
pos = celestial.calc_ut(jd, 0, 256)  # SUN, SEFLG_SPEED
print(f"Sun longitude: {pos.lon:.4f}°")
```

### JavaScript / TypeScript

```typescript
import * as celestial from "celestial-js";
const pos = celestial.calc_ut(celestial.julday(2025, 3, 20, 9.0, 1), 0, 256);
console.log(`Sun longitude: ${pos.lon.toFixed(4)}°`);
```

### PHP

```php
$jd  = celestial_julday(2025, 3, 20, 9.0, GREG_CAL);
$pos = celestial_calc_ut($jd, SE_SUN, FLG_SPEED);
echo "Sun longitude: " . round($pos[0], 4) . "°\n";
```

### CLI

```bash
celestial calc --date 2025-03-20
celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg
```

---

## Workspace layout

```
celestial-workspace/
├── core/                        celestial-core — pure-Rust engine, zero dependencies
│   └── src/
│       ├── astronomy/           IAU 2000B nutation, VSOP87, 3-pass light-time, rise/set
│       └── functions/
│           ├── chart.rs         Phases 1–4 (Western + Vedic helpers)
│           ├── hellenistic.rs   Phase 5: dignity, almuten, firdaria
│           ├── chinese.rs       Phase 6: Ba Zi, solar terms
│           ├── mesoamerican.rs  Phase 7: Tonalpohualli, Tzolkin, Haab
│           ├── indigenous.rs    Phase 8: Medicine Wheel, Egyptian decans
│           ├── vedic.rs         Jyotish utilities
│           ├── calc.rs          calc_ut, calc_many (parallel), nutation
│           └── ...              aspects, eclipses, crossings, calendars
│   └── tests/
│       ├── new_features_test.rs Phases 2–4 core tests
│       ├── phase1_test.rs       Phase 1: minor aspects, antiscia, dignities
│       ├── phase5_test.rs       Phase 5: Hellenistic / Persian
│       ├── phase6_test.rs       Phase 6: Chinese
│       ├── phase7_test.rs       Phase 7: Mesoamerican
│       └── phase8_test.rs       Phase 8: Indigenous / Egyptian
├── cli/                         celestial — 11 sub-commands + plugin system
│   └── src/cmd/render/
│       ├── mod.rs               Dispatch, shared helpers (~3200 lines)
│       ├── western.rs           Natal, cosmogram, returns, progressions, bi-wheel
│       ├── specialist.rs        Dial, composite, tri-wheel, ephemeris, local-space
│       ├── vedic.rs             Rasi, navamsa, dasha, ashtakavarga, shadbala
│       ├── hellenistic.rs       Hellenistic overlay, firdaria, profection
│       ├── chinese.rs           Ba Zi (Four Pillars)
│       ├── mesoamerican.rs      Aztec + Maya calendar chart
│       └── indigenous.rs        Medicine Wheel + Egyptian decans
├── bindings/
│   ├── python/                  PyO3 — 152 exported functions
│   ├── js/                      napi-rs — 155 exported functions + index.d.ts
│   └── php/                     ext-php-rs — 130 exported functions
├── fuzz/                        53 property-test suites (cargo run)
├── tests/fixtures/              reference_values.json — cross-language canonical values
└── docs/                        Extended documentation (see Documentation section)
```

The public API is a **single flat namespace** — `use celestial_core::*` gives you everything.

---

---

## Plugin architecture

Any executable named `celestial-<n>` on `$PATH` becomes a first-class subcommand:

```bash
celestial --list-plugins              # discover all installed plugins
celestial synastry --date 1985-01-01  # runs celestial-synastry if on PATH
```

---

## CI pipelines

Five independent pipelines, each triggered on changes to its crate or `core/`:

| Pipeline | Jobs |
|---|---|
| **celestial-core** | `lint` (fmt + clippy) → `test` (737 unit tests) ‖ `fuzz` (53 suites) |
| **celestial-cli** | `lint` (clippy) → `test` (81 tests) → `build` (3 OS) |
| **celestial-python** | `lint-rs` ‖ `lint-py` (black + ruff) → `test` (245 pure-logic) → `build` (maturin wheel) |
| **celestial-js** | `lint-rs` ‖ `lint-ts` (eslint + tsc) → `test` (162 pure-logic) → `build` (napi-rs addon) |
| **celestial-php** | `lint-rs` → `build` (ext-php-rs + pure-logic tests, PHP 8.1) |

`lint-rs` and `lint-py`/`lint-ts` always run in parallel with strict scope.

---

---

## License

AGPL-3.0, matching the Swiss Ephemeris it emulates.
