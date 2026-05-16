# Celestial — Code Audit

Date: 2026-05-16 · Rescan: 2026-05-16 (post architecture/perf refactor, develop @ d8e4d11). One table per category; first column is a stable ID. Completed items are removed (not listed); only open / deferred / decided items remain.

---

## 1. Cyclomatic complexity

No function in the workspace exceeds CC 10 — nothing open. Current peak is ~6–7 (`searches::years_diff`, `pipeline::run`). Many 11–15-arm `match` fns (houses/parse/registry dispatch) are flat lookup tables: high arm count, no real path-risk; intentionally not flagged.

---

## 2. Code duplication

| id | location(s) | duplicated | size | status |
|---|---|---|---|---|
| DUP-1 | bindings/{js,python,php}/src/lib.rs | ~201 per-export macro stubs hand-mirrored ×3 | ~6k LOC | OPEN (structural) — napi/pyo3/php proc-macros + native return shapes can't be unified by a plain crate; only codegen/macro from one spec removes it (see DP-4) |
| DUP-2 | bindings/{js,python,php} eclipse exports | ~15 eclipse stubs ×3 (sol/lun_eclipse_when*, _how, _where) | ~450 LOC | OPEN — per-lang result marshalling differs; codegen candidate |
| DUP-3 | bindings/{js,python,php} houses exports | `houses`/`houses_ex`/`houses_ex2` ×3 | ~90 LOC | OPEN — codegen candidate |
| DUP-4 | bindings/{js,python,php} `revjul`/`revjul_hms` | same call, 3 inconsistent return shapes (CalDate / tuple / map) | ~24 LOC | OPEN — normalize shape in `celestial-ffi`, then thin per-lang |
| DUP-5 | cli/src/cmd/render/{calendar_wheel,indigenous,mesoamerican}.rs | wheel CX/CY/R geometry + palette-fetch + `json!()` preamble per tradition | ~180 LOC | LOW — partly intentional (per-tradition layout); a shared coords/palette helper would still cut ~half |

---

## 3. Performance

| id | file:line | issue | impact | status |
|---|---|---|---|---|
| PERF-1 | core/src/astronomy/engine.rs:161 | heliocentric-speed path evaluates the full VSOP series 3× (jde, jde±0.5) when `FLG_SPEED` set on a heliocentric calc | HOT | OPEN — analytic d/dt VSOP, or reuse the central eval for a 1-day diff |
| PERF-2 | core/src/functions/searches.rs:316-318 | `next_aspect_with` recomputes `calc_ut(jd_ret, SPEED)` after the bisection already converged via `diff_at` | WARM | OPEN — stash the final bisection eval |
| PERF-3 | core/src/functions/searches.rs:411-412 | `next_aspect_cusp` re-runs `calc_ut` + `houses()` after convergence (already computed inside the last `diff_at`) | WARM | OPEN — reuse converged result |
| PERF-4 | core/src/functions/searches.rs:149 | `bisect_retro_station` recomputes `pos_at` after the loop; the converged midpoint already holds `speed_lon` | WARM | OPEN — return the converged sample |
| PERF-5 | core/src/functions/searches.rs:340-354 | `next_aspect_with2` runs two independent ±aspect scan loops over the same JD range from `jd_start` | WARM | OPEN — merge into one multi-target scan |
| PERF-6 | core/src/astronomy/engine.rs:74,268 | `compute_speed` central-difference ±0.5 d | HOT | WONTFIX — forward/back diff would reduce precision; central diff already minimal 2-eval 2nd-order (kept so it is not re-flagged) |

Precision invariant for any fix here: `calc` + moon phases + vedic/meso/bazi SVG must stay byte-identical to the 1986-05-30 PDF reference and Diana 1961-07-01 charts.

---

## 4. Security — MODERATE (memory-safe, no `unsafe`, FFI clean, no shell exec)

| id | severity | file:line | issue | fix |
|---|---|---|---|---|
| SEC-1 | HIGH | cli/src/cmd/render/builtin_svg.rs:181 | `--var title=` / TOML `[vars]` written raw into SVG `<text>` → markup injection | XML-escape all `vars` text values |
| SEC-2 | HIGH | cli/src/cmd/chart.rs:582 | `--name` written verbatim into SVG `<text>` | XML-escape `name` |
| SEC-3 | HIGH | cli/src/cmd/render/builtin_svg.rs:663-665 | `&b1[..len.min(3)]` slices on non-char-boundary → panic on multibyte body label | `chars().take(3).collect()` |
| SEC-4 | HIGH | cli/Cargo.toml:27 | `toml = "=0.4.10"` (2019, unmaintained) parsing untrusted `--config` | upgrade to `toml` 0.8.x |
| SEC-5 | MED | cli/src/cmd/render/pipeline.rs:230-241 | `--out`/config `out=` no traversal/abs-path check → arbitrary write | reject `..`/abs or confine to base dir |
| SEC-6 | MED | cli/src/cmd/render/pipeline.rs:215-224 | MiniJinja env: no fuel/recursion limit on `--template` → DoS | `set_fuel`/call-stack limit, strict undefined |
| SEC-7 | MED | cli/src/parse.rs:169 | `offsets[0]` assumes non-empty after the tz match | guard `offsets.is_empty()` |
| SEC-8 | LOW | core/src/functions/time.rs:92-103 | unbounded `f64 → i64 as` saturates silently on extreme JD | checked/`TryFrom` cast |

