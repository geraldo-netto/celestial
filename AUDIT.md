# Celestial — Code Audit

Rescan: 2026-05-17 (develop, post security + coverage hardening). One
table per category; stable IDs in the first column. Completed work is
removed (not listed). Rows are only **OPEN**, **DEFERRED** (real but
disproportionate to do safely now — its own effort), or **DECIDED**
(evaluated, intentionally not done — kept so a rescan doesn't re-flag).

| state | meaning |
|---|---|
| OPEN | actionable, bounded, byte-safe — do next |
| DEFERRED | real; needs an isolated session + precision soak |
| DECIDED | WONTFIX / DECLINED / N-A with rationale |

---

## 1. Complexity

Clean. No function exceeds CC ≈ 10 (peak ~6–7: `searches::years_diff`,
`pipeline::run`). High-arm `match` dispatch tables (houses/parse/
registry) are flat lookups — not flagged.

## 2. Code duplication

| id | location | issue | status |
|---|---|---|---|
| DUP-1 | bindings/{js,python,php}/src/lib.rs | ~201 per-export macro stubs ×3 (incl. the eclipse / houses / `*_many` families) | DEFERRED — napi/pyo3/php proc-macros, per-module registration and native return shapes can't be unified by a plain crate; only a spec-driven codegen over 3 *published* bindings on the precision path removes it. Shared `FfiError`/`pos6`/`pos6_tuple` halves already shipped. = ARCH-10/DP-4 |
| DUP-4 | bindings `revjul`/`revjul_hms` | 3 idiomatic return shapes (struct/tuple/map) | DECIDED — intentional per-language idioms downstream depends on; core call already shared. Normalizing = a published API break, not a dedupe |
| DUP-7 | cli/src/cmd/render/{vedic,specialist,hellenistic,calendar_wheel,omer_grid,…}.rs | `palette_with_defaults(&[…],&vars)` + title-default repeated in ~8 builders | OPEN — a `palette!` macro/helper; bounded, byte-identical. (The 3 self-titling builders already use `palette_vars`; these 8 differ in key sets / inject title upstream — low value, byte-safe.) |

DUP-5 (per-tradition wheel geometry) is N-A — distinct layout
constants, not duplication; shared halves already factored.

## 3. Performance (precision is the hard gate)

All hot/warm items are decided; each is regression-locked so a future
rewrite must reproduce current numbers.

| id | site | status |
|---|---|---|
| PERF-1 | engine.rs heliocentric-speed 3× VSOP | WONTFIX-as-stated; only precision-safe route is an analytic VSOP derivative (own soak). Locked: `perf1_heliocentric_mars_speed_regression_lock` |
| PERF-2/3 | searches.rs post-bisect `calc_ut`/`houses` | WONTFIX — scan strips SPEED; the post-bisect full-flag eval at the converged jd is authoritative, not redundant. Locked: `perf2345_search_regression_lock` |
| PERF-4 | searches.rs `bisect_retro_station` | N-A — already reuses the last loop sample; no recompute exists |
| PERF-5 | searches.rs `next_aspect_with2` dual scan | DECLINED — merge is a precision-sensitive rewrite for marginal gain |
| PERF-6 | engine.rs `compute_speed` ±0.5 d | WONTFIX — central diff is the minimal 2-eval 2nd-order form; forward/back loses precision |

Invariant: `calc` + moon phases + vedic/meso/bazi SVG stay
byte-identical to the 1986-05-30 PDF reference and the Diana
1961-07-01 chart.

## 4. Security

Clean — memory-safe, no `unsafe` in core/cli/bindings/ffi, FFI sound
(napi/pyo3/ext-php-rs), `plugin.rs` execs via arg array (no shell). All
eight previously-flagged items fixed and re-verified present:

| id | sev | fix | commit |
|---|---|---|---|
| SEC-1/2 | HIGH | `format::xml_escape` on title/palette/`--name` | `0d15c20` |
| SEC-3 | HIGH | UTF-8-safe `trunc_chars` | `0d15c20` |
| SEC-4 | HIGH | `toml` 0.4.10 → 0.8.23 | `f710c34` |
| SEC-5 | MED | config `out` confined (rel, no `..`) | `d014909` |
| SEC-6 | MED | MiniJinja `fuel` cap (50M) | `0ce4060` |
| SEC-7 | MED | `offsets.first()` guard | `95fa795` |
| SEC-8 | LOW | `day_of_week` jd sentinel | `fbd486c` |

## 5. Architecture / modularity / visibility

Dependency direction is correct (core → celestial-ffi → bindings; cli →
core+ffi). Render submodules use proper `pub(crate)`/`pub(super)`; no
over-exposed internals; no god-module besides the (already-split)
render tree. `core::Error` is `#[non_exhaustive]`.

| id | area | issue | status |
|---|---|---|---|
| ARCH-7 | core/src/lib.rs | 12 crate-root `pub use mod::*` globs | DEFERRED — load-bearing for core's own internal `crate::` paths through the precision compute; faithful de-glob ≈ exhaustive ~280-symbol mirror. Isolated effort |
| ARCH-8 | cli/src/cmd/render/mod.rs (~2.4k LOC) | residual = test module + wheel/format/tables/dignity helpers | DEFERRED — already a facade; safe extraction is many single-span-per-commit moves (bulk pass corrupts line math); low payoff vs precision-tree risk |
| ARCH-10 | bindings 201×3 stubs | no codegen | DEFERRED — = DUP-1/DP-4 |
| NV-1 | cli `ParseError` (parse.rs), `CliError` (error.rs) | not `#[non_exhaustive]` | OPEN (LOW) — internal-only today (converted early via `From`); add the attribute as cheap forward-proofing if either is ever surfaced through a binding |

## 6. Design pattern opportunities

| id | target | status |
|---|---|---|
| DP-1 | registry macro/trait over 28 `dispatch_*` | DISMISSED — bodies are heterogeneous (only `dispatch_natal` trivial); a macro is closure indirection over a clear hot-path adapter for ~140 LOC — net-negative |
| DP-2 | `JulianDay`/`Latitude`/… unit newtypes | DEFERRED — payoff needs threading through `calc_ut`/`houses_ex` = core-API rewrite on the precision path. Parse-layer newtypes already shipped (`7bfa598`) |
| DP-4 | binding codegen | DEFERRED — = ARCH-10/DUP-1 |
| DP-6 | SVG → MiniJinja templates | DEFERRED — the template half **conflicts with the byte-identical precision gate** (changes whitespace/layout); the `Palette` half is already realized (`SvgPalette`/`palette_vars`) |
| DP-11 | `OutputFormatter` over calc/moon/houses/chart | DECLINED — per-command JSON keys + text columns are bespoke; a trait abstracts only the 2-line json/text branch — leaky |

## 7. Test coverage

`cargo llvm-cov` 0.8.5, product code only (excludes `fuzz/` 0%-by-design
and `xtask/` tooling). Total **74.6% region / 74.2% line**;
~1278 tests (237 core-lib + 145 cli-lib + 20 integration files). Test
suite is flake-free (TEST-1 — plugin `$PATH` race — fixed `d9b7b79`).

Core engine is strong (80–99% per module; time 99%, panchanga 99.6%,
utils 99%, houses 97%); precision paths additionally regression-locked.

| area | region | note |
|---|---|---|
| `bindings/{js,python}/src/lib.rs` | 0% | exercised only by the JS/Python language harnesses — `llvm-cov` can't instrument them |
| `cli/src/cmd/render/{config 11%,registry 29%,pipeline 66%}`, `error.rs 33%`, `calendar.rs 37%`, `parse.rs 68%`, `format.rs 69%` | 11–69% | CLI orchestration — the main remaining test-debt (the 5 calc/eclipse/houses/omer/sabbats wrappers are now 77–97%) |
| `core/src/functions/{searches,phenomena,vedic,esbats,eclipses}.rs` | 81–89% | rare-edge branches; hot paths regression-locked |

No CI coverage gate.

---

## Recommended next

1. **OPEN, bounded, byte-safe:** DUP-7 (`palette!` helper) · NV-1
   (`#[non_exhaustive]` on `ParseError`/`CliError`).
2. **Test debt:** unit-cover the 11–37% CLI orchestration modules via
   the `compute()` seam; raise `searches.rs` branch coverage; add a CI
   coverage floor.
3. **Deferred isolated efforts (own session + precision soak each):**
   ARCH-7 lib.rs de-glob · ARCH-8 render helper extraction ·
   ARCH-10/DP-4/DUP-1 binding codegen · DP-2 unit newtypes ·
   DP-6 SVG templates · PERF-1 analytic VSOP derivative.
