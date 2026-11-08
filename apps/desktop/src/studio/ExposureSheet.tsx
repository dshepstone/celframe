import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import type { Layer } from "../backend";
import { targetLayer, useApp } from "../store";
import { EyeIcon, EyeOffIcon } from "./icons";
import { startScrub } from "./scrub";

/** Paper colours for the sheet's cells: white with dark ink in both themes. */
const PAPER = "bg-[#ffffff] text-[#1a1c1f]";
const PAPER_CURRENT = "bg-[#d9efec] text-[#1a1c1f]";
const INK_MUTED = "text-[#5d636b]";
const HOLD_LINE = "bg-[#4a4f57]";
const RULE_LIGHT = "border-b border-b-[#d4d7dc]";
const RULE_HALF = "border-b border-b-[#9aa0a8]";
const RULE_SECOND = "border-b-2 border-b-[#5d636b]";
const COLUMN_EDGE = "border-r border-r-[#c4c7cc]";

/** Rows past the scene's end, or past the current frame if that's later. */
const EXTRA_ROWS = 24;

type Cell =
  { kind: "start"; number: number } | { kind: "hold" } | { kind: "empty" };

/** What each frame of a layer shows: where a drawing starts, a hold, or nothing. */
export function cellsFor(layer: Layer, rows: number): Cell[] {
  const cells: Cell[] = Array.from({ length: rows }, () => ({ kind: "empty" }));
  for (const exposure of layer.exposures) {
    const number =
      layer.drawings.findIndex((d) => d.id === exposure.drawing) + 1;
    for (let i = 0; i < exposure.length; i++) {
      const frame = exposure.startFrame + i;
      if (frame >= rows) break;
      cells[frame] = i === 0 ? { kind: "start", number } : { kind: "hold" };
    }
  }
  return cells;
}

