# Celestial — TODO

Rescan: **2026-08-08** (targeted whole-project scan of production Rust for bit/float/integer, arithmetic, and branch optimizations; skipped `target`, `node_modules`, build caches, `tests/`, `benches/`, and `fuzz/`). Added: PERF-13..15, PERF-17..21, DUP-12, REL-44, REL-45, DS-1, VEC-1. PERF-16 was raised and then **rejected on measurement** — see "Audit picks deliberately rejected". Every row was read at the cited line before entry. Speed claims remain static reads (no benchmark was run); `core/benches/celestial_bench.rs` covers `math::*`, `calc::*`, `houses::*` and is the intended before/after gate.
2026-08-08 precision review: every row above was re-checked against the project's numeric gates with standalone probes rather than accepted as written. Classified **bit-identical** (no gate exposure): PERF-17 hoist + no-op-`mul_add` removal, PERF-18, PERF-19, PERF-20, PERF-21, DS-1. Classified **inside tolerance, measured**: PERF-13 (0.0 delta at the pinned `moon.rs` epochs; 4.0e-15° on `mean_obliquity` vs a 1e-12° gate), PERF-15 (9.3e-16° worst vs a 1e-12° gate over a 200k-sample sweep). Classified **breaks a bit-exact fingerprint, needs re-pinning in the same commit**: PERF-13 at `nodes.rs:436`, PERF-14 at `astronomy/phenomena.rs:180`. Classified **must not use the FMA form**: DUP-12 — `nutation::poly` with `mul_add` moves `dpsi` 5.5e-12″ at ±1 cy against a 1e-11″ gate whose furthest pinned epoch is t≈1.33 cy. REL-45 (`norm_deg` can return exactly `360.0`) was discovered by these probes, not by the original scan. Relevant std facts confirmed from source: `f64::sin_cos` is `(self.sin(), self.cos())`, and `f64::rem_euclid` is one `%` plus a branch — not two `fmod` calls, correcting the original PERF-16 cost estimate.
Checked-clean in this pass: `vsop87.rs` (already Horner + `mul_add` + `sin_cos`), `delta_t.rs` (Horner with FMA), `crossings.rs` refinement (Brent–Dekker, not bisection), `searches.rs` sign-change predicate, no production `unsafe`, no integer-overflow-prone index math in the calendar modules.
Prior rescan: **2026-07-23** (whole-project tracked-source scan: 258 files / 107,775 lines; skipped `target`, `node_modules`, cache directories, and ignored compiled/build outputs; source-controlled generated API docs and stubs were checked as contracts). Added confirmed findings: CLI-4..7, LINT-1, CONF-1/2, DOC-13, LEG-3, PLUG-1, PROD-2, REL-16, ROB-3, SEC-7. Static passes also found no new production `unsafe`, panic/todo stubs, broken relative Markdown links, tracked build artifacts, or unaccounted public-shaped dead callables.
2026-07-23 verification: `cargo test --workspace --exclude celestial-fuzz`; `cargo fmt --all -- --check`; minimal/default core feature checks; `cargo xtask parity` (270 each), `coverage` (309 core / 270 bound / 39 allow-listed), `shapes --check`, `apidoc --check`, `pyi --check`, `dts --check`; fuzz harness (88 suites); JS pure logic (162), typecheck, and ESLint; PHP `cargo check`, format, clippy, and stub syntax; Python native-import smoke; tracked JSON/TOML/YAML/shell syntax. Full JS native test is red as PROD-2; Python `pytest` is unavailable in this environment. Clippy's remaining warnings are recorded in CC-1 and LINT-1.
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
| BIND-3 | DECIDED | — | `utc_to_jd` takes 7 date scalars in Python/PHP but a single `UtcDate` object in JS (`bindings/js/src/lib.rs:605`). | KEEP — idiomatic JS object-packing, not drift. Recorded in `xtask/arity_allow.txt`. |
| BIND-9 | OPEN | L | PHP stubs and `docs/php.md` advertise 273 `celestial_*` aliases, but the built extension exports only the 273 unprefixed functions; the newly unskipped native suite failed immediately on nonexistent `celestial_version()`. | Choose one public naming contract: generate and runtime-test real aliases, or remove the fictional aliases from stub generation/docs and publish the unprefixed API as canonical. |

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

(no findings)

