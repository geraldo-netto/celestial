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
| DUP-5 | cli/src/cmd/render/{calendar_wheel,indigenous,mesoamerican}.rs | wheel CX/CY/R geometry + palette-fetch per tradition | ~120 LOC | LOW — partly intentional (per-tradition layout); the `json!()` preamble half is now resolved (typed contexts) |
| DUP-6 | core/src/functions/aspects.rs:119-270 | `match_aspect` / `_2` / `_3` / `_4` overloads re-implement the same diff/speed/factor/matched math (variants 1≡3, 2≡4) | ~60 LOC redundant | OPEN (HIGH) — collapse to one core fn taking a parametric orb spec; the variants become thin adapters (precision-sensitive: byte-verify aspect output) |
| DUP-7 | cli/src/cmd/render/{vedic,specialist,hellenistic,chinese,mesoamerican,indigenous,calendar_wheel,omer_grid}.rs | `palette_with_defaults(&[…],&vars)` + `vars.entry("title").or_insert_with(…)` repeated in ~10 builders | ~70 LOC | OPEN — a `palette!(vars, title, [(k,d)…])` helper/macro; bounded, byte-identical (same strings) |
| DUP-8 | bindings/python/src/lib.rs | `(pos.lon,pos.lat,pos.dist,pos.speed_lon,pos.speed_lat,pos.speed_dist)` tuple built ×5 (calc/calc_ut/calc_many/calc_ut_many/calc_pctr) | ~30 LOC | LOW — pyo3 `.into_py_any()` chain resists extraction; could use `celestial_ffi::pos6` + a tuple-from-array helper |

---

## 3. Performance

| id | file:line | issue | impact | status |
|---|---|---|---|---|
| PERF-1 | core/src/astronomy/engine.rs:161 | heliocentric-speed path evaluates the full VSOP series 3× (jde, jde±0.5) when `FLG_SPEED` set on a heliocentric calc | HOT | WONTFIX-as-stated / analytic deferred — `h(jde)` is needed for the position; speed is an O(h²) central difference. "Reuse central → 1-day diff" = forward/back O(h) = speed precision loss (identical to PERF-6). The only precision-safe route is an analytic VSOP-series derivative — large, precision-sensitive, its own soak. Locked by a J2000 heliocentric-Mars regression test so a future derivative rewrite must reproduce it (commit forthcoming) |
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
| ARCH-8 | cli/src/cmd/render/mod.rs (~2415 LOC) | residual bulk: ~80 test fns + geometry/format/const/dignity helpers still inline | move geometry→`wheel.rs`, format→`format.rs`, tables→`const.rs`, dignity→`dignity.rs`; tests stay | DEFERRED — post-Phase-5 mod.rs is already a facade; the bulk is the test module. Safe extraction is many single-span-per-commit moves (a bulk multi-span pass corrupts line math); low architectural payoff vs risk to the precision-verified tree. Isolated effort. `dignity.rs` single-span move verified trivial — the viable unit of work |
| ARCH-10 | bindings 201×3 stubs | no codegen; every export hand-written per language | generate all 3 from one signature spec / macro (see DP-4) | DEFERRED — stubs are heterogeneous (arg counts, `()` / `PyResult<()>` / `PyResult<PyObject>` returns, pyo3/napi/php attributes + per-module registration all differ); a macro needs ≈ per-fn arms (DP-1 dynamic) and the only real fix is a spec-driven codegen framework over 3 *published* bindings on the precision compute path — disproportionate to do atomically/verifiably; the spec table still enumerates all 201. Error shims already unified via `FfiError` (8fbed69). Own isolated effort with a dedicated binding-test soak |
| ARCH-11 | testability | `pipeline::run` does IO + dispatch inline (`std::fs` at pipeline.rs:217/241/245) | extract pure `compute(&RenderArgs)->Result<String,CliError>`; `run` wraps IO | OPEN |
| ARCH-12 | bindings/{js,python,php}/Cargo.toml | each binding dual-depends on **both** `celestial-core` and `celestial-ffi`; the ffi seam re-exports core but is not the sole dependency, and feature flags are threaded to both | depend only on `celestial-ffi` (re-exports the core API); define `timezone`/`calendar-traditions` once in ffi and propagate inward | OPEN — completes the Phase-7 facade intent |
| ARCH-13 | cli/src/cmd/render/args.rs | `RenderArgs` not range-validated before dispatch (lat∉[-90,90] / lon∉[-180,180] reach core unchecked) | `impl RenderArgs { fn validate(&self) }` called at the top of `pipeline::run` | OPEN — small; pairs with ARCH-11 |

---

## 6. Design pattern opportunities

