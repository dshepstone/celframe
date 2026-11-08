import {
  useEffect,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { valueAt } from "../animated";
import type { Camera, Interpolation } from "../backend";
import {
  CAMERA_VALUES,
  cameraKeyFrames,
  cameraScale,
  keyAll,
  resetCamera,
} from "../camera";
import { useApp } from "../store";
import { Panel, panelIconButton } from "./Panel";

const keyButton =
  "flex h-5 w-5 shrink-0 items-center justify-center rounded text-sm leading-none outline-none hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-focus-ring";
const textButton =
  "h-[22px] rounded px-1.5 text-xs text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 disabled:hover:bg-transparent";

/** Rounds a shown value for display: whole numbers stay whole, others keep two places. */
function tidy(value: number): number {
  return Math.round(value * 100) / 100;
}

/** One camera value at the current frame: a scrubbable label, a number and its key. */
function CameraValue({
  scene,
  camera,
  info,
}: {
  scene: string;
  camera: Camera;
  info: (typeof CAMERA_VALUES)[number];
}) {
  const { document: doc, runCommand } = useApp();
  const frame = doc?.currentFrame ?? 0;
  const animated = camera[info.property];
  const shown = tidy(valueAt(animated, frame) * info.scale);
  const [draft, setDraft] = useState(String(shown));
  useEffect(() => setDraft(String(shown)), [shown]);
  const key = animated.keys.find((k) => k.frame === frame);
  const keyed = animated.keys.length > 0;

  const setKey = (stored: number, interpolation: Interpolation) =>
    void runCommand({
      type: "setCameraKey",
      scene,
      property: info.property,
      key: { frame, value: stored, interpolation },
    });
  /** Applies a new shown value once, when typing or scrubbing ends. */
  const commit = (value: number) => {
    if (!Number.isFinite(value)) {
      setDraft(String(shown));
      return;
    }
    const stored = Math.min(info.max, Math.max(info.min, value / info.scale));
    if (tidy(stored * info.scale) === shown) {
      setDraft(String(shown));
      return;
    }
    if (keyed) setKey(stored, key?.interpolation ?? "ease");
    else
      void runCommand({
        type: "setCameraBaseValue",
        scene,
        property: info.property,
        value: stored,
      });
  };

  // Dragging the label left or right changes the value; Shift moves ten times faster.
  const scrub = (event: ReactPointerEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    const startX = event.clientX;
    const start = shown;
    let latest = start;
    const move = (e: PointerEvent) => {
      const step = info.step * (e.shiftKey ? 10 : 1);
      latest = tidy(start + Math.round((e.clientX - startX) / 2) * step);
      setDraft(String(latest));
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      commit(latest);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div
      role="group"
      aria-label={info.label}
      className="flex h-7 items-center gap-1.5 px-2.5 text-xs text-text-muted"
    >
      <span
        onPointerDown={scrub}
        title="Drag left or right to change"
        className="w-16 shrink-0 cursor-ew-resize select-none"
      >
        {info.label}
      </span>
      <input
        type="number"
        aria-label={`${info.label} value`}
        value={draft}
        step={info.step}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={() => commit(Number(draft))}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit(Number(draft));
          if (e.key === "Escape") setDraft(String(shown));
        }}
        className="h-5 min-w-0 flex-1 rounded border border-control-border bg-surface-sunken px-1 text-right text-xs text-text tabular-nums outline-none focus:border-accent"
      />
      <span className="w-3 shrink-0">{info.unit}</span>
      {key ? (
        <>
          <button
            type="button"
            aria-label={`Remove ${info.label} key`}
            title="Remove the key at this frame"
            onClick={() =>
              void runCommand({
                type: "removeCameraKey",
                scene,
                property: info.property,
                frame,
              })
            }
            className={`${keyButton} text-accent`}
          >
            ◆
          </button>
          <select
            aria-label={`${info.label} interpolation`}
            value={key.interpolation}
            onChange={(e) => setKey(key.value, e.target.value as Interpolation)}
            className="h-5 w-[52px] shrink-0 rounded border border-control-border bg-surface-sunken px-0.5 text-xs text-text"
          >
            <option value="linear">Linear</option>
            <option value="ease">Ease</option>
            <option value="step">Step</option>
          </select>
        </>
      ) : (
        <>
          <button
            type="button"
            aria-label={`Set ${info.label} key`}
            title="Key this value at this frame"
            onClick={() => setKey(valueAt(animated, frame), "ease")}
            className={`${keyButton} text-text-muted hover:text-text`}
          >
            ◇
          </button>
          <span className="w-[52px] shrink-0" />
        </>
      )}
    </div>
  );
}