## CLI / Option Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CLI-4 | OPEN | S | `DateJd::from_str` accepts bare `NaN`, `inf`, and `-inf` as Julian days. `render --date NaN --print-context` exits 0 with a null-filled chart; `--date inf` panics on integer overflow in `next_principal_phase`. | Reject non-finite and out-of-domain JDs at the shared parse boundary. `cli_fuzz::boundary_non_finite_and_extreme_floats` currently calls `parse_date` but never asserts rejection or exercises a downstream command. |
| CLI-5 | OPEN | S | Civil dates/times are normalized instead of validated: bad time components use `unwrap_or(0.0)`, month/day/time ranges are unchecked, and `2024-01-01 nope:nope` silently becomes midnight. A test explicitly accepts `1986-13-99 09:00`. | Parse the documented formats strictly, validate Gregorian fields and clock ranges, and add rejection tests for malformed and normalized-away input. |
| CLI-6 | OPEN | M | Numeric validation is command-specific and incomplete. `houses --lat NaN --json` succeeds and emits invalid JSON containing bare `NaN`; render accepts non-finite `--years` and produces null-heavy progressed charts. | Reuse finite/range validators for every geographic and calculation scalar at the CLI boundary, including `houses` lat/lon and render progression/secondary-coordinate inputs. |
| CLI-7 | OPEN | S | `houses --json` serializes `cusps` as a quoted string (`"cusps":"[...]"`) rather than a JSON array. | Replace the string-guessing `fmt::json_obj` path with a typed `serde_json` value/struct and assert field types after parsing command output. |

## Code Complexity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CC-1 | DECIDED | — | `cli/tests/cli_fuzz.rs:225 boundary_empty_null_and_oversize_strings` CC 16/10 and `cli/src/parse.rs:423 parse_tz_forms` CC 14/10. | Test-only assertion/matcher density; real branching is low. `cargo clippy --workspace --all-targets -- -W clippy::cognitive_complexity` reports only these two. |
| CC-6 | DECIDED | S | `cli/src/cmd/render/calendar_overlays.rs:509 render_day_cell` sits at exactly CC 10. | Compliant but no headroom. If extended, split moon-glyph and omer-badge rendering into helpers. |
| CC-7 | DECIDED | S | `core/src/body/mod.rs:119 Body::is_known_id` is a flat range ladder. | Still compliant; flat documented id windows are clearer than hiding the ranges in a table. |
| LINT-1 | OPEN | S | Warning-clean clippy is not maintained across all tracked Rust: `core/src/astronomy/houses.rs:923` triggers `byte_char_slices`, and standalone PHP `match_aspect3` / `match_aspect4` trigger `too_many_arguments`. | Apply the byte-slice suggestion; for the published flat FFI signatures, add the same targeted lint rationale already used by core/Python or refactor all bindings together. |

## Code Duplication

| id | status | effort | description | notes |
|---|---|---|---|---|
| DUP-1 | DECIDED | L | Binding return-adapter stubs remain repeated across JS/Python/PHP. | Per-language return shapes are intentionally different. `cargo xtask parity` confirms all 270 function exports stay aligned. |
| DUP-2 | DECIDED | S | `body_of(n)` validation helpers are present in each binding. | Per-language error mapping is the irreducible part; sharing would obscure simple error flow. |
| DUP-3 | DECIDED | — | Deterministic `core/tests/rel_clamps.rs` and randomized `fuzz/src/main.rs::test_rel_clamps` overlap on body-id edge coverage. | Intentional two-tier coverage split: `cargo test` regression plus opt-in fuzz/property run. |
| DUP-4 | DECIDED | — | `revjul` / `revjul_hms` have three language-specific return shapes. | Published API idioms differ; normalizing would break bindings. |
| DUP-5 | DECIDED | — | Per-tradition SVG wheel geometry uses similar-looking constants. | Distinct layout contracts; shared pieces are already factored. |
| DUP-12 | OPEN | S | Horner evaluation is implemented twice with different numerics: `nutation.rs:172 poly` folds with `acc * x + c` while `delta_t.rs:241 polynomial` folds with `acc.mul_add(x, c)`. Neither is visible to the other modules that need it. | **Consolidate on the plain `acc * x + c` form, not the FMA one.** Measured: switching `nutation::poly` to `mul_add` moves `dpsi` by 5.5e-12″ at ±1 century and 1.8e-10″ at ±17 centuries, against a 1.0e-11″ gate at `nutation.rs:385` whose furthest pinned epoch is t≈1.33 cy — i.e. right on the limit with only the 8 leading rows of 77 modelled, so the full series very likely breaks it. `delta_t` moving to the plain form is safe (its gates are 1e-6 s). Put the single `pub(crate)` helper in `astronomy/constants.rs`. Prerequisite for PERF-13/PERF-14. |

