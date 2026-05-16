# Celestial — Code Audit

Date: 2026-05-16 · Scope: Rust workspace (`core/`, `cli/`, `bindings/`, `xtask/`, `benches/`, `fuzz/`) — 143 files, ~68.5k LOC. Read-only analysis; no code changed.

---

## 1. Cyclomatic complexity > 10 — severity: MODERATE

| file:line | fn | est. CC | why |
|---|---|---|---|
| cli/src/cmd/render/mod.rs:1517 | `run` | ~22 | many `?`, nested if/else for tz/date branches, 4 early-return flags |
| cli/src/cmd/render/context.rs:534 | `compute_aspects` | ~16 | triple-nested loop (n×n×ASPECT_DEFS) + ASC/MC guard + orb check |
| core/src/functions/chart.rs:256 | `retrograde_station_ut` | ~15 | scan loop + sign-change branch + 4 state guards + 3-arm result match |
| core/src/functions/searches.rs:452 | `years_diff` | ~13 | mirrored fwd/back loops, inner break conds |
| cli/src/cmd/render/calendar_overlays.rs:508 | `render_day_cell` | ~12 | 9 overlay branches (lag/sabbat/moon/omer badges) |
| core/src/functions/searches.rs:756 | `lower_meridian_transit_ut` | ~11 | closure + scan while-loop + dual cond |
| cli/src/cmd/render/calendar_overlays.rs:673 | `render_default_calendar_svg` | ~11 | week/day loops + conditional SVG sections |
| cli/src/cmd/render/vedic.rs:18 | `render_north_indian_svg` | ~11 | 4 loops + if branches (12-house diamond) |
| cli/src/cmd/render/vedic.rs:747 | `render_dasha_svg` | ~11 | 3 nested loops + conditional rows |
| cli/src/cmd/render/specialist.rs:* | `build_*_context` (dial/composite/triwheel/graphic_ephemeris) | ~10–12 | optional-field if-let chains + loops |

Note: many 11–15-arm `match` fns (houses dispatch, parse, esbats, main subcommand) are flat lookup tables — high arm count, low real path risk. No outliers > ~25.

---

## 2. Code duplication — severity: MODERATE-HIGH

| location(s) | duplicated | size | dedupe |
|---|---|---|---|
| bindings/{python,php,js}/src/lib.rs | ~201-fn FFI surface hand-mirrored 3× | ~7.7k LOC | code-gen all 3 from one API spec/macro |
| cli/src/cmd/render/{mesoamerican,chinese,indigenous,vedic,hellenistic,specialist,omer_grid,calendar_*}.rs | identical SVG preamble (xml/svg/rect/title/date) ×~17 | ~250 LOC | `svg_header(ctx,w,h)->(String,Palette)` |
| cli/src/cmd/render/*.rs | panel-card `<rect>/<text>` blocks re-hand-written | dozens | `panel_card(s,x,y,w,h,heading,rows)` |
| cli/src/cmd/{calc,moon,houses,...}.rs | `parse_date` + json/text output scaffold ×~10 | ~10–20 each | `run_with_jd` + `emit()` helper |
| bindings */lib.rs `*_many` | verbatim copy of non-`many` sibling | ~6 pairs ×3 | generic `map_results` closure |

Core logic largely well-factored; duplication concentrated in tri-lingual bindings + render boilerplate.

---

## 3. Performance — severity: ONE SYSTEMIC HIGH-IMPACT BUG

| file:line | issue | impact | fix |
|---|---|---|---|
| core/src/astronomy/engine.rs:74,268 | `compute_speed` recomputes jde±0.5 without reusing central | hot | share central-diff neighbors |
| core/src/functions/moon_phases.rs:162-178 | `bisect_phase` Newton: ~6 calc_ut ×15 iters via finite-diff | hot | analytic ~12.19°/day derivative |
| core/src/functions/moon_phases.rs:181 | post-loop recomputes elongation already known | warm | reuse computed value |
| cli/src/cmd/chart.rs:389-462 | `push_str(&format!())` in 360°/sign/cusp/aspect loops | warm | `write!`/`writeln!` into buffer |
| core/src/astronomy/planetary.rs:36-63 | `apparent_planet` ~4-5 heliocentric evals ×3 under SPEED scans | hot | cache Earth heliocentric per jde |

---

## 4. Security — severity: MODERATE (memory-safe, no `unsafe`, FFI clean)

### HIGH — unescaped user input → SVG/script injection
| file:line | issue | fix |
|---|---|---|
| cli/src/cmd/render/builtin_svg.rs:180 | `--var title`/TOML `[vars]` raw into SVG `<text>` | XML-escape all `vars` values |
| cli/src/cmd/render/builtin_svg.rs:178-180 | palette strings raw into SVG attrs (attr breakout) | XML-attr-escape palette |
| cli/src/cmd/chart.rs:567 | `--name` verbatim into `<text>` | XML-escape `name` |

