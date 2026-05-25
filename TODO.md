# Celestial — TODO

Rescan: **2026-05-25** (categories per `AGENTS.md` §Rules ∪ prior AUDIT extras; tables only).
Fix pass same day cleared SEC-12, TEST-6, DOC-5, DEAD-2, DUP-9, DUP-10, DDD-1 — rows removed per AGENTS.md. DEAD-3 demoted to DECIDED.

## Security

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Reliability / correctness

| id | status | effort | description | notes |
|---|---|---|---|---|
| REL-2 | DECIDED | S | `panchanga::karana_name(0)` (`panchanga.rs:201`, public) underflows `0u8-2` → debug-only panic; release wraps to a valid index; internal callers pass `1..=60`. | Release-safe robustness nit. Not tracked for fix. |

## Wiring gaps

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Concurrency

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — no new concurrency primitives in product code; only the test-only `path_lock` in `plugin.rs`, poison-safe.)

## Test coverage

| id | status | effort | description | notes |
|---|---|---|---|---|
| TEST-2 | DECIDED | — | `bindings/{js,python}/src/lib.rs` 0% — exercised only by the JS/Python language harnesses; `llvm-cov` can't instrument them. | Not a real gap; the only sub-80% product area. |

## Code complexity

| id | status | effort | description | notes |
|---|---|---|---|---|
| CC-6 | DECIDED | S | `calendar_overlays.rs:509 render_day_cell` sits at exactly CC 10 — compliant, zero headroom. Next decoration would breach cap. | If extended, split moon-glyph + omer-badge blocks into `write_*` helpers. |
| CC-1 | DECIDED | — | `cli_fuzz.rs:250 boundary_…strings`; `parse.rs:418 parse_tz_forms` (test mod). Clippy `cognitive_complexity` 16/12 but pure `assert!`/`matches!` macro expansion; real cyclomatic ≤3. | Lint is `nursery`/disabled. No logic to split. |

## Code duplication

| id | status | effort | description | notes |
|---|---|---|---|---|
| DUP-1 | DECIDED | L | Bindings 201×3 per-export return-adapter stubs — shared *input* half already factored in `bindings/ffi`; residual = irreducible per-language *return* shapes (py tuple / php map / js struct). | Only a ~600-1000 LOC spec + 3-emitter codegen removes it, regenerating 3 published APIs with php unverifiable. Net-negative. = ARCH-10 / DP-4. |
| DUP-4 | DECIDED | — | `revjul`/`revjul_hms` 3 return shapes — intentional per-language idioms; core call already shared. | Normalizing = published-API break. |
| DUP-6 | DECIDED | S | `Xorshift64` PRNG copied across `cli_fuzz.rs:18` and `fuzz/src/main.rs:18` — test-only, 2 crates/targets, ~15 LOC. | Shared dev-dep crate disproportionate. |
| DUP-5 | DECIDED | — | Per-tradition wheel geometry — distinct layout constants, not duplication; shared halves already factored. | N-A. |

## Performance

| id | status | effort | description | notes |
|---|---|---|---|---|
| PERF-8 | DECIDED | S | `svg_common::svg_doc_open` chains 3 `String::replace` + 2 `format!` per chart (~6 allocs for preamble). | One call per chart; not hot. Combining needs a build-time concat or tiny templater. Net-neutral. |
| PERF-9 | DECIDED | S | `parse_chart_type` (`args.rs:49`) calls `registered_chart_types()` twice — each call joins all aliases into one `String` then splits it. | One-shot at CLI start; <1µs. Cosmetic micro-perf. |
| PERF-2/3 | DECIDED | — | `searches.rs` post-bisect `calc_ut`/`houses` full-flag re-eval is authoritative, not redundant (scan strips SPEED). | Locked by `perf2345_search_regression_lock`. |
| PERF-4 | DECIDED | — | `bisect_retro_station` already reuses the last loop sample; no recompute. | N-A. |
| PERF-5 | DECIDED | M | `next_aspect_with2` dual-scan merge — precision-sensitive rewrite for marginal gain. | Auto-decline per byte-identical gate. |
| PERF-6 | DECIDED | — | `compute_speed` ±0.5 d central diff is the minimal 2-eval 2nd-order form; forward/back loses precision. | Locked. |

## Scalability

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — aspect grids / `midpoint_table` / `spread_labels` are O(n²) on a fixed chart-body count; calendar batches bounded to a year/month; graphic-ephemeris sampling clamps `days ∈ [28, 366]`; MiniJinja fuel-capped 50M instr.)

## Architecture / SOLID / POSA

