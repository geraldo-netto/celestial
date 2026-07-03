# Celestial — TODO

Rescan: **2026-07-02c** (follow-up all-category rescan incl. the core-correctness cluster that aborted in 02b; 4 parallel audit agents — core astronomy math numerically verified with a scratch harness, core infra/concurrency, soft categories verified by executing the shipped binary, cli-render/xtask re-sweep). Current open rows from this pass remain in the tables below; rows fixed in the follow-up implementation were removed. Checked-clean: ELP2000 vs Meeus 47.a, computus, molad/dechiyot, tabular Hijri, coptic/ethiopic, no static-mut/atomics in core, SVG goldens pinned.
Rescan: **2026-07-02b** (deep all-category rescan; 5 parallel audit agents over core/cli/bindings/perf/security; findings source-verified — CLI ones confirmed by running the built binary; high-severity binding/CLI/security claims spot-checked before entry). New: CLI-1..3, SM-1/2, ROB-2, UX-1, REL-14/15, DEC-2, SEC-6, PLAT-1, PERF-11/12, DOC-12, LEG-2, BIND-8. NOTE: the core-correctness cluster (astronomy math / calendars) aborted on a session limit with no output — re-run that cluster next pass. Baseline: `cargo test --workspace --exclude celestial-fuzz` green, `clippy -W cognitive_complexity` = 0.
Rescan: **2026-07-02** (whole-project scan across all categories in `AGENTS.md`; 5 parallel category-cluster audits, findings verified against source before entry, speculative/false-positive claims dropped).
2026-07-02 docs review + fix pass: all workspace `*.md` cross-checked against source. Fixed and rows removed — DOC-7 (README/context_schema omer `.sefirah`→`week/day_sefirah`, moon `.short`→`phase_short`, hebrew `holidays[]`→`years[]`/`days[]`, gregorian annotation tags), DOC-8 (context_schema planet `body`→`name`/`retrograde`→`retro`/`sign` idx+`sign_name`, aspect `aspect`→`aspect_name`+`aspect_deg`), DOC-9 (JS/PHP `FLG_SIDEREAL` 64→65536), DOC-10 (binding `nutation` docs → one arg, 2-value tuple/array, dropped fictional `NutationResult.eps_true`), DOC-11 (test counts re-derived from a single run: workspace 1496, core 1174, cli 293, py 216, js 162). Verify: `cargo test -p celestial-cli` templates_render + context_schema tests green; workspace count confirmed by `cargo test --workspace --exclude celestial-fuzz`. Verified-correct and NOT flagged: TZ table 203 entries, 27 chart types, 194 parity, `SYNODIC_MONTH`, `api_reference.md` nutation signature.
2026-07-02 verification: `cargo test --workspace` (1496 passed); `cargo clippy --workspace --all-targets` (clean); `cargo xtask parity` (194 fns); `cargo xtask pyi --check`; `cargo xtask dts --check`; `cargo check -p celestial-core --no-default-features {,--features timezone,--features calendar-traditions}`. New findings: REL-11 (retrograde template key), REL-13 (Koch diurnal_semi_arc stub), ROB-1 (panchanga silent ephemeris fallback), DOC-8 (schema retro/aspect key drift), LEG-1 (get_ayanamsa_name alias). REL-11 fixed same pass (`p.retrograde` → `p.retro`) — row removed. Dropped false positives: profection `cusps[13]` (wrap is correct), hallucinated `angles[]`/`arabic_parts[]`/`fixed_stars[]` schema keys (never emitted), "TODO.md missing" (exists), CLI items duplicating existing DECIDED rows.
Prior rescan: **2026-06-06** (whole-project scan across all categories in `AGENTS.md`; current TODO rows ignored as requested, then fresh findings de-duplicated before entry).
2026-06-06 verification: `cargo clippy --workspace --all-targets -- -W clippy::cognitive_complexity` (only recorded CC-1 warnings); `cargo test --workspace`; `cargo xtask parity`; `cargo xtask pyi --check`; `cargo xtask dts --check`; `cargo xtask test-stubs`; `cargo check -p celestial-core --no-default-features`; `cargo check -p celestial-core --no-default-features --features timezone`; `cargo check -p celestial-core --no-default-features --features calendar-traditions`; `cargo test -p celestial-core moon_phase`; `cargo run --manifest-path fuzz/Cargo.toml --quiet`.
Prior rescan: **2026-06-05** (whole-project scan across all categories in `AGENTS.md`).
2026-06-05 verification: `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -W clippy::cognitive_complexity`; `cargo test -p celestial-core --no-default-features`; `cargo check -p celestial-ffi --no-default-features`; `cargo check -p celestial-js --no-default-features`; `cargo check -p celestial-py --no-default-features`; `cargo check -p celestial-core --no-default-features --features timezone`; `cargo check -p celestial-core --no-default-features --features calendar-traditions`; `cargo xtask parity`; `cargo xtask pyi --check`; `cargo xtask dts --check`; `cargo xtask test-stubs`.
2026-06-05 fix pass cleared: REL-9, PERF-10 — rows removed per AGENTS.md.
Prior 2026-05-28 rescan findings are now fixed.
Prior 2026-05-27 fix pass cleared: REL-2, REL-7, REL-8, DUP-6, DUP-11, VIS-1 (partial), VIS-2, DEAD-3 — rows removed per AGENTS.md.
Prior 2026-05-27 COV-1 raised coverage gates to 93; retained as DECIDED test-coverage context in Documentation notes only.