## Composition

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — render composition now flows through `ChartContext`, `CHART_REGISTRY`, and specialist dispatch helpers.)

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

(no findings)

## Documentation

| id | status | effort | description | notes |
|---|---|---|---|---|
| DOC-4 | DECIDED | M | `celestial-cli` public items are intentionally undocumented. | Binary crate/lib split only exists so `main.rs` and tests can share modules; `#![warn(missing_docs)]` remains core-only. |
| DOC-13 | OPEN | S | Current docs/CI comments retain obsolete inventory: README says 1496 tests / 84 fuzz suites; docs say 1174 core tests and 194 binding functions; the core workflow header promises ≥80% line / ≥90% function coverage while commands enforce 75% / 78%. Current checks report 1555 workspace tests, 88 fuzz suites, and 270 parity exports. | Update current-facing counts or make them generated/link to authoritative outputs; align the workflow header with its actual thresholds. Historical rescan records above should remain unchanged. |
| DOC-14 | OPEN | M | `docs/javascript.md` uses snake_case for 22 unique native calls (`calc_ut`, `set_sid_mode`, `moon_phase`, etc.), while the generated TypeScript API exports camelCase (`calcUt`, `setSidMode`, `moonPhase`, etc.); the guide's examples therefore fail at runtime. | Rewrite calls from `bindings/js/index.d.ts` and add an executable documentation smoke check so examples cannot drift from napi export names. |

## Legacy / Deprecation

