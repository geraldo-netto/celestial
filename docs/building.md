# celestial — Build Guide

This guide covers building each component independently and all together,
with all test, lint, and formatting commands.

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust + Cargo | stable ≥ 1.75 | `curl https://sh.rustup.rs -sSf \| sh` (Linux/macOS) · [rustup.rs](https://rustup.rs) (Windows) |
| Node.js | ≥ 18 | <https://nodejs.org> |
| Python | ≥ 3.8 | <https://python.org> |
| maturin | ≥ 1.4 | `pip install maturin` |
| PHP | ≥ 8.1 (optional) | system package manager |

---

## Full workspace build

```bash
# Build every Rust crate (core + CLI + all bindings)
cargo build

# Release build
cargo build --release

# Build only the crates used in production (exclude fuzz)
cargo build --workspace --exclude celestial-fuzz
```

---

## `celestial-core` — pure-Rust engine

```bash
# Build
cargo build --package celestial-core

# Lint (errors on correctness, suspicious, perf)
cargo clippy --package celestial-core \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Format check
cargo fmt --package celestial-core -- --check

# Apply formatting
cargo fmt --package celestial-core

# Unit + integration + doctests (single-threaded for deterministic output)
cargo test --package celestial-core -- --test-threads=1

# Run only a specific test file
cargo test --package celestial-core --test integration_tests -- --test-threads=1
cargo test --package celestial-core --test unit_tests       -- --test-threads=1
cargo test --package celestial-core --test wheel_test       -- --test-threads=1
cargo test --package celestial-core --test hellenistic_test -- --test-threads=1
cargo test --package celestial-core --test chinese_test     -- --test-threads=1
cargo test --package celestial-core --test mesoamerican_test -- --test-threads=1
cargo test --package celestial-core --test indigenous_test  -- --test-threads=1

# Property-based fuzz tests (all 74 suites, ~60 s)
cargo run --manifest-path fuzz/Cargo.toml

# Benchmarks
cargo bench --package celestial-core
```

---

## `celestial-cli` — command-line tool

```bash
# Build
cargo build --package celestial-cli

# Release build + install to ~/.cargo/bin/celestial
cargo install --path cli

# Lint
cargo clippy --package celestial-cli \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Tests (104 tests)
cargo test --package celestial-cli

# Release binary only
cargo build --release --package celestial-cli
# Output: target/release/celestial
```

---

## Python binding (`celestial-py`)

```bash
cd bindings/python

# Development install (editable wheel, fastest iteration)
pip install maturin
maturin develop                                       # Linux / macOS
maturin develop --target x86_64-pc-windows-msvc       # Windows (MSVC) 
maturin develop --target x86_64-pc-windows-gnu        # Windows (MinGW)

# Release wheel
maturin build --release                               # Linux / macOS
maturin build --release --target x86_64-pc-windows-msvc  # Windows
# Output: target/wheels/celestial_py-*.whl

# Install the built wheel
pip install target/wheels/celestial_py-*.whl

# Lint — Rust side
cargo clippy --package celestial-py \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Lint — Python side
pip install ruff mypy
ruff check python/
mypy python/ --ignore-missing-imports

# Type-check the .pyi stubs
mypy python/celestial_py/celestial_py.pyi --ignore-missing-imports

# Format — Python side
black python/

# Tests (245 passing, 21 skipped — skipped tests require a built wheel)
pip install pytest
python3 -m pytest python/celestial_py/tests/pure_logic_test.py -v

# Full test matrix (requires built wheel)
python3 -m pytest python/celestial_py/tests/ -v
```

---

## JavaScript / TypeScript binding (`celestial-js`)

```bash
cd bindings/js

# Install Node dependencies
npm ci                     # or: npm install

# Development build (debug, faster)
npm run build:dev          # calls: napi build --platform

# Release build
npm run build              # calls: napi build --platform --release
# Output: celestial.*.node

# Lint — Rust side
cargo clippy --package celestial-js \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Type-check TypeScript
npx tsc --noEmit           # or: npm run typecheck

# Lint TypeScript
npx eslint tests/          # or: npm run lint

# Pure-logic tests (216+ tests across JS/PHP pure-logic suites)
node tests/pure_logic.test.mjs

# Full test suite (requires built .node addon)
npm test
```

---

## PHP binding

```bash
cd bindings/php

# Build (standalone workspace — uses its own Cargo.toml)
cargo build --manifest-path bindings/php/Cargo.toml

# Release build
cargo build --manifest-path bindings/php/Cargo.toml --release
# Output: target/release/libcelestial.so  (Linux)
#         target/release/libcelestial.dylib (macOS)
#         target/release/celestial.dll    (Windows)

# Install the extension
# Add to php.ini:  extension=/path/to/libcelestial.so

# Lint — Rust side
cargo clippy --manifest-path bindings/php/Cargo.toml \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Static analysis — PHP side (requires phpstan and a built extension)
phpstan analyse tests/ --level=8 \
  --configuration=phpstan.neon \
  --autoload-file=phpstan-stubs.php
```

---

## Run all tests at once

```bash
# From the workspace root — fastest way to check everything
cargo test --workspace --exclude celestial-fuzz -- --test-threads=1  # all Rust tests
cargo run  --manifest-path fuzz/Cargo.toml                           # property tests

cd bindings/python
python3 -m pytest python/celestial_py/tests/pure_logic_test.py -q   # Python tests

cd ../js
node tests/pure_logic.test.mjs                                        # JS tests
```

---

## Lint everything at once

```bash
# Rust — entire workspace
cargo clippy --workspace --exclude celestial-fuzz \
  -- -D clippy::correctness -D clippy::suspicious -D clippy::perf

# Format check — entire workspace
cargo fmt --all -- --check

# Python
cd bindings/python
ruff check python/
black --check python/

# TypeScript
cd ../js
npx tsc --noEmit
npx eslint tests/
```


### Date and time input

All commands accepting `--date` also accept a separate `--time` flag:

```bash
# Equivalent — time embedded in date string:
celestial render --chart-type natal --date "1990-05-15 14:30" --lat 48.85 --lon 2.35

# Or separated — easier for scripts:
celestial render --chart-type natal --date 1990-05-15 --time 14:30 --lat 48.85 --lon 2.35

# Seconds accepted; display truncates to HH:MM:
celestial render --date 1990-05-15 --time 14:30:45  # shows "14:30 UT"
```

---

## xtask — developer automation

```bash
# Check that Python, JS, and PHP binding all export identical functions
cargo xtask parity

# Generate stub skeletons for functions missing from a binding
cargo xtask codegen            # preview only
cargo xtask codegen --apply    # write into binding files (review diff before committing)

# Regenerate bindings/php/phpstan-stubs.php from the PHP binding source
cargo xtask stubs

# Validate phpstan-stubs.php for PHP 8.0 syntax errors (no PHP binary required)
cargo xtask test-stubs
```


---

## CI pipeline summary

Five independent GitHub Actions pipelines each trigger on changes to their
crate or `core/`:

| Pipeline | Trigger path | Jobs |
|---|---|---|
| `celestial-core` | `core/**`, `fuzz/**` | lint → test (514) ‖ fuzz (74 suites, ~1M checks) |
| `celestial-cli` | `cli/**`, `core/**` | lint → test (81) → release build |
| `celestial-python` | `bindings/python/**`, `core/**` | clippy → ruff/mypy → pytest |
| `celestial-js` | `bindings/js/**`, `core/**` | clippy → tsc/eslint → node tests |
| `celestial-php` | `bindings/php/**`, `core/**` | clippy → phpstan |

| `binding-parity` | `bindings/**`, `xtask/**` | parity check → test-stubs validation |

See [`.github/workflows/`](../.github/workflows/) for the full YAML.

---

## Feature flags (`celestial-core`)

`celestial-core` uses Cargo feature flags to keep the default build full-featured
while allowing minimal builds (e.g. WASM, embedded, pure-astronomy tools) to opt out
of large optional modules.

| Feature | Default | What it adds | Size impact |
|---|---|---|---|
| `timezone` | ✓ | IANA timezone abbreviation table (203 entries, `TZ_TABLE`, `tz_abbr_find`) | ~1 400 lines |
| `calendar-traditions` | ✓ | Jewish, Omer, Easter, Islamic, Nowruz/Bahá'í, Vesak, Sabbats & Esbats, Coptic/Ethiopic, Zoroastrian Fasli, Tibetan Phugpa | ~2 900 lines |

### Usage

```toml
# Default — all features on (recommended for CLI and language bindings):
celestial-core = { path = "../core" }

# Minimal — pure astronomical calculations only (planets, houses, eclipses, aspects):
celestial-core = { path = "../core", default-features = false }

# Minimal + timezone lookup only:
celestial-core = { path = "../core", default-features = false, features = ["timezone"] }

# Minimal + religious calendars only:
celestial-core = { path = "../core", default-features = false, features = ["calendar-traditions"] }
```

### What's always included (no feature flag)

The astronomy core is always compiled regardless of features:

- **Calculations**: `calc`, `calc_ut`, `calc_many`, `fixstar_ut`
- **Houses**: `houses`, `houses_ex`, `houses_armc`, all house systems
- **Positions & motion**: `position`, `speed`, `retrograde_station_ut`, `next_aspect`
- **Moon**: `moon_phase`, `moon_phase_info`, `moon_phases_for_month`, `moon_illumination`
- **Eclipses**: `sol_eclipse_when_glob`, `lun_eclipse_when`, `sol_eclipse_how`
- **Vedic**: `ayanamsa`, `long_to_nakshatra`, `vimshottari_dasha`, `panchanga`
- **Hellenistic**: `full_dignity`, `almuten`, `annual_profection`, `firdaria`
- **Geo / utils**: `norm_deg`, `diff_deg_signed`, `lon_to_sign`, `azalt`, `refrac`
- **Time**: `julday`, `revjul`, `jdnow`, `deltat`, `sidereal_time`
- **ISO 8601 week**: `iso_week`, `day_of_year`, `weeks_in_iso_year`
- **Maya Long Count**: `maya_long_count`, `maya_long_count_str`
- **Yallop crescent visibility**: `yallop_q`, `best_time_method`
- **Vietnamese Âm Lịch**: `vietnamese_month_start_jd`, `vietnamese_chinese_boundary_differs`



## xtask — developer automation

```bash
cargo xtask parity          # Check Python / JS / PHP bindings expose identical fn sets
cargo xtask codegen         # Preview stubs for functions missing from bindings
cargo xtask codegen --apply # Write the generated stubs into each binding
cargo xtask stubs           # Regenerate bindings/php/phpstan-stubs.php
cargo xtask test-stubs      # Validate phpstan-stubs.php for PHP 8.0 syntax
cargo xtask pyi             # Regenerate bindings/python/.../celestial_py.pyi
cargo xtask pyi --check     # Verify .pyi is up-to-date (CI gate)
cargo xtask dts             # Regenerate bindings/js/index.d.ts
cargo xtask dts --check     # Verify .d.ts is up-to-date (CI gate)
```

The `--check` variants exit non-zero if the generated file is out of sync,
so CI catches stale stubs before merge.
