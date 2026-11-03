// Design tokens: the single source of colours and spacing for the Celframe UI.

export type ThemeName = "light" | "dark";

export const colorTokenNames = [
  "surface",
  "surface-raised",
  "surface-sunken",
  "canvas-backdrop",
  "text",
  "text-muted",
  "text-on-accent",
  "accent",
  "accent-hover",
  "border",
  "control-border",
  "focus-ring",
  "danger",
] as const;

export type ColorToken = (typeof colorTokenNames)[number];

export const themes: Record<ThemeName, Record<ColorToken, string>> = {
  light: {
    surface: "#f6f3ec",
    "surface-raised": "#fffdf8",
    "surface-sunken": "#ebe6db",
    "canvas-backdrop": "#d9d4c8",
    text: "#1f1c17",
    "text-muted": "#5c564b",
    "text-on-accent": "#ffffff",
    accent: "#0f6b63",
    "accent-hover": "#0b5650",
    border: "#d3ccbd",
    "control-border": "#857d6e",
    "focus-ring": "#b5400e",
    danger: "#a8201a",
  },
  dark: {
    surface: "#1b1d21",
    "surface-raised": "#25282d",
    "surface-sunken": "#141619",
    "canvas-backdrop": "#0f1012",
    text: "#f1eee7",
    "text-muted": "#b3aea3",
    "text-on-accent": "#08201e",
    accent: "#4fc4b8",
    "accent-hover": "#74d6cc",
    border: "#373b42",
    "control-border": "#7f838b",
    "focus-ring": "#ff9e6b",
    danger: "#ff8a80",
  },
};

export const spacing = {
  xs: "0.25rem",
  sm: "0.5rem",
  md: "0.75rem",
  lg: "1rem",
  xl: "1.5rem",
} as const;

export const radius = { sm: "0.25rem", md: "0.5rem", lg: "0.875rem" } as const;

/** Text and its background, which WCAG AA requires to reach 4.5:1. */
export const textPairs: ReadonlyArray<readonly [ColorToken, ColorToken]> = [
  ["text", "surface"],
  ["text", "surface-raised"],
  ["text", "surface-sunken"],
  ["text-muted", "surface"],
  ["text-muted", "surface-raised"],
  ["text-muted", "surface-sunken"],
  ["text-on-accent", "accent"],
  ["text-on-accent", "accent-hover"],
  ["danger", "surface"],
  ["danger", "surface-raised"],
];

/** Controls and focus indicators, which WCAG AA requires to reach 3:1. */
export const controlPairs: ReadonlyArray<readonly [ColorToken, ColorToken]> = [
  ["control-border", "surface"],
  ["control-border", "surface-raised"],
  ["control-border", "surface-sunken"],
  ["accent", "surface"],
  ["accent", "surface-raised"],
  ["accent", "surface-sunken"],
  ["focus-ring", "surface"],
  ["focus-ring", "surface-raised"],
  ["focus-ring", "surface-sunken"],
  ["focus-ring", "canvas-backdrop"],
];

function channelLuminance(value: number): number {
  const c = value / 255;
  return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const n = Number.parseInt(hex.slice(1), 16);
  return (
    0.2126 * channelLuminance((n >> 16) & 255) +
    0.7152 * channelLuminance((n >> 8) & 255) +
    0.0722 * channelLuminance(n & 255)
  );
}

/** WCAG contrast ratio of two `#rrggbb` colours, from 1 to 21. */
export function contrastRatio(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [
    number,
    number,
  ];
  return (hi + 0.05) / (lo + 0.05);
}

/** Applies a theme by setting CSS custom properties on an element. */
export function applyTheme(
  theme: ThemeName,
  root: { style: { setProperty(name: string, value: string): void } } & {
    dataset: Record<string, string | undefined>;
  },
): void {
  for (const name of colorTokenNames) {
    root.style.setProperty(`--color-${name}`, themes[theme][name]);
  }
  root.dataset.theme = theme;
}
