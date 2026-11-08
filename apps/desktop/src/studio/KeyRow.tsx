import { useState, type PointerEvent as ReactPointerEvent } from "react";
import type { Camera, Command, Layer } from "../backend";
import { cameraKeyFrames, moveCameraKeys } from "../camera";
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
 * A slim timeline row with a diamond at each keyed frame. Clicking one goes to its frame;
 * dragging one moves its keys. See docs/specs/keyframes.md.
 */
function KeyTrack({
  name,
  heading,
  keyName,
  keyFrames,
  frames,
  move,
}: {
  /** The row's accessible name. */
  name: string;
  /** The row's visible heading. */
  heading: string;
  /** Names each diamond, as in "Opacity key at frame 3". */
  keyName: string;
  keyFrames: number[];
  frames: number;
  /** The edit that moves the keys at one frame to another. */
  move: (from: number, to: number) => Command;
}) {
  const { document: doc, runCommand, setCurrentFrame } = useApp();
  const [dragging, setDragging] = useState<{ from: number; to: number } | null>(
    null,
  );
  const keyed = new Set(keyFrames);
  const current = doc?.currentFrame ?? 0;

  const press = (event: ReactPointerEvent, from: number) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const row = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const y = row.top + row.height / 2;
    let to = from;
    setDragging({ from, to });
    const onMove = (e: PointerEvent) => {
      const frame = keyFrameAt(e.clientX, y);
      if (frame !== null && frame !== to) {
        to = frame;
        setDragging({ from, to });
      }
    };
    const up = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", up);
      setDragging(null);
      if (to === from) {
        void setCurrentFrame(from);
        return;
      }
      // Moving keys onto others replaces them: one undo step either way.
      void runCommand(move(from, to));
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", up);
  };

  return (
    <tr aria-label={name}>
      <th
        scope="row"
        className="sticky left-0 z-10 h-3.5 border-r border-border bg-surface px-2 pl-4 text-left text-[10px] font-normal text-text-muted"
      >
        {heading}
      </th>
      {Array.from({ length: frames }, (_, frame) => {
        const key = keyed.has(frame);
        const shown = dragging
          ? frame === dragging.to || (key && frame !== dragging.from)
          : key;
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
                aria-label={`${keyName} key at frame ${frame + 1}`}
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

/** A keyed layer's opacity keys, under the layer in the timeline. */
export function KeyRow({
  scene,
  layer,
  frames,
}: {
  scene: string;
  layer: Layer;
  frames: number;
}) {
  const keys = new Map(layer.opacity.keys.map((k) => [k.frame, k]));
  return (
    <KeyTrack
      name={`${layer.name} keys`}
      heading="Opacity"
      keyName="Opacity"
      keyFrames={[...keys.keys()]}
      frames={frames}
      move={(from, to) => {
        const key = keys.get(from);
        return {
          type: "batch",
          commands: key
            ? [
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
              ]
            : [],
        };
      }}
    />
  );
}

/** The scene camera's keys, at the top of the timeline. A diamond stands for every value
 * keyed at that frame, and moving it moves them all. */
export function CameraKeyRow({
  scene,
  camera,
  frames,
}: {
  scene: string;
  camera: Camera;
  frames: number;
}) {
  return (
    <KeyTrack
      name="Camera keys"
      heading="Camera"
      keyName="Camera"
      keyFrames={cameraKeyFrames(camera)}
      frames={frames}
      move={(from, to) => moveCameraKeys(scene, camera, from, to)}
    />
  );
}
