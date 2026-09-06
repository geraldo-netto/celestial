# Celestial — TODO

## Adaptability

| id | status | effort | description | notes |
|---|---|---|---|---|
| BIND-3 | DECIDED | — | `utc_to_jd` takes 7 date scalars in Python/PHP but a single `UtcDate` object in JS (`bindings/js/src/lib.rs:605`). | KEEP — idiomatic JS object-packing, not drift. Recorded in `xtask/arity_allow.txt`. |
| BIND-9 | OPEN | L | PHP stubs and `docs/php.md` advertise `celestial_*` aliases that the extension does not export. | Fresh native build confirms `function_exists('version') == true` and `function_exists('celestial_version') == false`. Current native tests use unprefixed names and pass, so they no longer detect this documentation/stub mismatch. Generate and runtime-test real aliases, or remove fictional aliases from generation/docs and make the unprefixed API canonical. |
| BIND-10 | OPEN | S | Ship Python stubs at the module path consumers import, with a typing marker. | `bindings/python/python/celestial_py/` contains `__init__.py` and `celestial_py.pyi`, but no `__init__.pyi`, `_celestial_py.pyi`, or `py.typed`; `xtask/src/main.rs::cmd_pyi` writes the unrelated submodule filename. The package imports `_celestial_py`, so generated signatures are not its public typing interface. Verify type checking through an installed wheel and `import celestial_py`. |
| BIND-11 | OPEN | M | Preserve Python optional arguments and exported constants in generated stubs. | `celestial_py.pyi` declares every argument mandatory and no constants: e.g. `set_sid_mode` requires three parameters and `julday` five despite native defaults. `xtask/src/main.rs:1420–1428` emits parameter types without PyO3 signature defaults. Parse the native signature metadata and constant registrations, then type-check representative valid calls and invalid argument types. |
| BIND-12 | OPEN | M | Reject incomplete flattened position records and invalid body IDs in JS/PHP chart adapters. | `bindings/js/src/lib.rs:2443/2470/2487/2511` and matching PHP adapters use `chunks_exact(2/3)` and discard any remainder; body floats are cast directly to integers. An odd-length midpoint list or incomplete aspect triple silently loses user input, unlike Python's tuple extraction. Validate record lengths, finite integral IDs, and known-body ranges before conversion. |

## Architecture / Modularity / SOLID

| id | status | effort | description | notes |
|---|---|---|---|---|
| ARCH-10 | DECIDED | L | Bindings still expose broad per-language adapter surfaces rather than shared codegen-only APIs. | Same as DUP-1 / DP-4; byte/API churn and PHP unverifiable path made full codegen net-negative. |
| ARCH-11 | DECIDED | S | `bindings/ffi/src/lib.rs:18` uses `pub use celestial_core::*`, so new core public API can expand binding compile surfaces. | KEEP — bindings intentionally consume the whole published core facade. Explicit lists would duplicate 200+ root exports. |
| ARCH-12 | DECIDED | S | `core/src/lib.rs` re-exports raw Swiss-Ephemeris-style constants alongside typed `Body` / `CalcFlags` APIs. | Back-compat layer for bindings and existing Rust users; tightening would be a published API break. |
| SONAR-PHP-2 | DECIDED | — | Sonar `php:S112` flags four generic `RuntimeException` throws in the standalone PHP golden-test executable. | KEEP — the process has one uncaught failure path; dedicated exception subclasses add no handling or diagnostic value. |

## Business / Design Patterns / DDD

| id | status | effort | description | notes |
|---|---|---|---|---|

## CLI / Option Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CLI-4 | OPEN | S | `DateJd::from_str` accepts bare `NaN`, `inf`, and `-inf` as Julian days. `render --date NaN --print-context` exits 0 with a null-filled chart; `--date inf` panics on integer overflow in `next_principal_phase`. | Reject non-finite and out-of-domain JDs at the shared parse boundary. `cli_fuzz::boundary_non_finite_and_extreme_floats` currently calls `parse_date` but never asserts rejection or exercises a downstream command. |
| CLI-5 | OPEN | S | Civil dates/times are normalized instead of validated: bad time components use `unwrap_or(0.0)`, month/day/time ranges are unchecked, and `2024-01-01 nope:nope` silently becomes midnight. A test explicitly accepts `1986-13-99 09:00`. | Parse the documented formats strictly, validate Gregorian fields and clock ranges, and add rejection tests for malformed and normalized-away input. |
| CLI-6 | OPEN | M | Numeric validation is command-specific and incomplete. `houses --lat NaN --json` succeeds and emits invalid JSON containing bare `NaN`; render accepts non-finite `--years` and produces null-heavy progressed charts. | Reuse finite/range validators for every geographic and calculation scalar at the CLI boundary, including `houses` lat/lon and render progression/secondary-coordinate inputs. |
| CLI-7 | OPEN | S | `houses --json` serializes `cusps` as a quoted string (`"cusps":"[...]"`) rather than a JSON array. | Replace the string-guessing `fmt::json_obj` path with a typed `serde_json` value/struct and assert field types after parsing command output. |
| CLI-8 | OPEN | S | Preserve `render --month` through universal-overlay composition. | `pipeline.rs::build_calendar_context` uses the requested month, but `apply_universal_overlays` rebuilds Gregorian data from the original JD. `render --chart-type calendar --date 2024-01-01 --month 2 --print-context` reports top-level month 2 with a January Gregorian grid. Derive overlay dates from the effective calendar month and validate its range. |
| CLI-9 | OPEN | S | Apply the same timezone conversion to both graphic-ephemeris endpoints. | `cli/src/cmd/render/registry.rs:228–229` calls `parse_date(date2)` directly, whereas the primary date and other secondary dates use the selected timezone. Equal local start/end timestamps with a nonzero timezone therefore produce an artificial span. Route this endpoint through the shared secondary-date parser and test equivalent UTC/local ranges. |

