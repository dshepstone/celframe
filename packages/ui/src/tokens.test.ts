import { describe, expect, it } from "vitest";
import {
  applyTheme,
  colorTokenNames,
  contrastRatio,
  controlPairs,
  textPairs,
  themes,
  type ThemeName,
} from "./tokens";

const themeNames: ThemeName[] = ["light", "dark"];

describe("contrastRatio", () => {
  it("matches known values", () => {
    expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(contrastRatio("#ffffff", "#ffffff")).toBeCloseTo(1, 5);
    expect(contrastRatio("#ffffff", "#000000")).toBeCloseTo(21, 5);
  });
});

describe.each(themeNames)("%s theme", (name) => {
  const theme = themes[name];

  it("defines every token as #rrggbb", () => {
    for (const token of colorTokenNames) {
      expect(theme[token]).toMatch(/^#[0-9a-f]{6}$/);
    }
  });

  it.each(textPairs)("%s on %s meets AA for text (4.5:1)", (fg, bg) => {
    expect(contrastRatio(theme[fg], theme[bg])).toBeGreaterThanOrEqual(4.5);
  });

  it.each(controlPairs)("%s on %s meets AA for controls (3:1)", (fg, bg) => {
    expect(contrastRatio(theme[fg], theme[bg])).toBeGreaterThanOrEqual(3);
  });
});

describe("applyTheme", () => {
  it("sets every token and records the theme", () => {
    const set = new Map<string, string>();
    const dataset: Record<string, string | undefined> = {};
    applyTheme("dark", {
      style: { setProperty: (key, value) => void set.set(key, value) },
      dataset,
    });
    expect(set.get("--color-accent")).toBe(themes.dark.accent);
    expect(set.size).toBe(colorTokenNames.length);
    expect(dataset.theme).toBe("dark");
  });
});
