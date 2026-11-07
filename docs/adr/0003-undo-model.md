# ADR-0003: Undo model

Status: accepted

## Context

Every edit must be undoable, and the Rust core is the single source of truth. Commands also need to cross Tauri IPC from the UI. A pen stroke arrives as many small updates but must undo as one step.

## Decision

- **Commands are a serializable enum.** The UI sends them as tagged JSON, and adding one forces every `match` to handle it.
- **Each command returns its inverse when applied.** Undo applies the inverse, which returns the command for redo. No full document snapshots are stored.
- **A failed command changes nothing.** It returns an error, and history stays as it was.
- **Stroke merging is explicit.** `begin_stroke` opens a merge for one stroke, and appending or truncating samples of that stroke keeps only the original undo entry. Truncating lets a shape redraw itself while it's dragged. Undoing removes the whole stroke, and that removal captures the finished stroke for redo.
- **A merge closes** on `end_stroke`, on undo or redo, and on any other command.
- **Dirty state uses state IDs.** Each edit gets a new ID, so undoing back to the saved point reads as clean, while a saved point lost to a new edit stays dirty.

## Consequences

- Memory use grows with the number of edits, not with document size. There's no history limit yet.
- Layer locks aren't enforced in commands, so undo can't get stuck when a layer was locked after an edit. The UI enforces locks.
- Commands need IDs chosen by the caller, so redo restores the same IDs. Undo history isn't saved in files.