/** Rows of frames against columns of layers, typed into by drawing number. See docs/specs/exposure-sheet.md. */
export function ExposureSheet() {
  const {
    document: doc,
    activeLayer,
    setActiveLayer,
    setCurrentFrame,
    editTiming,
    runCommand,
  } = useApp();
  const [draft, setDraft] = useState("");
  const entry = useRef<HTMLInputElement>(null);
  const scene = doc?.project.scenes[0];
  const current = doc?.currentFrame ?? 0;

  // Keep the current row in sight as the frame changes, for example during playback.
  useEffect(() => {
    entry.current?.scrollIntoView?.({ block: "nearest", inline: "nearest" });
  }, [current, activeLayer]);

  if (!scene || !doc) return null;
  const layers = [...scene.layers].reverse();
  const target = targetLayer(scene.layers, activeLayer);
  const rows = Math.max(doc.sceneLength, current + 1) + EXTRA_ROWS;
  const columns = layers.map((layer) => cellsFor(layer, rows));

  const choose = (layer: string, frame: number) => {
    setActiveLayer(layer);
    void setCurrentFrame(frame);
    setDraft("");
    // Typing goes into the current cell's entry, which moves with it.
    setTimeout(() => entry.current?.focus(), 0);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    const index = layers.findIndex((l) => l.id === target?.id);
    const move = (layerStep: number, frameStep: number) => {
      event.preventDefault();
      const next = layers[index + layerStep];
      if (layerStep !== 0 && next) setActiveLayer(next.id);
      if (frameStep !== 0)
        void setCurrentFrame(Math.max(0, current + frameStep));
      setDraft("");
    };
    switch (event.key) {
      case "ArrowUp":
        return move(0, -1);
      case "ArrowDown":
        return move(0, 1);
      case "ArrowLeft":
        return move(-1, 0);
      case "ArrowRight":
        return move(1, 0);
      case "Escape":
        event.preventDefault();
        setDraft("");
        return;
      case "Delete":
      case "Backspace":
        if (draft === "") {
          event.preventDefault();
          void editTiming({ type: "clearFrame" });
        }
        return;
      case "Enter": {
        event.preventDefault();
        const number = Number(draft);
        if (draft === "") return;
        setDraft("");
        if (!Number.isInteger(number) || number < 1) {
          useApp.getState().reportError(`"${draft}" isn't a drawing number.`);
          return;
        }
        void editTiming({ type: "expose", number }).then(() =>
          useApp.getState().setCurrentFrame(current + 1),
        );
        return;
      }
    }
  };

  return (
    <section
      aria-label="Exposure sheet"
      className="flex h-full min-h-0 flex-col bg-surface"
    >
      <div className="min-h-0 flex-1 overflow-auto">
        <table
          className="border-collapse text-xs tabular-nums"
          aria-label="Exposures by frame"
        >
          <thead className="sticky top-0 z-10 bg-surface-raised">
            <tr>
              <th
                scope="col"
                className="sticky left-0 z-20 h-6 w-12 min-w-12 border-r border-b border-border bg-surface-raised px-1.5 text-right font-normal text-text-muted"
              >
                Frame
              </th>
              {layers.map((layer) => (
                <th
                  key={layer.id}
                  scope="col"
                  onClick={() => setActiveLayer(layer.id)}
                  className={`h-6 w-24 min-w-24 max-w-32 cursor-pointer border-r border-b border-border px-1 text-left font-medium ${
                    layer.id === target?.id
                      ? "bg-accent-muted text-text"
                      : "text-text-muted"
                  }`}
                >
                  <span className="flex items-center gap-0.5">
                    <button
                      type="button"
                      aria-label={`${layer.visible ? "Hide" : "Show"} ${layer.name}`}
                      aria-pressed={layer.visible}
                      onClick={(e) => {
                        e.stopPropagation();
                        void runCommand({
                          type: "setLayerVisible",
                          scene: scene.id,
                          layer: layer.id,
                          visible: !layer.visible,
                        });
                      }}
                      className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring aria-[pressed=false]:opacity-60"
                    >
                      {layer.visible ? <EyeIcon /> : <EyeOffIcon />}
                    </button>
                    <span
                      className={`truncate ${layer.visible ? "" : "opacity-50"}`}
                    >
                      {layer.name}
                    </span>
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: rows }, (_, frame) => {
              const rule =
                frame % 24 === 23
                  ? RULE_SECOND
                  : frame % 12 === 11
                    ? RULE_HALF
                    : RULE_LIGHT;
              const isCurrent = frame === current;
              const paper = isCurrent ? PAPER_CURRENT : PAPER;
              return (
                <tr key={frame}>
                  <th
                    scope="row"
                    data-scrub-frame={frame}
                    onPointerDown={(e) => startScrub(e, "y")}
                    className={`sticky left-0 z-[1] h-5 cursor-ns-resize border-r border-r-[#9aa0a8] px-1.5 text-right font-normal select-none ${rule} ${paper} ${
                      isCurrent ? "font-semibold" : INK_MUTED
                    }`}
                  >
                    <button
                      type="button"
                      aria-label={`Frame ${frame + 1}`}
                      onClick={() => void setCurrentFrame(frame)}
                      className="w-full cursor-ns-resize text-right outline-none focus-visible:outline-2 focus-visible:outline-focus-ring"
                    >
                      {frame + 1}
                    </button>
                  </th>
                  {layers.map((layer, column) => {
                    const cell = columns[column]?.[frame] ?? { kind: "empty" };
                    const here = isCurrent && layer.id === target?.id;
                    return (
                      <td
                        key={layer.id}
                        onClick={() => choose(layer.id, frame)}
                        aria-label={
                          cell.kind === "start"
                            ? `${layer.name}, frame ${frame + 1}: drawing ${cell.number}`
                            : undefined
                        }
                        className={`relative h-5 cursor-cell p-0 text-center ${COLUMN_EDGE} ${rule} ${paper} ${
                          here
                            ? "outline-2 -outline-offset-2 outline-accent"
                            : ""
                        } ${layer.visible ? "" : "opacity-50"}`}
                      >
                        {here && cell.kind === "hold" ? (
                          <span
                            aria-hidden="true"
                            className={`pointer-events-none absolute inset-y-0 left-1/2 w-px -translate-x-1/2 ${HOLD_LINE}`}
                          />
                        ) : null}
                        {here ? (
                          <input
                            ref={entry}
                            aria-label={`Drawing number for ${layer.name} at frame ${frame + 1}`}
                            inputMode="numeric"
                            value={draft}
                            placeholder={
                              cell.kind === "start" ? String(cell.number) : ""
                            }
                            onChange={(e) =>
                              setDraft(e.target.value.replace(/[^0-9]/g, ""))
                            }
                            onKeyDown={onKeyDown}
                            className="h-full w-full bg-transparent text-center text-[#1a1c1f] outline-none placeholder:text-[#1a1c1f]"
                          />
                        ) : cell.kind === "start" ? (
                          <span className="font-medium">{cell.number}</span>
                        ) : cell.kind === "hold" ? (
                          <span
                            aria-hidden="true"
                            className={`absolute inset-y-0 left-1/2 w-px -translate-x-1/2 ${HOLD_LINE}`}
                          />
                        ) : null}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
