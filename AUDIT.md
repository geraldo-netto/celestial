# Celestial — Code Audit

Date: 2026-05-16 · Rescan: 2026-05-16 (post architecture/perf refactor, develop @ d8e4d11). One table per category; first column is a stable ID. Completed items are removed (not listed); only open / deferred / decided items remain.

---

## 1. Cyclomatic complexity

No function in the workspace exceeds CC 10 — nothing open. Current peak is ~6–7 (`searches::years_diff`, `pipeline::run`). Many 11–15-arm `match` fns (houses/parse/registry dispatch) are flat lookup tables: high arm count, no real path-risk; intentionally not flagged.

---

## 2. Code duplication

| id | location(s) | duplicated | size | status |
|---|---|---|---|---|
| DUP-1 | bindings/{js,python,php}/src/lib.rs | ~201 per-export macro stubs hand-mirrored ×3 | ~6k LOC | DEFERRED — same item as ARCH-10/DP-4: napi/pyo3/php proc-macros + per-module registration + native return shapes can't be unified by a plain crate; the only real fix is a spec-driven codegen framework over 3 *published* bindings on the precision compute path (disproportionate to do atomically/verifiably; spec still enumerates all 201). Error-shim + `pos6` halves already shipped via `celestial-ffi` (8fbed69). Own isolated effort with a binding-test soak |
| DUP-2 | bindings/{js,python,php} eclipse exports | ~15 eclipse stubs ×3 (sol/lun_eclipse_when*, _how, _where) | ~450 LOC | DEFERRED (subset of DUP-1) — each binding already calls the shared `celestial::*_eclipse_*` (core result re-exported via celestial-ffi); the residual is trivial per-lang result shaping (struct/tuple/Vec) not worth a crate fn. The bulk is the per-lang macro attribute+signature ×3 → same codegen-class as DUP-1/ARCH-10/DP-4 |
| DUP-3 | bindings/{js,python,php} houses exports | `houses`/`houses_ex`/`houses_ex2` ×3 | ~90 LOC | DEFERRED (subset of DUP-1) — marshalling is **not** even consistent across bindings (js trims trailing-zero cusps via a count/filter; py returns `cusps[1..]` raw), so no shared celestial-ffi helper is possible; the residual is the per-lang macro stub shell ×3 = same codegen-class as DUP-1/ARCH-10/DP-4 |
| DUP-4 | bindings/{js,python,php} `revjul`/`revjul_hms` | same call, 3 return shapes (js `CalDate` struct / py tuple / php `HashMap`) | ~24 LOC | DECLINED — the differing shapes are **intentional per-language idioms** that downstream consumers depend on; the core `celestial::revjul[_hms]` call is already shared (1 line each). "Normalize the shape" = a published-binding API break (semver decision), not a byte-safe dedupe. Residual stub shell is DUP-1 codegen-class |
| DUP-5 | cli/src/cmd/render/{calendar_wheel,indigenous,mesoamerican}.rs | wheel CX/CY/R geometry + palette-fetch per tradition | — | RESOLVED/N-A — on inspection the per-tradition layout consts hold **distinct values** (MW 350/300/200, LS 450/450/320, BAZI grid 160/280/60/70 …) = intentional per-tradition layout, not duplicated code; no shared `polar`/`wx` helper is repeated across tradition files. The genuinely-shared halves are already factored: preamble (`svg_common::svg_doc_open`/`panel_card`, d488454), palette (`palette_vars` 7ba7b26 / `SvgPalette`), object construction (typed contexts). Nothing extractable remains |
| DUP-7 | cli/src/cmd/render/{vedic,specialist,hellenistic,chinese,mesoamerican,indigenous,calendar_wheel,omer_grid}.rs | `palette_with_defaults(&[…],&vars)` + `vars.entry("title").or_insert_with(…)` repeated in ~10 builders | ~70 LOC | OPEN — a `palette!(vars, title, [(k,d)…])` helper/macro; bounded, byte-identical (same strings) |

---

## 3. Performance