| id | status | effort | description | notes |
|---|---|---|---|---|
| LEG-3 | OPEN | M | Public Swiss-compatible setup APIs advertise state changes but silently no-op: `set_ephe_path` / `set_jpl_file` always return `Ok(())`, `set_tid_acc` / `set_lapse_rate` discard input, and `tid_acc` always returns 0. They are exported through the bindings as working setters. | Either implement observable supported behavior or explicitly mark/deprecate compatibility no-ops and return an unsupported error where signatures permit. Keep `FLG_JPL`'s documented built-in fallback distinct from accepting a file that is never used. |

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
| SONAR-JS-3 | DECIDED | — | Sonar `javascript:S7721` asks to hoist two helpers out of test-suite callbacks. | KEEP — each callback executes once; moving local fixture helpers outward adds scope without avoiding repeated allocation. |
| PERF-13 | OPEN | S | Polynomials are written as expanded powers instead of Horner form: `nutation.rs:40` expands `t`…`t⁵` (15 multiplies for a 6-term series), `moon.rs:29/34/39/42/51/59` expand `t²`–`t⁴` for six fundamental arguments, plus `nodes.rs:77`, `rise_set.rs:223`, `planetary.rs:220/234`, `ayanamsa.rs:213`. | **Precision measured, stands.** At the three pinned `moon.rs` epochs the Horner and Horner+FMA results for `L'` are **bit-identical** to the expanded form (delta exactly 0); `mean_obliquity` shifts 4.0e-15° against a 1.0e-12° gate (`nutation.rs:383`). Horner is also the numerically *better* form — the expanded version is the one accumulating error. **Blocker:** `nodes.rs:436` is a bit-exact `assert_eq!(f64_fingerprint(…))`, so touching `nodes.rs:77` forces re-pinning that hash in the same commit. |
| PERF-14 | OPEN | S | `functions/phenomena.rs:217` (`yallop_q`) and `astronomy/phenomena.rs:105–115` (`visual_magnitude`, Moon/Mercury/Venus arms) evaluate quartics via `powi(2)`/`powi(3)`/`powi(4)` plus separate `i * i`. | Stands on arithmetic grounds — up to 9 multiplies per arm versus 4 FMAs. **Blocker:** `astronomy/phenomena.rs:180` is a bit-exact fingerprint gate over `compute_phenomena`, so this one cannot land without re-pinning the hash. Weigh that against the size of the win before starting; the visibility magnitudes are reported to ~0.01 mag. |
| PERF-15 | OPEN | S | Term-argument accumulation in the two hottest series loops uses plain mul/add: `moon.rs:77` and `moon.rs:97` build `dc*d_r + mc*ms_r + mpc*mp_r + fc*f_r` (120 rows per call), `nutation.rs:126` builds a 5-term dot product over 77 rows. | **Precision measured, stands.** Randomized sweep (200k argument sets over representative ELP rows) puts the FMA-vs-plain divergence at **9.3e-16° worst case** in lunar longitude against the 1.0e-12° gate at `moon.rs:326` — a 1000× margin. `vsop87.rs:28` already uses `mul_add` for exactly this; the lunar and nutation series were never converted. |
| PERF-17 | OPEN | S | `fixstars.rs:1150` recomputes `23.4393_f64.to_radians().sin_cos()` on every `star_ecliptic_pos` call although it is a constant (the 97-star catalog scan at `fixstars.rs:1117` calls it per star). Same function has `y_lon = (…).mul_add(1.0, …)` (line 1162) — an FMA whose multiplier is 1.0 — and `1.0 / (star.plx / 1000.0) * 206_265.0` (line 1172), two divisions where one suffices. | Hoisting is **bit-identical** (`f64::sin_cos` is literally `(self.sin(), self.cos())` in std, so no algorithm swap is involved). Dropping the no-op `mul_add` is **bit-identical** — verified over 200k samples, `a.mul_add(1.0, b) == a + b` always, since `a * 1.0` is exact. The parallax fold to `206_265_000.0 / plx` is **not** bit-identical (~1 ulp, differs in 3 of 4 sampled parallaxes) but is the *more* accurate single-rounding form; no bit-exact gate covers it — the `fixstars` fingerprint hashes the static `CATALOG`, not computed positions. Split it out if a bit-identical diff is wanted. |
| PERF-18 | OPEN | S | `planetary.rs:58` light-time iteration recomputes each coordinate difference twice: `((xt - xe) * (xt - xe) + (yt - ye) * (yt - ye) + (zt - ze) * (zt - ze)).sqrt()`. | **Bit-identical** — binding `dx`/`dy`/`dz` evaluates the same subtractions to the same values. Zero precision risk. The surrounding code at lines 46–49 and 64–69 already does it this way. Runs twice per `apparent_planet`. |
| PERF-19 | OPEN | S | Both lunar series loops select the eccentricity factor per term with `match mc.abs() as i32 { 1 => e, 2 => e2, _ => 1.0 }` (`moon.rs:79`, `moon.rs:99`) — a float `abs`, a float→int cast, and a compare chain for each of the 120 terms per call. | **Bit-identical** — same three `f64` values, only the selection mechanism changes. `mc` is always in `-2..=2`, so `[1.0, e, e2][mc.abs() as usize]` is a branchless load; better still, fold the E-power index into the static table as an extra column so the cast disappears. Pairs with DS-1. |
| PERF-20 | OPEN | S | Loop-invariant trigonometry recomputed inside cusp loops in `astronomy/houses.rs`: `eps_r.cos()` inside `meridian` (line 650), `morinus` (line 667), `campanus` (line 602) and both Koch loops (lines 462, 477); the Koch loops also call `armc_r.sin()`/`armc_r.cos()` two to three times per iteration even though `sin_armc` is already bound at line 452. | **Bit-identical** — hoisting a deterministic libm call out of a loop cannot change its result, and `f64::sin_cos` returns exactly `(self.sin(), self.cos())`. The `houses` gate at line 1184 is a 1e-10 tolerance anyway. Covered by the `houses::houses_ex_*` benches. |
| PERF-21 | OPEN | M | `cli/src/cmd/render/geometry.rs:11–18` splits wheel projection into `wx`/`wy`, so each of the 68 paired call sites evaluates `wheel_angle` (a `rem_euclid`) twice, `to_radians` twice, and an independent `sin` and `cos` on the identical angle. `builtin_svg.rs:520–521` and `:525–526` repeat the pattern inline. | **Bit-identical**, therefore safe against the byte-exact SVG goldens: `sin_cos` is defined as `(self.sin(), self.cos())`, and the hoisted `wheel_angle` is one evaluation of an expression that was previously computed twice to the same value. Add `wxy(cx, cy, r, lon, asc) -> (f64, f64)` and convert the paired sites; keep `wx`/`wy` for the few unpaired uses. |

## Platform