## Code Complexity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CC-1 | OPEN | M | Split the 19 tests exceeding the mandatory complexity limit of 10; the previous test-only exemption conflicts with `AGENTS.md`. | Reconfirmed by `cargo clippy --workspace --all-targets -- -W clippy::cognitive_complexity`: `core/src/astronomy/crossings.rs:374/403` (11/25), `heliacal.rs:465/528` (17/12), `solar_cycle.rs:278/331` (12/11); `core/src/functions/calc.rs:848` (14), `eclipses.rs:495` (20), `esbats.rs:431` (13), `time.rs:606/766` (13/13), `utils.rs:606/639` (16/16), `vedic.rs:383` (11), `geoformat.rs:308/342` (13/12), `omer.rs:609` (11); `cli/tests/cli_fuzz.rs:225` (16), `cli/src/parse.rs:423` (14). Preserve assertions and mutation coverage while splitting. |
| CC-6 | DECIDED | S | `cli/src/cmd/render/calendar_overlays.rs:509 render_day_cell` sits at exactly CC 10. | Compliant but no headroom. If extended, split moon-glyph and omer-badge rendering into helpers. |
| CC-7 | DECIDED | S | `core/src/body/mod.rs:119 Body::is_known_id` is a flat range ladder. | Still compliant; flat documented id windows are clearer than hiding the ranges in a table. |
| LINT-1 | OPEN | S | Restore warning-clean clippy across tracked Rust. Current checks report 33 excessive-precision literals, `houses.rs:900` byte-character slices, `houses.rs:1112` type complexity, `vedic.rs:474` production items after tests, four `chunks_exact_to_as_chunks` warnings per JS/PHP adapter, and PHP `match_aspect3` / `match_aspect4` argument-count warnings. | Verified with workspace and standalone PHP `--all-targets` clippy on Rust 1.98.1. Apply literal/type/layout fixes; preserve published binding signatures with targeted rationale. Resolve PLAT-2 before adopting newer APIs suggested by clippy. Complexity violations are tracked separately in CC-1. |

## Code Duplication

| id | status | effort | description | notes |
|---|---|---|---|---|
| DUP-1 | DECIDED | L | Binding return-adapter stubs remain repeated across JS/Python/PHP. | Per-language return shapes are intentionally different. `cargo xtask parity` confirms all 270 function exports stay aligned. |
| DUP-2 | DECIDED | S | `body_of(n)` validation helpers are present in each binding. | Per-language error mapping is the irreducible part; sharing would obscure simple error flow. |
| DUP-3 | DECIDED | — | Deterministic `core/tests/rel_clamps.rs` and randomized `fuzz/src/main.rs::test_rel_clamps` overlap on body-id edge coverage. | Intentional two-tier coverage split: `cargo test` regression plus opt-in fuzz/property run. |
| DUP-4 | DECIDED | — | `revjul` / `revjul_hms` have three language-specific return shapes. | Published API idioms differ; normalizing would break bindings. |
| DUP-5 | DECIDED | — | Per-tradition SVG wheel geometry uses similar-looking constants. | Distinct layout contracts; shared pieces are already factored. |

## Composition

| id | status | effort | description | notes |
|---|---|---|---|---|

## Concurrency

| id | status | effort | description | notes |
|---|---|---|---|---|

## Configuration Discoverability

| id | status | effort | description | notes |
|---|---|---|---|---|
| CONF-1 | OPEN | S | `render::compute` validates arguments before `load_config`, so config-provided values bypass validation. A config with `lat = 999` renders successfully and exposes 999 in the context. | Merge config first, then validate the effective arguments; add config-path tests for non-finite and out-of-range values. |
| CONF-2 | OPEN | M | Config precedence infers whether the CLI supplied a value by comparing it with sentinel defaults (`now`, `0.0`, `P`). Explicit `--lat 0 --lon 0`, `--date now`, or `--hsys P` can therefore be overwritten by config despite the documented “CLI flags take precedence” rule. | Preserve Clap value-source information or model defaultable fields as `Option<T>` until after config merging. |

## Data Structure

| id | status | effort | description | notes |
|---|---|---|---|---|
| DS-1 | OPEN | M | The two largest series tables store small integer multipliers as `f64`. `nutation.rs:182 IAU2000B_COEFFICIENTS` is `&[[f64; 9]]` × 77 rows (5,544 B) whose first five columns only ever hold values in `-3..=3`; `moon.rs LONGITUDE_DISTANCE_TERMS`/`LATITUDE_TERMS` are `[f64; 6]`/`[f64; 5]` × 120 rows whose first four columns are the Delaunay multipliers `D, M, M', F`. | **Bit-identical** if done as specified: `i8` → `f64` widening is exact for `-3..=3`, so every product and sum is unchanged. Narrowing the multiplier columns cuts both tables roughly in half and improves L1 residency for the per-call loops. Keep the amplitude columns `f64` — `-20_905_355.0` needs 25 mantissa bits, so `f32` would **not** be exact and would silently lose lunar-distance precision. Enables PERF-19 (index the E-power table directly) and VEC-1. |

## Decoupling

| id | status | effort | description | notes |
|---|---|---|---|---|
| DEC-1 | DECIDED | S | `cli/src/cmd/render/svg_common.rs::write_year_axis` calls `celestial_core::revjul` / `julday` directly from a shared render helper. | Acceptable thin veneer: axis ticks need JD/Gregorian conversion; closure injection would be heavier than the coupling. |

## Dependency

| id | status | effort | description | notes |
|---|---|---|---|---|

## Design Thinking

| id | status | effort | description | notes |
|---|---|---|---|---|

## Documentation