| id | location | current | pattern | payoff |
|---|---|---|---|---|
| DP-1 | render/registry.rs | `CHART_REGISTRY` + 28 `dispatch_*` | DISMISSED on inspection — the wrappers are **not** near-identical: only `dispatch_natal` is the trivial shape; the other 27 have genuinely distinct bodies (return-JD computation, var inserts, bi/tri-wheel multi-date, per-family builders). Real shared code is only the ~5-line sig/`Ok(..into())` boilerplate (~140 LOC, not −400); a macro/trait there is closure indirection over a clear, debuggable hot-path adapter — net-negative. O(n) alias `.find()` over ~28 entries is not a real cost. |
| DP-2 | core `PlanetPos`, `jd/lat/lon: f64`, `hsys: u8` | primitive obsession; units stringly-documented | newtypes `JulianDay`/`Latitude`/`Longitude`/`HouseSystem` (in `celestial-ffi`) | DEFERRED — the compile-checked-units payoff only exists once the newtypes are threaded through `calc_ut`/`houses_ex`/… i.e. a core public-API rewrite on the exact precision compute path + every CLI/binding call site (Phase-4-class, can't atomic-verify). Unused wrapper types are net-zero clutter; a lat/lon-only CLI seam is marginal churn. Partial already shipped: `parse::{HouseSys,Tz,BodyId,DateJd}` FromStr newtypes (7bfa598). Own isolated effort with a precision soak |
| DP-4 | bindings/{js,python,php}/src/lib.rs | 201×3 stubs + per-lang `PlanetPos`/error shim | codegen/macro `#[export(shape,langs)]` over `celestial-ffi` | DEFERRED with ARCH-10 (same item) — partial single-family macro is net-negative (DP-1 dynamic); only a full spec-driven codegen realizes it. Error-shim half already done via `FfiError` |
| DP-6 | render/builtin_svg.rs + tradition renderers | hand-rolled SVG `push_str`/`write!` strings, untyped `vars[...]` | MiniJinja templates + parsed `Palette` struct | DEFERRED / partly done — the template-engine half **conflicts with the mandated byte-identical-vs-PDF/Diana precision gate**: moving renderers to `.tt` changes whitespace/attribute layout, so it cannot be done while the precision invariant holds (mutually exclusive — would need the gate relaxed). The `Palette` half is substantially realized via `svg_common::SvgPalette` (bg/accent/text trio, used by the vedic/etc renderers, commit d488454); the remaining ad-hoc `vars[...]` reads are per-renderer-distinct colour sets (omer 6 keys, specialist 12, …) — not shared duplication, so a generic getter is lateral (DP-1 dynamic) |
| DP-7 | core/src/functions/aspects.rs:119-270 | `match_aspect{,_2,_3,_4}` overloads | parametric `OrbSpec` struct + one core matcher; variants become adapters | removes the DUP-6 redundant math; precision-sensitive (byte-verify aspect output) |
| DP-8 | core/src/functions/motion.rs:102-107 | `event_type: i32` + `(bool,bool,bool)` tuple matching | `#[repr(u8)] enum RiseSetEvent { Rise, Transit, Set }` | type-safe event dispatch; invalid combinations unrepresentable |
| DP-9 | core/src/functions/config.rs:8-15 | hand-rolled `thread_local! { Cell }` per setting (`SID_MODE`/`TOPO_POS`/`DELTA_T_USERDEF`) | a small typed config-registry abstraction | adding a setting stops repeating thread-local boilerplate; one audit surface |
| DP-10 | cli/src/i18n.rs:66-90 | `Lang::from_locale` hardcoded prefix `match` arms | const `(prefix, Lang)` table + lookup | new language = one table row, no match edit |
| DP-11 | cli/src/cmd/{calc,moon,houses,chart}.rs | per-command ad-hoc `Row` struct + json/text branch | `OutputFormatter<T: Serialize>` (`.table()`/`.json()`) | re-examined earlier as leaky; revisit only if a 4th+ command needs it — currently borderline, not net-positive |

---

## Recommended order

1. **SEC-1..4 (HIGH)** — XML-escape user SVG values + char-boundary fix + `toml` upgrade. Small, contained, urgent.
2. **PERF-1 analytic derivative** — only as its own precision-soak effort (reuse-central is a precision loss; regression-locked).
3. **DUP-6 / DP-7** — collapse the four `match_aspect` overloads to one parametric matcher (~60 LOC, HIGH; byte-verify aspect output).
4. **ARCH-11 + ARCH-13** — pure `compute()` seam + `RenderArgs::validate`; cheap, unlocks render-logic testing.
5. **ARCH-12** — bindings depend only on `celestial-ffi` (finish the Phase-7 facade).
6. **DUP-7 / ARCH-13 / DP-8 / DP-10 / DP-9** — bounded, byte-safe cleanups (palette+title helper, event enum, locale table, config registry).
7. **PERF-2..5** — post-bisection recompute reuse in `searches.rs` (WARM, precision-safe).
8. **Deferred isolated efforts (own session + precision soak each):** ARCH-7 lib.rs de-glob · ARCH-8 render helper single-span moves · ARCH-10/DP-4 binding codegen · DP-2 unit newtypes · DP-6 SVG templates.
