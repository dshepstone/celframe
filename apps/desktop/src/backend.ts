import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Mirrors the view sent by the Rust core. The core stays the source of truth.
export interface Exposure {
  startFrame: number;
  length: number;
  drawing: string;
}

export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  drawings: { id: string; strokes: Stroke[] }[];
  /** Sorted by start frame, with no overlaps. */
  exposures: Exposure[];
  /** From 0 to 1. Keyable. */
  opacity: Animated;
}

/** How a key's value travels to the next key's. See docs/specs/keyframes.md. */
export type Interpolation = "linear" | "ease" | "step";

export interface Key {
  frame: number;
  value: number;
  interpolation: Interpolation;
}

/** A value with optional keys. */
export interface Animated {
  value: number;
  keys: Key[];
}

/** A keyable value on a layer. */
export type LayerProperty = "opacity";

export interface Scene {
  id: string;
  name: string;
  width: number;
  height: number;
  frameRate: number;
  /** Bottom layer first. */
  layers: Layer[];
  camera: Camera;
}

/** The scene camera. See docs/specs/scene-camera.md. */
export interface Camera {
  /** The scene point at the middle of the frame. */
  x: Animated;
  y: Animated;
  /** Distance from the scene; 1000 frames it exactly. */
  z: Animated;
  /** Degrees, counter-clockwise. */
  rotation: Animated;
  /** Lens magnification; 1 is standard. */
  zoom: Animated;
}

export type CameraProperty = keyof Camera;

export interface Project {
  id: string;
  name: string;
  scenes: Scene[];
}

/** A document edit for the core, as tagged JSON. See celframe-core's Command. */
export type Command =
  | { type: "setLayerVisible"; scene: string; layer: string; visible: boolean }
  | { type: "setLayerLocked"; scene: string; layer: string; locked: boolean }
  | { type: "renameLayer"; scene: string; layer: string; name: string }
  | { type: "removeLayer"; scene: string; layer: string }
  | { type: "moveLayer"; scene: string; layer: string; index: number }
  | {
      type: "setKey";
      scene: string;
      layer: string;
      property: LayerProperty;
      key: Key;
    }
  | {
      type: "removeKey";
      scene: string;
      layer: string;
      property: LayerProperty;
      frame: number;
    }
  | {
      type: "setBaseValue";
      scene: string;
      layer: string;
      property: LayerProperty;
      value: number;
    }
  | { type: "setCameraKey"; scene: string; property: CameraProperty; key: Key }
  | {
      type: "removeCameraKey";
      scene: string;
      property: CameraProperty;
      frame: number;
    }
  | {
      type: "setCameraBaseValue";
      scene: string;
      property: CameraProperty;
      value: number;
    }
  | { type: "batch"; commands: Command[] };

/** A change to the canvas view. Anchors are CSS pixels from the window's top left. */
export type ViewChange =
  | { type: "zoom"; factor: number; anchor?: { x: number; y: number } }
  | { type: "pan"; dx: number; dy: number }
  | { type: "rotate"; degrees: number; anchor?: { x: number; y: number } }
  | { type: "fit" };

/** The view after a change. 100% is one CSS pixel per scene unit. */
export interface ViewInfo {
  zoomPercent: number;
  rotationDegrees: number;
  /** True while the whole scene is fitted to the canvas. */
  fitted: boolean;
}

/** What the next stroke looks like and where it goes. */
/** Which part of a drawing a stroke is on. Colour art draws below line art. */
export type Art = "line" | "colour";

export interface Stroke {
  id: string;
  baseWidth: number;
  color: [number, number, number, number];
  art: Art;
  samples: { x: number; y: number; pressure: number }[];
}

export interface PenOptions {
  width: number;
  color: [number, number, number, number];
  /** null draws on the topmost visible, unlocked layer. */
  layer: string | null;
  art: Art;
  /** Draw, erase with `width` as the eraser's diameter, or drag out a shape. */
  tool:
    | { type: "draw" }
    | { type: "erase"; wholeStrokes: boolean }
    | { type: "shape"; shape: ShapeOption }
    | { type: "select"; lasso: boolean; scaleWidths: boolean };
}

export type ShapeKind = "line" | "rectangle" | "ellipse" | "polygon";

export type ShapeOption =
  | { type: "line" }
  | { type: "rectangle" }
  | { type: "ellipse" }
  | { type: "polygon"; sides: number };

export interface DocumentView {
  project: Project;
  canUndo: boolean;
  canRedo: boolean;
  dirty: boolean;
  /** The project's file, or null if it has never been saved. */
  path: string | null;
  /** IDs of the selected strokes. */
  selection: string[];
  /** The box around the selection's centrelines: [minX, minY, maxX, maxY]. */
  selectionBounds: [number, number, number, number] | null;
  /** The current frame, counted from 0. */
  currentFrame: number;
  /** The first scene's length in frames. */
  sceneLength: number;
}

