# Celestial — TODO

Rescan: **2026-06-06** (whole-project scan across all categories in `AGENTS.md`; current TODO rows ignored as requested, then fresh findings de-duplicated before entry).
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

(no findings)

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

(no findings — `Command` dispatch, plugin fallback, chart registry, `--calendar` value enum, timezone-required render path, and invalid chart/calendar cases are covered by workspace tests.)

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

(no findings — production shared configuration is thread-local; test-only PATH mutation uses a poison-safe mutex.)

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
| DOC-7 | OPEN | S | Calendar overlay docs/schema drift from live context: `README.md:355` names Omer `.sefirah`, `README.md:357` names Moon `.short`, and `cli/templates/context_schema.txt:168` documents `hebrew.holidays[]`; live code emits `day_sefirah` / `week_sefirah`, `phase_short`, and `hebrew.days[]` with no holiday list. | Template-author docs only; `cargo test --workspace`, `renders_year_calendar`, and template fixtures still pass because templates use the live keys. Also align stale schema tag names `sabbat` / `hebrew_holiday` with actual `sabbat_name` / Hebrew date fields. |
| DOC-4 | DECIDED | M | `celestial-cli` public items are intentionally undocumented. | Binary crate/lib split only exists so `main.rs` and tests can share modules; `#![warn(missing_docs)]` remains core-only. |

## Legacy / Deprecation

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — raw constants and legacy binding shapes are retained as back-compat decisions under ARCH/DUP.)

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

## Platform

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — Unix and non-Unix plugin execution paths compile; no-default-feature binding checks pass.)

## Plugin Extensibility

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — plugin discovery ignores non-executables, deduplicates names, sorts output, and returns actionable unknown-command errors.)

## Product Engineering

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Purpose

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings)

## Reliability / Correctness

| id | status | effort | description | notes |
|---|---|---|---|---|
| REL-10 | OPEN | S | `core/src/functions/moon_phases.rs:287` accepts `jd >= jd_from - 0.01`, so `next_principal_phase` / `next_new_moon` can return a phase up to 0.01 days before the requested start despite docs/tests saying at or strictly after `jd_from`. | Boundary issue only; broad moon/reference/property tests pass. `fuzz/src/main.rs:4233` also documents widened synodic tolerance and references a missing `next_new_moon` TODO, so either tighten the predicate and add a boundary regression or explicitly document an inclusive tolerance contract. |

## Robustness / Recovery

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — invalid parse/config/plugin/template paths return errors; fuzz/property suites remain green from prior fix pass.)

## Scalability

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — calendar batches are bounded to month/year; aspect/midpoint grids are fixed by chart-body count; MiniJinja fuel is capped.)

## Security

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — config-driven output path traversal is rejected; template execution has a fuel cap; user strings are XML-escaped where built-in SVG consumes them.)

## State Machine Integrity

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — CLI subcommand dispatch and render output mode state are covered by tests.)

## UI / UX

| id | status | effort | description | notes |
|---|---|---|---|---|

(no findings — localized help keys are complete; timezone-required errors point to explicit remediation.)

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

## Unused Functions / Methods

| id | status | effort | description | notes |
|---|---|---|---|---|
| UNUSED-1 | DECIDED | — | No new unused public-shaped production callables found in this rescan. | `cargo xtask parity`, binding stub checks, workspace tests, and prior public API review all remain green; existing library-only APIs are accounted for under WIRE-1. |