## Adaptability

| id | status | effort | description | notes |
|---|---|---|---|---|
| BIND-1 | OPEN | S | `lun_eclipse_how` PHP wrapper (`bindings/php/src/lib.rs:2014`) drops the optional `geopos` observer arg present in Python/JS, so PHP callers cannot get local eclipse circumstances. | Surfaced by `cargo xtask coverage` arity check; parked in `xtask/arity_allow.txt`. Add the `Option<[f64;3]>` param to the PHP wrapper to match, then remove its allow-list line. |
| BIND-2 | OPEN | S | `match_aspect3`/`match_aspect4` expose an extra `def_orb` tuning arg in Python (`bindings/python/src/lib.rs:725,747`) that JS/PHP omit — inconsistent published arity. | Same gate. Either add `def_orb` to JS+PHP or drop it from Python; then remove the two allow-list lines. |
| BIND-3 | DECIDED | — | `utc_to_jd` takes 7 date scalars in Python/PHP but a single `UtcDate` object in JS (`bindings/js/src/lib.rs:605`). | KEEP — idiomatic JS object-packing, not drift. Recorded in `xtask/arity_allow.txt`. |
| BIND-5 | OPEN | M | `houses_ex` parameter ORDER differs: PHP is `(jd, flags, lat, lon, hsys)` but Python/JS are `(jd, lat, lon, hsys, flags)`. Code ported between PHP and Py/JS silently passes flags where lat is expected. | Surfaced by GATE-2 ordered-signature check; parked in `xtask/arity_allow.txt`. Align PHP to the Py/JS order (breaking for PHP callers) then remove the allow line. |
| BIND-6 | OPEN | M | `rise_trans` differs: Python omits `ephe_flags` and puts `flags` last `(…, temp, flags)`; JS/PHP put flags 3rd `(planet, flags, event_type, …)`. | Same gate. Reconcile to one param order across all three, then remove the allow line. |
| BIND-7 | OPEN | S | `sabbat_jd` takes `kind` as a `String` name in PHP but a `u8` enum index in Python/JS. | Same gate. Pick one (string name is friendlier) and align the other two, or document as intentional and keep the allow line. |
| GATE-5 | OPEN | M | No cross-language RUNTIME behaviour parity: nothing calls the built addons and diffs `calc_ut`/`houses` numbers across Python/JS/PHP/core. `reference_values.json` only covers ~15 categories in pure-logic (re-implemented math, not the native call). Deferred from the GATE-1..4 pass: needs built native addons executed in CI (an infra change), unlike the source-level gates. | Plan: (1) `cargo xtask golden` generates `tests/fixtures/binding_golden.json` from core for a fixed input set (`--check` keeps it synced to core); (2) each binding's *native* test suite (post-build job) loads the built addon and asserts numeric equality vs the fixture; (3) wire those native suites to actually run in the js/python/php build jobs. |
| BIND-8 | OPEN | M | `next_sabbat`/`next_esbat` return only `[jd]`/`f64` in JS (`bindings/js/src/lib.rs:2369,2386`) and PHP (`bindings/php/src/lib.rs:1885,2402`), dropping the name that Python returns as `(name, jd)` (`bindings/python/src/lib.rs:1851,1890`); the JS comment "name is available via the kind index" is false — no index is returned. | Return the kind index or name from JS+PHP to match Python, or document that callers must pair with `next_sabbat_name`. Not caught by the shapes gate (coarse kind only). |

