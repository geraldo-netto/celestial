# celestial — Documentation

**celestial** is a pure-Rust Swiss Ephemeris port with CLI, Python, JavaScript and PHP bindings,
supporting eight astrological traditions across 27 chart types.

## Documents

| File | Contents |
|---|---|
| [api_reference.md](api_reference.md) | Complete Rust API reference — return types, code examples |
| [building.md](building.md) | Build guide — how to build each component, run all tests, lint everything |
| [python.md](python.md) | Python (PyO3) binding — installation, all functions, constants |
| [javascript.md](javascript.md) | JavaScript / TypeScript (napi-rs) — typed API, full guide |
| [php.md](php.md) | PHP (ext-php-rs) — installation, phpstan stubs, all functions |

The main [README.md](../README.md) contains a full Table of Contents, CLI reference, and
quick-start examples for all languages.

---


## Recent additions

The engine now covers **19 additional functions** across 7 features — see
[`api_reference.md`](api_reference.md) for signatures.

| Feature | Highlights |
|---|---|
| ISO 8601 week | `iso_week`, `day_of_year`, `weeks_in_iso_year` |
| Maya Long Count | `maya_long_count`, dotted notation `"13.0.0.0.0"` |
| Yallop crescent visibility | `yallop_q` — q-value + class A–F for Islamic lunar sighting |
| Coptic / Ethiopic calendar | JD ↔ date, leap rule, 13-month epagomenal structure |
| Zoroastrian Fasli | New Year locked to astronomical vernal equinox (Nowruz) |
| Tibetan Phugpa | Losar (New Year) + Rabjung cycle year names |
| Vietnamese Âm Lịch | UTC+7 month boundaries — diverges from Chinese ~4% of days |

All 194 functions are exported from **Python**, **JavaScript**, and **PHP** (verify with `cargo xtask parity`).

## Feature flags

`celestial-core` can be built minimal (no timezone table, no liturgical calendars):

```toml
celestial-core = { path = "...", default-features = false }
```

Two opt-out features, both on by default:
- `timezone` — IANA abbreviation table (`TZ_TABLE`, `tz_abbr_find`)
- `calendar-traditions` — Jewish, Omer, Easter, Islamic, Nowruz/Bahá'í, Vesak,
  Sabbats/Esbats, Coptic/Ethiopic, Zoroastrian Fasli, Tibetan Phugpa

## Quick start

### Rust

```rust
use celestial_core::*;

let jd  = julday(2025, 3, 20, 9.0, GREG_CAL);
let pos = calc_ut(jd, SUN, FLG_SPEED).unwrap();
println!("Sun longitude: {:.4}°", pos.lon);
```

### CLI

```bash
celestial calc --date 2025-03-20
celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg
```

Any executable named `celestial-<n>` on `$PATH` becomes a first-class subcommand:

```bash
celestial --list-plugins              # discover all installed plugins
celestial synastry --date 1985-01-01  # runs celestial-synastry if on PATH
```

See [README.md](../README.md#cli) for the full command reference and all 27 chart types.

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

---

## Workspace layout

```
celestial/
├── core/
│   ├── src/
│   │   ├── lib.rs              crate root — re-exports all domain modules
│   │   ├── body/               Body, CalcFlags, HouseSystem, SiderealMode
│   │   ├── position.rs         calc_ut, calc_many, CalcOptions, fixed stars
│   │   ├── time.rs             julday, revjul, UTC conversion
│   │   ├── houses.rs           house cusp systems
│   │   ├── motion.rs           crossings, rise/set, eclipses, RiseTransOptions
│   │   ├── moon.rs             phases, illumination, esbats, sabbats
│   │   ├── chart.rs            aspects, AspectOrbs, progressions, traditions
│   │   ├── vedic.rs            Jyotish, Panchānga
│   │   ├── geo.rs              coordinate formatting, timezones
│   │   ├── calendar/           Hebrew, Christian, Islamic, Hindu, Buddhist, Persian, Celtic
│   │   ├── constants.rs        numeric constants (body indices, flags, modes)
│   │   ├── error.rs            structured Error enum (#[non_exhaustive])
│   │   └── functions/          pub(crate) implementation — 28 submodules
│   │       ├── hellenistic.rs  dignity, almuten, firdaria
│   │       ├── chinese.rs      Ba Zi, solar terms
│   │       ├── mesoamerican.rs Tonalpohualli, Tzolkin, Haab
│   │       └── indigenous.rs   Medicine Wheel, Egyptian decans
├── cli/src/cmd/render/         27 chart-type SVG builders + 6 calendar overlays
├── bindings/python/            PyO3 — 194 functions + celestial_py.pyi stubs (201 typed)
├── bindings/js/                napi-rs — 194 functions + index.d.ts (276 typed — structs + consts)
├── bindings/php/               ext-php-rs — 194 functions + phpstan-stubs.php (402 symbols)
└── tests/fixtures/             reference_values.json — cross-language test fixture
```

The public API is a **single flat namespace** — `use celestial_core::*` gives you everything.

---

## Benchmarks

```bash
# Rust micro-benchmarks
cargo bench --package celestial-core

# Precision + performance comparison vs pyephem and astropy
pip install astropy pyephem
python3 benches/precision_comparison.py
```

Precision vs Meeus benchmarks confirmed at ~3.2″ Sun / ~0.7″ Moon. Jupiter / Saturn carry ~5′–30′ residuals against canonical ephemerides — see [README.md#accuracy](../README.md#accuracy) for the full per-body table. `SYNODIC_MONTH = 29.530_588_853` days.
## CI pipelines

Five independent pipelines, each triggered on changes to its crate or `core/`:

| Pipeline | Jobs |
|---|---|
| **celestial-core** | `lint` (fmt + clippy) → `test` (1101 tests) ‖ `fuzz` (84 suites, ~1M property checks) |
| **celestial-cli** | `lint` (clippy) → `test` (154 tests) → `build` (3 OS) |
| **celestial-python** | `lint-rs` ‖ `lint-py` (black + ruff) → `test` (216 pure-logic) → `build` (maturin wheel) |
| **celestial-js** | `lint-rs` ‖ `lint-ts` (eslint + tsc) → `test` (142 pure-logic) → `build` (napi-rs addon) |
| **celestial-php** | `lint-rs` → `build` (ext-php-rs + pure-logic tests, PHP 8.1) |

`lint-rs` and `lint-py`/`lint-ts` always run in parallel with strict scope.

---
