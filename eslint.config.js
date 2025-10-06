import js from "@eslint/js";
import { tanstackConfig } from "@tanstack/eslint-config";
import pluginImportX from "eslint-plugin-import-x";
import pnpmPlugin from "eslint-plugin-pnpm";
import * as postCssModules from "eslint-plugin-postcss-modules";
import * as reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import jsoncParser from "jsonc-eslint-parser";
import * as tseslint from "typescript-eslint";
import yamlParser from "yaml-eslint-parser";

export default tseslint.config(
  ...tanstackConfig,
  {
    ignores: ["dist", "coverage", "*.bundled_*.mjs", "eslint.config.js"],
  },
  {
    files: ["**/*.{js,ts,jsx,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        project: [
          "./packages/*/tsconfig.json",
          "./packages/*/tsconfig.*.json",
          "!./packages/tsconfig/tsconfig.*.json",
        ],
      },
    },
    plugins: {
      "postcss-modules": postCssModules,
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
      "import": pluginImportX,
    },
    extends: [
      js.configs.recommended,
      tseslint.configs.strict,
      tseslint.configs.stylistic,
    ],
    linterOptions: {
      reportUnusedDisableDirectives: "error",
    },
    settings: {
      "import/resolver": ["typescript"],
      "postcss-modules": {
        camelCase: true,
        include: "**/*.module.css",
      },
    },
    rules: {
      // Import rules
      "import/consistent-type-specifier-style": ["warn", "prefer-top-level"],
      "import/first": ["warn"],
      "import/no-anonymous-default-export": ["error", {
        allowArray: true,
        allowObject: true,
      }],
      "import/order": ["warn", {
        "groups": ["type", ["builtin", "external", "internal"], ["parent", "sibling", "index"]],
        "newlines-between": "always",
      }],
      "sort-imports": "off",
      "no-restricted-imports": ["error", "node:test"],
      // Redundant with TypeScript
      "import/no-unresolved": "off",
      // Typescript
      "@typescript-eslint/consistent-type-definitions": ["error", "type"],
      "@typescript-eslint/consistent-type-imports": ["warn"],
      "@typescript-eslint/no-unused-vars": ["warn", {
        varsIgnorePattern: "^_",
        argsIgnorePattern: "^_",
      }],
      "@typescript-eslint/array-type": ["error", { default: "array-simple" }],
      // PostCSS
      "postcss-modules/no-undef-class": "error",
      "postcss-modules/no-unused-class": "warn",
      // React
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "react-hooks/rules-of-hooks": ["error"],
      "react-hooks/exhaustive-deps": ["warn"],
    },
  },
  {
    files: ["**/*.test.{js,jsx,ts,tsx}"],
    rules: {
      "postcss-modules/no-unused-class": "off",
    },
  },
  {
    files: ["package.json", "**/package.json"],
    languageOptions: {
      parser: jsoncParser,
    },
    plugins: {
      pnpm: pnpmPlugin,
    },
    rules: {
      "pnpm/json-enforce-catalog": "off",
      "pnpm/json-valid-catalog": "error",
      "pnpm/json-prefer-workspace-settings": "error",
    },
  },
  {
    files: ["pnpm-workspace.yaml"],
    languageOptions: {
      parser: yamlParser,
    },
    plugins: {
      pnpm: pnpmPlugin,
    },
    rules: {
      "pnpm/yaml-no-unused-catalog-item": "error",
      "pnpm/yaml-no-duplicate-catalog-item": "error",
    },
  },
);