/**
 * The scene camera at the current frame: pan, distance, rotation and zoom, each keyable.
 * The Camera view looks through it. See docs/specs/scene-camera.md.
 */
export function CameraPanel() {
  const { document: doc, runCommand, setCurrentFrame } = useApp();
  const scene = doc?.project.scenes[0];
  if (!scene) return null;
  const { camera } = scene;
  const frame = doc.currentFrame;
  const keys = cameraKeyFrames(camera);
  const previous = [...keys].reverse().find((k) => k < frame);
  const next = keys.find((k) => k > frame);
  const scale = cameraScale(camera, frame);
  const distance = valueAt(camera.z, frame);
  const truck = (factor: number) => {
    const z = Math.min(1e5, Math.max(1, distance * factor));
    void runCommand(
      camera.z.keys.length > 0
        ? {
            type: "setCameraKey",
            scene: scene.id,
            property: "z",
            key: {
              frame,
              value: z,
              interpolation:
                camera.z.keys.find((k) => k.frame === frame)?.interpolation ??
                "ease",
            },
          }
        : {
            type: "setCameraBaseValue",
            scene: scene.id,
            property: "z",
            value: z,
          },
    );
  };

  return (
    <Panel
      id="camera"
      title="Camera"
      detail={`Frame ${frame + 1}`}
      actions={
        <>
          <button
            type="button"
            className={panelIconButton}
            aria-label="Previous camera key"
            title="Go to the previous camera key"
            disabled={previous === undefined}
            onClick={() =>
              previous !== undefined && void setCurrentFrame(previous)
            }
          >
            ‹◆
          </button>
          <button
            type="button"
            className={panelIconButton}
            aria-label="Next camera key"
            title="Go to the next camera key"
            disabled={next === undefined}
            onClick={() => next !== undefined && void setCurrentFrame(next)}
          >
            ◆›
          </button>
          <button
            type="button"
            className={textButton}
            title="Key every camera value at this frame"
            onClick={() => void runCommand(keyAll(scene.id, camera, frame))}
          >
            Key all
          </button>
        </>
      }
    >
      <div className="flex flex-col py-1">
        {CAMERA_VALUES.map((info) => (
          <CameraValue
            key={info.property}
            scene={scene.id}
            camera={camera}
            info={info}
          />
        ))}
      </div>
      <div className="flex items-center gap-1 border-t border-border px-2.5 py-1.5">
        <button
          type="button"
          className={textButton}
          title="Move the camera closer to the scene"
          onClick={() => truck(0.8)}
        >
          Truck in
        </button>
        <button
          type="button"
          className={textButton}
          title="Move the camera away from the scene"
          onClick={() => truck(1.25)}
        >
          Truck out
        </button>
        <span className="flex-1" />
        <button
          type="button"
          className={textButton}
          title="Remove every camera key and return to the standard camera"
          onClick={() => void runCommand(resetCamera(scene.id, camera))}
        >
          Reset
        </button>
      </div>
      <p
        aria-label="Camera framing"
        className="px-2.5 pb-2 text-[11px] text-text-muted tabular-nums"
      >
        Frames {Math.round(scene.width / scale)} ×{" "}
        {Math.round(scene.height / scale)} of the scene (
        {Math.round(scale * 100)}% of standard)
      </p>
    </Panel>
  );
}
