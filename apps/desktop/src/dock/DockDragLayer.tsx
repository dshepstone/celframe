import { useEffect } from "react";
import { useApp } from "../store";
import type { AreaId, DropTarget } from "./layout";
import { PANEL_INFO } from "./panels";

/** The drop target under a point, from the elements there, topmost first. */
export function dropTargetAt(
  x: number,
  y: number,
  elements: readonly Element[],
): DropTarget | null {
  for (const element of elements) {
    const host = element.closest<HTMLElement>(
      "[data-dock-tabs], [data-dock-body], [data-dock-area-drop]",
    );
    if (!host) continue;
    const area = host.dataset.area as AreaId;
    const stack = host.dataset.stack ?? "";
    if (host.hasAttribute("data-dock-area-drop")) return { kind: "area", area };
    if (host.hasAttribute("data-dock-tabs")) {
      const tabs = [...host.querySelectorAll("[data-dock-tab]")];
      const index = tabs.filter((tab) => {
        const rect = tab.getBoundingClientRect();
        return rect.left + rect.width / 2 < x;
      }).length;
      return { kind: "tabs", area, stack, index };
    }
    const rect = host.getBoundingClientRect();
    const after =
      area === "bottom"
        ? x > rect.left + rect.width / 2
        : y > rect.top + rect.height / 2;
    return { kind: "split", area, stack, after };
  }
  return null;
}

/** Follows a dragged tab: shows it at the pointer, finds the target, and drops or cancels. */
export function DockDragLayer() {
  const { dockDrag, setDockDrag, moveDockTab } = useApp();
  const dragging = dockDrag !== null;

  useEffect(() => {
    if (!dragging) return;
    const move = (e: PointerEvent) => {
      const drag = useApp.getState().dockDrag;
      if (!drag) return;
      const elements = document.elementsFromPoint?.(e.clientX, e.clientY) ?? [];
      setDockDrag({
        ...drag,
        x: e.clientX,
        y: e.clientY,
        target: dropTargetAt(e.clientX, e.clientY, elements),
      });
    };
    const drop = () => {
      const drag = useApp.getState().dockDrag;
      setDockDrag(null);
      if (drag?.target) moveDockTab(drag.panel, drag.target);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        setDockDrag(null);
      }
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", drop);
    const cancel = () => setDockDrag(null);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("keydown", key, true);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", drop);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("keydown", key, true);
    };
  }, [dragging, setDockDrag, moveDockTab]);

  if (!dockDrag) return null;
  return (
    <div
      aria-hidden="true"
      style={{ left: dockDrag.x + 12, top: dockDrag.y + 8 }}
      className="pointer-events-none fixed z-50 rounded border border-accent bg-surface-raised px-2.5 py-1 text-2xs font-semibold tracking-[0.06em] text-text uppercase shadow-2xl"
    >
      {PANEL_INFO[dockDrag.panel].title}
    </div>
  );
}
