import { applyTheme, type ThemeName } from "@celframe/ui";
import { create } from "zustand";
import { backend, type DocumentView, type RecoveryInfo } from "./backend";

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

export interface AppState {
  document: DocumentView | null;
  error: string | null;
  /** Waiting on the unsaved-changes prompt. */
  pendingAction: FileAction | null;
  /** Unsaved work from an earlier run, waiting for Restore or Discard. */
  recovery: RecoveryInfo | null;
  theme: ThemeName;
  sidePanelOpen: boolean;
  paletteOpen: boolean;
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
  setPaletteOpen: (open: boolean) => void;
}

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

  return {
    document: null,
    error: null,
    pendingAction: null,
    recovery: null,
    theme: "light",
    sidePanelOpen: true,
    paletteOpen: false,
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
    setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  };
});
