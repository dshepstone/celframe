import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Mirrors the view sent by the Rust core. The core stays the source of truth.
export interface Project {
  id: string;
  name: string;
  scenes: { id: string; name: string }[];
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
  closeWindow: async (): Promise<void> => {
    await invoke("close_window");
  },
  /** The core asks before closing a window with unsaved changes. */
  onCloseRequested: (handler: () => void): Promise<UnlistenFn> =>
    listen("close-requested", handler),
};
