# Celestial — Code Audit

Rescan: **2026-05-22** (develop, full 16-category re-audit, parallel
multi-agent sweep + per-claim verification; **fix pass applied same
day**). One table per category; stable IDs in the first column.
Completed work is removed (not listed).

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

> **This rescan found 16 OPEN; 13 were fixed the same day**, leaving
> **3 OPEN** (DUP-7, DOC-1, DOC-2) + 1 DEFERRED (DUP-8). Fixed and
> removed: the recurring feature-gate wiring class struck a third time
> in the language bindings (**WIRE-2**, commit `d6a3ead`) — the
> structural root, a feature-matrix CI that only `cargo check`ed, was
> closed by making it `cargo test` under `-D warnings` (**TEST-4/5**,
> `d1df606`); the phantom `chart` subcommand + 816-LOC orphan
> (**WIRE-3/DEAD-1/DOC-3**, `785c61a`); **PERF-7** doubled nutation
> (`e6395b8`, byte-identical); **CC-2..CC-5** (`16f4894`); and
> **REL-1/SEC-11** binding `chunks_exact` (`7728fa2`). Remaining OPEN
> are documentation-weight (DOC-1/2) + one small dedupe (DUP-7).

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
`svg_common::esc_var`) — removed per completed-work policy. **SEC-11**
(JS binding `chunks` panic unwinding across the `extern "C"` boundary
into V8 = UB) was found and **fixed this cycle** (`7728fa2`, with
REL-1) — removed. No OPEN security findings.

## 2. Reliability

Triaged all 440 `unwrap/expect/panic!/unreachable!` sites: the vast
majority are inside `#[cfg(test)]` mods; `let _ = write!(String,…)` is
infallible; float→int casts saturate (Rust ≥1.45, no UB on NaN/±Inf);
CLI `parse.rs` input is length-guarded; `plugin.rs` crosses no
privilege boundary. The one real cluster — unchecked `chunks()`
indexing on attacker-controlled binding input (REL-1) — was **fixed
this cycle** (`7728fa2`, `chunks_exact`) and removed.

| id | status | effort | description |
|---|---|---|---|
| REL-2 | DECIDED | S | `panchanga::karana_name(0)` (`panchanga.rs:201`, public) underflows `0u8-2` → debug-only panic; release wraps to a valid index and all internal callers pass `1..=60`. Release-safe robustness nit, not tracked. |

## 3. Wiring gaps

**New category.** A recurring failure mode: an item is defined/gated on
one side and referenced/gated differently on the other. The class has
now been hit and fixed **four times**: ARCH-7 (core `lib.rs`
re-exports), `6da2ec9` (core test imports), **WIRE-2** (`d6a3ead`, the
three language bindings — ~35 calendar-traditions symbols referenced
unconditionally, `cargo build -p celestial-py/-js --no-default-features`
was failing 37 errors), and the structural root **TEST-4** (`d1df606`,
matrix CI now `cargo test`s feature-off, see §4). The phantom `chart`
subcommand + its 816-LOC orphan (**WIRE-3/DEAD-1/DOC-3**, `785c61a`)
were the second cluster. All fixed and removed. No OPEN wiring findings.

---

# Quality gates

## 4. Test coverage

`cargo llvm-cov` 0.8.5; product code (core/src + cli/src, tests
excluded) = **94.9% region / 94.5% line / 94.7% function**; every
source file ≥80%. CI floor enforced (`--fail-under-lines 80
--fail-under-functions 90`, fuzz/xtask/binding-tests excluded). Only
known-uncovered product fn is `esbats::bisect_fallback_full_moon`
(reachable only on primary-solver failure). **TEST-4** (matrix CI only
`cargo check`ed, never executed feature-off behavior — the structural
root of the gate-bug class) and **TEST-5** (feature-off-only dead
imports) were both **fixed this cycle** (`d1df606`): the matrix step now
`cargo test`s every combo under `RUSTFLAGS=-D warnings`. Removed.

| id | status | effort | description |
|---|---|---|---|
| TEST-2 | DECIDED | — | `bindings/{js,python}/src/lib.rs` 0% — exercised only by the JS/Python language harnesses; `llvm-cov` can't instrument them. Not a real gap; the only sub-80% product area. |

## 5. Code complexity

Hard rule (CLAUDE.md): no fn CC > 10, tests included; flat lookup
`match` exempt. Most product fns peak ~6–7. The four fns over/at the cap
this rescan — **CC-2** `retrograde_station_ut` (~13), **CC-3**
`compute_aspects`, **CC-4** `elapsed_days`, **CC-5** `Tz::from_str`
(~11) — were all **fixed this cycle** (`16f4894`: helper extraction +
flat-match rewrite) and removed. Invariant restored.

| id | status | effort | description |
|---|---|---|---|
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
declined. **PERF-7** (doubled 77-term nutation eval on the hot path:
`apparent_planet/sun/moon` + `sidereal_time_deg` each ran `nutation`
then `true_obliquity`, which recomputes it) was **fixed this cycle**
(`e6395b8`, byte-identical — reuse the in-scope `nut.deps`) and removed.

| id | status | effort | description |
|---|---|---|---|
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

DOC-3 (stale `main.rs:3` doc header listing the removed `chart` builtin)
was **fixed this cycle** (`785c61a`, WIRE-3 cluster) and removed.

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

The 13 high-value OPEN findings from this rescan (WIRE-2, WIRE-3,
DEAD-1, DOC-3, TEST-4, TEST-5, PERF-7, CC-2..CC-5, REL-1, SEC-11) were
fixed and committed the same day (`785c61a`, `d6a3ead`, `d1df606`,
`e6395b8`, `16f4894`, `7728fa2`). **3 OPEN remain**, all bounded and
byte-safe:

1. **DOC-2** (S) — fix the broken/ambiguous intra-doc links so docs.rs
   renders clean; add `#![warn(missing_docs)]` to surface DOC-1 churn.
2. **DUP-7** (S/M) — extract a `write_year_axis` helper shared by the
   hellenistic + vedic year-axis ruler (~18 LOC dup). Keep SVG output
   byte-identical (regression-locked).
3. **DOC-1** (L) — backfill `///` on the 378 undocumented public items;
   its own session (mostly `constants.rs` + `functions/aspects.rs`).

DEFERRED: **DUP-8** (f64 SVG-preamble overload). All DECIDED rows kept
so a rescan doesn't re-flag.