| id | status | effort | description | notes |
|---|---|---|---|---|
| DOC-4 | DECIDED | M | `celestial-cli` public items are intentionally undocumented. | Binary crate/lib split only exists so `main.rs` and tests can share modules; `#![warn(missing_docs)]` remains core-only. |
| DOC-13 | OPEN | S | Replace obsolete test/API counts and align CI coverage descriptions with enforced thresholds. | README says 1496 tests / 84 fuzz suites; docs say 1174 core tests and 194 binding functions. The current workspace run reports 1847 passes and 2 failures, the fuzz harness has 88 suites, and parity reports 270 exports. The core workflow header promises ≥80% line / ≥90% function coverage while commands enforce 75% / 78%. Generate counts or link authoritative outputs and correct the workflow header. |
| DOC-14 | OPEN | M | `docs/javascript.md` uses snake_case for 22 unique native calls (`calc_ut`, `set_sid_mode`, `moon_phase`, etc.), while the generated TypeScript API exports camelCase (`calcUt`, `setSidMode`, `moonPhase`, etc.); the guide's examples therefore fail at runtime. | Rewrite calls from `bindings/js/index.d.ts` and add an executable documentation smoke check so examples cannot drift from napi export names. |
| DOC-15 | OPEN | S | Correct mathematical units and transform-direction prose against the core contract. | `docs/api_reference.md:91` and `docs/javascript.md:137` label `mean_sidtime` as degrees although core returns hours. `core/src/functions/utils.rs:218` says positive obliquity transforms ecliptic to equatorial, opposite the implemented Swiss rotation: positive obliquity maps ecliptic longitude 90°/latitude 0° to negative declination. Fix prose and link language guides to generated signatures instead of duplicating them. |

## Legacy / Deprecation

| id | status | effort | description | notes |
|---|---|---|---|---|
| LEG-3 | OPEN | M | Public Swiss-compatible setup APIs advertise state changes but silently no-op: `set_ephe_path` / `set_jpl_file` always return `Ok(())`, `set_tid_acc` / `set_lapse_rate` discard input, and `tid_acc` always returns 0. They are exported through the bindings as working setters. | Either implement observable supported behavior or explicitly mark/deprecate compatibility no-ops and return an unsupported error where signatures permit. Keep `FLG_JPL`'s documented built-in fallback distinct from accepting a file that is never used. |

## Multithreading

| id | status | effort | description | notes |
|---|---|---|---|---|

## Observability

| id | status | effort | description | notes |
|---|---|---|---|---|

## OKR

| id | status | effort | description | notes |
|---|---|---|---|---|

## PDCA

| id | status | effort | description | notes |
|---|---|---|---|---|
| PDCA-1 | OPEN | S | Include every contract/dependency input in CI path filters. Documentation-only edits never trigger `binding-parity`; edits to `bindings/ffi/**` never trigger the JS/Python/PHP native workflows, and shared `tests/test-util/**` edits do not trigger the CLI workflow. | `.github/workflows/{binding-parity,celestial-js,celestial-python,celestial-php,celestial-cli}.yml` push and pull-request filters omit those paths. Add generated API docs/language guides and transitive source/test inputs so required gates run for the changes they protect. |

## Performance

