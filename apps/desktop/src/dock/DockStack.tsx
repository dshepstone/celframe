import type { KeyboardEvent, PointerEvent as ReactPointerEvent } from "react";
import { useApp } from "../store";
import type { AreaId, PanelId, Stack } from "./layout";
import { PANEL_INFO } from "./panels";

/** Dragging starts once the pointer has moved this far, so clicks still select. */
const DRAG_THRESHOLD = 4;

/** Watches a pointer press on a tab, and starts a drag once it moves far enough. */
function startTabDrag(event: ReactPointerEvent, panel: PanelId) {
  if (event.button !== 0) return;
  const { setDockDrag } = useApp.getState();
  const startX = event.clientX;
  const startY = event.clientY;
  const move = (e: PointerEvent) => {
    if (Math.hypot(e.clientX - startX, e.clientY - startY) < DRAG_THRESHOLD)
      return;
    // From here the drag layer follows the pointer and handles the drop.
    stop();
    setDockDrag({ panel, x: e.clientX, y: e.clientY, target: null });
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

/** One stack: a tab strip and the active panel. Also a drop target while dragging. */
export function DockStack({ area, stack }: { area: AreaId; stack: Stack }) {
  const { activatePanel, dockDrag } = useApp();
  const target = dockDrag?.target;
  const insertAt =
    target?.kind === "tabs" && target.stack === stack.id ? target.index : null;
  const split =
    target?.kind === "split" && target.stack === stack.id ? target : null;
  const sideways = area === "bottom";

  const onKeyDown = (event: KeyboardEvent, index: number) => {
    const step =
      event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
    if (step === 0) return;
    event.preventDefault();
    const next =
      stack.tabs[(index + step + stack.tabs.length) % stack.tabs.length];
    if (next) {
      activatePanel(next);
      document.getElementById(`dock-tab-${next}`)?.focus();
    }
  };

  return (
    <div className="flex h-full min-h-0 min-w-0 flex-col">
      <div
        role="tablist"
        aria-label={`${area} panels`}
        data-dock-tabs=""
        data-area={area}
        data-stack={stack.id}
        className="relative flex h-7 shrink-0 items-stretch overflow-x-auto border-b border-border bg-surface-raised [scrollbar-width:none]"
      >
        {stack.tabs.map((panel, index) => {
          const active = panel === stack.active;
          return (
            <button
              key={panel}
              id={`dock-tab-${panel}`}
              type="button"
              role="tab"
              aria-selected={active}
              aria-controls={`dock-panel-${panel}`}
              tabIndex={active ? 0 : -1}
              data-dock-tab=""
              onClick={() => activatePanel(panel)}
              onKeyDown={(e) => onKeyDown(e, index)}
              onPointerDown={(e) => startTabDrag(e, panel)}
              className={`relative flex shrink-0 items-center px-3 text-2xs font-semibold tracking-[0.06em] uppercase outline-none focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus-ring ${
                active
                  ? "bg-surface text-text shadow-[inset_0_2px_0_var(--color-accent)]"
                  : "text-text-muted hover:text-text"
              } ${dockDrag?.panel === panel ? "opacity-40" : ""}`}
            >
              {insertAt === index ? <InsertLine /> : null}
              {PANEL_INFO[panel].title}
            </button>
          );
        })}
        <span className="relative min-w-6 flex-1">
          {insertAt === stack.tabs.length ? <InsertLine /> : null}
        </span>
      </div>
      <div
        id={`dock-panel-${stack.active}`}
        role="tabpanel"
        aria-labelledby={`dock-tab-${stack.active}`}
        data-dock-body=""
        data-area={area}
        data-stack={stack.id}
        className="relative min-h-0 flex-1 overflow-auto bg-surface"
      >
        {PANEL_INFO[stack.active].render()}
        {split ? (
          <div
            aria-hidden="true"
            className={`pointer-events-none absolute bg-accent/25 outline-2 -outline-offset-2 outline-accent ${
              sideways
                ? `inset-y-0 w-1/2 ${split.after ? "right-0" : "left-0"}`
                : `inset-x-0 h-1/2 ${split.after ? "bottom-0" : "top-0"}`
            }`}
          />
        ) : null}
      </div>
    </div>
  );
}

function InsertLine() {
  return (
    <span
      aria-hidden="true"
      className="pointer-events-none absolute inset-y-1 left-0 w-0.5 rounded-full bg-accent"
    />
  );
}
