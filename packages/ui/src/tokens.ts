// Design tokens: the single source of colours and spacing for the Celframe UI.

export type ThemeName = "light" | "dark";

export const colorTokenNames = [
  "surface",
  "surface-raised",
  "surface-sunken",
  "surface-hover",
  "canvas-backdrop",
  "text",
  "text-muted",
  "text-on-accent",
  "accent",
  "accent-hover",
  "accent-muted",
  "border",
  "control-border",
  "focus-ring",
  "danger",
] as const;

export type ColorToken = (typeof colorTokenNames)[number];

/** Graphite neutrals with one teal accent. See docs/specs/visual-design.md. */
export const themes: Record<ThemeName, Record<ColorToken, string>> = {
  light: {
    surface: "#e8e9eb",
    "surface-raised": "#f3f4f5",
    "surface-sunken": "#dadcdf",
    "surface-hover": "#dee0e3",
    "canvas-backdrop": "#b9bcc1",
    text: "#1a1c1f",
    "text-muted": "#50555c",
    "text-on-accent": "#ffffff",
    accent: "#0d6b65",
    "accent-hover": "#0a5752",
    "accent-muted": "#c9e2df",
    border: "#c4c7cc",
    "control-border": "#767b84",
    "focus-ring": "#983a0a",
    danger: "#a4241c",
  },
  dark: {
    surface: "#232529",
    "surface-raised": "#2a2d32",
    "surface-sunken": "#1b1d20",
    "surface-hover": "#32363c",
    "canvas-backdrop": "#3a3d43",
    text: "#e4e6e9",
    "text-muted": "#a0a5ad",
    "text-on-accent": "#0a1716",
    accent: "#43b2aa",
    "accent-hover": "#5cc4bc",
    "accent-muted": "#1e3b3a",
    border: "#16181a",
    "control-border": "#80858e",
    "focus-ring": "#e8a05c",
    danger: "#ff8f86",
  },
};

export const spacing = {
  xs: "0.25rem",
  sm: "0.5rem",
  md: "0.75rem",
  lg: "1rem",
  xl: "1.5rem",
} as const;

export const radius = { sm: "3px", md: "4px", lg: "6px" } as const;

/** Text and its background, which WCAG AA requires to reach 4.5:1. */
export const textPairs: ReadonlyArray<readonly [ColorToken, ColorToken]> = [
  ["text", "surface"],
  ["text", "surface-raised"],
  ["text", "surface-sunken"],
  ["text", "surface-hover"],
  ["text", "accent-muted"],
  ["text-muted", "surface"],
  ["text-muted", "surface-raised"],
  ["text-muted", "surface-sunken"],
  ["text-muted", "surface-hover"],
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
  ["accent", "accent-muted"],
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
