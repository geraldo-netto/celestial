# Celestial — project rules

## Code complexity

- Do NOT write any function with cyclomatic complexity > 10. Applies to
  all code, **including tests**. No exceptions.
- If logic needs more, split into helpers, table-drive it, or restructure
  until each function is ≤ 10.
- Flat lookup `match` (one arm = one mapping, no nested logic) is exempt:
  high arm count, no real path branching.
