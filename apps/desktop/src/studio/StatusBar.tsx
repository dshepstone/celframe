import { targetLayer, useApp } from "../store";

/** One line of facts about the open document. */
export function StatusBar() {
  const { document: doc, activeLayer } = useApp();
  const scene = doc?.project.scenes[0];
  const layer = scene ? targetLayer(scene.layers, activeLayer) : null;
  const strokes = scene?.layers.reduce(
    (sum, l) => sum + l.drawings.reduce((n, d) => n + d.strokes.length, 0),
    0,
  );

  return (
    <footer
      aria-label="Status"
      className="flex h-6 items-center gap-4 border-t border-border bg-surface-raised px-3 text-xs text-text-muted"
    >
      {scene ? (
        <>
          <span>{scene.name}</span>
          <span>
            {scene.width} × {scene.height}
          </span>
          <span>{scene.frameRate} fps</span>
          <span>Frame 1</span>
          <span>
            Drawing on: {layer ? layer.name : "no visible, unlocked layer"}
          </span>
          <span>{strokes} strokes</span>
        </>
      ) : null}
      <span className="flex-1" />
      <span className="truncate">
        {doc?.path ?? "Not saved yet"}
        {doc?.dirty ? " · Unsaved changes" : ""}
      </span>
    </footer>
  );
}
