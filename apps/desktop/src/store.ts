import { applyTheme, type ThemeName } from "@celframe/ui";
import { create } from "zustand";
import {
  backend,
  type Command,
  type DocumentView,
  type Layer,
  type Art,
  type PenOptions,
  type SelectionEdit,
  type ShapeKind,
  type RecoveryInfo,
  type ViewChange,
  type ViewInfo,
} from "./backend";
import {
  type Shelf,
  loadShelves,
  moveAction,
  nextShelfName,
  storeShelves,
} from "./shelves";

const THEME_KEY = "celframe.theme";

function readSavedTheme(): ThemeName | null {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    return saved === "light" || saved === "dark" ? saved : null;
  } catch {
    return null;
  }
}

function saveTheme(theme: ThemeName): void {
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {
    // The choice just won't be remembered next time.
  }
}

export function initialTheme(): ThemeName {
  return (
    readSavedTheme() ??
    (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
  );
}

/** Actions that replace or close the document, so they ask about unsaved changes first. */
export type FileAction = "new" | "open" | "close";

export type UnsavedChoice = "save" | "discard" | "cancel";

export type Rgba = [number, number, number, number];

/** The tools that work so far. */
export type Tool = "pen" | "eraser" | "shape" | "select" | "hand" | "zoom";

/** The layer new strokes go to: the chosen one, or else the topmost visible, unlocked one. */
export function targetLayer(
  layers: readonly Layer[],
  chosen: string | null,
): Layer | null {
  const picked = chosen ? layers.find((l) => l.id === chosen) : undefined;
  if (picked) return picked;
  return [...layers].reverse().find((l) => l.visible && !l.locked) ?? null;
}

export interface AppState {
  document: DocumentView | null;
  error: string | null;
  /** Waiting on the unsaved-changes prompt. */
  pendingAction: FileAction | null;
  /** Unsaved work from an earlier run, waiting for Restore or Discard. */
  recovery: RecoveryInfo | null;
  theme: ThemeName;
  sidePanelOpen: boolean;
  timelineOpen: boolean;
  paletteOpen: boolean;
  brushWidth: number;
  brushColor: Rgba;
  /** The layer chosen in the Layers panel, if any. */
  activeLayer: string | null;
  /** The part of the drawing the pen draws on. */
  art: Art;
  /** The eraser's diameter in scene units. */
  eraserWidth: number;
  /** The eraser removes every stroke it touches instead of cutting it. */
  eraseWholeStrokes: boolean;
  shapeKind: ShapeKind;
  polygonSides: number;
  /** Width and height fields keep their ratio. */
  lockProportions: boolean;
  /** Scaling the selection also scales stroke widths. */
  scaleStrokeWidths: boolean;
  tool: Tool;
  /** Space is held, which pans with any tool. */
  spaceHeld: boolean;
  /** The canvas view, once the core has reported it. */
  view: ViewInfo | null;
  load: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  /** Ends a pen stroke and shows the document it produced. */
  finishStroke: () => Promise<void>;
  reportError: (error: unknown) => void;
  restoreRecovery: () => Promise<void>;
  discardRecovery: () => Promise<void>;
  requestFileAction: (action: FileAction) => Promise<void>;
  resolvePending: (choice: UnsavedChoice) => Promise<void>;
  save: () => Promise<void>;
  saveAs: () => Promise<void>;
  setTheme: (theme: ThemeName) => void;
  toggleTheme: () => void;
  setSidePanelOpen: (open: boolean) => void;
  setTimelineOpen: (open: boolean) => void;
  setPaletteOpen: (open: boolean) => void;
  setBrushWidth: (width: number) => void;
  setBrushColor: (color: Rgba) => void;
  setActiveLayer: (layer: string | null) => void;
  runCommand: (command: Command) => Promise<void>;
  addLayer: () => Promise<void>;
  penOptions: () => PenOptions;
  setTool: (tool: Tool) => void;
  setArt: (art: Art) => void;
  setEraserWidth: (width: number) => void;
  setEraseWholeStrokes: (whole: boolean) => void;
  setShapeKind: (kind: ShapeKind) => void;
  setPolygonSides: (sides: number) => void;
  toggleArt: () => void;
  selectAll: () => Promise<void>;
  clearSelection: () => Promise<void>;
  deleteSelection: () => Promise<void>;
  nudgeSelection: (dx: number, dy: number) => Promise<void>;
  editSelection: (edit: SelectionEdit) => Promise<void>;
  setLockProportions: (lock: boolean) => void;
  setScaleStrokeWidths: (scale: boolean) => void;
  setSpaceHeld: (held: boolean) => void;
  changeView: (change: ViewChange) => Promise<void>;
  /** The shelves under the menus. Null until loaded with `initShelves`. */
  shelves: Shelf[] | null;
  activeShelf: string | null;
  /** Loads stored shelves, keeping only actions in `known`. Does nothing once loaded. */
  initShelves: (known: ReadonlySet<string>) => void;
  setActiveShelf: (id: string) => void;
  addShelf: () => void;
  renameShelf: (id: string, name: string) => void;
  removeShelf: (id: string) => void;
  addToShelf: (id: string, action: string) => void;
  removeFromShelf: (id: string, index: number) => void;
  moveOnShelf: (id: string, index: number, by: number) => void;
}

export const BRUSH_WIDTH_MIN = 0.5;
export const BRUSH_WIDTH_MAX = 200;
export const POLYGON_SIDES_MIN = 3;
export const POLYGON_SIDES_MAX = 12;

export const useApp = create<AppState>((set, get) => {
  // The core owns the document, so the UI only ever stores what it sends back.
  // Resolves true when the core sent a new view, false on error or a cancelled picker.
  const sync = async (request: Promise<DocumentView | null>) => {
    try {
      const view = await request;
      if (view) set({ document: view, error: null });
      return view !== null;
    } catch (error) {
      set({ error: String(error) });
      return false;
    }
  };

  const perform = async (action: FileAction) => {
    if (action === "new") {
      await sync(backend.newDocument());
    } else if (action === "open") {
      await sync(backend.openDocument());
    } else {
      try {
        await backend.closeWindow();
      } catch (error) {
        set({ error: String(error) });
      }
    }
  };

  /** Replaces the shelves and stores them. */
  const putShelves = (shelves: Shelf[], activeShelf = get().activeShelf) => {
    storeShelves(shelves);
    set({ shelves, activeShelf });
  };
  const editShelf = (id: string, change: (shelf: Shelf) => Shelf) => {
    const shelves = get().shelves;
    if (shelves) putShelves(shelves.map((s) => (s.id === id ? change(s) : s)));
  };

  return {
    document: null,
    error: null,
    pendingAction: null,
    recovery: null,
    theme: "light",
    sidePanelOpen: true,
    timelineOpen: true,
    paletteOpen: false,
    brushWidth: 10,
    brushColor: [0, 0, 0, 255],
    activeLayer: null,
    art: "line",
    eraserWidth: 20,
    eraseWholeStrokes: false,
    shapeKind: "rectangle",
    polygonSides: 5,
    lockProportions: true,
    scaleStrokeWidths: false,
    tool: "pen",
    spaceHeld: false,
    view: null,
    load: async () => {
      await sync(backend.getDocument());
      try {
        set({ recovery: await backend.getRecovery() });
      } catch (error) {
        set({ error: String(error) });
      }
    },
    finishStroke: () => sync(backend.penUp()).then(() => undefined),
    reportError: (error) => set({ error: String(error) }),
    // On failure the prompt stays open, so the person can still discard the copy.
    restoreRecovery: async () => {
      if (await sync(backend.restoreRecovery())) set({ recovery: null });
    },
    discardRecovery: async () => {
      try {
        await backend.discardRecovery();
        set({ recovery: null, error: null });
      } catch (error) {
        set({ error: String(error) });
      }
    },
    undo: () => sync(backend.undo()).then(() => undefined),
    redo: () => sync(backend.redo()).then(() => undefined),
    requestFileAction: async (action) => {
      if (get().document?.dirty) {
        set({ pendingAction: action });
      } else {
        await perform(action);
      }
    },
    resolvePending: async (choice) => {
      const action = get().pendingAction;
      set({ pendingAction: null });
      if (!action || choice === "cancel") return;
      // A failed or cancelled save must not lose the changes.
      if (choice === "save" && !(await sync(backend.saveDocument()))) return;
      await perform(action);
    },
    save: () => sync(backend.saveDocument()).then(() => undefined),
    saveAs: () => sync(backend.saveDocumentAs()).then(() => undefined),
    setTheme: (theme) => {
      applyTheme(theme, document.documentElement);
      saveTheme(theme);
      set({ theme });
    },
    toggleTheme: () => {
      get().setTheme(get().theme === "dark" ? "light" : "dark");
    },
    setSidePanelOpen: (sidePanelOpen) => set({ sidePanelOpen }),
    setTimelineOpen: (timelineOpen) => set({ timelineOpen }),
    setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
    setBrushWidth: (width) =>
      set({
        brushWidth: Math.min(BRUSH_WIDTH_MAX, Math.max(BRUSH_WIDTH_MIN, width)),
      }),
    setBrushColor: (brushColor) => set({ brushColor }),
    setActiveLayer: (activeLayer) => {
      // A selection belongs to one layer and part.
      if (activeLayer !== get().activeLayer) void get().clearSelection();
      set({ activeLayer });
    },
    runCommand: async (command) => {
      await sync(backend.executeCommand(command));
      // A removed layer can't stay chosen.
      const layers = get().document?.project.scenes[0]?.layers ?? [];
      const active = get().activeLayer;
      if (active && !layers.some((l) => l.id === active)) {
        set({ activeLayer: null });
      }
    },
    addLayer: async () => {
      if (await sync(backend.addLayer())) {
        const layers = get().document?.project.scenes[0]?.layers ?? [];
        set({ activeLayer: layers[layers.length - 1]?.id ?? null });
      }
    },
    setTool: (tool) => set({ tool }),
    setSpaceHeld: (spaceHeld) => set({ spaceHeld }),
    changeView: async (change) => {
      try {
        set({ view: await backend.changeView(change) });
      } catch (error) {
        set({ error: String(error) });
      }
    },
    penOptions: () => {
      const state = get();
      const erasing = state.tool === "eraser";
      return {
        width: erasing ? state.eraserWidth : state.brushWidth,
        color: state.brushColor,
        layer: state.activeLayer,
        art: state.art,
        tool: erasing
          ? { type: "erase", wholeStrokes: state.eraseWholeStrokes }
          : state.tool === "shape"
            ? {
                type: "shape",
                shape:
                  state.shapeKind === "polygon"
                    ? { type: "polygon", sides: state.polygonSides }
                    : { type: state.shapeKind },
              }
            : state.tool === "select"
              ? { type: "select" }
              : { type: "draw" },
      };
    },
    setShapeKind: (shapeKind) => set({ shapeKind }),
    setPolygonSides: (sides) =>
      set({
        polygonSides: Math.min(
          POLYGON_SIDES_MAX,
          Math.max(POLYGON_SIDES_MIN, Math.round(sides)),
        ),
      }),
    setEraserWidth: (width) =>
      set({
        eraserWidth: Math.min(
          BRUSH_WIDTH_MAX,
          Math.max(BRUSH_WIDTH_MIN, width),
        ),
      }),
    setEraseWholeStrokes: (eraseWholeStrokes) => set({ eraseWholeStrokes }),
    shelves: null,
    activeShelf: null,
    initShelves: (known) => {
      if (get().shelves) return;
      const shelves = loadShelves(known);
      set({ shelves, activeShelf: shelves[0]?.id ?? null });
    },
    setActiveShelf: (activeShelf) => set({ activeShelf }),
    addShelf: () => {
      const shelves = get().shelves ?? [];
      const shelf: Shelf = {
        id: crypto.randomUUID(),
        name: nextShelfName(shelves),
        actions: [],
      };
      putShelves([...shelves, shelf], shelf.id);
    },
    renameShelf: (id, name) => {
      const trimmed = name.trim();
      if (trimmed) editShelf(id, (s) => ({ ...s, name: trimmed }));
    },
    removeShelf: (id) => {
      const shelves = get().shelves ?? [];
      if (shelves.length <= 1) return;
      const index = shelves.findIndex((s) => s.id === id);
      const rest = shelves.filter((s) => s.id !== id);
      const active =
        get().activeShelf === id
          ? (rest[Math.max(0, index - 1)]?.id ?? null)
          : get().activeShelf;
      putShelves(rest, active);
    },
    addToShelf: (id, action) =>
      editShelf(id, (s) =>
        s.actions.includes(action)
          ? s
          : { ...s, actions: [...s.actions, action] },
      ),
    removeFromShelf: (id, index) =>
      editShelf(id, (s) => ({
        ...s,
        actions: s.actions.filter((_, i) => i !== index),
      })),
    moveOnShelf: (id, index, by) =>
      editShelf(id, (s) => ({
        ...s,
        actions: moveAction(s.actions, index, by),
      })),
    setArt: (art) => {
      if (art !== get().art) void get().clearSelection();
      set({ art });
    },
    toggleArt: () => get().setArt(get().art === "line" ? "colour" : "line"),
    selectAll: async () => {
      const { activeLayer, art } = get();
      await sync(backend.selectAll({ layer: activeLayer, art }));
    },
    clearSelection: async () => {
      if ((get().document?.selection.length ?? 0) === 0) return;
      await sync(backend.clearSelection());
    },
    deleteSelection: async () => {
      if ((get().document?.selection.length ?? 0) === 0) return;
      await sync(backend.deleteSelection());
    },
    editSelection: async (edit) => {
      if ((get().document?.selection.length ?? 0) === 0) return;
      await sync(backend.editSelection(edit));
    },
    setLockProportions: (lockProportions) => set({ lockProportions }),
    setScaleStrokeWidths: (scaleStrokeWidths) => set({ scaleStrokeWidths }),
    nudgeSelection: async (dx, dy) => {
      if ((get().document?.selection.length ?? 0) === 0) return;
      await sync(backend.nudgeSelection(dx, dy));
    },
  };
});