Notes: no `unsafe` in core/cli/bindings/ffi; napi/pyo3/ext-php-rs FFI sound; `plugin.rs` execs via arg array (no shell).

---

## 5. Architecture

| id | area | problem | improvement | status |
|---|---|---|---|---|
| ARCH-7 | core/src/lib.rs | 12 crate-root `pub use mod::*` globs flatten whole surface | curate explicit re-exports | DEFERRED — load-bearing for core internals (precision compute uses `crate::calc_ut` etc); faithful de-glob ≈ exhaustive ~280-symbol mirror over the precision path. Own isolated effort |
| ARCH-8 | cli/src/cmd/render/mod.rs (~2415 LOC) | residual bulk: ~80 test fns + geometry/format/const/dignity helpers still inline | move geometry→`wheel.rs`, format→`format.rs`, tables→`const.rs`, dignity→`dignity.rs`; tests stay | OPEN |
| ARCH-9 | per-tradition context typing | `ChartContext` typed only at the dispatch boundary; the 21 builders still compose raw `serde_json::Value` | per-tradition typed structs (`NatalContext`, `VedicContext`, …) impl `Serialize`+`Deref` | OPEN |
| ARCH-10 | bindings 201×3 stubs | no codegen; every export hand-written per language | generate all 3 from one signature spec / macro (see DP-4) | OPEN |
| ARCH-11 | testability | `pipeline::run` does IO + dispatch inline | extract pure `compute(&RenderArgs)->Result<String,CliError>`; `run` wraps IO | OPEN |

---

## 6. Design pattern opportunities

| id | location | current | pattern | payoff |
|---|---|---|---|---|
| DP-1 | render/registry.rs:22-434 | `CHART_REGISTRY` + 28 near-identical `dispatch_*`; O(n) alias `.find()` | `trait ChartBuilder` impls or declarative macro from `(aliases,title,build,render)` | −~400 LOC; new type = 1 row; per-type metadata |
| DP-2 | core `PlanetPos`, `jd/lat/lon: f64`, `hsys: u8` | primitive obsession; units stringly-documented | newtypes `JulianDay`/`Latitude`/`Longitude`/`HouseSystem` (in `celestial-ffi`) | compile-checked units across CLI + bindings |
| DP-3 | cli/src/cmd/render/args.rs | `chart_type: String`, `calendars: Vec<String>`, `hsys: char` parsed at runtime | clap `#[derive(ValueEnum)]` enums | invalid `--chart-type` fails at parse, help lists valid values |
| DP-4 | bindings/{js,python,php}/src/lib.rs | 201×3 stubs + per-lang `PlanetPos`/error shim | codegen/macro `#[export(shape,langs)]` over `celestial-ffi` | ~3k LOC culled; one signature per export (realizes ARCH-10, DUP-1/2/3) |
| DP-5 | render/context.rs:31-613 + tradition builders | `json!({...})` + `p["k"].as_f64().unwrap_or(0.0)` everywhere | typed builder structs + `ContextError` | template-field typos & missing data caught before render (realizes ARCH-9) |
| DP-6 | render/builtin_svg.rs + tradition renderers | hand-rolled SVG `push_str`/`write!` strings, untyped `vars[...]` | MiniJinja templates + parsed `Palette` struct | ~500 LOC → templates; per-tradition theming; checked at build |

---

## Recommended order

1. **SEC-1..4 (HIGH)** — XML-escape user SVG values + char-boundary fix + `toml` upgrade. Small, contained, urgent.
2. **PERF-1** — heliocentric-speed VSOP triple-eval (only remaining HOT, precision-safe via analytic derivative or central reuse).
3. **ARCH-8 / ARCH-11** — finish the render decomposition (helpers out, pure `compute` seam); cheap, unlocks testing.
4. **DP-4 / ARCH-10** — binding codegen: largest LOC reduction left (~3k), removes DUP-1/2/3.
5. **ARCH-9 / DP-5** — per-tradition typed contexts.
6. **ARCH-7** — lib.rs de-glob, as its own isolated effort with a dedicated precision soak.
