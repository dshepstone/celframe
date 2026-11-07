import type { Layer } from "../backend";
import { targetLayer, useApp } from "../store";

const CELL = 22;
const MIN_FRAMES = 48;

/** How many frame columns to show: past the last exposure, at least two seconds at 24 fps. */
export function frameCount(layers: readonly Layer[]): number {
  const last = Math.max(
    0,
    ...layers.flatMap((l) => l.exposures.map((e) => e.startFrame + e.length)),
  );
  return Math.max(MIN_FRAMES, Math.ceil((last + 12) / 12) * 12);
}

/** Rows of layers against frames, showing each exposure and its drawing number. */
export function Timeline() {
  const { document: doc, activeLayer, setActiveLayer } = useApp();
  const scene = doc?.project.scenes[0];
  if (!scene) return null;
  const layers = [...scene.layers].reverse();
  const frames = frameCount(scene.layers);
  const target = targetLayer(scene.layers, activeLayer);

  return (
    <section
      aria-label="Timeline"
      className="flex h-full min-h-0 flex-col bg-surface"
    >
      <header className="flex items-center gap-3 border-b border-border px-3 py-1.5 text-xs">
        <h2 className="font-semibold tracking-wide text-text-muted uppercase">
          Timeline
        </h2>
        <span className="text-text-muted">
          Frame 1 of {frames} · {scene.frameRate} fps · Playback and timing
          tools arrive in Phase 3
        </span>
      </header>
      <div className="flex min-h-0 flex-1 overflow-auto">
        <table
          className="self-start border-collapse text-xs"
          aria-label="Exposures"
        >
          <thead className="sticky top-0 z-10 bg-surface-raised">
            <tr>
              <th
                scope="col"
                className="sticky left-0 z-20 w-40 min-w-40 bg-surface-raised px-2 py-1 text-left font-medium text-text-muted"
              >
                Layer
              </th>
              {Array.from({ length: frames }, (_, i) => (
                <th
                  key={i}
                  scope="col"
                  style={{ width: CELL, minWidth: CELL }}
                  className={`border-l border-border py-1 text-center font-normal ${
                    i === 0
                      ? "bg-accent text-text-on-accent"
                      : i % 6 === 5
                        ? "text-text"
                        : "text-text-muted"
                  }`}
                >
                  {(i + 1) % 6 === 0 || i === 0 ? i + 1 : ""}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {layers.map((layer) => (
              <tr
                key={layer.id}
                onClick={() => setActiveLayer(layer.id)}
                className={layer.id === target?.id ? "bg-surface-sunken" : ""}
              >
                <th
                  scope="row"
                  className={`sticky left-0 z-10 truncate border-t border-border px-2 py-1 text-left font-normal ${
                    layer.id === target?.id
                      ? "bg-surface-sunken text-text"
                      : "bg-surface text-text-muted"
                  } ${layer.visible ? "" : "line-through"}`}
                >
                  {layer.name}
                </th>
                <ExposureCells layer={layer} frames={frames} />
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ExposureCells({ layer, frames }: { layer: Layer; frames: number }) {
  const cells = [];
  let frame = 0;
  for (const exposure of layer.exposures) {
    if (exposure.startFrame >= frames) break;
    for (; frame < exposure.startFrame; frame++) {
      cells.push(
        <td key={frame} className="border-t border-l border-border" />,
      );
    }
    const span = Math.min(exposure.length, frames - exposure.startFrame);
    const number =
      layer.drawings.findIndex((d) => d.id === exposure.drawing) + 1;
    cells.push(
      <td
        key={frame}
        colSpan={span}
        className="border-t border-l border-border p-0.5"
        aria-label={`Drawing ${number}, frames ${exposure.startFrame + 1} to ${exposure.startFrame + span}`}
      >
        <div className="flex h-5 items-center rounded-sm bg-accent px-1 text-text-on-accent">
          {number}
        </div>
      </td>,
    );
    frame = exposure.startFrame + span;
  }
  for (; frame < frames; frame++) {
    cells.push(<td key={frame} className="border-t border-l border-border" />);
  }
  return <>{cells}</>;
}
