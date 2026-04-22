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

# Property-based fuzz tests (all 67 suites, ~30 s)
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

# Tests (81 tests)
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

# Pure-logic tests (172 tests, no native build required)
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
| `celestial-core` | `core/**`, `fuzz/**` | lint → test (776) ‖ fuzz (67 suites) |
| `celestial-cli` | `cli/**`, `core/**` | lint → test (81) → release build |
| `celestial-python` | `bindings/python/**`, `core/**` | clippy → ruff/mypy → pytest |
| `celestial-js` | `bindings/js/**`, `core/**` | clippy → tsc/eslint → node tests |
| `celestial-php` | `bindings/php/**`, `core/**` | clippy → phpstan |

| `binding-parity` | `bindings/**`, `xtask/**` | parity check → test-stubs validation |

See [`.github/workflows/`](../.github/workflows/) for the full YAML.