| id | status | effort | description | notes |
|---|---|---|---|---|


## Plugin Extensibility

| id | status | effort | description | notes |
|---|---|---|---|---|
| PLUG-1 | OPEN | S | `discover()` inserts a plugin name into `seen` before checking executability. A non-executable `celestial-foo` earlier on `PATH` suppresses a later executable one from `--list-plugins`, while `try_exec` independently scans onward and can dispatch it. | Mark a name seen only after accepting an executable candidate; add a two-directory PATH test that keeps discovery and dispatch consistent. |

## Product Engineering

| id | status | effort | description | notes |
|---|---|---|---|---|

## Purpose

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Reliability / Correctness

| id | status | effort | description | notes |
|---|---|---|---|---|
| MUT-1 | OPEN | L | Full `cargo-mutants` baseline finds surviving core mutants, beginning with unasserted rise/set dispatch arms, body mappings, and derived constant values. | User stopped the fresh sweep at 1,979/6,835 mutants: 9 missed and 3 timed out are tracked below; 4,856 remain untested. Resume from the preserved `/tmp/celestial-mutation-shard-*` checkpoints when requested. |
| MUT-5 | OPEN | S | `crossings::CROSSING_TOL_X` survives changing the day/second conversion from division to remainder, weakening crossing-time convergence by five orders of magnitude. | Add an exact public crossing precision regression, then rerun the mutant with the full package suite. |
| MUT-6 | OPEN | S | Three `crossings::bracket_crossing` predicate mutants (`*` to `+`, `<=` to `>`, and `&&` to `||`) make the full package test run time out instead of failing fast. | Pin sign-change/antipodal rejection boundaries directly and ensure false brackets cannot drive downstream refinement into long-running searches. |
| MUT-7 | OPEN | S | `planetary::apparent_planet` has eight surviving mutations across Earth-coordinate subtraction and the initial geocentric distance sum-of-squares arithmetic. | Add exact multi-epoch public geocentric position fingerprints that exercise every rectangular axis and the light-time corrected distance calculation. |
| REL-43 | OPEN — parked | L | `losar_jd` assumes Losar is always the second new moon after the winter solstice; it returns 2025-01-29, while the official 2025 Tibetan Losar date is 2025-02-28 because the shortcut omits Phugpa leap-month rules. | Needs the full Phugpa true-month/intercalation, true-date correction, and skipped/repeated-day algorithm; do not substitute a one-year table or heuristic. Validate Janson's 2019–2027 vectors plus official dates when implemented. |
| SONAR-BIND-1 | DECIDED | — | Sonar `typescript:S7758` prefers `codePointAt` at four FFI house-system call sites. | KEEP — `charCodeAt(0)` intentionally supplies a required one-byte ASCII integer; `codePointAt` widens the type with `undefined` without supporting a valid extra input. |
| SONAR-PY-3 | DECIDED | — | Sonar `python:S1244` flags two exact float equalities in binding tests. | KEEP — half-day Julian values are exactly representable and coordinate transformation must preserve the distance component bit-for-bit. |
| REL-16 | OPEN | L | Public TT/ET and UT variants have contradictory time-scale behavior. `fixstar_ut`, `fixstar2_ut`, `nod_aps_ut`, `ayanamsa_ut`, and `ayanamsa_ex_ut` pass UT straight into TT math; `solcross` / `mooncross` are documented as ET but search with `calc_ut`; `helio_cross_ut` is an exact alias of the ET function. Tests often require equality for the same numeric JD, locking in the mismatch. | Define the scale of every input/output, apply ΔT conversion at one boundary, and replace alias-equality tests with equivalent-instant tests (UT input versus TT input shifted by ΔT) plus external reference values. |
| REL-45 | OPEN | S | `norm_deg` can return exactly `360.0`, violating the `[0, 360)` contract both copies document (`functions/utils.rs:257`, `astronomy/constants.rs:68`). `f64::rem_euclid` is `let r = x % rhs; if r < 0.0 { r + rhs.abs() } else { r }` — for any tiny negative input the `r + 360.0` rounds up to exactly `360.0`. Verified: `(-1e-18f64).rem_euclid(360.0) == 360.0`, likewise for `-1e-30` and `-f64::MIN_POSITIVE`. | Real, not theoretical: `panchanga.rs:278` already carries a comment about this class of overflow and defends with `.min(26)` / `.min(59)` clamps, so the symptom was seen once and patched locally instead of at the source. Any `(norm_deg(x) / 30.0) as usize` sign/house index — e.g. `indigenous.rs:40`, `chart.rs:1011`, `vedic.rs:180` — can produce an out-of-range slot. Fix in `norm_deg` (`if r >= 360.0 { 0.0 } else { r }`) and add a test for negative denormal input. Not a performance change; found while measuring PERF-16. |
| REL-44 | OPEN | S | `crossings.rs:64 opposite_sign` detects a sign change with `first * second < 0.0`, and `bracket_crossing` (`crossings.rs:99`) uses the same `d0 * d1 <= 0.0` test. The product underflows to `0.0` when both residuals are tiny (e.g. `1e-200 * 1e-200`) and overflows to infinity at the other extreme, so a genuine bracket can be silently missed or a false one accepted. | `searches.rs:23 brackets_zero` already does this correctly via `a.is_sign_negative() != b.is_sign_negative()` — a sign-bit compare with no rounding. Make `crossings.rs` use the same predicate and add a bracket test with denormal-scale residuals. |
| REL-17 | OPEN | S | `mesoamerican_calendars.svg.tt` fails with `undefined value (in t:75)` for the valid instant `2000-01-01 00:00 UTC`, although the built-in Mesoamerican renderer and `--print-context` succeed with complete data. | Reproduce with the bundled template, isolate the zero-index calendar value that MiniJinja rejects, and add a second template integration case covering this instant. |