| id | status | effort | description | notes |
|---|---|---|---|---|
| ARCH-11 | DECIDED | S | `bindings/ffi/src/lib.rs:17` `pub use celestial_core::*;` glob re-export — any new pub in core silently expands the FFI surface and the 3 binding APIs. | Intentional per `lib.rs:8-14` comment: bindings need the *whole* core API. Explicit list = 200+ items duplicating `lib.rs`. = ARCH-10 class. |
| ARCH-12 | DECIDED | S | `core::lib.rs` flat re-exports ~300 raw `i32` constants (`SUN`/`MOON`/`FLG_*`/`SIDM_*`/`ECL_*`/`TIDAL_*`) alongside the typed `body::*`/`CalcFlags`. Two parallel APIs. | Back-compat layer for the bindings + published API; tightening = published-API break. Documented at `lib.rs:45`. |
| ARCH-10 | DECIDED | L | Bindings 201×3 stubs, no codegen. | = DUP-1 / DP-4 (measured net-negative; php unverifiable). |

## Modularity & SoC / decoupling

| id | status | effort | description | notes |
|---|---|---|---|---|
| MOD-1 | DECIDED | S | `cli/cmd/render/svg_common.rs::write_year_axis` (`svg_common.rs:107-140`) hard-codes `celestial_core::revjul`/`julday` calls inside a shared render helper — domain call from the rendering layer. | Idiomatic enough (axis ticks need JD↔Gregorian); pass-closures alternative is heavier. Acceptable thin-veneer use of core API. |

## Visibility

| id | status | effort | description | notes |
|---|---|---|---|---|
| VIS-2 | DECIDED | S | `cli/src/format.rs` exposes 9 `pub fn` (`xml_escape`, `lon_zodiac`, `deg_dms`, `dist_au`, `speed_dday`, `rule`, `lpad`, `rpad`, `json_obj`, `json_array`). Used cross-module in cli; no external consumers. Could be `pub(crate)`. | Same class as VIS-1 / DOC-4: cli is a binary crate; `pub` has no library effect. `lib.rs` only exists to enable integration tests + fuzz. Cosmetic. |
| VIS-1 | DECIDED | S | `render/{specialist,hellenistic,vedic,pipeline}.rs` `build_*`/`render_*` are bare `pub fn` where siblings use `pub(super)` (`mod.rs:486/1627/1890`). | No real leak (parent submodules are private `mod`); cosmetic. |

## Design patterns

| id | status | effort | description | notes |
|---|---|---|---|---|
| DP-12 | DECIDED | S | `builtin_svg::Palette` (`<'a>` borrow) vs `svg_common::SvgPalette` (owned `String`, escaped) — two near-identical palette structs with different escape contracts. | Unifying = either eager escape (breaks byte-gate if any char ≠ identity) or lazy escape (wraps every site). Touches byte-identical gate → DECIDED. |
| DP-1 | DECIDED | M | Registry macro over the ~12 heterogeneous `dispatch_*` (distinct return-jd / `--years` / date2 logic) — a macro there is closure indirection over a clear hot-path adapter. | The 15 uniform dispatchers were already collapsed into `specialist_dispatch!`. Dismissed. |
| DP-6 | DECIDED | L | SVG → MiniJinja templates: `Palette` half shipped; templating the 29 renderers changes whitespace → breaks byte-identical gate. | Nothing byte-safe remains. |
| DP-11 | DECIDED | M | `OutputFormatter` trait over calc/moon/houses/chart — per-command JSON keys + text columns are bespoke; trait abstracts only the 2-line json/text branch. | Leaky. |
| DP-4 | DECIDED | L | Binding codegen. | = ARCH-10 / DUP-1. |

## Documentation

| id | status | effort | description | notes |
|---|---|---|---|---|
| DOC-4 | DECIDED | M | `celestial-cli` has 34 undocumented `pub` items. Binary crate; `pub` exists only so `main.rs` can use the lib — not a published library API. | `#![warn(missing_docs)]` deliberately core-only. Not a real gap. |

## Business patterns / DDD

| id | status | effort | description | notes |
|---|---|---|---|---|

## Observability

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — core has no production `println!`/`eprintln!`; CLI errors carry actionable context via `ParseError`; `plugin::try_exec` formats both "failed to exec" and "unknown command — run --help"; lib/CLI observability is light by design.)

## Usability / ergonomics

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — `--help` text i18n-routed via `localize()`; `BUILTIN_COMMANDS` stays in sync with the `Command` enum; plugin error messages point at `celestial --help`; REL-3 fix already guards `parse_numeric_offset`.)

## Unused functions / methods

| id | status | effort | description | notes |
|---|---|---|---|---|
| DEAD-3 | DECIDED | S | `celestial_ffi::pos6` used only once (php `lib.rs:53`); js inlines its own struct shape, python uses `pos6_tuple`. | KEEP — docstring frames `pos6` as the array counterpart to `pos6_tuple`; php is a legitimate consumer and the only ffi-shape binding. Logged so a future rescan doesn't re-flag. |