## Architecture / Modularity / SOLID

| id | status | effort | description | notes |
|---|---|---|---|---|
| ARCH-10 | DECIDED | L | Bindings still expose broad per-language adapter surfaces rather than shared codegen-only APIs. | Same as DUP-1 / DP-4; byte/API churn and PHP unverifiable path made full codegen net-negative. |
| ARCH-11 | DECIDED | S | `bindings/ffi/src/lib.rs:18` uses `pub use celestial_core::*`, so new core public API can expand binding compile surfaces. | KEEP — bindings intentionally consume the whole published core facade. Explicit lists would duplicate 200+ root exports. |
| ARCH-12 | DECIDED | S | `core/src/lib.rs` re-exports raw Swiss-Ephemeris-style constants alongside typed `Body` / `CalcFlags` APIs. | Back-compat layer for bindings and existing Rust users; tightening would be a published API break. |

## Business / Design Patterns / DDD

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## CLI / Option Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CLI-1 | OPEN | S | `render --lat/--lon/--lat2/--lon2` (`cli/src/cmd/render/args.rs:89,93,151,155`) lack `allow_hyphen_values`, so the space form `render --lat -23.55` dies with `error: unexpected argument '-2' found` — every southern/western coordinate is blocked unless written `--lat=-23.55`. Verified: `houses.rs:19,23` already sets the flag. | Add `allow_hyphen_values = true` to the four render lat/lon args. |
| CLI-3 | OPEN | S | `--calendar` help/enum text (`cli/src/cmd/render/args.rs:~113`) lists only 5 values (gregorian, omer, sabbats, moon, hebrew), omitting `gregorian-year`/`year-calendar` (`CalendarKind::GregorianYear`) that `resolve_overlay` and README both accept. | Add `gregorian-year` to the help string. |
| CLI-4 | OPEN | S | `dispatch_profection` (`cli/src/cmd/render/registry.rs:230`) uses `args.return_year` directly as the profection AGE, so `render --chart-type profection --return-year 2026` computes age 2026 (verified: `"profection_age": 2026`, house 11) instead of the profection for target year 2026. | Derive age as `return_year − birth_year`, or reject `--return-year` for profection and require `--years`. |

## Code Complexity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CC-1 | DECIDED | — | `cli/tests/cli_fuzz.rs:225 boundary_empty_null_and_oversize_strings` CC 16/10 and `cli/src/parse.rs:423 parse_tz_forms` CC 14/10. | Test-only assertion/matcher density; real branching is low. `cargo clippy --workspace --all-targets -- -W clippy::cognitive_complexity` reports only these two. |
| CC-6 | DECIDED | S | `cli/src/cmd/render/calendar_overlays.rs:509 render_day_cell` sits at exactly CC 10. | Compliant but no headroom. If extended, split moon-glyph and omer-badge rendering into helpers. |
| CC-7 | DECIDED | S | `core/src/body/mod.rs:119 Body::is_known_id` is a flat range ladder. | Still compliant; flat documented id windows are clearer than hiding the ranges in a table. |

## Code Duplication

| id | status | effort | description | notes |
|---|---|---|---|---|
| DUP-1 | DECIDED | L | Binding return-adapter stubs remain repeated across JS/Python/PHP. | Per-language return shapes are intentionally different. `cargo xtask parity` confirms all 194 function exports stay aligned. |
| DUP-2 | DECIDED | S | `body_of(n)` validation helpers are present in each binding. | Per-language error mapping is the irreducible part; sharing would obscure simple error flow. |
| DUP-3 | DECIDED | — | Deterministic `core/tests/rel_clamps.rs` and randomized `fuzz/src/main.rs::test_rel_clamps` overlap on body-id edge coverage. | Intentional two-tier coverage split: `cargo test` regression plus opt-in fuzz/property run. |
| DUP-4 | DECIDED | — | `revjul` / `revjul_hms` have three language-specific return shapes. | Published API idioms differ; normalizing would break bindings. |
| DUP-5 | DECIDED | — | Per-tradition SVG wheel geometry uses similar-looking constants. | Distinct layout contracts; shared pieces are already factored. |