## Robustness / Recovery

| id | status | effort | description | notes |
|---|---|---|---|---|
| ROB-3 | OPEN | M | Normal broken-pipe use panics: piping a rendered chart to `head` exits 101 from a stdout write failure. Output is spread across `print!` / `println!`, so other verbose commands have the same failure mode. | Route stdout through fallible buffered writes and treat `BrokenPipe` as a clean early exit; add a subprocess pipe-closure regression test. |

## Scalability

| id | status | effort | description | notes |
|---|---|---|---|---|


## Security

| id | status | effort | description | notes |
|---|---|---|---|---|
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
| WIRE-1 | DECIDED | — | Public Rust API functions such as `ayanamsa_ex`, `current_file_data`, `gauquelin_sector`, `get_orbital_elements`, `heliacal_pheno_ut`, `nod_aps_ut`, `set_tid_acc`, `tid_acc`, and `vis_limit_mag` have no CLI call site. | KEEP — `celestial-core` is a published library crate; these functions are intentionally root-re-exported and parity smoke-tested for external consumers. |
| WIRE-2 | DECIDED | S | `Body::is_known_id` is only used internally by `try_from_raw` plus tests. | KEEP — useful published predicate for callers that want a cheap bool pre-check. |
| WIRE-3 | DECIDED | S | `BodyError::OutOfRange { id }` is constructed but bindings stringify it instead of pattern matching. | KEEP — typed Rust variant preserves future extensibility while bindings expose language-native string errors. |

## Unused Functions / Methods

| id | status | effort | description | notes |
|---|---|---|---|---|
| UNUSED-1 | DECIDED | — | No new unused public-shaped production callables found in this rescan. | `cargo xtask parity`, binding stub checks, workspace tests, and prior public API review all remain green; existing library-only APIs are accounted for under WIRE-1. |

## Audit picks deliberately rejected

| id | status | effort | description | notes |
|---|---|---|---|---|
| PERF-16 | REJECTED | S | Proposed collapsing `functions/utils.rs:298 diff_deg_signed` from `wrap_signed_180(norm_deg(p1) - norm_deg(p2))` to `wrap_signed_180(p1 - p2)`, on the claim that `rem_euclid` is homomorphic over subtraction mod 360. | **Measured and withdrawn — it loses precision on out-of-range input.** Randomized sweeps: inputs already in `[0, 360)` are bit-identical (0/500,000 mismatches), but over ±1e6° 49,342/500,000 diverge with a worst error of 1.2e-10°, and over ±1e17° the worst error is **352°** — the argument reduction has to happen *before* the subtraction or the small operand is annihilated. The current triple-`rem_euclid` form is deliberate robustness for a public API that accepts arbitrary `f64`. The original entry also overstated the cost: `f64::rem_euclid` is one `%` plus a branch, not two `fmod` calls, so the remaining saving is ~1 `fmod` and not worth the exposure. |
