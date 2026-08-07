# celestial — Build Guide

This guide covers building each component independently and all together,
with all test, lint, and formatting commands.

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust + Cargo | stable ≥ 1.82 (workspace MSRV) | `curl https://sh.rustup.rs -sSf \| sh` (Linux/macOS) · [rustup.rs](https://rustup.rs) (Windows) |
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

# Property-based fuzz tests (all 84 suites, ~1M property checks, ~15 s)
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

# Tests (293 tests)
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

# Tests (216 pure-logic passing; extension tests skipped without a built wheel)
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

# Pure-logic tests (162 passing — no built addon required)
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
celestial render --chart-type natal --date "1990-05-15 14:30" --timezone UTC --lat 48.85 --lon 2.35

# Or separated — easier for scripts:
celestial render --chart-type natal --date 1990-05-15 --time 14:30 --timezone UTC --lat 48.85 --lon 2.35

# Seconds accepted; display truncates to HH:MM:
celestial render --date 1990-05-15 --time 14:30:45 --timezone UTC
```

---

## xtask — developer automation

```bash
# Check that Python, JS, and PHP binding all export identical functions
cargo xtask parity

# Core→binding coverage + cross-binding arity + constant parity.
# Fails if a core public fn (pub use in core/src/lib.rs) is bound by no binding
# and is not listed in xtask/core_unbound_allow.txt, if bindings disagree on a
# fn's parameter count (unless in xtask/arity_allow.txt), or if a constant is not
# exported by every binding (unless in xtask/const_allow.txt).
cargo xtask coverage
cargo xtask coverage --write-allow   # re-baseline the intentionally-unbound list

# Return-shape contract — snapshot of each binding's return KIND per fn.
cargo xtask shapes            # regenerate xtask/binding_shapes.txt
cargo xtask shapes --check    # CI gate: fail if a return shape drifted

# Generated binding API reference (constant values + per-language signatures).
cargo xtask apidoc            # regenerate docs/generated/binding_api.md
cargo xtask apidoc --check    # CI gate: fail if out of sync with source

# Native binding numeric contract generated from celestial-core.
cargo xtask golden            # regenerate tests/fixtures/binding_golden.json
cargo xtask golden --check    # CI gate: fail if the fixture is out of sync

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
| `celestial-core` | `core/**`, `fuzz/**` | lint → test (1174) ‖ fuzz (84 suites, ~1M checks) |
| `celestial-cli` | `cli/**`, `core/**` | lint → test (293) → release build |
| `celestial-python` | `bindings/python/**`, `core/**` | clippy → ruff/mypy → pytest (216 pure-logic) |
| `celestial-js` | `bindings/js/**`, `core/**` | clippy → tsc/eslint → node tests (162 pure-logic) |
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
| `calendar-traditions` | ✓ | Jewish, Omer, Easter, Islamic, Nowruz/Bahá'í, Vesak, Sabbats & Esbats, Coptic/Ethiopic, Zoroastrian Fasli, Tibetan calendar helpers | ~2 900 lines |

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
