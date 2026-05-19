# Celestial — Code Audit

Rescan: 2026-05-17b (develop, post coverage-hardening + full
multi-category re-audit). One table per category; stable IDs in the
first column. Completed work is removed (not listed). Rows are only
**OPEN**, **DEFERRED** (real but disproportionate to do safely now —
its own effort), or **DECIDED** (evaluated, intentionally not done —
kept so a rescan doesn't re-flag).

| state | meaning |
|---|---|
| OPEN | actionable, bounded, byte-safe — do next |
| DEFERRED | real; needs an isolated session + precision soak |
| DECIDED | WONTFIX / DECLINED / N-A with rationale |

> **No OPEN findings.** SEC-10 (SVG injection in specialist
> renderers) and TEST-3 (CI coverage floor) were both found and
> **fixed this cycle** — removed per the completed-work policy. Only
> DEFERRED / DECIDED items remain (each with rationale below).

---

## 1. Complexity

Product code clean — no `core/src` or `cli/src` (non-test) function
exceeds CC ≈ 10 (peak ~6–7: `searches::years_diff`, `pipeline::run`).
High-arm `match` dispatch tables (houses/parse/registry) are flat
lookups — not flagged. Newly added fuzz suites + `cli_coverage.rs`
helpers verified all ≤10.

| id | location | issue | status |
|---|---|---|---|
| CC-1 | cli/tests/cli_fuzz.rs:250 `boundary_…strings`; cli/src/parse.rs:418 `parse_tz_forms` (test mod) | clippy `cognitive_complexity` 16 / 12 vs 10 — **purely `assert!`/`matches!` macro-expansion**; real cyclomatic ≤3, no splittable branch | DECIDED — the lint is `nursery`/allow-by-default and is not enabled in any crate, so the `clippy.toml` threshold is dormant for these. No logic to restructure. Kept so a rescan doesn't re-flag |

## 2. Code duplication

| id | location | issue | status |
|---|---|---|---|
| DUP-1 | bindings/{python,php,js}/src/lib.rs | hand-written per-export delegating stubs ×3 (python 201 `#[pyfunction]`, php 201 `#[php_function]`, js ~196 `#[napi]`) | DEFERRED — napi/pyo3/php proc-macros, per-module registration and native return shapes can't be unified by a plain crate; only spec-driven codegen over 3 *published* bindings on the precision path removes it. Shared `FfiError`/`pos6`/`pos6_tuple` halves still factored in `bindings/ffi/src/lib.rs` (consumed by py/php/js). = ARCH-10/DP-4 |
| DUP-4 | bindings `revjul`/`revjul_hms` | 3 idiomatic return shapes (struct/tuple/map) | DECIDED — intentional per-language idioms downstream depends on; core call already shared. Normalizing = a published API break, not a dedupe |
| DUP-6 | cli/tests/cli_fuzz.rs:18 vs fuzz/src/main.rs:18 | `Xorshift64` PRNG (+`random_string`) copy-pasted across two test targets (self-documented at cli_fuzz.rs:16) | DECIDED — test-only, 2 copies in different crates/targets; a shared dev-dep crate is disproportionate to ~15 LOC. Below the dedupe bar |

DUP-5 (per-tradition wheel geometry) is N-A — distinct layout
constants, not duplication; shared halves already factored.

## 3. Performance (precision is the hard gate)

All hot/warm items are decided; each is regression-locked so a future
rewrite must reproduce current numbers. Re-verified this rescan: no new
hot-path waste (recent core changes are correctness/refactor only; the
DEC-1 RAII guard is one TLS read + one TLS write, zero loop cost).

| id | site | status |
|---|---|---|
| PERF-1 | engine.rs heliocentric-speed 3× VSOP | WONTFIX-as-stated; only precision-safe route is an analytic VSOP derivative (own soak). Locked: `perf1_heliocentric_mars_speed_regression_lock` (core/tests/unit_tests.rs:2735) |
| PERF-2/3 | searches.rs post-bisect `calc_ut`/`houses` | WONTFIX — scan strips SPEED (searches.rs:289/390/462, crossings.rs:170/313); the post-bisect full-flag eval at the converged jd is authoritative, not redundant. Locked: `perf2345_search_regression_lock` (unit_tests.rs:2769) |
| PERF-4 | searches.rs `bisect_retro_station` | N-A — already reuses the last loop sample; no recompute exists |
| PERF-5 | searches.rs `next_aspect_with2` dual scan | DECLINED — merge is a precision-sensitive rewrite for marginal gain |
| PERF-6 | engine.rs `compute_speed` ±0.5 d | WONTFIX — central diff is the minimal 2-eval 2nd-order form; forward/back loses precision |

Invariant: `calc` + moon phases + vedic/meso/bazi SVG stay
byte-identical to the 1986-05-30 PDF reference and the Diana
1961-07-01 chart.

## 4. Security

Clean. `unsafe`-free in core/cli/bindings/ffi source; FFI sound
(napi/pyo3/ext-php-rs); `plugin.rs` execs via arg array (no shell);
`cargo audit` clean (90 deps, 0 advisories; toml 0.8.23, minijinja
2.19.0). SEC-1..SEC-10 all fixed (removed per the completed-work
policy). SEC-10 (this cycle): the SEC-1/SEC-9 SVG escaping never
reached the 9 specialist/calendar renderers — fixed by escaping every
user-controlled `ctx["vars"]` title/colour at the read boundary
(`svg_common::esc_var` + escaped `SvgPalette`/`CalendarPalette`),
byte-identical for non-malicious input, regression-locked
(`cli_coverage::render_specialist_var_injection_escaped`).

## 5. Architecture / modularity / visibility

Dependency graph acyclic and correctly layered: core depends on nothing
in-workspace (only `serde`); ffi → core; {js,php,python} → ffi only;
**cli → core only** (not core+ffi — tighter than a prior note claimed).
Render submodules are private `mod` (mod.rs:52-64) — `RenderArgs`/`run`
are the only intended crate-public render surface; visibility chain
sound. All public error/value enums `#[non_exhaustive]` (`core::Error`
error.rs:18; `CliError` cli/error.rs:25; `ParseError` parse.rs:24;
`CalendarKind` render/args.rs:14).

Decoupling: well-decoupled (DEC-1 sidereal-mode leak fixed in a prior
cycle — removed per the completed-work policy). DEC-2 (29 renderers
read `ctx["field"]` stringly) DECLINED — typed contexts deliberately
stop at the builder/serialize boundary; a field-enum across 29
renderers is the DP-1/DP-6 net-negative dynamic. DEC-3
(renderer/builder fn-ptr → trait) = DP-1.
DEC-4 (xtask line-scan → `syn` AST) DEFERRED — offline tooling, now
regression-tested; `syn` build-cost not warranted until binding churn.

| id | area | issue | status |
|---|---|---|---|
| ARCH-8 | cli/src/cmd/render/mod.rs | 2444 LOC total but **~63% is test code** (`mod tests` ~1023, `mod tests_vedic` ~525); non-test residual ≈ ~900 LOC of facade re-exports + wheel/SI/dignity helpers | DEFERRED — already a facade; safe extraction is many single-span-per-commit moves (bulk pass corrupts line math); low payoff vs precision-tree risk |
| ARCH-10 | bindings 201×3 stubs | no codegen | DEFERRED — = DUP-1/DP-4 |
| VIS-1 | render/{specialist,hellenistic,vedic,pipeline}.rs | `build_*`/`render_*` are bare `pub fn` where siblings use `pub(super)` (e.g. mod.rs:486/1627/1890) — inconsistent | DECIDED — no real leak (parent submodules are private `mod`, unreachable outside `render`); cosmetic only |

## 6. Design pattern opportunities

| id | target | status |
|---|---|---|
| DP-1 | registry macro/table over `dispatch_*` | DISMISSED — the heterogeneous ~12 (`dispatch_solar_return`/`_lunar_return`/`_progressed`/`_solar_arc`/`_biwheel`/`_triwheel`/`_composite`/`_ephemeris`/`_profection` — distinct return-jd / `--years` / date2 logic) stay hand-written; a macro there is closure indirection over a clear hot-path adapter. (The 15 *uniform* dispatchers — DP-1b — were collapsed into the `specialist_dispatch!` macro `059434f`; removed per the completed-work policy) |
| DP-2 | `JulianDay`/`Latitude`/… unit newtypes | DEFERRED — payoff needs threading through `calc_ut`/`houses_ex` = core-API rewrite on the precision path. Parse-layer newtypes already shipped (`7bfa598`) |
| DP-4 | binding codegen | DEFERRED — = ARCH-10/DUP-1 |
| DP-6 | SVG → MiniJinja templates | DEFERRED — the template half **conflicts with the byte-identical precision gate** (changes whitespace/layout); the `Palette` half is already realized (`SvgPalette`/`palette_vars`) |
| DP-11 | `OutputFormatter` over calc/moon/houses/chart | DECLINED — per-command JSON keys + text columns are bespoke; a trait abstracts only the 2-line json/text branch — leaky |

## 7. Test coverage

`cargo llvm-cov` 0.8.5, tests + the in-repo property/fuzz harness
merged (`run_prop_tests` is a `[[bin]]`, so it must be run explicitly —
`cargo llvm-cov --workspace` alone never ran it).

Product code (core/src + cli/src, tests excluded) = **94.9% region /
94.5% line** (core 94.1%/93.9%, cli 95.7%/95.1%). Every core + cli
source file is ≥80% region and line; lowest core modules (`searches`
81, `phenomena` 82, `vedic` 83, `esbats` 82) are rare-edge branches
with hot paths regression-locked. 86 property suites + 21 cli_smoke +
19 cli_coverage integration tests, all green.

A CI coverage floor is enforced (TEST-3, done this cycle, removed per
the completed-work policy): the `celestial-core` workflow merges
tests + the fuzz harness and gates on `cargo llvm-cov report
--fail-under-lines 80` over product code only (fuzz/xtask/binding-
tests/generated tables excluded via `--ignore-filename-regex`).
Current scoped TOTAL: 94.9% region / 94.5% line.

| id | area | issue | status |
|---|---|---|---|
| TEST-2 | bindings/{js,python}/src/lib.rs | 0% — exercised only by the JS/Python language harnesses; `llvm-cov` cannot instrument them | DECIDED — not instrumentable, not a real gap; the only sub-80% product area |

---

## Recommended next

No OPEN findings. Remaining work is the long-horizon deferred set:

1. **Deferred isolated efforts (own session + precision soak each):**
   ARCH-8 render helper extraction ·
   ARCH-10/DP-4/DUP-1 binding codegen · DP-2 unit newtypes ·
   DP-6 SVG templates · PERF-1 analytic VSOP derivative.
