import { targetLayer, useApp } from "../store";

const viewButton =
  "flex h-4 min-w-4 items-center justify-center rounded px-1 text-text hover:bg-surface-hover outline-none focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-40";

/** One line of facts about the open document, and the view controls. */
export function StatusBar() {
  const { document: doc, activeLayer, art, view, changeView } = useApp();
  const scene = doc?.project.scenes[0];
  const layer = scene ? targetLayer(scene.layers, activeLayer) : null;
  const strokes = scene?.layers.reduce(
    (sum, l) => sum + l.drawings.reduce((n, d) => n + d.strokes.length, 0),
    0,
  );

  return (
    <footer
      aria-label="Status"
      className="flex h-[22px] shrink-0 items-center gap-3 border-t border-border bg-surface-raised px-2.5 text-xs text-text-muted tabular-nums [&>span:not(:last-child)]:pr-3 [&>span:not(:last-child)]:shadow-[1px_0_0_var(--color-border)]"
    >
      {scene ? (
        <>
          <span>{scene.name}</span>
          <span>
            {scene.width} × {scene.height}
          </span>
          <span>{scene.frameRate} fps</span>
          <span>
            Frame {(doc?.currentFrame ?? 0) + 1} of {doc?.sceneLength ?? 1}
          </span>
          <span>
            Drawing on: {layer ? layer.name : "no visible, unlocked layer"}
          </span>
          <span>{art === "line" ? "Line art" : "Colour art"}</span>
          <span>{strokes} strokes</span>
          {doc && doc.selection.length > 0 ? (
            <span>{doc.selection.length} selected</span>
          ) : null}
        </>
      ) : null}
      <span className="flex-1" />
      <span role="group" aria-label="View" className="flex items-center gap-1">
        <button
          type="button"
          className={viewButton}
          aria-label="Zoom out"
          onClick={() => void changeView({ type: "zoom", factor: 0.8 })}
        >
          −
        </button>
        <span className="w-12 text-center" aria-label="Zoom level">
          {view ? `${Math.round(view.zoomPercent)}%` : "Fit"}
        </span>
        <button
          type="button"
          className={viewButton}
          aria-label="Zoom in"
          onClick={() => void changeView({ type: "zoom", factor: 1.25 })}
        >
          +
        </button>
        <button
          type="button"
          className={viewButton}
          aria-label="Rotate left"
          onClick={() => void changeView({ type: "rotate", degrees: 15 })}
        >
          ⟲
        </button>
        <span className="w-10 text-center" aria-label="Rotation angle">
          {Math.round(view?.rotationDegrees ?? 0) % 360}°
        </span>
        <button
          type="button"
          className={viewButton}
          aria-label="Rotate right"
          onClick={() => void changeView({ type: "rotate", degrees: -15 })}
        >
          ⟳
        </button>
        <button
          type="button"
          className={viewButton}
          disabled={!view || view.fitted}
          onClick={() => void changeView({ type: "fit" })}
        >
          Fit
        </button>
      </span>
      <span className="truncate">
        {doc?.path ?? "Not saved yet"}
        {doc?.dirty ? " · Unsaved changes" : ""}
      </span>
    </footer>
  );
}
