import { useEffect, type RefObject } from "react";
import { backend, type PointerSample } from "./backend";
import { useApp } from "./store";

function toSample(event: PointerEvent): PointerSample {
  // A mouse has no pressure. Touch that reports none uses full pressure too.
  const pressure =
    event.pointerType === "mouse" || event.pressure <= 0 ? 1 : event.pressure;
  return { x: event.clientX, y: event.clientY, pressure };
}

/**
 * Sends pen, mouse and touch strokes drawn on the element to the core. Samples are batched to
 * at most one call per displayed frame. See docs/specs/pen-input.md.
 */
export function usePenInput<T extends HTMLElement>(
  ref: RefObject<T | null>,
  enabled: boolean,
): void {
  const finishStroke = useApp((state) => state.finishStroke);
  const reportError = useApp((state) => state.reportError);

  useEffect(() => {
    const element = ref.current;
    if (!element || !enabled) return;
    let pointer: number | null = null;
    let queue: PointerSample[] = [];
    let frame = 0;

    const flush = () => {
      frame = 0;
      if (queue.length === 0) return;
      const batch = queue;
      queue = [];
      backend.penMove(batch).catch(reportError);
    };
    const down = (event: PointerEvent) => {
      if (pointer !== null) return;
      if (event.pointerType === "mouse" && event.button !== 0) return;
      pointer = event.pointerId;
      element.setPointerCapture?.(event.pointerId);
      event.preventDefault();
      backend.penDown(toSample(event)).catch(reportError);
    };
    const move = (event: PointerEvent) => {
      if (event.pointerId !== pointer) return;
      // A pen reports more samples than there are frames. Keep them all.
      const events = event.getCoalescedEvents?.() ?? [];
      queue.push(...(events.length > 0 ? events : [event]).map(toSample));
      if (frame === 0) frame = requestAnimationFrame(flush);
    };
    const end = () => {
      if (frame !== 0) cancelAnimationFrame(frame);
      flush();
      void finishStroke();
    };
    const up = (event: PointerEvent) => {
      if (event.pointerId !== pointer) return;
      pointer = null;
      end();
    };

    element.addEventListener("pointerdown", down);
    element.addEventListener("pointermove", move);
    element.addEventListener("pointerup", up);
    element.addEventListener("pointercancel", up);
    return () => {
      element.removeEventListener("pointerdown", down);
      element.removeEventListener("pointermove", move);
      element.removeEventListener("pointerup", up);
      element.removeEventListener("pointercancel", up);
      if (pointer !== null) end();
    };
  }, [ref, enabled, finishStroke, reportError]);
}