## Composition

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — render composition now flows through `ChartContext`, `CHART_REGISTRY`, and specialist dispatch helpers.)

## Concurrency

| id | status | effort | description | notes |
|---|---|---|---|---|

| CONC-2 | OPEN | S | Fuzz suites leak sidereal mode: `test_ayanamsa` (`fuzz/src/main.rs:391`) sets a random sid mode per iteration and never restores; `test_sidereal_all_modes` (:3695) "restores" to LAHIRI instead of the default Fagan-Bradley — suites after `run_core_suites` execute under a leftover mode, making results order/N-dependent. | Wrap both in a save/restore guard like panchanga's `SidModeGuard` (fuzz/src/panchanga.rs:225). |

## Configuration Discoverability

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — render config precedence, unsafe config `out` rejection, `--var` override handling, and config error paths are tested.)

## Data Structure

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Decoupling

| id | status | effort | description | notes |
|---|---|---|---|---|
| DEC-1 | DECIDED | S | `cli/src/cmd/render/svg_common.rs::write_year_axis` calls `celestial_core::revjul` / `julday` directly from a shared render helper. | Acceptable thin veneer: axis ticks need JD/Gregorian conversion; closure injection would be heavier than the coupling. |
| DEC-2 | OPEN | M | `secondary_progressions` computes progressed house cusps then discards them (`_houses`) in JS (`bindings/js/src/lib.rs:2416`) and PHP (`bindings/php/src/lib.rs:1767`), while Python (`bindings/python/src/lib.rs:1927`) returns `(positions, cusps)` — progressed houses are unreachable in JS/PHP. | Return the cusps alongside positions in JS+PHP to match Python, or document the intentional omission. |

## Dependency

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — dependency set is small and feature checks pass for default, minimal, timezone-only, and calendar-only core builds.)

## Design Thinking

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Documentation

| id | status | effort | description | notes |
|---|---|---|---|---|
| DOC-4 | DECIDED | M | `celestial-cli` public items are intentionally undocumented. | Binary crate/lib split only exists so `main.rs` and tests can share modules; `#![warn(missing_docs)]` remains core-only. |
| DOC-12 | OPEN | S | `docs/index.md:71` documents the plugin naming as `celestial-<n>` (literal), but `plugin.rs:3` uses `celestial-<name>` — the `<n>` placeholder is wrong. | Change `celestial-<n>` to `celestial-<name>`. |
| DOC-13 | OPEN | S | README.md:193 claims "All subcommands accept `--json`", but `celestial render --json` errors with `unexpected argument '--json'` (verified; the other 12 subcommands do accept it). | Scope the claim to the computation subcommands, or note `render` uses `--print-context` for JSON. |

## Legacy / Deprecation

| id | status | effort | description | notes |
|---|---|---|---|---|
| LEG-1 | OPEN | S | `get_ayanamsa_name` is a redundant alias of `ayanamsa_name` exported by all three bindings (`bindings/python/src/lib.rs:1541`, `bindings/js/src/lib.rs:2564`, `bindings/php/src/lib.rs:2433` — the PHP one is literally commented "Legacy alias"). Both wrap the same `celestial::ayanamsa_name`. | Counts twice against the 194-function parity surface. If external consumers depend on it, mark `#[deprecated]` and schedule removal; otherwise drop the alias and regenerate stubs. Kept intentionally today, but no deprecation path is recorded. |
| LEG-2 | OPEN | S | Two more byte-identical legacy aliases across all three bindings, same class as LEG-1: `get_ayanamsa` = `ayanamsa` and `house_name_str` = `house_name` (`bindings/python/src/lib.rs:1580`, `bindings/js/src/lib.rs:796,2682`, `bindings/php/src/lib.rs:2156,2617`). Each inflates the parity surface. | Fold into LEG-1's deprecation decision: `#[deprecated]` + scheduled removal, or drop and regenerate stubs. |

## Multithreading

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — `calc_many` / `calc_ut_many` scoped-thread paths are covered and preserve input order.)

