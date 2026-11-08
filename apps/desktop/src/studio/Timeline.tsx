import type { MouseEvent, ReactNode } from "react";
import type { Layer, TimingEdit } from "../backend";
import { targetLayer, useApp } from "../store";

const CELL = 22;
const MIN_FRAMES = 48;

/** How many frame columns to show: past the last exposure and the current frame, at least
 * two seconds at 24 fps, in whole half-seconds. */
export function frameCount(layers: readonly Layer[], currentFrame = 0): number {
  const last = Math.max(
    currentFrame + 1,
    ...layers.flatMap((l) => l.exposures.map((e) => e.startFrame + e.length)),
  );
  return Math.max(MIN_FRAMES, Math.ceil((last + 12) / 12) * 12);
}

const transportButton =
  "flex h-6 min-w-6 items-center justify-center rounded px-1.5 text-sm text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 aria-pressed:text-accent";

function TransportButton({
  label,
  shortcut,
  onClick,
  pressed,
  children,
}: {
  label: string;
  shortcut?: string;
  onClick: () => void;
  pressed?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      aria-pressed={pressed}
      title={shortcut ? `${label} (${shortcut})` : label}
      onClick={onClick}
      className={transportButton}
    >
      {children}
    </button>
  );
}

const TIMING: {
  edit: TimingEdit;
  label: string;
  short: string;
  shortcut: string;
}[] = [
  {
    edit: { type: "newDrawing" },
    label: "New drawing",
    short: "New",
    shortcut: "Alt+N",
  },
  {
    edit: { type: "hold", by: 1 },
    label: "Hold longer",
    short: "+1",
    shortcut: "Alt+=",
  },
  {
    edit: { type: "hold", by: -1 },
    label: "Hold shorter",
    short: "−1",
    shortcut: "Alt+-",
  },
  {
    edit: { type: "insertFrame" },
    label: "Insert frame",
    short: "Ins",
    shortcut: "Alt+I",
  },
  {
    edit: { type: "deleteFrame" },
    label: "Delete frame",
    short: "Del",
    shortcut: "Alt+D",
  },
];

/** First, previous, play, next and last; the frame counter; loop; and the timing edits. */
function Transport() {
  const {
    document: doc,
    playing,
    loop,
    togglePlayback,
    setLoop,
    setCurrentFrame,
    editTiming,
  } = useApp();
  if (!doc) return null;
  const frame = doc.currentFrame;
  return (
    <div
      role="toolbar"
      aria-label="Transport"
      className="flex h-8 shrink-0 items-center gap-0.5 border-b border-border bg-surface-raised px-2"
    >
      <TransportButton
        label="First frame"
        shortcut="Home"
        onClick={() => void setCurrentFrame(0)}
      >
        ⏮
      </TransportButton>
      <TransportButton
        label="Previous frame"
        shortcut="Alt+←"
        onClick={() => void setCurrentFrame(Math.max(0, frame - 1))}
      >
        ◀
      </TransportButton>
      <TransportButton
        label={playing ? "Stop" : "Play"}
        shortcut="Shift+Space"
        onClick={togglePlayback}
        pressed={playing}
      >
        {playing ? "■" : "▶"}
      </TransportButton>
      <TransportButton
        label="Next frame"
        shortcut="Alt+→"
        onClick={() => void setCurrentFrame(frame + 1)}
      >
        ▶|
      </TransportButton>
      <TransportButton
        label="Last frame"
        shortcut="End"
        onClick={() => void setCurrentFrame(doc.sceneLength - 1)}
      >
        ⏭
      </TransportButton>
      <output
        aria-label="Current frame"
        className="mx-2 min-w-24 text-sm text-text tabular-nums"
      >
        Frame {frame + 1}
        <span className="text-text-muted"> of {doc.sceneLength}</span>
      </output>
      <label className="flex items-center gap-1.5 text-xs text-text-muted">
        <input
          type="checkbox"
          checked={loop}
          onChange={(e) => setLoop(e.target.checked)}
          className="accent-[var(--color-accent)]"
        />
        Loop
      </label>
      <span className="mx-2 h-4 w-px bg-border" aria-hidden="true" />
      <div role="group" aria-label="Timing" className="flex gap-0.5">
        {TIMING.map(({ edit, label, short, shortcut }) => (
          <button
            key={label}
            type="button"
            aria-label={label}
            title={`${label} (${shortcut})`}
            onClick={() => void editTiming(edit)}
            className="h-6 rounded border border-control-border bg-surface px-2 text-xs text-text outline-none hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-focus-ring"
          >
            {short}
          </button>
        ))}
      </div>
    </div>
  );
}