/** Onion skin settings. See docs/specs/onion-skin.md. */
export interface OnionSkin {
  enabled: boolean;
  before: number;
  after: number;
  byFrames: boolean;
  step: number;
  /** The nearest neighbour's opacity, from 0 to 1. */
  opacity: number;
}

/** A timing edit at the current frame. See docs/specs/timing.md. */
export type TimingEdit =
  | { type: "newDrawing" }
  | { type: "hold"; by: number }
  | { type: "insertFrame" }
  | { type: "deleteFrame" }
  | { type: "expose"; number: number }
  | { type: "clearFrame" };

/** The centre's views. See docs/specs/views.md. */
export type ViewMode = "drawing" | "camera";

export type Arrange = "front" | "forward" | "backward" | "back";

/** An edit to the whole selection. See docs/specs/transform-and-arrange.md. */
export type SelectionEdit =
  | {
      type: "setBounds";
      x: number;
      y: number;
      width: number;
      height: number;
      scaleWidths: boolean;
    }
  | { type: "rotate"; degrees: number }
  | { type: "flip"; horizontal: boolean }
  | { type: "arrange"; to: Arrange };

/** A recovery copy left by a run that didn't exit normally. */
export interface RecoveryInfo {
  originalPath: string | null;
  savedAtMs: number;
}

/** Where the canvas area is, in CSS pixels, and the colour around the paper. */
export interface CanvasLayout {
  x: number;
  y: number;
  width: number;
  height: number;
  /** Device pixels per CSS pixel. */
  scale: number;
  backdrop: [number, number, number, number];
}

/** A pointer position in CSS pixels from the window's top left, with pen pressure. */
export interface PointerSample {
  x: number;
  y: number;
  pressure: number;
  /** Modifier keys, which shapes use. */
  shift?: boolean;
  alt?: boolean;
}

export const backend = {
  getDocument: () => invoke<DocumentView>("get_document"),
  undo: () => invoke<DocumentView>("undo"),
  redo: () => invoke<DocumentView>("redo"),
  newDocument: () => invoke<DocumentView>("new_document"),
  // These resolve to null when the person cancels the file picker.
  openDocument: () => invoke<DocumentView | null>("open_document"),
  saveDocument: () => invoke<DocumentView | null>("save_document"),
  saveDocumentAs: () => invoke<DocumentView | null>("save_document_as"),
  getRecovery: () => invoke<RecoveryInfo | null>("get_recovery"),
  restoreRecovery: () => invoke<DocumentView>("restore_recovery"),
  discardRecovery: async (): Promise<void> => {
    await invoke("discard_recovery");
  },
  /** Resolves false when there is no native canvas to show through the page. */
  setCanvasLayout: (layout: CanvasLayout) =>
    invoke<boolean>("set_canvas_layout", { layout }),
  executeCommand: (command: Command) =>
    invoke<DocumentView>("execute_command", { command }),
  addLayer: () => invoke<DocumentView>("add_layer"),
  changeView: (change: ViewChange) =>
    invoke<ViewInfo>("change_view", { change }),
  penDown: async (
    sample: PointerSample,
    options: PenOptions,
  ): Promise<void> => {
    await invoke("pen_down", { sample, options });
  },
  penMove: async (samples: PointerSample[]): Promise<void> => {
    await invoke("pen_move", { samples });
  },
  penUp: () => invoke<DocumentView>("pen_up"),
  selectAll: (part: { layer: string | null; art: Art }) =>
    invoke<DocumentView>("select_all", { part }),
  clearSelection: () => invoke<DocumentView>("clear_selection"),
  deleteSelection: () => invoke<DocumentView>("delete_selection"),
  nudgeSelection: (dx: number, dy: number) =>
    invoke<DocumentView>("nudge_selection", { dx, dy }),
  setOnionSkin: async (onion: OnionSkin): Promise<void> => {
    await invoke("set_onion_skin", { onion });
  },
  setCurrentFrame: (frame: number) =>
    invoke<DocumentView>("set_current_frame", { frame }),
  editTiming: (edit: TimingEdit, layer: string | null) =>
    invoke<DocumentView>("edit_timing", { edit, layer }),
  /** Shows the Drawing or Camera view. Null while the canvas isn't ready. */
  setViewMode: (mode: ViewMode, layer: string | null) =>
    invoke<ViewInfo | null>("set_view_mode", { mode, layer }),
  setHandlesVisible: async (visible: boolean): Promise<void> => {
    await invoke("set_handles_visible", { visible });
  },
  editSelection: (edit: SelectionEdit) =>
    invoke<DocumentView>("edit_selection", { edit }),
  closeWindow: async (): Promise<void> => {
    await invoke("close_window");
  },
  /** The core asks before closing a window with unsaved changes. */
  onCloseRequested: (handler: () => void): Promise<UnlistenFn> =>
    listen("close-requested", handler),
};