| id | status | effort | description | notes |
|---|---|---|---|---|
| PERF-8 | DECIDED | S | `svg_common::svg_doc_open` chains several small string allocations per chart. | One call per chart; combining needs templating/build-time concat and is net-neutral. |
| PERF-9 | DECIDED | S | `parse_chart_type` calls `registered_chart_types()` twice on invalid input. | One-shot CLI parse path; cosmetic micro-perf. |
| PERF-2/3 | DECIDED | — | Search code re-evaluates final `calc_ut` / `houses` with full flags after bisection. | Authoritative final result, not redundant; locked by regression tests. |
| PERF-5 | DECIDED | M | `next_aspect_with2` dual-scan merge could be unified. | Precision-sensitive rewrite for marginal gain; byte-identical gate made it a deliberate no-op. |
| SONAR-JS-3 | DECIDED | — | Sonar `javascript:S7721` asks to hoist two helpers out of test-suite callbacks. | KEEP — each callback executes once; moving local fixture helpers outward adds scope without avoiding repeated allocation. |
| PERF-13 | OPEN | S | Polynomials are written as expanded powers instead of Horner form: `nutation.rs:40` expands `t`…`t⁵` (15 multiplies for a 6-term series), `moon.rs:29/34/39/42/51/59` expand `t²`–`t⁴` for six fundamental arguments, plus `nodes.rs:77`, `rise_set.rs:223`, `planetary.rs:220/234`, `ayanamsa.rs:213`. | **Precision measured, stands.** At the three pinned `moon.rs` epochs the Horner and Horner+FMA results for `L'` are **bit-identical** to the expanded form (delta exactly 0); `mean_obliquity` shifts 4.0e-15° against a 1.0e-12° gate (`nutation.rs:383`). Horner is also the numerically *better* form — the expanded version is the one accumulating error. **Blocker:** `nodes.rs:436` is a bit-exact `assert_eq!(f64_fingerprint(…))`, so touching `nodes.rs:77` forces re-pinning that hash in the same commit. |
| PERF-14 | OPEN | S | `functions/phenomena.rs:217` (`yallop_q`) and `astronomy/phenomena.rs:105–115` (`visual_magnitude`, Moon/Mercury/Venus arms) evaluate quartics via `powi(2)`/`powi(3)`/`powi(4)` plus separate `i * i`. | Stands on arithmetic grounds — up to 9 multiplies per arm versus 4 FMAs. **Blocker:** `astronomy/phenomena.rs:180` is a bit-exact fingerprint gate over `compute_phenomena`, so this one cannot land without re-pinning the hash. Weigh that against the size of the win before starting; the visibility magnitudes are reported to ~0.01 mag. |
| PERF-15 | OPEN | S | Term-argument accumulation in the two hottest series loops uses plain mul/add: `moon.rs:77` and `moon.rs:97` build `dc*d_r + mc*ms_r + mpc*mp_r + fc*f_r` (120 rows per call), `nutation.rs:126` builds a 5-term dot product over 77 rows. | **Precision measured, stands.** Randomized sweep (200k argument sets over representative ELP rows) puts the FMA-vs-plain divergence at **9.3e-16° worst case** in lunar longitude against the 1.0e-12° gate at `moon.rs:326` — a 1000× margin. `vsop87.rs:28` already uses `mul_add` for exactly this; the lunar and nutation series were never converted. |
| PERF-17 | OPEN | S | Hoist fixed obliquity trigonometry and remove the unit-multiplier FMA in `fixstars.rs::star_ecliptic_pos`. | The catalog scan calls the transform per star; `23.4393_f64.to_radians().sin_cos()` is constant, and `y_lon` multiplies by 1.0 inside `mul_add`. Prior 200k-sample measurements found the FMA removal bit-identical; hoisting deterministic trigonometry preserves results. The previous distance-division fold is superseded by REL-64: catalog parallax is already in arcseconds, so preserving the old factor of 1000 would retain a correctness bug. |
| PERF-18 | OPEN | S | `planetary.rs:58` light-time iteration recomputes each coordinate difference twice: `((xt - xe) * (xt - xe) + (yt - ye) * (yt - ye) + (zt - ze) * (zt - ze)).sqrt()`. | **Bit-identical** — binding `dx`/`dy`/`dz` evaluates the same subtractions to the same values. Zero precision risk. The surrounding code at lines 46–49 and 64–69 already does it this way. Runs twice per `apparent_planet`. |
| PERF-19 | OPEN | S | Both lunar series loops select the eccentricity factor per term with `match mc.abs() as i32 { 1 => e, 2 => e2, _ => 1.0 }` (`moon.rs:79`, `moon.rs:99`) — a float `abs`, a float→int cast, and a compare chain for each of the 120 terms per call. | **Bit-identical** — same three `f64` values, only the selection mechanism changes. `mc` is always in `-2..=2`, so `[1.0, e, e2][mc.abs() as usize]` is a branchless load; better still, fold the E-power index into the static table as an extra column so the cast disappears. Pairs with DS-1. |
| PERF-20 | OPEN | S | Loop-invariant trigonometry recomputed inside cusp loops in `astronomy/houses.rs`: `eps_r.cos()` inside `meridian` (line 650), `morinus` (line 667), `campanus` (line 602) and both Koch loops (lines 462, 477); the Koch loops also call `armc_r.sin()`/`armc_r.cos()` two to three times per iteration even though `sin_armc` is already bound at line 452. | **Bit-identical** — hoisting a deterministic libm call out of a loop cannot change its result, and `f64::sin_cos` returns exactly `(self.sin(), self.cos())`. The `houses` gate at line 1184 is a 1e-10 tolerance anyway. Covered by the `houses::houses_ex_*` benches. |
| PERF-21 | OPEN | M | `cli/src/cmd/render/geometry.rs:11–18` splits wheel projection into `wx`/`wy`, so each of the 68 paired call sites evaluates `wheel_angle` (a `rem_euclid`) twice, `to_radians` twice, and an independent `sin` and `cos` on the identical angle. `builtin_svg.rs:520–521` and `:525–526` repeat the pattern inline. | **Bit-identical**, therefore safe against the byte-exact SVG goldens: `sin_cos` is defined as `(self.sin(), self.cos())`, and the hoisted `wheel_angle` is one evaluation of an expression that was previously computed twice to the same value. Add `wxy(cx, cy, r, lon, asc) -> (f64, f64)` and convert the paired sites; keep `wx`/`wy` for the few unpaired uses. |

## Platform

| id | status | effort | description | notes |
|---|---|---|---|---|
| PLAT-2 | OPEN | M | Make the advertised Rust 1.82 minimum an inherited and tested package contract. | `Cargo.toml` sets only `[workspace.package].rust-version`; `cargo metadata --no-deps` reports `rust_version: null` for every member because none inherits it. `docs/building.md:12` promises 1.82, while code uses newer integer `is_multiple_of` and CI tests only stable. Declare the actual supported minimum, align dependencies/source, and add an MSRV build/test job. |

## Plugin Extensibility

| id | status | effort | description | notes |
|---|---|---|---|---|
| PLUG-1 | OPEN | S | `discover()` inserts a plugin name into `seen` before checking executability. A non-executable `celestial-foo` earlier on `PATH` suppresses a later executable one from `--list-plugins`, while `try_exec` independently scans onward and can dispatch it. | Mark a name seen only after accepting an executable candidate; add a two-directory PATH test that keeps discovery and dispatch consistent. |

## Product Engineering

| id | status | effort | description | notes |
|---|---|---|---|---|
| PROD-3 | OPEN | M | Remove the obsolete external-ephemeris requirement from native JS/Python tests and repair the hidden reference failures. | Core requires no data files, but `bindings/js/tests/celestial.test.ts:27–32` and `bindings/python/python/celestial_py/tests/conftest.py:54–58` skip suites unless `SWISSEPH_EPHE_PATH` is set; native CI never sets it. Freshly rebuilt extensions, tested outside the repo: default JS 27 pass/7 skip; enabled 29 pass/5 fail. Default Python 238 pass/16 skip; enabled 240 pass/14 fail. Separate obsolete numeric/shape/exception fixtures from engine defects before changing assertions; preserve independent references within supported accuracy. |
| PROD-4 | OPEN | S | Make the JS pure-logic approximate assertion reject non-finite results. | `bindings/js/tests/pure_logic.test.mjs:73` throws only when `Math.abs(val - exp) >= tol`; NaN or undefined therefore passes every finite expected-value check. Require finite actual/expected values and a valid tolerance before comparison, and mutation-validate failure behavior rather than trusting a green pure-logic suite. |

## Purpose

| id | status | effort | description | notes |
|---|---|---|---|---|

## Reliability / Correctness

