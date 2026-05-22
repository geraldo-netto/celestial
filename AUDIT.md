# Celestial — Code Audit

Rescan: **2026-05-22** (develop, full 16-category re-audit, parallel
multi-agent sweep + per-claim verification). One table per category;
stable IDs in the first column. Completed work is removed (not listed).

| state | meaning |
|---|---|
| OPEN | actionable, bounded, byte-safe — do next |
| DEFERRED | real; needs an isolated session + precision soak |
| DECIDED | WONTFIX / DECLINED / N-A with rationale (kept so a rescan doesn't re-flag) |

| effort | scale |
|---|---|
| S | bounded, < 1 h, no precision risk |
| M | isolated session (multi-file or needs a soak) |
| L | large / cross-crate / published-API surface |

> **OPEN findings this rescan: 14** (was 0). The headline is a
> **recurring feature-gate wiring class**: ARCH-7 fixed it in the core
> `lib.rs` re-exports, `6da2ec9` fixed it in the core test files, and
> this rescan finds the **same bug a third time in the language
> bindings** (WIRE-2) — plus the structural reason it keeps shipping:
> the feature-matrix CI only `cargo check`s, never tests (TEST-4). A
> second cluster is the phantom `chart` subcommand + its 816-line
> orphaned source file (WIRE-3 / DEAD-1).

Category order: **correctness & safety** first (Security, Reliability,
Wiring gaps), then **quality gates** (Test coverage, Complexity,
Duplication), **performance & scale** (Performance, Scalability),
**structure** (Architecture/SOLID/POSA, Modularity & SoC, Visibility,
Design patterns), and **domain & docs** (Documentation, DDD,
Usability/ergonomics).

---

# Correctness & safety

## 1. Security

`unsafe`-free in core/cli/bindings/ffi source; FFI via
napi/pyo3/ext-php-rs; `plugin.rs` execs via arg array (no shell);
`cargo audit` clean (90 deps, 0 advisories — re-verified this rescan).
SEC-1..SEC-10b all fixed (SVG/XML injection escaping reached every
renderer incl. specialist/calendar/south_indian via
`svg_common::esc_var`) — removed per completed-work policy. One **new**
finding: a binding panic that crosses an FFI boundary as UB.

| id | status | effort | description |
|---|---|---|---|
| SEC-11 | OPEN | S | `bindings/js/src/lib.rs:2216,2238,2255,2279` — REL-1's `chunks(2\|3)` index panic in `calcChartAspects/Auto`, `midpointTable`, `solarArcDirections` unwinds across the generated `extern "C"` boundary into V8: `#[napi]` fns here lack `catch_unwind` and the workspace is `panic=unwind` (no `panic="abort"`) ⇒ **UB**. Fix: eliminate the panic (REL-1) and/or add `#[napi(catch_unwind)]`. |

## 2. Reliability

Triaged all 440 `unwrap/expect/panic!/unreachable!` sites: the vast
majority are inside `#[cfg(test)]` mods; `let _ = write!(String,…)` is
infallible; float→int casts saturate (Rust ≥1.45, no UB on NaN/±Inf);
CLI `parse.rs` input is length-guarded; `plugin.rs` crosses no
privilege boundary. The one real cluster is unchecked `chunks()`
indexing on attacker-controlled binding input.

| id | status | effort | description |
|---|---|---|---|
| REL-1 | OPEN | S | JS & PHP bindings build positions via `positions.chunks(2\|3).map(\|c\| (.., c[1], c[2]))` on a caller-supplied `Vec<f64>` (`js/src/lib.rs:2216,2238,2255,2279`; `php/src/lib.rs:1563,1584,1601,1625`); a non-multiple length makes the final short chunk panic on `c[1]`/`c[2]`. Python is safe (typed `Vec<(i32,f64[,f64])>` — pyo3 enforces arity). Fix: `chunks_exact(N)` or `.filter(\|c\| c.len()==N)`. |
| REL-2 | DECIDED | S | `panchanga::karana_name(0)` (`panchanga.rs:201`, public) underflows `0u8-2` → debug-only panic; release wraps to a valid index and all internal callers pass `1..=60`. Release-safe robustness nit, not tracked. |

## 3. Wiring gaps

**New category.** A recurring failure mode: an item is defined/gated on
one side and referenced/gated differently on the other. ARCH-7 (core
`lib.rs` re-exports) and `6da2ec9` (core test imports) were prior
instances of this exact class — both fixed. The bindings were never
checked under feature-off; they have it too.

| id | status | effort | description |
|---|---|---|---|
| WIRE-2 | OPEN | M | `bindings/{python,js,php}/src/lib.rs` reference ~35 `calendar-traditions`-gated core symbols **unconditionally** while only ~10 are `#[cfg]`-gated. Verified: `cargo build -p celestial-py --no-default-features` and `-p celestial-js --no-default-features` each fail with **37 errors** (`omer_*`, `sabbats/esbats_for_year`, `jewish_*`, `hijri_*`, `easter_*`, `bahai_*`, `nowruz_*`, `vesak_jd`, `uposatha_days`, `SabbatKind`, …). Same ARCH-7 def/use cfg asymmetry, now in bindings (python gates Coptic/Ethiopic/Fasli/Tibetan at `lib.rs:1478-1539` but leaves the rest + the `#[pymodule]` adds bare). Fix: add `#[cfg(feature="calendar-traditions")]` to every binding wrapper **and** its module-registration line whose core symbol is gated, in all three. |
| WIRE-3 | OPEN | S | Phantom `chart` subcommand: `main.rs:116` lists `"chart"` in `BUILTIN_COMMANDS` (and the `main.rs:3` doc header advertises it — see DOC-3), but there is **no** `Chart` variant in `enum Command`. So `celestial chart …` is flagged builtin (`main.rs:130`), **skips plugin dispatch**, then clap dies "unrecognized subcommand" — and a user's `celestial-chart` PATH plugin is permanently shadowed. Fix: drop `"chart"` from `BUILTIN_COMMANDS` (`render` supersedes it). |
| DEAD-1 | OPEN | M | `cli/src/cmd/chart.rs` (816 LOC: `ChartArgs`/`run`/`compute_chart`/`print_json`/`render_svg`) is **never declared as a module** (`cmd/mod.rs` has no `mod chart`), so Cargo never compiles it — no dead-code warning, silent rot; functionality fully duplicated by `render --chart-type natal`. Last touched by an SVG-escape commit so it may retain pre-SEC-fix unescaped patterns. Fix: delete (pair with WIRE-3). |

---

# Quality gates

## 4. Test coverage

`cargo llvm-cov` 0.8.5; product code (core/src + cli/src, tests
excluded) = **94.9% region / 94.5% line / 94.7% function**; every
source file ≥80%. CI floor enforced (`--fail-under-lines 80
--fail-under-functions 90`, fuzz/xtask/binding-tests excluded). Only
known-uncovered product fn is `esbats::bisect_fallback_full_moon`
(reachable only on primary-solver failure). The new findings are about
**which build configs the suite actually runs in**, not raw %.

| id | status | effort | description |
|---|---|---|---|
| TEST-4 | OPEN | M | The feature-matrix CI job only runs `cargo check` per combo (`.github/workflows/celestial-core.yml:218-225`); the `test` job runs **default (all-features) only**. So `calendar-traditions`-off / `timezone`-off *behavior* is compile-checked, never executed — the structural reason the `6da2ec9` test-gate bug and WIRE-2 both reached review. Fix: add a `cargo test --no-default-features` (± each feature) run to the matrix. |
| TEST-5 | OPEN | S | The `6da2ec9` gate fix left feature-off warnings the `-D warnings` gate can't see (it runs all-features): unused imports `unit_tests.rs:1453,1631,1632`, dead `const` `unit_tests.rs:1634`, unused glob `helpers_test.rs:490` — surface only under `--no-default-features`. Fix alongside TEST-4. |
| TEST-2 | DECIDED | — | `bindings/{js,python}/src/lib.rs` 0% — exercised only by the JS/Python language harnesses; `llvm-cov` can't instrument them. Not a real gap; the only sub-80% product area. |

## 5. Code complexity

Hard rule (CLAUDE.md): no fn CC > 10, tests included; flat lookup
`match` exempt. Most product fns peak ~6–7. CC counted manually (no CC
tool installed). Four fns now exceed/skirt the cap — CC-2 is a clear
violation, CC-3/4/5 are marginal (~11).

| id | status | effort | description |
|---|---|---|---|
| CC-2 | OPEN | M | `retrograde_station_ut` `core/src/functions/chart.rs:257` — CC ~13: scan loop + sign-change `if` + two `if/else-if` each with 3-term `&&` chains + final 3-arm tuple `match` (not a flat dispatch). Real violation. Fix: extract a `classify_station` helper. |
| CC-3 | OPEN | S | `compute_aspects` `cli/src/cmd/render/context.rs:589` — CC ~11: 3 nested `for` + ASC/MC exclusion `&& \|\| &&` + orb `if` + `"square"\|\|"opposition"` in the `json!`. Marginal. |
| CC-4 | OPEN | S | `elapsed_days` `core/src/functions/omer.rs:148` — CC ~11: Hebrew dechiyot postponement boolean clusters (`if` with two `\|\|`-joined `&&&&` clauses + `alt%7 == 0\|\|3\|\|5`). Marginal. |
| CC-5 | OPEN | S | `<Tz as FromStr>::from_str` `cli/src/parse.rs:132` — CC ~11: `UTC\|GMT\|Z\|UT` string dispatch written as a `\|\|`-chain (not a flat `match`, so not exempt) + numeric `+/-` branch. Rewrite the chain as a flat `match` → < 10. |
| CC-1 | DECIDED | — | `cli_fuzz.rs:250 boundary_…strings`; `parse.rs:418 parse_tz_forms` (test mod) — clippy `cognitive_complexity` 16/12 but pure `assert!`/`matches!` macro expansion; real cyclomatic ≤3. Lint is `nursery`/disabled. No logic to split. |

## 6. Code duplication

| id | status | effort | description |
|---|---|---|---|
| DUP-7 | OPEN | M | Year-axis ruler block duplicated ~18 LOC: `hellenistic.rs:315-334` vs `vedic.rs:879-898` — same `birth_year..=end_year`, `.step_by(5)` loop, `julday`+x mapping, range-clamp `continue`, gridline `<line>`+`<text>` emit; diffs are only clamp bounds / colour var / font-size. Extract `write_year_axis(s, colour, font, clamp, …)`. |
| DUP-8 | DEFERRED | S | Inline SVG preamble (`<?xml…><svg viewBox><rect bg/>`) repeated verbatim in ~8 renderers (`south_indian.rs:243`, `vedic.rs:65,265`, `omer_grid.rs:209`, `hellenistic.rs:304`, `specialist.rs:446,653`, `calendar_overlays.rs:690`, `chinese.rs:161`). `svg_common::svg_doc_open()` already dedupes it but only for `u32` dims; these use `f64` dims. An `f64` overload consolidates it — but `svg_common`'s own doc sanctions the split as intentional. Escaping is consistently applied across all copies (no verbatim-copy escaping bug). |
| DUP-1 | DECIDED | L | bindings 201×3 per-export return-adapter stubs — shared *input* half already factored in `bindings/ffi`; residual is irreducible per-language *return* shapes (py tuple / php map / js struct). Only a ~600–1000 LOC spec+3-emitter codegen removes it, regenerating 3 *published* APIs with php unverifiable. Net-negative. = ARCH-10 / DP-4. |
| DUP-4 | DECIDED | — | `revjul`/`revjul_hms` 3 return shapes — intentional per-language idioms; core call already shared. Normalizing = published API break. |
| DUP-6 | DECIDED | S | `Xorshift64` PRNG copied across `cli_fuzz.rs:18` and `fuzz/src/main.rs:18` — test-only, 2 crates/targets, ~15 LOC; a shared dev-dep crate is disproportionate. |
| DUP-5 | DECIDED | — | Per-tradition wheel geometry — distinct layout constants, not duplication; shared halves already factored. N-A. |

---

# Performance & scale

## 7. Performance (precision is the hard gate)

Outputs must stay byte-identical to the 1986-05-30 PDF reference and
the Diana 1961-07-01 chart; any result-changing perf idea is auto-
declined. One **new** precision-neutral win found.

| id | status | effort | description |
|---|---|---|---|
| PERF-7 | OPEN | S | Doubled nutation eval on the hottest path: `apparent_planet/sun/moon` (`planetary.rs:79/89, 119/128, 145/150`) and `sidereal_time_deg` (`houses.rs:771-772`) call `nutation(jde)` then `true_obliquity(jde)`, which **recomputes the same 77-term IAU 2000B series** (`nutation.rs:48-51`). Runs on every probe of the bracket/bisection loops (100k+ iters). Fix: `let eps = mean_obliquity(jde) + nut.deps/3600.0;` reusing the in-scope `nut`. **Byte-identical** (pure fn of `jde`; reproduces `true_obliquity`'s arithmetic verbatim). |
| PERF-2/3 | DECIDED | — | searches.rs post-bisect `calc_ut`/`houses` full-flag re-eval is authoritative, not redundant (scan strips SPEED). Locked: `perf2345_search_regression_lock`. |
| PERF-4 | DECIDED | — | `bisect_retro_station` already reuses the last loop sample; no recompute. N-A. |
| PERF-5 | DECIDED | M | `next_aspect_with2` dual-scan merge is a precision-sensitive rewrite for marginal gain. |
| PERF-6 | DECIDED | — | `compute_speed` ±0.5 d central diff is the minimal 2-eval 2nd-order form; forward/back loses precision. |

## 8. Scalability

Clean. Aspect grids / `midpoint_table` / `spread_labels` are O(n²) but
n is the fixed chart body count (~13–23), not user-controllable.
Graphic-ephemeris sampling clamps `days ∈ [28, 366]`; calendar/moon/omer
batches are bounded to one year/month; `fixstars::find_star` is a
single pass; the MiniJinja path is fuel-capped (50M instr). No
O(n²)-or-worse over a user-controllable span; multi-century requests
stay bounded per body via the fixed transit-window table. No tracked
findings.

---

# Structure

## 9. Architecture / SOLID / POSA

Dependency graph re-verified **acyclic and layered**: core → only
`serde`; ffi → core; {js,php,python} → ffi; **cli → core only**. All
public error/value enums `#[non_exhaustive]`. `render/mod.rs` (1727 LOC)
is ~70% tests + banners — a facade, not a god-module. No new SRP/SOLID
violation.

| id | status | effort | description |
|---|---|---|---|
| ARCH-10 | DECIDED | L | bindings 201×3 stubs, no codegen — = DUP-1 / DP-4 (measured net-negative; php unverifiable). |

## 10. Modularity & separation of concerns

Clean. Domain logic stays in core; render/CLI layers consume it; no new
circular coupling; feature gating separates `timezone` /
`calendar-traditions` cleanly **within core** (the leaks are at the
binding/test edges — WIRE-2, TEST-5). No tracked findings.

## 11. Visibility

| id | status | effort | description |
|---|---|---|---|
| VIS-1 | DECIDED | S | `render/{specialist,hellenistic,vedic,pipeline}.rs` `build_*`/`render_*` are bare `pub fn` where siblings use `pub(super)` (e.g. `mod.rs:486/1627/1890). No real leak (parent submodules are private `mod`); cosmetic. |

## 12. Design patterns

| id | status | effort | description |
|---|---|---|---|
| DP-1 | DECIDED | M | Registry macro over the ~12 heterogeneous `dispatch_*` (distinct return-jd / `--years` / date2 logic) — a macro there is closure indirection over a clear hot-path adapter. The 15 *uniform* dispatchers were already collapsed into `specialist_dispatch!`. Dismissed. |
| DP-6 | DECIDED | L | SVG → MiniJinja templates: `Palette` half shipped; templating the 29 renderers changes whitespace → breaks the byte-identical gate. Nothing byte-safe remains. |
| DP-11 | DECIDED | M | `OutputFormatter` trait over calc/moon/houses/chart — per-command JSON keys + text columns are bespoke; the trait abstracts only the 2-line json/text branch. Leaky. |
| DP-4 | DECIDED | L | Binding codegen — = ARCH-10 / DUP-1. |

---

# Domain & docs

## 13. Documentation

| id | status | effort | description |
|---|---|---|---|
| DOC-1 | OPEN | L | 378 public items lack `///` docs (rustdoc `-W missing_docs`, all-features): `constants.rs` 207, `functions/aspects.rs` 79, `body/mod.rs` 39, `eclipses.rs` 15, … No `#![warn(missing_docs)]`/`deny` anywhere in core or cli. |
| DOC-2 | OPEN | S | Broken/ambiguous intra-doc links in core: `lib.rs:25` `[\`houses\`]` ambiguous (fn vs module); unresolved `FLG_EQUATORIAL`, `position::calc_ut`, `calc_chart_aspects_with_orb`, `astronomy::delta_t_for_year`; several public-doc links point to private items (`FESTIVAL_RULES`, `bisect_zero`, `ASPECT_BASE_ORBS`) → render broken on docs.rs. |
| DOC-3 | OPEN | S | `main.rs:3` doc header lists built-ins as `"calc houses chart render …"` — `chart` no longer exists (renamed `render`). README is correct; only the code comment is stale. (= WIRE-3 cluster.) |

## 14. Business patterns / DDD

Clean. Domain model coherent (ephemeris / chart / calendar / time
bounded contexts map to modules; ubiquitous language consistent). The
`JulianDay`/`Longitude`/`Latitude`/`Degrees` newtypes (DP-2, done) are
fully threaded through public moon/motion/chart/solar/vedic signatures —
**no raw `f64` for JD/lon/lat**, so no primitive-obsession gap. Any
further newtyping touches the published API → DECIDED, not OPEN. No
tracked findings (the orphaned `chart.rs` is DEAD-1).

## 15. Usability / ergonomics

CLI flag naming / `--help` / i18n localization consistent across the 12
live subcommands; no dead i18n keys. The one footgun is the phantom
`chart` builtin shadowing PATH plugins — tracked as **WIRE-3**. No
separate findings.

---

## Recommended next

Byte-safe, high-value, do first:

1. **WIRE-2** (M) — gate the binding calendar-traditions wrappers; the
   feature-off bindings are currently un-buildable. Same fix shape as
   ARCH-7 / `6da2ec9`. Verify with `cargo build -p celestial-py
   --no-default-features` (php standalone — not a workspace member).
2. **WIRE-3 + DEAD-1 + DOC-3** (S+M) — drop the `"chart"` builtin,
   delete the 816-LOC orphan, fix the doc header. One coherent cleanup.
3. **TEST-4** (M) — make the feature matrix `cargo test`, not just
   `check`; this closes the class that produced WIRE-2 and `6da2ec9`.
   Then **TEST-5** (S) clears the feature-off warnings it surfaces.
4. **PERF-7** (S) — byte-identical; removes one 77-term series eval per
   hot-path probe. Re-lock against the PDF reference.
5. **CC-2** (M) then **CC-3/4/5** (S) — restore the CC ≤ 10 invariant.
6. **REL-1 / SEC-11** (S) — `chunks_exact` in the JS/PHP bindings
   (SEC-11 is the JS UB elevation).

Then **DUP-7** (S/M), **DOC-1/2** (L/S). DEFERRED: **DUP-8**. All
DECIDED rows kept so a rescan doesn't re-flag.
