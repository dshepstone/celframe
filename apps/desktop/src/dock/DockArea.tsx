import {
  Fragment,
  useRef,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { useApp } from "../store";
import { AREA_LIMITS, type AreaId } from "./layout";
import { DockStack } from "./DockStack";

/** Follows a pointer drag, reporting how far it has moved since the press. */
function track(
  event: ReactPointerEvent,
  onMove: (dx: number, dy: number) => void,
) {
  event.preventDefault();
  const startX = event.clientX;
  const startY = event.clientY;
  const move = (e: PointerEvent) =>
    onMove(e.clientX - startX, e.clientY - startY);
  const stop = () => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", stop);
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", stop);
}

const LABELS: Record<AreaId, string> = {
  left: "left",
  right: "right",
  bottom: "bottom",
};

/**
 * A dock area: its stacks with dividers between them, and an edge that resizes the area.
 * Empty, it takes no space, except for a drop strip while a tab is being dragged.
 */
export function DockArea({ area }: { area: AreaId }) {
  const { dock, dockDrag, setAreaSize, resizeStacks } = useApp();
  const stacks = dock.areas[area];
  const size = dock.sizes[area];
  const box = useRef<HTMLDivElement>(null);
  const sideways = area === "bottom";

  if (stacks.length === 0) {
    if (!dockDrag) return null;
    const over =
      dockDrag.target?.kind === "area" && dockDrag.target.area === area;
    return (
      <div
        data-dock-area-drop=""
        data-area={area}
        aria-label={`Open the ${LABELS[area]} area`}
        className={`flex shrink-0 items-center justify-center border-dashed border-accent text-2xs text-text-muted ${
          sideways ? "h-10 border-t-2" : "w-10 border-x-2"
        } ${over ? "bg-accent/30" : "bg-surface/80"}`}
      >
        {sideways ? "Drop here" : ""}
      </div>
    );
  }

  const total = stacks.reduce((sum, s) => sum + s.weight, 0);
  const [min, max] = AREA_LIMITS[area];
  // Dragging the inner edge: left grows rightwards, right and bottom grow towards the canvas.
  const edgeSign = area === "left" ? 1 : -1;

  return (
    <div
      ref={box}
      aria-label={`${LABELS[area]} panels`}
      role="region"
      style={sideways ? { height: size } : { width: size }}
      className={`relative flex shrink-0 bg-surface ${
        sideways ? "flex-row border-t" : "flex-col"
      } ${area === "left" ? "border-r" : area === "right" ? "border-l" : ""} border-border`}
    >
      <div
        role="separator"
        aria-orientation={sideways ? "horizontal" : "vertical"}
        aria-label={`Resize the ${LABELS[area]} area`}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={size}
        tabIndex={0}
        onPointerDown={(e) =>
          track(e, (dx, dy) =>
            setAreaSize(area, size + edgeSign * (sideways ? dy : dx)),
          )
        }
        onKeyDown={(e) => {
          // Arrow keys here resize the area, not nudge the selection.
          if (!e.key.startsWith("Arrow")) return;
          e.preventDefault();
          e.stopPropagation();
          const grow =
            e.key ===
            (sideways
              ? "ArrowUp"
              : area === "left"
                ? "ArrowRight"
                : "ArrowLeft");
          setAreaSize(area, size + (grow ? 16 : -16));
        }}
        className={`absolute z-10 outline-none hover:bg-accent/40 focus-visible:bg-accent/60 ${
          sideways
            ? "inset-x-0 -top-[3px] h-[6px] cursor-row-resize"
            : `inset-y-0 w-[6px] cursor-col-resize ${area === "left" ? "-right-[3px]" : "-left-[3px]"}`
        }`}
      />
      {stacks.map((stack, index) => (
        <Fragment key={stack.id}>
          {index > 0 ? (
            <div
              role="separator"
              aria-orientation={sideways ? "vertical" : "horizontal"}
              aria-label="Resize panels"
              onPointerDown={(e) => {
                const length = sideways
                  ? (box.current?.clientWidth ?? 0)
                  : (box.current?.clientHeight ?? 0);
                let last = 0;
                track(e, (dx, dy) => {
                  const moved = sideways ? dx : dy;
                  resizeStacks(area, index - 1, moved - last, length);
                  last = moved;
                });
              }}
              className={`shrink-0 bg-border hover:bg-accent/60 ${
                sideways ? "w-px cursor-col-resize" : "h-px cursor-row-resize"
              } relative before:absolute ${
                sideways
                  ? "before:inset-y-0 before:-inset-x-1"
                  : "before:inset-x-0 before:-inset-y-1"
              }`}
            />
          ) : null}
          <div
            style={{ flexGrow: stack.weight / total, flexBasis: 0 }}
            className={`min-h-0 min-w-0 ${sideways ? "h-full" : "w-full"}`}
          >
            <DockStack area={area} stack={stack} />
          </div>
        </Fragment>
      ))}
    </div>
  );
}
