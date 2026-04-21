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

## Architecture

```
celestial-workspace/
├── core/src/functions/
│   ├── chart.rs         Phases 1–4
│   ├── hellenistic.rs   Phase 5: Hellenistic / Persian
│   ├── chinese.rs       Phase 6: Chinese astrology
│   ├── mesoamerican.rs  Phase 7: Mesoamerican calendars
│   └── indigenous.rs    Phase 8: Indigenous / Egyptian
├── cli/src/cmd/render/  24 SVG chart builders, one per tradition
├── bindings/python/     PyO3 — 152 functions
├── bindings/js/         napi-rs — 155 functions + TypeScript types
├── bindings/php/        ext-php-rs — 130 functions + phpstan stubs
└── tests/fixtures/      reference_values.json — cross-language test fixture
```

---

## License

AGPL-3.0, matching the Swiss Ephemeris it emulates.
