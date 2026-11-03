import { invoke } from "@tauri-apps/api/core";

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
}

export const backend = {
  getDocument: () => invoke<DocumentView>("get_document"),
  undo: () => invoke<DocumentView>("undo"),
  redo: () => invoke<DocumentView>("redo"),
};
