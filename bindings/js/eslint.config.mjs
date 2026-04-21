// eslint.config.mjs — ESLint configuration for celestial-js
// Focus: real bugs (no-undef, no-unused-vars, etc.)
// Formatting rules are intentionally excluded.
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
      // pure_logic.test.mjs runs in Node.js — add standard Node globals
      globals: {
        console: "readonly",
        process: "readonly",
        Buffer: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
      },
    },
    rules: {
      "no-undef": "error",
      "no-unreachable": "error",
      "no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      "no-constant-condition": "error",
      "no-duplicate-case": "error",
      "use-isnan": "error",
      "no-console": "off",
    },
  },

  // ── TypeScript files (.ts) ────────────────────────────────────────────────
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
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-require-imports": "off", // native addon needs require()
      "no-unreachable": "error",
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },
);
