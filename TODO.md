# Celestial — TODO

Rescan: **2026-05-27** (post-d163f8b newtype-threading refactor; categories per `AGENTS.md` §Rules).
2026-05-27 fix pass cleared: REL-2, REL-7, REL-8, DUP-6, DUP-11, VIS-1 (partial), VIS-2, DEAD-3 — rows removed per AGENTS.md.
2026-05-27 second-pass rescan post-fix: 3 new test fns refactored to satisfy CC ≤ 10 (was 11/11/15); WIRE-2/3 reviewed → KEEP; DUP-2/DUP-3/CC-7 logged DECIDED.
Prior rescan 2026-05-25 cleared SEC-12, TEST-6, DOC-5, DEAD-2, DUP-9, DUP-10, DDD-1.

## Security

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Reliability / correctness

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — REL-2/REL-7/REL-8 fixed 2026-05-27; clamps + `Body::try_from_raw` validated at FFI seam, 9 new core tests + 88 fuzz suites green.)

## Wiring gaps

| id | status | effort | description | notes |
|---|---|---|---|---|
| WIRE-1 | DECIDED | — | 12 `pub fn` in `core/src/functions/{config,calc,phenomena}.rs` + `astronomy/heliacal.rs:334` have tests but **zero** prod callers in `cli`, `bindings/{ffi,js,python,php}`: `ayanamsa_ex`, `ayanamsa_ex_ut`, `current_file_data`, `gauquelin_sector`, `get_orbital_elements`, `heliacal_pheno_ut`, `nod_aps_ut`, `orbit_max_min_true_distance`, `set_lapse_rate`, `set_tid_acc`, `tid_acc`, `vis_limit_mag`. | KEEP — `celestial-core` is a published library crate; these are part of the public Rust API surface re-exported in `lib.rs:201-208`. Bindings rely on `pub use celestial_core::*` glob (= ARCH-11), so any external Rust consumer of the crate gets them. Tests function as smoke-tests for the published API. Logged so future rescans don't re-flag. |
| WIRE-2 | DECIDED | S | `Body::is_known_id` — only internal caller is `try_from_raw`. Tests use it for consistency cross-check. Zero external prod caller in cli/bindings. | KEEP — published library predicate: lets external Rust consumers pre-check ids without paying the `BodyError` allocation. Useful in hot loops where callers want a `bool` instead of a `Result`. Documented in `lib.rs:92-103`. |
| WIRE-3 | DECIDED | S | `BodyError::OutOfRange { id }` is constructed by `try_from_raw` but never pattern-matched by any binding — `body_of` shims convert via `.to_string()` in all 3 languages. | KEEP — typed variant lets future error kinds be added without breaking external Rust consumers; `#[non_exhaustive]` could be added but is forward-compat noise. Bindings can keep stringifying. |

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
| CC-1 | DECIDED | — | `cli_fuzz.rs:227 boundary_empty_null_and_oversize_strings` CC 16/10; `parse.rs:422 parse_tz_forms` (test mod) CC 14/10. Pure `assert!`/`matches!` macro expansion; real cyclomatic ≤3. | Lint is `nursery`/disabled. No logic to split. |
| CC-7 | DECIDED | S | `Body::is_known_id` (`core/src/body/mod.rs:119`) CC 6/10. Currently compliant — flat `if` ladder over 5 documented id windows. | One extra range → CC 8; still compliant. Acceptable headroom. |

## Code duplication

| id | status | effort | description | notes |
|---|---|---|---|---|
| DUP-1 | DECIDED | L | Bindings 201×3 per-export return-adapter stubs — shared *input* half already factored in `bindings/ffi`; residual = irreducible per-language *return* shapes (py tuple / php map / js struct). | Only a ~600-1000 LOC spec + 3-emitter codegen removes it, regenerating 3 published APIs with php unverifiable. Net-negative. = ARCH-10 / DP-4. |
| DUP-2 | DECIDED | S | `body_of(n)` + `.iter().map(\|&p\| body_of(p)).collect::<…Result<Vec<_>>>()?` triplicated in `bindings/{js,python,php}/src/lib.rs`. Each wraps `Body::try_from_raw` with the per-language error type. | Same class as DUP-1: per-language error mapping is the irreducible per-binding part. Shared abstraction would require a trait object obscuring error-flow. |
| DUP-3 | DECIDED | — | `core/tests/rel_clamps.rs` (deterministic regression) and `fuzz/src/main.rs::test_rel_clamps` (randomized property-based) overlap on `Body::try_from_raw` edge-grid coverage. | Intentional split: `cargo test` runs the regression suite; the fuzz binary is opt-in. Removing either loses coverage at one tier. |
| DUP-4 | DECIDED | — | `revjul`/`revjul_hms` 3 return shapes — intentional per-language idioms; core call already shared. | Normalizing = published-API break. |
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
| VIS-1b | DECIDED | — | `cli/src/cmd/render/pipeline.rs::run` stays `pub fn` because `cli/src/main.rs` reaches it through `celestial_cli::cmd::render::run` (lib/bin boundary). | Inner `build_*`/`render_*` in `specialist.rs`/`hellenistic.rs`/`vedic.rs` were narrowed to `pub(super)` 2026-05-27. |

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

(no findings)

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
| (none found) | — | — | Clippy + manual scan (core pub fns + CLI visibility items) returned zero unref callables. All pub fns have test + binding + CLI + lib.rs refs. | Published core-API coverage intentional (WIRE-1). |