| id | status | effort | description | notes |
|---|---|---|---|---|
| REL-47 | OPEN | M | Apply requested coordinate transformations consistently across body and fixed-star dispatch. | `core/src/astronomy/engine.rs:48–50,60–62,90–174` returns nodes/Chiron/Pluto before sidereal/equatorial/topocentric/heliocentric processing, and the VSOP heliocentric branch before equatorial/sidereal transformations. `core/src/functions/calc.rs:129–141` uses fixed-star flags only as echoed return metadata; native Sirius coordinates are identical with and without `FLG_EQUATORIAL`. Add per-body/composed-flag checks against explicit transformations; reject unsupported combinations instead of claiming they were applied. |
| REL-48 | OPEN | S | Restore the failing Omer overlay regression before claiming a green workspace baseline. | `cargo test --workspace --exclude celestial-fuzz` on 2026-09-06 fails `cmd::render::tests::overlays_merge_into_non_calendar_context` at `cli/src/cmd/render/mod.rs:306`: expected Omer day 23, returned 22. Determine whether the overlay's sunset convention or the fixture is wrong; preserve the intended calendar boundary in a mutation-validated regression. |
| REL-49 | OPEN | S | Update the stale Almuten property oracle for additive dignity scores. | `fuzz/src/main.rs:2499` still requires `alm_score` in `-5..=5`, while `core/src/functions/hellenistic.rs:337–385` sums domicile/exaltation/triplicity/term/decan. `cargo run --manifest-path fuzz/Cargo.toml --quiet` reports 87 passing suites and `full_dignity` failing 14,000 checks on valid scores 6–11. Assert the documented sum and correct bounds, then mutation-validate the revised oracle. |
| REL-50 | OPEN | S | Reconcile Ba Zi and medicine-wheel SVG output with committed fixtures after validating the intended calendar/totem values. | The full workspace run fails `cli/tests/svg_snapshot.rs:101` first on `bazi`. Separate comparison of all 27 renderers finds exactly two mismatches: Ba Zi day/month pillars and element counts (28 changed lines), and medicine-wheel Beaver/Earth/Turtle versus Deer/Air/Butterfly (4 changed lines). Update only verified intended goldens; make the snapshot check report every mismatching renderer. |
| REL-51 | OPEN | M | Compute a complete topocentric position and derive speeds from the same transformed coordinates. | `core/src/astronomy/engine.rs:68–83,184–268` changes only longitude/RA; latitude/declination, distance, and ecliptic speed remain geocentric. Native Moon probe at JD 2463456.789, observer 12.5°E/37.5°N: longitude changes 206.6422906° to 206.9870961°, while the other five components remain identical. Verify all components against observer-vector geometry. |
| REL-52 | OPEN | M | Correct the orbital-element adapter's named fields. | `core/src/functions/calc.rs:267–277` assigns longitude of perihelion to `arg_perihelion`, and mean longitude to `mean_anomaly`, although `core/src/astronomy/nodes.rs:94–105` defines those source fields explicitly. Subtract ascending-node longitude and perihelion longitude respectively, normalize angles, and supply meaningful epoch/motion instead of zeros; clarify the advertised osculating versus implemented mean-element model. |
| REL-53 | OPEN | M | Anchor rise/set calculations at the civil midnight containing the input and enforce the public search-time convention. | `core/src/astronomy/rise_set.rs:92` uses `jd_ut.floor() + 0.5`, moving the day anchor at noon. Native sunrise results jump between adjacent civil days across noon; input JD 2460400.75 returns the already-past event 2460400.7059082. Use `(jd + 0.5).floor() - 0.5` for a civil-date calculation and advance if the API promises the next event; test before/after noon and before/after the event. |
| REL-54 | OPEN | L | Make location-specific eclipse searches actually select locally visible events and compute local circumstances. | `core/src/functions/eclipses.rs:76–94,188–201` delegates to global searches; lunar location is discarded, and solar attributes merely echo coordinates. Native probes at opposite longitudes return identical eclipse times/classifications; lunar attributes are all zero. Add horizon/path filtering and local contacts, or return unsupported instead of labeling a global event locally visible. The same issue affects `lun_occult_when_loc:386–420`, which additionally treats ecliptic longitude/latitude as RA/declination. |
| REL-55 | OPEN | M | Return no-eclipse results outside an event and classify lunar eclipses from the actual kind. | `core/src/functions/eclipses.rs:98–170,205–227`: solar `how` uses the nearest lunation, solar `where` silently searches another date, and lunar `how` ignores the returned kind and always emits a nonzero type. Native `lun_eclipse_how(2451545.0)` reports penumbral flags with zero magnitude. Evaluate the requested instant/location; distinguish total, partial, penumbral, and absent eclipses. |
| REL-56 | OPEN | M | Handle solar illumination separately from the reflected-light phase triangle. | `core/src/astronomy/phenomena.rs:54–63` applies the planet formula to the Sun; `core/src/functions/phenomena.rs:126–141` supplies its Earth distance as both triangle sides. Native `pheno_ut` at JD 2463456.789 reports Sun phase angle 59.131684° and illumination 0.756533. The regression at `functions/phenomena.rs:280–289` pins that incorrect result. Add an independent solar contract instead of retaining a computed fingerprint. |
| REL-57 | OPEN | S | Use the secondary-progression rate in the CLI progressed chart. | `cli/src/cmd/render/derived.rs:28` advances by `years * 365.25`, whereas `core/src/functions/chart.rs::secondary_progressions` correctly advances one day per year. A 30-year CLI progression advances 10,957.5 days instead of 30. Share the core calculation and compare the rendered/context positions with its output. |
| REL-58 | OPEN | M | Transform angular velocities along with positions in `coord_transform_with_speed`. | `core/src/functions/utils.rs:246–250` rotates positions and copies velocities unchanged. For `[40, 20, 1, 1, 2, 0]` with obliquity −23.4393°, returned angular speeds are `(1, 2)` but position finite differences give approximately `(0.171066, 2.205163)`. Rotate Cartesian position/velocity vectors or apply the spherical-coordinate Jacobian; test transformed speeds, which existing position round trips cannot validate. |
| REL-59 | OPEN | S | Remove the duplicate equation-of-equinoxes correction from `sidtime0`. | `core/src/functions/time.rs:270–277` names an already apparent sidereal time `gmst`, then adds the supplied nutation correction again. At J2000, passing the engine's own obliquity/nutation makes `sidtime0` differ from `sidtime` by −0.852183 seconds. Start from `mean_sidereal_time_deg` and test equivalent internally computed versus supplied corrections. |
| REL-60 | OPEN | S | Correct the equation-of-time sign to match the documented apparent-minus-mean convention. | `core/src/functions/time.rs:328–333` computes apparent RA minus mean RA, reversing the solar-time difference. At 2024-02-11 noon UT, native `time_equ` gives +14.202 minutes; the [NOAA solar equations](https://www.gml.noaa.gov/grad/solcalc/solareqns.PDF) give a negative correction. `core/tests/external_reference.rs:1095–1120` describes the negative February value but only asserts its absolute magnitude. Test signed February/November references and dependent apparent-solar-time output. |
| REL-61 | OPEN | M | Use floor-based century division and nonnegative day fractions for BCE/negative-JD calendar conversions. | `core/src/functions/time.rs::julday/revjul` uses truncating `a / 4` / `alpha / 4`; `revjul` also uses signed `fract()`. Native Gregorian year lengths are wrong for −400 (365), −300 (366), and 0 (365); `revjul(-1.0, JUL_CAL)` returns hour −12. Add independent proleptic-calendar vectors around negative century/leap boundaries and valid clock-range checks. |
| REL-62 | OPEN | M | Convert between local apparent and mean solar time using the equation of time. | `core/src/functions/time.rs:337–349` implements `lat_to_lmt` / `lmt_to_lat` solely as subtracting/adding longitude divided by 360. Both inputs are already local time; at Greenwich they incorrectly become identity conversions throughout the year. Use longitude to obtain the UT epoch for the solar correction, apply the corrected signed equation of time, and verify independent nonzero-correction references plus round trips. |
| REL-63 | OPEN | M | Compute the current orbital distance instead of returning the semi-major axis. | `core/src/functions/calc.rs:282–303` sets `orbit_max_min_true_distance().dtrue = el.semi_major` regardless of mean anomaly. Solve the supported orbital model at the requested epoch or use its heliocentric position, and compare perihelion/aphelion and intermediate dates; distinguish model approximations from a constant placeholder. |
| REL-64 | OPEN | S | Remove the thousandfold unit error in catalog-star distances. | `core/src/astronomy/fixstars.rs:27` defines parallax in arcseconds, but `star_ecliptic_pos:1172` divides it by 1000 before converting parsecs to AU. Sirius has `plx = 0.380`; native output is 542,802,631.579 AU instead of approximately 542,802.632 AU. Correct the conversion and the fingerprints at lines 1212–1229; derive reference distances independently from catalog parallax. Correctness takes precedence over the previous PERF-17 arithmetic fold. |
| REL-65 | OPEN | M | Include catalog proper motion in returned fixed-star angular speeds. | `core/src/astronomy/fixstars.rs:1183–1186` ignores its star argument and returns the same longitude speed and zero latitude speed for every star, while `star_ecliptic_pos` evolves both coordinates using each star's proper motion. Derive velocities from the same position model and coordinate frame; compare finite differences for stars with distinct nonzero proper motions. |
| MUT-1 | OPEN | L | Full `cargo-mutants` baseline finds surviving core mutants, beginning with unasserted rise/set dispatch arms, body mappings, and derived constant values. | User stopped the fresh sweep at 1,979/6,835 mutants: 9 missed and 3 timed out are tracked below; 4,856 remain untested. Resume from the preserved `/tmp/celestial-mutation-shard-*` checkpoints when requested. |
| MUT-5 | OPEN | S | `crossings::CROSSING_TOL_X` survives changing the day/second conversion from division to remainder, weakening crossing-time convergence by five orders of magnitude. | Add an exact public crossing precision regression, then rerun the mutant with the full package suite. |
| MUT-6 | OPEN | M | Reproduce and diagnose previously recorded crossing-predicate mutant timeouts against current source. | Nine prior predicate mutants timed out: five in `opposite_sign`, three in `brackets_root`, and `&&` to `\|\|` in `bracket_crossing`; `opposite_sign -> true` also timed out at 300 seconds. Current `bracket_crossing` and `refine_crossing` already have finite probe/iteration bounds, so the previous claim of an unbounded inner loop is unsupported. Restore the failing baseline first, rerun these mutants, and trace repeated callers or excessive bounded work before choosing a production limit. No mutation rerun during this rescan. |
| MUT-7 | OPEN | S | `planetary::apparent_planet` has eight surviving mutations across Earth-coordinate subtraction and the initial geocentric distance sum-of-squares arithmetic. | Add exact multi-epoch public geocentric position fingerprints that exercise every rectangular axis and the light-time corrected distance calculation. |
| REL-43 | OPEN — parked | L | `losar_jd` assumes Losar is always the second new moon after the winter solstice; it returns 2025-01-29, while the official 2025 Tibetan Losar date is 2025-02-28 because the shortcut omits Phugpa leap-month rules. | Needs the full Phugpa true-month/intercalation, true-date correction, and skipped/repeated-day algorithm; do not substitute a one-year table or heuristic. Validate Janson's 2019–2027 vectors plus official dates when implemented. |
| SONAR-BIND-1 | DECIDED | — | Sonar `typescript:S7758` prefers `codePointAt` at four FFI house-system call sites. | KEEP — `charCodeAt(0)` intentionally supplies a required one-byte ASCII integer; `codePointAt` widens the type with `undefined` without supporting a valid extra input. |
| SONAR-PY-3 | DECIDED | — | Sonar `python:S1244` flags two exact float equalities in binding tests. | KEEP — half-day Julian values are exactly representable and coordinate transformation must preserve the distance component bit-for-bit. |
| REL-16 | OPEN | L | Public TT/ET and UT variants have contradictory time-scale behavior. `fixstar_ut`, `fixstar2_ut`, `nod_aps_ut`, `ayanamsa_ut`, and `ayanamsa_ex_ut` pass UT straight into TT math; `solcross` / `mooncross` are documented as ET but search with `calc_ut`; `helio_cross_ut` is an exact alias of the ET function. Tests often require equality for the same numeric JD, locking in the mismatch. | Define the scale of every input/output, apply ΔT conversion at one boundary, and replace alias-equality tests with equivalent-instant tests (UT input versus TT input shifted by ΔT) plus external reference values. |
| REL-46 | OPEN | S | `norm_rad` has the identical wrap defect just fixed in `norm_deg`: both copies (`functions/utils.rs:264`, `astronomy/constants.rs:61`) document `[0, 2π)` but return exactly `TAU` for tiny negative inputs. Verified while fixing REL-45: `(-1e-18f64).rem_euclid(TAU) == TAU`, likewise `-1e-30` and `-f64::MIN_POSITIVE`. | Same one-line clamp as `norm_deg` (`if r >= TAU { 0.0 } else { r }`) plus a denormal-input test in each copy. Lower blast radius than REL-45 — no `(norm_rad(x) / k) as usize` slot index was found — so it is correctness hygiene on the documented contract rather than a live out-of-range bug. Deliberately left out of the REL-45 commit to keep that change scoped to the reported item. |

## Robustness / Recovery

| id | status | effort | description | notes |
|---|---|---|---|---|
| ROB-4 | OPEN | M | Validate calendar boundaries and widen arithmetic before subtraction/addition at public binding entry points. | Native Python probes raise Rust `PanicException`: `coptic_to_jd(2024, 0, 1)` at `core/src/functions/coptic.rs:107`; Gregorian/Solar-Hijri conversions at `nowruz.rs:47/53` for signed-integer endpoints; `maya_long_count(-1e30)` at `mesoamerican.rs:200`. Coptic year subtraction also precedes widening. Return documented errors/sentinels for invalid inputs, use checked/widened arithmetic, and exercise debug/release plus native-language boundaries. |
| ROB-5 | OPEN | S | Keep shared fuzz RNG range arithmetic in a widened type until the final conversion. | `tests/test-util/src/lib.rs:47–55` casts offsets to signed integers before adding the lower bound. A standalone debug probe using seed 1 and 1000 draws from each valid full signed range panics in both `range_i32` and `range_i64`. Widen the addition or use a justified wrapping mapping, then test wide intervals and endpoint containment in debug/release. |
| ROB-3 | OPEN | M | Normal broken-pipe use panics: piping a rendered chart to `head` exits 101 from a stdout write failure. Output is spread across `print!` / `println!`, so other verbose commands have the same failure mode. | Route stdout through fallible buffered writes and treat `BrokenPipe` as a clean early exit; add a subprocess pipe-closure regression test. |

## Scalability

| id | status | effort | description | notes |
|---|---|---|---|---|
| SCALE-1 | OPEN | M | Bound graphic-ephemeris sampling from the actual requested interval. | `cli/src/cmd/render/specialist.rs:330–356` clamps a `days` label to 366, limiting the sampling step to four days, but allocates and iterates from the full unbounded date span. A finite billion-day interval requests about 250 million samples per planet before rendering. Reject unsupported spans or derive a bounded sample count/step from the full interval, checking allocation arithmetic; verify large-range handling without allocating huge buffers. |

## Security

| id | status | effort | description | notes |
|---|---|---|---|---|
| SEC-8 | OPEN | M | Escape user-controlled values in bundled SVG templates. | `cli/src/cmd/render/pipeline.rs:262` disables autoescaping for all templates; `example.svg.tt:21` and `year_calendar.svg.tt:10` interpolate raw titles. Rendering either with `--var 'title=A & B'` produces malformed XML; markup supplied through CLI/config variables is also emitted verbatim. Enable XML-appropriate escaping or explicit escaping at interpolation sites, preserving intentional markup with an explicit contract; parse generated XML in regression checks. |
| SEC-7 | OPEN | M | Config path containment rejects absolute paths and `..` but follows symlinks. A relative config `out = "escape/chart.svg"` writes outside cwd when `escape` is a symlink; config `template` can likewise read outside. This contradicts the code's untrusted-config containment claim. | Resolve/open relative to a trusted cwd directory handle with symlink-safe containment (including existing ancestors and the final target); add symlink escape tests for both read and write paths. |

## State Machine Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|

## UI / UX

| id | status | effort | description | notes |
|---|---|---|---|---|

## Vectorization

| id | status | effort | description | notes |
|---|---|---|---|---|
| VEC-1 | OPEN | M | Re-assessed: the three series loops (`vsop87.rs:28 eval_series`, `moon.rs:77/97`, `nutation.rs:125`) *are* pure reductions over static tables — the shape autovectorizers handle — but the array-of-structs row layout (`Term(f64,f64,f64)`, `[f64; 9]`, `[f64; 6]`) forces a gather per lane, so LLVM leaves them scalar. | Structure-of-arrays columns plus the narrowed multipliers from DS-1 would let the `mul_add` + `sin_cos` reductions vectorize. Note `sin_cos` is a libm call and will not vectorize without an SLEEF-style vector math dependency — measure the argument-accumulation half alone before taking on a dependency. Do PERF-13/15 and DS-1 first; they may close most of the gap on their own. **Precision caveat:** unlike DS-1, this one is *not* free — a vectorized reduction sums in lane order and then folds partial accumulators, which is a different summation order and therefore a different result. That is the one change in this batch that must be re-validated against every fingerprint and tolerance rather than reasoned about, so it should be the last thing attempted, if at all. |

## Wiring Gaps

| id | status | effort | description | notes |
|---|---|---|---|---|
| WIRE-4 | OPEN | M | Implement or explicitly reject advertised calculation flags that the engine never reads: `FLG_TRUEPOS`, `FLG_J2000`, `FLG_SPEED3`, `FLG_NOABERR`, and `FLG_BARYCTR`. | `core/src/constants.rs:161–186` promises changed calculation behavior; `core/src/astronomy/engine.rs::calc_tt/compute_speed` always uses apparent positions and only checks `FLG_SPEED` for velocity. Returned flags echo ignored requests. Keep the already documented JPL/Moshier/NONUT/XYZ/RADIANS limitations distinct. |
| WIRE-5 | OPEN | M | Honor supported house-coordinate flags in `houses_ex` and `houses_ex2`, or reject them explicitly. | `core/src/functions/houses.rs:53–69` discards `_flags`, so sidereal requests return tropical cusps and angles. Route flags through house computation and test the ayanamsa offset and speed contract. |
| WIRE-6 | OPEN | M | Wire rise/set options through the actual solver. `CALC_ITRANSIT` falls back to sunrise, and custom horizon, pressure, temperature, observer altitude, and fixed-star name are discarded. | `core/src/functions/motion.rs:115–135,143–207`: `RiseSetEvent` has no lower-transit variant; `rise_trans_true_hor` drops `_horhgt`. `RiseTransOptions` advertises these settings. Implement supported behavior or return an explicit unsupported-input error, with observable option-effect tests. |
| WIRE-7 | OPEN | M | Wire user-defined ayanamsa epoch/value through configuration and sidereal calculations. | `core/src/functions/config.rs:106` drops `_t0`/`_ayan_t0`; `astronomy/ayanamsa.rs::SID_MODE_TABLE` omits the `User` variant, so mode 255 falls back to Lahiri. Native `set_sid_mode(255, 2451545.0, 10.0)` followed by `ayanamsa(2451545.0)` returns 23.85357 instead of the supplied epoch value. Preserve both parameters in thread snapshots and test batch propagation. |
| WIRE-8 | OPEN | M | Resolve heliacal object names to the requested body/star instead of substituting another object. | `core/src/functions/phenomena.rs:165–184` maps every catalog star to Venus and every unknown name to Mercury. Native `heliacal_ut` returns byte-identical results for `Sirius` and `Venus`; a nonexistent name returns a Mercury event. Use catalog coordinates for supported fixed stars and reject unknown/unsupported objects; apply the same resolution to `heliacal_pheno_ut` and `vis_limit_mag`. |
| WIRE-9 | OPEN | M | Connect `gauquelin_sector` to diurnal-sector geometry. | `core/src/functions/phenomena.rs:29–57` ignores method, body latitude, and all horizon inputs, returning an integer equal-ecliptic bin `(lon - asc) / 10`. Its own contract promises 18 sectors above and 18 below the horizon. Reuse the available sector/diurnal calculation where appropriate and test horizon/meridian anchors at nonzero latitude. |
| WIRE-10 | OPEN | M | Render the directed positions in built-in solar-arc charts and recompute their derived geometry. | `cli/src/cmd/render/derived.rs:46–82` stores `directed_planets`, changing only longitude; registry dispatch calls `render_progressed_svg`, which renders the original natal `planets`. Fresh SVG output for 10 versus 30 years differs only in the title. Feed directed positions into the renderer and rebuild sign/degree/geometry and relevant house/aspect data consistently. |
| WIRE-11 | OPEN | M | Draw the second ring and cross-chart aspects in the built-in biwheel renderer. | `cli/src/cmd/render/derived.rs:90–142` builds `outer_planets` and `cross_aspects`, then `render_biwheel_svg` delegates to the single natal wheel without consuming either. Changing only the secondary date from 1990 to 2025 produces byte-identical SVG; the bundled synastry template does consume the outer ring. Connect the built-in renderer and verify visible changes from secondary date/location inputs. |
| WIRE-1 | DECIDED | — | Public Rust APIs including `get_orbital_elements`, `heliacal_pheno_ut`, `current_file_data`, and several setup/calculation helpers have no CLI call site. | KEEP — `celestial-core` is a published library crate, so absence of an internal caller does not imply dead code. Root exports are either bound or explicitly recorded in `xtask/core_unbound_allow.txt`; missing advertised behavior is tracked separately under the relevant open findings. |
| WIRE-2 | DECIDED | S | `Body::is_known_id` is only used internally by `try_from_raw` plus tests. | KEEP — useful published predicate for callers that want a cheap bool pre-check. |
| WIRE-3 | DECIDED | S | `BodyError::OutOfRange { id }` is constructed but bindings stringify it instead of pattern matching. | KEEP — typed Rust variant preserves future extensibility while bindings expose language-native string errors. |

## Unused Functions / Methods

| id | status | effort | description | notes |
|---|---|---|---|---|

## Audit picks deliberately rejected

| id | status | effort | description | notes |
|---|---|---|---|---|
| PERF-16 | REJECTED | S | Proposed collapsing `functions/utils.rs:298 diff_deg_signed` from `wrap_signed_180(norm_deg(p1) - norm_deg(p2))` to `wrap_signed_180(p1 - p2)`, on the claim that `rem_euclid` is homomorphic over subtraction mod 360. | **Measured and withdrawn — it loses precision on out-of-range input.** Randomized sweeps: inputs already in `[0, 360)` are bit-identical (0/500,000 mismatches), but over ±1e6° 49,342/500,000 diverge with a worst error of 1.2e-10°, and over ±1e17° the worst error is **352°** — the argument reduction has to happen *before* the subtraction or the small operand is annihilated. The current triple-`rem_euclid` form is deliberate robustness for a public API that accepts arbitrary `f64`. The original entry also overstated the cost: `f64::rem_euclid` is one `%` plus a branch, not two `fmod` calls, so the remaining saving is ~1 `fmod` and not worth the exposure. |
| REL-17 | REJECTED | S | Previously reported Mesoamerican bundled-template failure is not reproducible with the current source and matching chart type. | Fresh build: `render --chart-type mesoamerican --date "2000-01-01 00:00" --tz UTC --template cli/templates/mesoamerican_calendars.svg.tt` exits 0 and emits valid XML; the 2024-01-01 case also passes. Reopen only with a failing invocation and matching source revision. |
