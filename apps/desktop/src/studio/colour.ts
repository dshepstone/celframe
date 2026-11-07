import type { Rgba } from "../store";

export function rgbaToHex([r, g, b]: Rgba): string {
  return `#${[r, g, b].map((c) => c.toString(16).padStart(2, "0")).join("")}`;
}

export function hexToRgbaColour(hex: string): Rgba {
  const value = Number.parseInt(hex.replace("#", ""), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255, 255];
}

/** Starter swatches: inks, skin and hair bases, and a few accents. */
export const SWATCHES: ReadonlyArray<{ name: string; hex: string }> = [
  { name: "Ink", hex: "#16130f" },
  { name: "Graphite", hex: "#4a4740" },
  { name: "Line blue", hex: "#2a5bd7" },
  { name: "Line red", hex: "#c8322a" },
  { name: "Line green", hex: "#2f8a4c" },
  { name: "Paper", hex: "#ffffff" },
  { name: "Skin light", hex: "#f2c9a8" },
  { name: "Skin deep", hex: "#8a5a3c" },
  { name: "Hair", hex: "#3b2a1e" },
  { name: "Sky", hex: "#8cc4e8" },
  { name: "Leaf", hex: "#7fb069" },
  { name: "Sun", hex: "#f4c542" },
];