## Observability

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — CLI errors include actionable parse/config/plugin context; core remains library-quiet by design.)

## OKR

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## PDCA

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Performance

| id | status | effort | description | notes |
|---|---|---|---|---|
| PERF-8 | DECIDED | S | `svg_common::svg_doc_open` chains several small string allocations per chart. | One call per chart; combining needs templating/build-time concat and is net-neutral. |
| PERF-9 | DECIDED | S | `parse_chart_type` calls `registered_chart_types()` twice on invalid input. | One-shot CLI parse path; cosmetic micro-perf. |
| PERF-2/3 | DECIDED | — | Search code re-evaluates final `calc_ut` / `houses` with full flags after bisection. | Authoritative final result, not redundant; locked by regression tests. |
| PERF-5 | DECIDED | M | `next_aspect_with2` dual-scan merge could be unified. | Precision-sensitive rewrite for marginal gain; byte-identical gate made it a deliberate no-op. |
| PERF-11 | OPEN | S | `core/src/astronomy/vsop87.rs:74` `eval_vsop_with_deriv` calls `eval_series` then `eval_series_deriv` over the same term slice, building each term's phase `c·τ+b` twice and running `.cos()`/`.sin()` as two separate transcendentals instead of one `sin_cos()` — ~2× transcendental cost on the helio+speed path (moon.rs:86 already pairs them). | Fuse into one term loop computing `arg.sin_cos()` once per term, accumulating value + derivative together. |
| PERF-12 | OPEN | S | `core/src/functions/searches.rs:412` `next_aspect_cusp`'s scan recomputes all 12 Placidus cusps at every 0.05-day step across an up-to-400-day window but reads only `hr.cusps[cusp]`, discarding ~11/12 of the iterative semi-arc work each step. | Compute only the requested cusp, or hoist ARMC/obliquity-invariant sub-results out of the per-step loop. |

## Platform

| id | status | effort | description | notes |
|---|---|---|---|---|

| PLAT-1 | OPEN | M | Windows plugin dispatch is broken: `is_exec` (`cli/src/plugin.rs:70`) only checks `is_file()`, `discover` keeps the extension in the name (`celestial-synastry.exe` → `"synastry.exe"`), but `try_exec` (:49) searches for the extensionless `celestial-<sub>`, so a real `.exe` plugin is never found and non-executable `celestial-*.txt` files are listed as plugins. Windows is in the CI matrix (`celestial-cli.yml:95`). | Strip a trailing `.exe`/PATHEXT from the discovered name and probe candidate names with executable extensions on non-Unix. |

## Plugin Extensibility

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — plugin discovery ignores non-executables, deduplicates names, sorts output, and returns actionable unknown-command errors.)

## Product Engineering

| id | status | effort | description | notes |
|---|---|---|---|---|
| PROD-1 | OPEN | M | 23 of 36 `celestial render` examples in README use bare `--date YYYY-MM-DD` and/or omit `--timezone` (README.md:446,449-466,477-491,506-521,534-541,571,591,618,345,393; docs/index.md:68; docs/building.md:246-252), so every one fails on the shipped binary with "has no time-of-day" / "missing --timezone" — verified by executing cosmogram, solar-return, bazi, mesoamerican, dial, natal+overlays, template-run, and the `--time` form. The examples contradict the tz-requirement section README itself introduces at :226-262. | Either update every example to carry time+`--tz`, or relax `require_datetime` (pipeline.rs:309) for chart types that don't need a birth instant (mesoamerican, ephemeris, calendar…). Also fix docs/building.md:252's now-false `shows "14:30 UT"` claim. |

## Purpose

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Reliability / Correctness

