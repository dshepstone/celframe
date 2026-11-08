import { valueAt } from "./animated";
import type { Camera, CameraProperty, Command, Key } from "./backend";

/** Matches `STANDARD_DISTANCE` in the core: the distance that frames the scene exactly. */
export const STANDARD_DISTANCE = 1000;

/** Each camera value as the Camera panel shows it. Ranges match the core's. */
export const CAMERA_VALUES: readonly {
  property: CameraProperty;
  label: string;
  unit: string;
  /** Shown value per stored value. */
  scale: number;
  /** Shown units per pixel when scrubbing the label. */
  step: number;
  min: number;
  max: number;
  default: number;
}[] = [
  {
    property: "x",
    label: "Pan X",
    unit: "",
    scale: 1,
    step: 1,
    min: -1e6,
    max: 1e6,
    default: 0,
  },
  {
    property: "y",
    label: "Pan Y",
    unit: "",
    scale: 1,
    step: 1,
    min: -1e6,
    max: 1e6,
    default: 0,
  },
  {
    property: "z",
    label: "Distance",
    unit: "",
    scale: 1,
    step: 5,
    min: 1,
    max: 1e5,
    default: STANDARD_DISTANCE,
  },
  {
    property: "rotation",
    label: "Rotation",
    unit: "°",
    scale: 1,
    step: 0.5,
    min: -3600,
    max: 3600,
    default: 0,
  },
  {
    property: "zoom",
    label: "Zoom",
    unit: "%",
    scale: 100,
    step: 1,
    min: 0.01,
    max: 100,
    default: 1,
  },
];

/** Frames that have a key on any camera value, in order. */
export function cameraKeyFrames(camera: Camera): number[] {
  const frames = new Set<number>();
  for (const { property } of CAMERA_VALUES)
    for (const key of camera[property].keys) frames.add(key.frame);
  return [...frames].sort((a, b) => a - b);
}

/** How many times larger the camera shows the scene than the standard camera does. */
export function cameraScale(camera: Camera, frame: number): number {
  const z = Math.max(1, valueAt(camera.z, frame));
  const zoom = Math.max(0.01, valueAt(camera.zoom, frame));
  return (zoom * STANDARD_DISTANCE) / z;
}

/** Keys every camera value at a frame with its value there, as one edit. */
export function keyAll(scene: string, camera: Camera, frame: number): Command {
  return {
    type: "batch",
    commands: CAMERA_VALUES.map(({ property }) => {
      const existing = camera[property].keys.find((k) => k.frame === frame);
      const key: Key = {
        frame,
        value: valueAt(camera[property], frame),
        interpolation: existing?.interpolation ?? "ease",
      };
      return { type: "setCameraKey", scene, property, key };
    }),
  };
}

/** Moves every camera key at `from` to `to`, as one edit. */
export function moveCameraKeys(
  scene: string,
  camera: Camera,
  from: number,
  to: number,
): Command {
  const commands: Command[] = [];
  for (const { property } of CAMERA_VALUES) {
    const key = camera[property].keys.find((k) => k.frame === from);
    if (!key) continue;
    commands.push(
      { type: "removeCameraKey", scene, property, frame: from },
      { type: "setCameraKey", scene, property, key: { ...key, frame: to } },
    );
  }
  return { type: "batch", commands };
}

/** Removes every camera key and returns each value to the standard camera's, as one edit. */
export function resetCamera(scene: string, camera: Camera): Command {
  const commands: Command[] = [];
  for (const { property, default: value } of CAMERA_VALUES) {
    for (const key of camera[property].keys)
      commands.push({
        type: "removeCameraKey",
        scene,
        property,
        frame: key.frame,
      });
    if (camera[property].value !== value)
      commands.push({ type: "setCameraBaseValue", scene, property, value });
  }
  return { type: "batch", commands };
}