| id | file:line | issue | impact | status |
|---|---|---|---|---|
| PERF-1 | core/src/astronomy/engine.rs:161 | heliocentric-speed path evaluates the full VSOP series 3× (jde, jde±0.5) when `FLG_SPEED` set on a heliocentric calc | HOT | WONTFIX-as-stated / analytic deferred — `h(jde)` is needed for the position; speed is an O(h²) central difference. "Reuse central → 1-day diff" = forward/back O(h) = speed precision loss (identical to PERF-6). The only precision-safe route is an analytic VSOP-series derivative — large, precision-sensitive, its own soak. Locked by a J2000 heliocentric-Mars regression test so a future derivative rewrite must reproduce it (commit forthcoming) |
| PERF-2 | core/src/functions/searches.rs:316-318 | `next_aspect_with` post-bisect `calc_ut(jd_ret, flags)` | WARM | WONTFIX — not redundant: the scan's `diff_at` uses `flags & !SPEED` (line 289), so the post-bisect full-flag eval at the exact converged jd is the authoritative result; "reuse the scan eval" drops SPEED = precision loss (PERF-1/6 rationale). Locked by `perf2345_search_regression_lock` |
| PERF-3 | core/src/functions/searches.rs:411-412 | `next_aspect_cusp` post-bisect `calc_ut` + `houses()` | WARM | WONTFIX — same as PERF-2 (scan_flags strips SPEED at line 390); the final eval is authoritative, not redundant. Regression-locked |
| PERF-4 | core/src/functions/searches.rs:149 | `bisect_retro_station` final sample | WARM | RESOLVED/N-A — audit was inaccurate: the fn already reuses `pm` from the last loop iteration; there is **no** post-loop recompute. Regression-locked |
| PERF-5 | core/src/functions/searches.rs:340-354 | `next_aspect_with2` runs two independent ±aspect searches | WARM | DECLINED — merging into one multi-target scan is a precision-sensitive rewrite (must reproduce the exact nearest-crossing selection) for marginal gain over a correct, clear path (DP-1 net-negative dynamic). Regression-locked |
| PERF-6 | core/src/astronomy/engine.rs:74,268 | `compute_speed` central-difference ±0.5 d | HOT | WONTFIX — forward/back diff would reduce precision; central diff already minimal 2-eval 2nd-order (kept so it is not re-flagged) |

Precision invariant for any fix here: `calc` + moon phases + vedic/meso/bazi SVG must stay byte-identical to the 1986-05-30 PDF reference and Diana 1961-07-01 charts.

---

## 4. Security — clean (memory-safe, no `unsafe`, FFI sound, no shell exec)

All previously-flagged items resolved — nothing open:

- SEC-1/2 (HIGH) — `format::xml_escape` on title/palette (builtin_svg) and `--name` (chart). `0d15c20`
- SEC-3 (HIGH) — UTF-8-safe `trunc_chars` replaces byte slices. `0d15c20`
- SEC-4 (HIGH) — `toml` 0.4.10 → 0.8.23. `f710c34`
- SEC-5 (MED) — config-supplied `out` confined to relative-no-`..`; CLI `--out` unrestricted. `d014909`
- SEC-6 (MED) — MiniJinja `fuel` cap (50M) on `--template`; recursion default-bounded. `0ce4060`
- SEC-7 (MED) — `offsets.first()` guard (was latent index panic). `95fa795`
- SEC-8 (LOW) — `day_of_week` non-finite/absurd-jd sentinel guard (matches `revjul`). `fbd486c`

Notes: no `unsafe` in core/cli/bindings/ffi; napi/pyo3/ext-php-rs FFI sound; `plugin.rs` execs via arg array (no shell). Re-verify on each rescan; do not re-flag the resolved items.

---

## 5. Architecture

| id | area | problem | improvement | status |
|---|---|---|---|---|
| ARCH-7 | core/src/lib.rs | 12 crate-root `pub use mod::*` globs flatten whole surface | curate explicit re-exports | DEFERRED — load-bearing for core internals (precision compute uses `crate::calc_ut` etc); faithful de-glob ≈ exhaustive ~280-symbol mirror over the precision path. Own isolated effort |
| ARCH-8 | cli/src/cmd/render/mod.rs (~2415 LOC) | residual bulk: ~80 test fns + geometry/format/const/dignity helpers still inline | move geometry→`wheel.rs`, format→`format.rs`, tables→`const.rs`, dignity→`dignity.rs`; tests stay | DEFERRED — post-Phase-5 mod.rs is already a facade; the bulk is the test module. Safe extraction is many single-span-per-commit moves (a bulk multi-span pass corrupts line math); low architectural payoff vs risk to the precision-verified tree. Isolated effort. `dignity.rs` single-span move verified trivial — the viable unit of work |
| ARCH-10 | bindings 201×3 stubs | no codegen; every export hand-written per language | generate all 3 from one signature spec / macro (see DP-4) | DEFERRED — stubs are heterogeneous (arg counts, `()` / `PyResult<()>` / `PyResult<PyObject>` returns, pyo3/napi/php attributes + per-module registration all differ); a macro needs ≈ per-fn arms (DP-1 dynamic) and the only real fix is a spec-driven codegen framework over 3 *published* bindings on the precision compute path — disproportionate to do atomically/verifiably; the spec table still enumerates all 201. Error shims already unified via `FfiError` (8fbed69). Own isolated effort with a dedicated binding-test soak |

---

## 6. Design pattern opportunities