| id | status | effort | description | notes |
|---|---|---|---|---|
| REL-10 | OPEN | S | `core/src/functions/moon_phases.rs:287` accepts `jd >= jd_from - 0.01`, so `next_principal_phase` / `next_new_moon` can return a phase up to 0.01 days before the requested start despite docs/tests saying at or strictly after `jd_from`. | Boundary issue only; broad moon/reference/property tests pass. `fuzz/src/main.rs:4233` also documents widened synodic tolerance and references a missing `next_new_moon` TODO, so either tighten the predicate and add a boundary regression or explicitly document an inclusive tolerance contract. |
| REL-13 | OPEN | M | `core/src/astronomy/houses.rs:453 diurnal_semi_arc` is a degenerate stub: `to_deg((lat_r.tan() * 0.0_f64.tan()).asin()) + 90.0` always returns `90.0` (since `tan(0) = 0`), and its `_eps_r` argument is unused — so Koch cusps (houses.rs:396) always divide a fixed 90° semi-arc instead of the latitude/obliquity-dependent value. | Verified degenerate by inspection. Either implement the real diurnal semi-arc (`asin(tan φ · tan δ) + 90`) or document Koch as an approximation. No Koch reference golden currently pins this, so add one with the fix. |
| REL-14 | OPEN | M | JS `housesEx` (`bindings/js/src/lib.rs:427`) returns the full 13-element `r.cusps` (incl. the unused SwissEph index-0 placeholder), whereas JS `houses` (:403) and Python/PHP `houses_ex` return `cusps[1..]` (12 real cusps) — so `cusps[i]` is off by one house for `housesEx` callers. Verified. | Slice `r.cusps[1..].to_vec()` in JS `housesEx` to match the siblings. Shapes gate can't see it (coarse kind only). |
| REL-15 | OPEN | M | `houses_ex2` cusp count disagrees across bindings: JS (`bindings/js/src/lib.rs:450`) and PHP (`bindings/php/src/lib.rs:2171`) return 13 cusps incl. index-0, but Python (`bindings/python/src/lib.rs:347`) returns 12 (`cusps[1..]`) — `cusps[i]` maps to a different house per language. | Strip index 0 in JS+PHP (or add it in Python) so all three agree with their own `houses`. |
| REL-19 | OPEN | S | `sol_eclipse_how` (`core/src/functions/eclipses.rs:105`) sets `ret_flags` from `attr[1] > 0.0`, but `attr[1]` is the constant solar diameter 0.5266 — so it still returns `ECL_TOTAL` for arbitrary input dates. | Key classification off `check_solar_eclipse`'s kind. The magnitude formula is already corrected; this remaining row is only the `ret_flags` classifier bug. |
| REL-20 | OPEN | S | `hebrew_year_from_jd` (`core/src/functions/jewish.rs:336-339`) starts at the mean-year estimate and only searches FORWARD, but the estimate can overshoot by 1 — wrong Hebrew year (and wrong `jd_to_hebrew_date`) for dates just before Rosh Hashanah. Verified 13 failures in 5600–5800 (e.g. 1853-10-01 → 5614, should be 5613). | Start the search at estimate−1 or add a backward-correction loop like `omer_from_jd` has. |
| REL-21 | OPEN | S | `easter_orthodox` (`core/src/functions/easter.rs:64-72`) hardcodes a 13-day Julian→Gregorian offset outside 1700–2099, but 1583–1699 is 10 days (3 days wrong) and ≥2100 is 14 (1 day wrong). Arithmetic-verified: Orthodox Easter 1600 = Gregorian Apr 2, code returns Apr 5. | Compute the offset from the century formula instead of the 3-branch table. |
| REL-23 | OPEN | S | `engine.rs:73`: when `FLG_SIDEREAL` and `FLG_TOPOCTR` are combined, the topocentric branch unconditionally overwrites `lon` from tropical `geo`, discarding the ayanamsa subtraction applied at :69 — sidereal+topocentric returns tropical topocentric longitudes. | Apply topo first, then subtract `sidereal_ayanamsa(jde)` when the sidereal bit is set. |
| REL-24 | OPEN | M | `calc_pctr` (`core/src/functions/calc.rs:41`) takes `jd_et` (TT) but routes through the UT-aliased path, applying ΔT twice (~69 s epoch shift at J2000); and its `dist` is `\|body.dist − center.dist\|` of geocentric distances — not the body–center distance (zero for equidistant bodies 90° apart). | Call `calc_tt`; derive planetocentric coords by Cartesian vector subtraction, not element-wise differences. |
| REL-25 | OPEN | S | Derived-Debug leak: `format!("{:?}", …)` on tuple-struct `Body` renders literal `Body(6)` etc. as user-visible labels in dasha bars (`cli/src/cmd/render/vedic.rs:739`), firdaria major/minor lords (`hellenistic.rs:231-232`), hellenistic dignity Term/Decan/Triplicity/Almuten columns (:71-76), profection legend "Lord: Body(3)" (:449), and dial `body1`/`body2` (`specialist.rs:71-77`). Verified in rendered SVG output. | Map `Body` through the `BODIES` name table (or a `body_name` helper) at all six sites; re-pin affected goldens. |
| REL-26 | OPEN | S | `DASHA_COLORS` (`vedic.rs:848`) and `FIRD_COLORS` (`hellenistic.rs:272`) are keyed `"SUN"`/`"RAHU"`/`"KETU"` which can never match the `"Body(N)"` strings compared at vedic.rs:919 / hellenistic.rs:379 — every firdaria bar renders fallback `#888` and every dasha bar the uniform planet color; the per-planet palette is fully dead. Verified in output. | Key both tables by the canonical name written into the context (Rahu/Ketu need explicit mapping); fixes together with REL-25. |
| REL-27 | OPEN | S | 90° midpoint dial renders wrong: `render_dial_svg` (`specialist.rs:859`) maps `dial_lon` (0–90°) 1:1 onto circle angle, so all planets crowd into one quadrant (verified: all glyph coords in one quarter) and the degree ticks (:843-853) mark only that quarter arc — a 90° dial must spread 0–90° over the full 360°. | Multiply `dial_lon` (and tick degrees) by 4 before the angle transform; re-pin the dial golden. |
| REL-28 | OPEN | S | xtask `scan_fn_name` (`xtask/src/main.rs:83`) uses `(start..n.min(start+5)).next()` which yields only `start` — inspects exactly 1 line despite the "at most 5 lines" contract, so `legacy_aliases()` silently misses any alias wrapper with a doc/cfg line between the attribute and `fn`. Latent today (verified: every current `#[php_function]` is immediately followed by fn or `#[allow]`), but one inserted doc line silently un-excludes an alias from parity. | Iterate the range (`for j in start..n.min(start+5)`). |