### MEDIUM
| file:line | issue | fix |
|---|---|---|
| cli/Cargo.toml `toml = "=0.4.10"` | 2019 unmaintained TOML parser on untrusted `--config` | upgrade to `toml` 0.8.x |
| cli/src/cmd/chart.rs:719 / render mod.rs:939 | `--out`/config `out=` no traversal/abs-path check → arbitrary write | reject `..`/abs or confine to base dir |
| cli/src/cmd/render/mod.rs:911-923 | MiniJinja env no fuel/recursion sandbox on `--template` → DoS | set fuel limit, strict undefined |
| cli/src/cmd/render/builtin_svg.rs:664 | `&aname[..len.min(4)]` non-char-boundary slice → panic | char-aware truncation |

### LOW
parse.rs:108 fragile `offsets[0]` (safe by arm order); time.rs:61-104 unbounded `f64→i64 as` saturates silently; fuzz/src/main.rs:52 `% (hi-lo)` panics if hi==lo (test-only).

Notes: no `unsafe` in core/cli/bindings; FFI (napi/pyo3/ext-php-rs) clean; no command injection in plugin.rs (PATH exec, no shell); untrusted CLI parsing uses checked `parse()`.

---

## 5. Architecture

| area | problem | improvement |
|---|---|---|
| cli/src/cmd/render/mod.rs (3541 LOC) | god file: args, config, tz, 28 dispatch wrappers, registry, template, schema, tests | split into `args.rs`/`config.rs`/`pipeline.rs`/`registry.rs`; thin facade |
| CLI error handling | all `Result<_,String>` + `.map_err(e.to_string())`, loses typed core::Error | one `thiserror CliError` (`Parse/Config/Compute(#[from])/Io`); String only at `main` |
| core domain vs functions/ | `moon.rs`/`houses.rs` are `pub use functions::*` shims; impl is `pub(crate)` | collapse shims or document `functions/` as canonical private layer |
| core/src/lib.rs | 23 glob `pub use *::*` flatten whole API surface | curate explicit re-exports; lean on `prelude` |
| bindings js/python/php | 3×~2.5k LOC mirror same ~150 fns + per-lang error shims | extract `celestial-ffi` facade crate; bindings = thin marshalling |
| parse.rs | `parse_date/tz/body/hsys/sid_mode` → `Result<_,String>`, no shared error | `TryFrom`/`FromStr` on newtypes + one `ParseError` |
| context build vs render | `dispatch_*` returns `(serde_json::Value, fn)`; schema implicit, test-enforced only | typed `ChartContext`; serialize to JSON only at template boundary |
| testability | `run()` does IO inline; logic only via full CLI path | `build_output(args)->Result<String,CliError>` pure; IO stays in `run` |

---

## 6. Design pattern opportunities

| location | current | pattern | payoff |
|---|---|---|---|
| `CHART_REGISTRY` + 28 `dispatch_*` | hand-written near-identical wrappers | Strategy trait or declarative macro from `(aliases,title,build,render)` | −~400 LOC; new type = 1 row |
| `jd/lat/lon: f64`, `offset/24.0`, `hsys: u8` positional | primitive obsession; unguarded arithmetic | newtypes `JulianDay`/`Degrees`/`UtcOffset`/`HouseSystem` | compiler-enforced units; tz/JD becomes method |
| parse.rs `parse_*` | ad-hoc `fn(&str)->Result<_,String>` | `FromStr`/`TryFrom` for newtypes | composes with clap `value_parser`, one error type |
| `dispatch_*` arg threading (`jd,&args,&vars` ×28) | long positional rebuilt per wrapper | `ChartContextBuilder` | one construction path |
| `fn(&Value)->String` + JSON traversal | stringly-typed every renderer | typed `ChartContext` + serde (+Visitor for overlays) | schema compile-checked; `context_schema.txt` generated |
| core `Error` (5 legacy String variants) | `Calc(String)` catch-alls dominate | finish structured-variant migration + thiserror | programmatic error handling in bindings |
| `to_napi`/`to_py`/`to_php` shims | same conversion ×3 | `From<core::Error>` in shared ffi crate | one site, consistent messages |

---

## Recommended order

1. **HIGH security** — XML-escape user-controlled SVG values (small, urgent).
2. **Perf** — strip SPEED flag in scan loops (big win, low risk).
3. **Architecture** — split `render/mod.rs` + introduce `CliError`; unlocks registry-as-Strategy, typed `ChartContext`, testable `build_output` seam.
