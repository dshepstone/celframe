import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["**/dist/**", "**/target/**", "**/storybook-static/**"] },
  js.configs.recommended,
  ...tseslint.configs.strict,
);