## Robustness / Recovery

| id | status | effort | description | notes |
|---|---|---|---|---|
| ROB-1 | OPEN | M | `core/src/functions/panchanga.rs:235,244` call `calc_ut(...).unwrap_or(<zeroed Planet>)` for Sun and Moon, so an ephemeris failure silently substitutes a 0°/0-speed position and the function returns a plausible-but-wrong panchanga (tithi/nakshatra) instead of surfacing the error. | Verified pattern. Consider propagating the `calc_ut` error (return `Result`) or at least a sentinel the caller can detect; today failure is indistinguishable from a real new-moon result. |
| ROB-2 | OPEN | M | `render --date2/--date3` for biwheel/composite/triwheel/lunar-return (`cli/src/cmd/render/registry.rs:85,149,171,195`) are parsed with `parse_date` but never timezone-adjusted and skip `require_datetime`, so the 2nd/3rd subject is computed in raw UT while `--date` is converted local→UT at `pipeline.rs:321` (verified: outer-planet longitudes unchanged between `--tz UTC` and `--tz +12`). | Apply the same `--timezone` offset (or add `--timezone2`) and the `require_datetime` check to date2/date3 before building those charts. |
| ROB-3 | OPEN | S | `rise_set_inner` (`core/src/astronomy/rise_set.rs:126-139`) returns not-found when `cos_h0` is out of ±1 BEFORE checking `event`, so a Transit request for a circumpolar body fails although it transits daily. Verified: `rise_trans(Sun, MTRANSIT)` at 80°N midsummer returns `Err(CircumpolarBody)`. | Skip the cos_h0 rise/set feasibility gate when `event == Transit`. |

## Scalability

| id | status | effort | description | notes |
|---|---|---|---|---|

| SCALE-1 | OPEN | S | `parallel_calc` (`core/src/functions/calc.rs:294`) spawns one OS thread per body with no cap, despite its own doc claiming "up to a cap" (calc.rs:238); bindings feed it an unbounded user `Vec` (python:180, js:313, php:280) — a 100k-element array attempts 100k thread spawns and a spawn failure panics the scope. | Chunk bodies over `available_parallelism()` (or clamp thread count); fix the doc. |

## Security

| id | status | effort | description | notes |
|---|---|---|---|---|

