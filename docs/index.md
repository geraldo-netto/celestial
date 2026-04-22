# celestial — Documentation

**celestial** is a pure-Rust Swiss Ephemeris port with CLI, Python, JavaScript and PHP bindings,
supporting eight astrological traditions across 24 chart types.

## Documents

| File | Contents |
|---|---|
| [api_reference.md](api_reference.md) | Complete function reference — all 8 phases, all return types |
| [api_reference.md](api_reference.md) | Rust crate API with extended code examples |
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
│   │       ├── hellenistic.rs  Phase 5: dignity, almuten, firdaria
│   │       ├── chinese.rs      Phase 6: Ba Zi, solar terms
│   │       ├── mesoamerican.rs Phase 7: Tonalpohualli, Tzolkin, Haab
│   │       └── indigenous.rs   Phase 8: Medicine Wheel, Egyptian decans
├── cli/src/cmd/render/         24 SVG chart builders, one per tradition
├── bindings/python/            PyO3 — 152 functions + celestial_py.pyi stubs
├── bindings/js/                napi-rs — 155 functions + index.d.ts
├── bindings/php/               ext-php-rs — 130 functions + phpstan stubs
└── tests/fixtures/             reference_values.json — cross-language test fixture
```

The public API is a **single flat namespace** — `use celestial_core::*` gives you everything.

---

## CLI & plugins

### Plugin architecture

Any executable named `celestial-<n>` on `$PATH` becomes a first-class subcommand:

```bash
celestial --list-plugins              # discover all installed plugins
celestial synastry --date 1985-01-01  # runs celestial-synastry if on PATH
```

---

## Benchmarks

```bash
# Rust micro-benchmarks
cargo bench --package celestial-core

# Precision + performance comparison vs pyephem and astropy
pip install astropy pyephem
python3 benches/precision_comparison.py
```

Precision vs Meeus benchmarks confirmed at ~3.2″ Sun / ~0.7″ Moon. `SYNODIC_MONTH = 29.530_588_853` days.
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
