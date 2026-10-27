import js from "@eslint/js";
import tseslint from "typescript-eslint";
import hooks from "eslint-plugin-react-hooks";
export default tseslint.config(
  {
    ignores: [
      "dist",
      "node_modules",
      "playwright-report",
      "test-results",
      "tmp",
      "src-tauri/target",
      "src-tauri/target-*",
      "src-tauri/gen",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["scripts/admin/*.mjs"],
    languageOptions: {
      globals: {
        Buffer: "readonly",
        console: "readonly",
        process: "readonly",
      },
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    plugins: { "react-hooks": hooks },
    rules: hooks.configs.recommended.rules,
  },
);