| SEC-6 | OPEN | M | An untrusted `--config` can set `[render] template` to any absolute path (`cli/src/cmd/render/config.rs:90` assigns `r.template` with no guard), and MiniJinja emits non-`{{}}` file content verbatim into the "SVG" — an arbitrary-file-read/disclosure. The sibling `out` field right below (:101-112) IS path-guarded under the same explicitly-stated "config may be untrusted" SEC-5 threat model, so `template` is an inconsistency in that model. | Apply the same absolute/`..` rejection to config-sourced `template`; leave the explicit `--template` CLI flag unrestricted (mirror the `out` split). |

## State Machine Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|

| SM-1 | OPEN | M | `crossing` for any body other than sun/moon without `--helio` falls into `_ => helio_cross_ut(...)` (`cli/src/cmd/crossing.rs:51`), so `crossing --body mars --lon 120` silently returns the *heliocentric* crossing (verified: identical JD to `--helio`) but labels it a plain geocentric "Mars crossing". | Reject non-sun/moon geocentric requests with an explicit error, or force/announce `(heliocentric)` in the label. |
| SM-2 | OPEN | S | `raw.iter().any(|a| a == "--list-plugins")` (`cli/src/main.rs:93`) scans the whole arg vector and returns before clap dispatch, so `celestial calc --date … --list-plugins` (verified) silently discards the `calc` computation and just lists plugins. | Only honor `--list-plugins` when no subcommand is present, or let clap own the global flag and handle it after arg-match. |

## UI / UX

| id | status | effort | description | notes |
|---|---|---|---|---|

| UX-1 | OPEN | M | `--chart-type calendar` with no `--calendar` flags renders a bare grid whose subtitle still advertises "overlays: gregorian · omer · sabbats · moon" (`cli/src/cmd/render/pipeline.rs:344`), but 0 day annotations appear (verified vs 2 with `--calendar moon`): `build_calendar_context` defaults `calendars` to all four (pipeline.rs:55-72) while `apply_universal_overlays` keys off the empty `args.calendars`. | Drive `apply_universal_overlays` from the context's `calendars` for the calendar chart-type (or default `overlay_cals` to all-four) so advertised overlays are populated. |
| UX-2 | OPEN | S | Stray semicolon typo renders literally in the profection SVG legend: output reads `Age 40: House 5 ;profection — Lord: …` (`cli/src/cmd/render/hellenistic.rs:484`; sibling stray `;` in the comment at :473). Verified in output. | Delete the stray `;` from both format strings. |

## Vectorization

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — current math is scalar and bounded; no obvious SIMD/vectorization win surfaced.)

## Wiring Gaps

| id | status | effort | description | notes |
|---|---|---|---|---|
| WIRE-1 | DECIDED | — | Public Rust API functions such as `ayanamsa_ex`, `current_file_data`, `gauquelin_sector`, `get_orbital_elements`, `heliacal_pheno_ut`, `nod_aps_ut`, `set_tid_acc`, `tid_acc`, and `vis_limit_mag` have no CLI call site. | KEEP — `celestial-core` is a published library crate; these functions are intentionally root-re-exported and parity smoke-tested for external consumers. |
| WIRE-2 | DECIDED | S | `Body::is_known_id` is only used internally by `try_from_raw` plus tests. | KEEP — useful published predicate for callers that want a cheap bool pre-check. |
| WIRE-3 | DECIDED | S | `BodyError::OutOfRange { id }` is constructed but bindings stringify it instead of pattern matching. | KEEP — typed Rust variant preserves future extensibility while bindings expose language-native string errors. |
| WIRE-4 | OPEN | S | `monthly_profection(&cusps_arr, age, 0)` (`cli/src/cmd/render/hellenistic.rs:443`) hardcodes month 0, so the exported `month_house`/`month_lon` context fields always duplicate the annual profection values — no CLI knob ever reaches them. | Feed a real month (from the render date) or delete the two always-redundant fields. |

## Unused Functions / Methods

| id | status | effort | description | notes |
|---|---|---|---|---|
| UNUSED-1 | DECIDED | — | No new unused public-shaped production callables found in this rescan. | `cargo xtask parity`, binding stub checks, workspace tests, and prior public API review all remain green; existing library-only APIs are accounted for under WIRE-1. |