| id | location | current | pattern | payoff |
|---|---|---|---|---|
| DP-1 | render/registry.rs | `CHART_REGISTRY` + 28 `dispatch_*` | DISMISSED on inspection — the wrappers are **not** near-identical: only `dispatch_natal` is the trivial shape; the other 27 have genuinely distinct bodies (return-JD computation, var inserts, bi/tri-wheel multi-date, per-family builders). Real shared code is only the ~5-line sig/`Ok(..into())` boilerplate (~140 LOC, not −400); a macro/trait there is closure indirection over a clear, debuggable hot-path adapter — net-negative. O(n) alias `.find()` over ~28 entries is not a real cost. |
| DP-2 | core `PlanetPos`, `jd/lat/lon: f64`, `hsys: u8` | primitive obsession; units stringly-documented | newtypes `JulianDay`/`Latitude`/`Longitude`/`HouseSystem` (in `celestial-ffi`) | DEFERRED — the compile-checked-units payoff only exists once the newtypes are threaded through `calc_ut`/`houses_ex`/… i.e. a core public-API rewrite on the exact precision compute path + every CLI/binding call site (Phase-4-class, can't atomic-verify). Unused wrapper types are net-zero clutter; a lat/lon-only CLI seam is marginal churn. Partial already shipped: `parse::{HouseSys,Tz,BodyId,DateJd}` FromStr newtypes (7bfa598). Own isolated effort with a precision soak |
| DP-4 | bindings/{js,python,php}/src/lib.rs | 201×3 stubs + per-lang `PlanetPos`/error shim | codegen/macro `#[export(shape,langs)]` over `celestial-ffi` | DEFERRED with ARCH-10 (same item) — partial single-family macro is net-negative (DP-1 dynamic); only a full spec-driven codegen realizes it. Error-shim half already done via `FfiError` |
| DP-6 | render/builtin_svg.rs + tradition renderers | hand-rolled SVG `push_str`/`write!` strings, untyped `vars[...]` | MiniJinja templates + parsed `Palette` struct | DEFERRED / partly done — the template-engine half **conflicts with the mandated byte-identical-vs-PDF/Diana precision gate**: moving renderers to `.tt` changes whitespace/attribute layout, so it cannot be done while the precision invariant holds (mutually exclusive — would need the gate relaxed). The `Palette` half is substantially realized via `svg_common::SvgPalette` (bg/accent/text trio, used by the vedic/etc renderers, commit d488454); the remaining ad-hoc `vars[...]` reads are per-renderer-distinct colour sets (omer 6 keys, specialist 12, …) — not shared duplication, so a generic getter is lateral (DP-1 dynamic) |
| DP-11 | cli/src/cmd/{calc,moon,houses,chart}.rs | per-command ad-hoc `Row` struct + json/text branch | `OutputFormatter<T: Serialize>` (`.table()`/`.json()`) | DECLINED (evaluated) — each command's JSON keys and text-table columns are bespoke (calc body/lon/lat/dist/speed; moon phase/date; houses angle/cusp; chart nested planets/houses/aspects). A trait would still need a per-command typed result struct **and** per-command text layout; it only abstracts the 2-line `if json {} else {}` with no shared body — a leaky abstraction (DP-1 net-negative dynamic). Kept as a decided item so it is not re-flagged |

---

## 7. Test coverage

`cargo llvm-cov` (0.8.5), workspace, **product code only** (excludes
`fuzz/` harness 0% by-design and `xtask/` build tooling):

| scope | region | line | fn |
|---|---|---|---|
| product total | **73.3%** | 72.6% | 60.1% |
| `core/` engine (functions/, astronomy/) | ~80–99% per module | — | — |

1255 `#[test]` total; 237 core-lib + 122 cli-lib + 20 integration files.
Core compute is well covered (time 99%, panchanga 99.6%, utils 99%,
houses 97%, most calendars 95–99%); precision paths are additionally
regression-locked (PDF/Diana charts, PERF-1, perf2345).

Low-coverage hotspots (not bugs — instrumentation/structure):

| area | region | why |
|---|---|---|
| `bindings/{js,python}/src/lib.rs` | 0% | exercised only by the JS/Python language test harnesses, which `llvm-cov` doesn't instrument (not Rust unit tests) |
| `cli/src/cmd/{crossing,eclipse,houses,omer,sabbats}.rs` | 0% | thin `run()` wrappers exercised by `assert_cmd` integration tests that spawn the binary as a separate process (uninstrumented) |
| `cli/src/cmd/render/{config,registry,pipeline}.rs`, `cli/src/cmd/calendar.rs`, `cli/src/parse.rs` | 11–69% | CLI orchestration; partially covered. `compute()` seam (ARCH-11) + `RenderArgs::validate` (ARCH-13) now have unit tests |
| `core/src/functions/{searches,phenomena,vedic,esbats,eclipses}.rs` | 81–89% | large search/branch surfaces; the hot precision paths are regression-locked, the gap is rare-edge branches |

No coverage gate is enforced in CI. Raising CLI-command/`searches`
branch coverage is the main test-debt item; the engine itself is solid.

---

## Recommended order

1. **Test debt** — cover the 0% CLI `run()` wrappers with in-process tests (now feasible via the `compute()` seam) + raise `searches.rs` branch coverage; consider a CI coverage floor.
2. **Deferred isolated efforts (own session + precision soak each):** ARCH-7 lib.rs de-glob · ARCH-8 render helper single-span moves · ARCH-10/DP-4 binding codegen · DP-2 unit newtypes · DP-6 SVG templates · PERF-1 analytic VSOP derivative.

(All §4 security items are resolved; §1 complexity clean; §2/§3/§5/§6 hold only deferred/decided decisions.)
