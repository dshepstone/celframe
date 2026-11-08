import type { PointerEvent as ReactPointerEvent } from "react";
import { useApp } from "../store";

/** The frame of the `[data-scrub-frame]` element under a point, if any. */
function frameAt(x: number, y: number): number | null {
  for (const element of document.elementsFromPoint?.(x, y) ?? []) {
    const host = element.closest<HTMLElement>("[data-scrub-frame]");
    const frame = Number(host?.dataset.scrubFrame);
    if (host && Number.isInteger(frame)) return frame;
  }
  return null;
}

/**
 * Starts scrubbing from a press on a frame number: the current frame follows the pointer
 * along `axis` until release. Frame numbers mark themselves with `data-scrub-frame`.
 * See docs/specs/timing.md.
 */
export function startScrub(event: ReactPointerEvent, axis: "x" | "y") {
  if (event.button !== 0) return;
  const start = (event.target as Element).closest<HTMLElement>(
    "[data-scrub-frame]",
  );
  if (!start) return;
  event.preventDefault();
  const state = useApp.getState();
  state.stopPlayback();
  // The other axis stays on the row or column of frame numbers while the pointer wanders.
  const box = start.getBoundingClientRect();
  const fixed =
    axis === "x" ? box.top + box.height / 2 : box.left + box.width / 2;
  let last = Number(start.dataset.scrubFrame);
  void state.setCurrentFrame(last);

  const move = (e: PointerEvent) => {
    const frame =
      axis === "x" ? frameAt(e.clientX, fixed) : frameAt(fixed, e.clientY);
    if (frame !== null && frame !== last) {
      last = frame;
      void useApp.getState().setCurrentFrame(frame);
    }
  };
  const stop = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", stop);
    window.removeEventListener("pointercancel", stop);
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", stop);
  window.addEventListener("pointercancel", stop);
}
