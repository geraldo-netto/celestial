# Agents Behavior Guide

File define expected behavior + usage model for AI agents in this repo.
Primary source for agent conduct, editing norms, response expectations — follow
behavior defined here when interacting with this workspace.

## Purpose

- Standard guidelines for agent interactions.
- Consistent behavior when using AI tooling in workspace.

## General Agent Behavior

- Prefer short, actionable responses. Terse fine; fragments OK.
- Respect workspace context. No guessing when info missing.
- Code changes: describe what changed + why.
- File edits: include exact context around replacements to avoid ambiguity.

## Rules

- No assume. No hide confusion. Surface tradeoffs, ask user when unclear.
- Write minimum code that solve problem. No speculative or unneeded changes.
- Touch only what must. Clean own mess only. Leave workspace cleaner than found.
- Define success criteria before changes. Verify + iterate until satisfied.
- Keep code complexity <= 10 for any function, class, method. All code,
  **including tests**. No exceptions. Logic need more → split helpers, table-drive,
  or restructure until each function <= 10. Flat lookup `match` (one arm = one mapping, no
  nested logic) exempt: high arm count, no real path branching.
- No code duplication. Apply SOLID only when improve clarity or structure.
- Default no comments. Add only when WHY non-obvious (hidden constraint, subtle
  invariant, workaround for specific bug). Record assumptions/design intent in commit notes,
  not inline.
- Prefer explicit, maintainable solutions over clever shortcuts.
- Propose business/design patterns + DDD only when improve clarity or structure.
- Bindings must not drift from `celestial-core`. `core` = source of truth, not
  bindings. After ANY change to core public fn, binding signature, binding
  constant, or doc that restates them, run binding gates + keep green:
  - `cargo xtask parity` — three bindings export same fn names.
  - `cargo xtask coverage` — every core flat public fn (`pub use` in `core/src/lib.rs`)
    bound in all three languages OR listed in `xtask/core_unbound_allow.txt` with
    reason; cross-binding arities agree or recorded in `xtask/arity_allow.txt`.
    Adding core fn without deciding "bind it or allow-list it" = drift we forbid.
  - `cargo xtask shapes --check` — binding return KIND (scalar/tuple/array/object)
    unchanged; real change must update `xtask/binding_shapes.txt` in same commit.
  - `cargo xtask apidoc --check` — `docs/generated/binding_api.md` (constant values +
    per-language signatures) matches source. Hand-written language guides in `docs/*.md`
    must LINK to generated file for constants/signatures, never restate numbers
    or shapes (exactly how DOC-9/DOC-10 drifted). Prose-only explanation fine.
  - `cargo xtask pyi --check` / `cargo xtask dts --check` — stubs in sync.
  Run in `binding-parity` CI pipeline; red gate blocks merge.
- ALWAYS record review findings in `TODO.md` — never report only in chat. Any time
  scan, review, audit, or "look for issues" (not just major changes), add each finding to
  matching category table in `TODO.md` before/while reporting.
- ALWAYS remove completed items from `TODO.md` — once finding implemented + tested + merged,
  delete row from table outright. No "shipped" sub-sections, no struck-through entries.
  `git log` = durable record. Exceptions: "Open — parked" section keeps open-but-deferred
  items with why-not-now annotation; "Audit picks deliberately rejected" section keeps
  rationale so future passes don't re-pick same items.
- Major changes: rescan whole project, create/update `TODO.md` with one table per review category.
  Each table use format: `id | status | effort | description | notes`.
  - adaptability
  - architecture/modularity/SOLID
  - business/design patterns/DDD
  - CLI / option integrity
  - code complexity
  - code duplication
  - composition
  - concurrency
  - configuration discoverability
  - data structure
  - decoupling
  - dependency
  - design thinking
  - documentation
  - legacy / deprecation
  - multithreading
  - observability when application has it
  - okr
  - pdca
  - performance
  - platform
  - plugin extensibility
  - product engineering
  - purpose
  - reliability/correctness
  - robustness / recovery
  - scalability
  - security
  - state machine integrity
  - UI / UX
  - vectorization
  - wiring gaps — modules/helpers/cfg knobs that exist + pass tests but no real production call site (orphan exports, cfg flags never read, advertised backends not wired in). Shipped feature only "shipped" when dispatcher actually invokes it.
  - unused functions/methods — public-shaped callables (no leading `_`) imported by no production code, no tests, no plugins. Different from wiring gaps: not half-wired, fully dead. Includes `__init__.py` re-exports no caller pulls + class methods only ever called from one private site. Each finding: keep / inline / delete decision recorded in `notes`.

## File Editing

- No overwrite existing files unless user explicitly asks or file missing.
- Text edits: preserve surrounding context, keep modifications minimal.
- Use repo-specific structure + conventions when adding or updating files.

## Communications

- Use headings + bullets for readability.
- Highlight changed files + key points.
- Keep final answers brief + professional.