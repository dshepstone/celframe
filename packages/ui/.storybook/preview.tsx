import type { Preview } from "@storybook/react-vite";
import { useEffect } from "react";
import "../src/styles.css";
import { applyTheme, type ThemeName } from "../src/tokens";

const preview: Preview = {
  globalTypes: {
    theme: {
      description: "Theme",
      toolbar: { items: ["light", "dark"], dynamicTitle: true },
    },
  },
  initialGlobals: { theme: "light" },
  decorators: [
    (Story, context) => {
      const theme = context.globals.theme as ThemeName;
      useEffect(() => {
        applyTheme(theme, document.documentElement);
        document.body.style.background = "var(--color-surface)";
      }, [theme]);
      return <Story />;
    },
  ],
};

export default preview;
