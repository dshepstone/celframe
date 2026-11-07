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
}

export interface Scene {
  id: string;
  name: string;
  width: number;
  height: number;
  frameRate: number;
  /** Bottom layer first. */
  layers: Layer[];
}

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
  | { type: "moveLayer"; scene: string; layer: string; index: number };

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
  /** Draw, or erase with `width` as the eraser's diameter. */
  tool: { type: "draw" } | { type: "erase"; wholeStrokes: boolean };
}

export interface DocumentView {
  project: Project;
  canUndo: boolean;
  canRedo: boolean;
  dirty: boolean;
  /** The project's file, or null if it has never been saved. */
  path: string | null;
}

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
  closeWindow: async (): Promise<void> => {
    await invoke("close_window");
  },
  /** The core asks before closing a window with unsaved changes. */
  onCloseRequested: (handler: () => void): Promise<UnlistenFn> =>
    listen("close-requested", handler),
};
