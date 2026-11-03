import { applyTheme, type ThemeName } from "@celframe/ui";
import { create } from "zustand";
import { backend, type DocumentView } from "./backend";

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

export interface AppState {
  document: DocumentView | null;
  error: string | null;
  theme: ThemeName;
  sidePanelOpen: boolean;
  paletteOpen: boolean;
  load: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  setTheme: (theme: ThemeName) => void;
  toggleTheme: () => void;
  setSidePanelOpen: (open: boolean) => void;
  setPaletteOpen: (open: boolean) => void;
}

export const useApp = create<AppState>((set, get) => {
  // The core owns the document, so the UI only ever stores what it sends back.
  const sync = async (request: Promise<DocumentView>) => {
    try {
      set({ document: await request, error: null });
    } catch (error) {
      set({ error: String(error) });
    }
  };

  return {
    document: null,
    error: null,
    theme: "light",
    sidePanelOpen: true,
    paletteOpen: false,
    load: () => sync(backend.getDocument()),
    undo: () => sync(backend.undo()),
    redo: () => sync(backend.redo()),
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
