import { useEffect, type RefObject } from "react";
import { useApp } from "./store";

/** How much one wheel notch (100 units of deltaY) zooms. */
const WHEEL_ZOOM_PER_100 = 1.15;

/**
 * Zoom, pan and rotate on the canvas. The wheel zooms at the pointer. The middle button, the
 * Pan tool, or holding Space while dragging pans. The Zoom tool zooms in on a click, and out
 * with Alt. See docs/specs/canvas-navigation.md.
 */
export function useCanvasNavigation<T extends HTMLElement>(
  ref: RefObject<T | null>,
  enabled: boolean,
): void {
  const tool = useApp((state) => state.tool);
  const spaceHeld = useApp((state) => state.spaceHeld);
  const changeView = useApp((state) => state.changeView);

  useEffect(() => {
    const element = ref.current;
    if (!element || !enabled) return;
    let pan: { pointer: number; x: number; y: number } | null = null;
    let pending = { dx: 0, dy: 0 };
    let frame = 0;

    const flush = () => {
      frame = 0;
      const { dx, dy } = pending;
      pending = { dx: 0, dy: 0 };
      if (dx !== 0 || dy !== 0) void changeView({ type: "pan", dx, dy });
    };
    const wheel = (event: WheelEvent) => {
      event.preventDefault();
      // Lines and pages scroll further per event than pixels do.
      const scale =
        event.deltaMode === 1 ? 33 : event.deltaMode === 2 ? 400 : 1;
      const factor = WHEEL_ZOOM_PER_100 ** ((-event.deltaY * scale) / 100);
      void changeView({
        type: "zoom",
        factor,
        anchor: { x: event.clientX, y: event.clientY },
      });
    };
    const down = (event: PointerEvent) => {
      const pans =
        event.button === 1 ||
        (event.button === 0 && (tool === "hand" || spaceHeld));
      if (pans && pan === null) {
        event.preventDefault();
        pan = { pointer: event.pointerId, x: event.clientX, y: event.clientY };
        element.setPointerCapture?.(event.pointerId);
      } else if (event.button === 0 && tool === "zoom" && !spaceHeld) {
        event.preventDefault();
        void changeView({
          type: "zoom",
          factor: event.altKey ? 0.5 : 2,
          anchor: { x: event.clientX, y: event.clientY },
        });
      }
    };
    const move = (event: PointerEvent) => {
      if (pan === null || event.pointerId !== pan.pointer) return;
      pending.dx += event.clientX - pan.x;
      pending.dy += event.clientY - pan.y;
      pan.x = event.clientX;
      pan.y = event.clientY;
      if (frame === 0) frame = requestAnimationFrame(flush);
    };
    const up = (event: PointerEvent) => {
      if (pan === null || event.pointerId !== pan.pointer) return;
      pan = null;
      if (frame !== 0) cancelAnimationFrame(frame);
      flush();
    };
    // The middle button would otherwise start the browser's auto-scroll.
    const auxDown = (event: MouseEvent) => {
      if (event.button === 1) event.preventDefault();
    };

    element.addEventListener("wheel", wheel, { passive: false });
    element.addEventListener("pointerdown", down);
    element.addEventListener("pointermove", move);
    element.addEventListener("pointerup", up);
    element.addEventListener("pointercancel", up);
    element.addEventListener("mousedown", auxDown);
    return () => {
      if (frame !== 0) cancelAnimationFrame(frame);
      element.removeEventListener("wheel", wheel);
      element.removeEventListener("pointerdown", down);
      element.removeEventListener("pointermove", move);
      element.removeEventListener("pointerup", up);
      element.removeEventListener("pointercancel", up);
      element.removeEventListener("mousedown", auxDown);
    };
  }, [ref, enabled, tool, spaceHeld, changeView]);
}
