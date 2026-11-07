import { useApp, SIDE_PANEL_MAX, SIDE_PANEL_MIN } from "../store";

/** The side panel's left edge, dragged to resize it. Arrow keys resize it too. */
export function SidePanelResizer() {
  const { sidePanelWidth, setSidePanelWidth } = useApp();
  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize side panel"
      aria-valuemin={SIDE_PANEL_MIN}
      aria-valuemax={SIDE_PANEL_MAX}
      aria-valuenow={sidePanelWidth}
      tabIndex={0}
      onPointerDown={(event) => {
        event.preventDefault();
        const startX = event.clientX;
        const startWidth = sidePanelWidth;
        const target = event.currentTarget;
        target.setPointerCapture?.(event.pointerId);
        const move = (e: PointerEvent) =>
          setSidePanelWidth(startWidth + startX - e.clientX);
        const up = () => {
          target.removeEventListener("pointermove", move);
          target.removeEventListener("pointerup", up);
        };
        target.addEventListener("pointermove", move);
        target.addEventListener("pointerup", up);
      }}
      onKeyDown={(e) => {
        // Arrow keys here resize the panel, not nudge the selection.
        if (e.key.startsWith("Arrow")) e.stopPropagation();
        if (e.key === "ArrowLeft") setSidePanelWidth(sidePanelWidth + 16);
        if (e.key === "ArrowRight") setSidePanelWidth(sidePanelWidth - 16);
      }}
      className="absolute top-0 bottom-0 -left-[3px] z-10 w-[6px] cursor-col-resize outline-none hover:bg-accent/40 focus-visible:bg-accent/60"
    />
  );
}