/** Rows of layers against frames, with a playhead, showing each exposure and its drawing number. */
export function Timeline() {
  const {
    document: doc,
    activeLayer,
    setActiveLayer,
    setCurrentFrame,
  } = useApp();
  const scene = doc?.project.scenes[0];
  if (!scene || !doc) return null;
  const layers = [...scene.layers].reverse();
  const current = doc.currentFrame;
  const frames = frameCount(scene.layers, current);
  const target = targetLayer(scene.layers, activeLayer);

  /** Clicking a cell picks its layer and frame. */
  const pick = (layer: string, start: number) => (event: MouseEvent) => {
    const offset = Math.floor(event.nativeEvent.offsetX / CELL);
    setActiveLayer(layer);
    void setCurrentFrame(start + Math.max(0, offset));
  };

  return (
    <section
      aria-label="Timeline"
      className="flex h-full min-h-0 flex-col bg-surface"
    >
      <Transport />
      <div className="flex min-h-0 flex-1 overflow-auto">
        <table
          className="self-start border-collapse text-xs"
          aria-label="Exposures"
        >
          <thead className="sticky top-0 z-10 bg-surface">
            <tr>
              <th
                scope="col"
                className="sticky left-0 z-20 h-5 w-44 min-w-44 border-r border-border bg-surface px-2 text-left font-normal text-text-muted"
              >
                Layer
              </th>
              {Array.from({ length: frames }, (_, i) => (
                <th
                  key={i}
                  scope="col"
                  style={{ width: CELL, minWidth: CELL }}
                  aria-current={i === current ? "true" : undefined}
                  className={`relative h-5 cursor-pointer border-l border-border p-0 text-center font-normal ${
                    i === current
                      ? "bg-accent-muted text-text"
                      : i % 6 === 5
                        ? "text-text hover:bg-surface-hover"
                        : "text-text-muted hover:bg-surface-hover"
                  }`}
                >
                  <button
                    type="button"
                    aria-label={`Go to frame ${i + 1}`}
                    onClick={() => void setCurrentFrame(i)}
                    className="h-full w-full outline-none focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus-ring"
                  >
                    {(i + 1) % 6 === 0 || i === 0 || i === current ? i + 1 : ""}
                  </button>
                  {i === current ? (
                    // The playhead: a line from the frame number down through every row.
                    <span
                      aria-hidden="true"
                      className="pointer-events-none absolute top-full left-1/2 h-[2000px] w-0.5 -translate-x-1/2 bg-accent/80"
                    />
                  ) : null}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {layers.map((layer) => (
              <tr
                key={layer.id}
                className={layer.id === target?.id ? "bg-accent-muted/60" : ""}
              >
                <th
                  scope="row"
                  onClick={() => setActiveLayer(layer.id)}
                  className={`sticky left-0 z-10 h-[26px] cursor-pointer truncate border-t border-r border-border px-2 text-left font-normal ${
                    layer.id === target?.id
                      ? "bg-accent-muted text-text"
                      : "bg-surface text-text-muted"
                  } ${layer.visible ? "" : "line-through"}`}
                >
                  {layer.name}
                </th>
                <ExposureCells layer={layer} frames={frames} pick={pick} />
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ExposureCells({
  layer,
  frames,
  pick,
}: {
  layer: Layer;
  frames: number;
  pick: (layer: string, start: number) => (event: MouseEvent) => void;
}) {
  const cells = [];
  let frame = 0;
  const empty = (at: number) => (
    <td
      key={at}
      onClick={pick(layer.id, at)}
      className="cursor-pointer border-t border-l border-border"
    />
  );
  for (const exposure of layer.exposures) {
    if (exposure.startFrame >= frames) break;
    for (; frame < exposure.startFrame; frame++) cells.push(empty(frame));
    const span = Math.min(exposure.length, frames - exposure.startFrame);
    const number =
      layer.drawings.findIndex((d) => d.id === exposure.drawing) + 1;
    cells.push(
      <td
        key={frame}
        colSpan={span}
        onClick={pick(layer.id, exposure.startFrame)}
        className="cursor-pointer border-t border-l border-border p-0.5"
        aria-label={`Drawing ${number}, frames ${exposure.startFrame + 1} to ${exposure.startFrame + span}`}
      >
        <div className="pointer-events-none flex h-[18px] items-center rounded-sm border-l-2 border-accent bg-accent-muted px-1 text-text">
          {number}
        </div>
      </td>,
    );
    frame = exposure.startFrame + span;
  }
  for (; frame < frames; frame++) cells.push(empty(frame));
  return <>{cells}</>;
}
