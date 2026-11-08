import { useState, type PointerEvent as ReactPointerEvent } from "react";
import type { Layer } from "../backend";
import { useApp } from "../store";

/** The frame of the key-row cell under a point, if any. */
function keyFrameAt(x: number, y: number): number | null {
  for (const element of document.elementsFromPoint?.(x, y) ?? []) {
    const cell = element.closest<HTMLElement>("[data-key-frame]");
    const frame = Number(cell?.dataset.keyFrame);
    if (cell && Number.isInteger(frame)) return frame;
  }
  return null;
}

/**
 * A slim row under a keyed layer in the timeline, with a diamond at each key. Clicking one goes
 * to its frame; dragging one moves the key. See docs/specs/keyframes.md.
 */
export function KeyRow({
  scene,
  layer,
  frames,
}: {
  scene: string;
  layer: Layer;
  frames: number;
}) {
  const { document: doc, runCommand, setCurrentFrame } = useApp();
  const [dragging, setDragging] = useState<{ from: number; to: number } | null>(
    null,
  );
  const keys = new Map(layer.opacity.keys.map((k) => [k.frame, k]));
  const current = doc?.currentFrame ?? 0;

  const press = (event: ReactPointerEvent, from: number) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const row = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const y = row.top + row.height / 2;
    let to = from;
    setDragging({ from, to });
    const move = (e: PointerEvent) => {
      const frame = keyFrameAt(e.clientX, y);
      if (frame !== null && frame !== to) {
        to = frame;
        setDragging({ from, to });
      }
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setDragging(null);
      const key = keys.get(from);
      if (!key) return;
      if (to === from) {
        void setCurrentFrame(from);
        return;
      }
      // Moving a key onto another replaces it: one undo step either way.
      void runCommand({
        type: "batch",
        commands: [
          {
            type: "removeKey",
            scene,
            layer: layer.id,
            property: "opacity",
            frame: from,
          },
          {
            type: "setKey",
            scene,
            layer: layer.id,
            property: "opacity",
            key: { ...key, frame: to },
          },
        ],
      });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <tr aria-label={`${layer.name} keys`}>
      <th
        scope="row"
        className="sticky left-0 z-10 h-3.5 border-r border-border bg-surface px-2 pl-4 text-left text-[10px] font-normal text-text-muted"
      >
        Opacity
      </th>
      {Array.from({ length: frames }, (_, frame) => {
        const key = keys.get(frame);
        const shown = dragging
          ? frame === dragging.to || (key && frame !== dragging.from)
          : Boolean(key);
        return (
          <td
            key={frame}
            data-key-frame={frame}
            className={`h-3.5 border-l border-border p-0 text-center leading-none ${
              frame === current ? "bg-accent-muted/60" : ""
            }`}
          >
            {shown ? (
              <button
                type="button"
                aria-label={`Opacity key at frame ${frame + 1}`}
                onPointerDown={(e) => key && press(e, frame)}
                className={`cursor-ew-resize text-[10px] leading-none outline-none focus-visible:outline-2 focus-visible:outline-focus-ring ${
                  dragging && frame === dragging.to
                    ? "text-accent"
                    : "text-text"
                }`}
              >
                ◆
              </button>
            ) : null}
          </td>
        );
      })}
    </tr>
  );
}
