// eslint.config.mjs — ESLint configuration for celestial-js
// Targets: tests/*.mjs (pure-logic runner) and tests/*.ts (integration tests)
// Focus: real bugs (no-undef, no-unused-vars, no-unreachable, etc.)
// Formatting rules are intentionally excluded (prettier handles formatting,
// and we don't enforce it in CI).
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  // ── JavaScript / ESM files (.mjs) ────────────────────────────────────────
  {
    files: ["tests/**/*.mjs"],
    ...js.configs.recommended,
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
    },
    rules: {
      // Bugs
      "no-undef":           "error",
      "no-unreachable":     "error",
      "no-unused-vars":     ["error", { argsIgnorePattern: "^_" }],
      "no-constant-condition": "error",
      "no-duplicate-case":  "error",
      "use-isnan":          "error",
      // Style rules intentionally off — we only want bug-catching
      "no-console":         "off",
    },
  },

  // ── TypeScript files (.ts) ────────────────────────────────────────────────
  // tsc handles type safety; eslint adds logic-level checks on top.
  {
    files: ["tests/**/*.ts"],
    extends: tseslint.configs.recommendedTypeChecked,
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        project: "./tsconfig.json",
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // Keep only rules that catch real bugs, not style
      "@typescript-eslint/no-explicit-any":    "off",  // native addon returns any
      "@typescript-eslint/no-floating-promises":"error",
      "@typescript-eslint/no-unsafe-assignment":"off",  // addon interop
      "@typescript-eslint/no-unsafe-call":     "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "no-unreachable":                        "error",
      "no-unused-vars":                        "off",   // tsc catches this
      "@typescript-eslint/no-unused-vars":     ["error", { argsIgnorePattern: "^_" }],
    },
  }
);
