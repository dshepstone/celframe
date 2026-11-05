import { themes, type ThemeName } from "@celframe/ui";
import { useEffect, useRef, useState, type RefObject } from "react";
import { backend } from "./backend";

/** "#rrggbb" to opaque RGBA bytes. */
export function hexToRgba(hex: string): [number, number, number, number] {
  const value = Number.parseInt(hex.replace("#", ""), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255, 255];
}

/**
 * Keeps the native canvas drawing under the element, and reports whether it does. Until it
 * confirms, the element should stay opaque so an undrawn window never shows through.
 */
export function useNativeCanvas<T extends HTMLElement>(
  theme: ThemeName,
): [RefObject<T | null>, boolean] {
  const ref = useRef<T>(null);
  const [active, setActive] = useState(false);

  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const backdrop = hexToRgba(themes[theme]["canvas-backdrop"]);
    const send = () => {
      const rect = element.getBoundingClientRect();
      backend
        .setCanvasLayout({
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
          scale: window.devicePixelRatio,
          backdrop,
        })
        .then(setActive, () => setActive(false));
    };
    send();
    // The side panel changes the canvas size, and moving to another display changes the scale.
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(send);
    observer?.observe(element);
    window.addEventListener("resize", send);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", send);
    };
  }, [theme]);

  return [ref, active];
}
