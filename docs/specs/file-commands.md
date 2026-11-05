# File commands

This spec defines how a person creates, opens and saves `.celf` projects in the desktop app.

## Commands

| Command | Shortcut         | What it does                                                                 |
| ------- | ---------------- | ---------------------------------------------------------------------------- |
| New     | Ctrl/Cmd+N       | Replaces the open project with an empty "Untitled" project.                  |
| Open    | Ctrl/Cmd+O       | Asks for a `.celf` file and opens it.                                        |
| Save    | Ctrl/Cmd+S       | Saves to the project's file. A project that was never saved acts as Save As. |
| Save As | Ctrl/Cmd+Shift+S | Asks for a location, saves there, and makes that the project's file.         |

All four are in the command palette. A menu bar is out of scope for Phase 1.

## Ownership

The Rust core owns the open file path, just as it owns the document. File pickers open from
the core, so the UI never sends a path. The UI learns the current file from the document view.

## Unsaved changes

A project has unsaved changes when its undo history isn't at the last saved state. Undoing
back to that state makes the project clean again.

Before New, Open or closing the window discards unsaved changes, the app asks:

> Save changes to "Project name" before …?

- **Save** saves first. If saving fails or is cancelled, the original action doesn't happen.
- **Don't save** continues and discards the changes.
- **Cancel**, or Escape, does nothing.

## Saving

- Saves are atomic, as `docs/file-format.md` describes: a failed save leaves the old file intact.
- A Save As path with no extension gets `.celf`. If a file with that new name already exists,
  the save is refused rather than replacing a file the person didn't pick.
- Save writes to the project's file at exactly its path, even if it has no extension.
- After a successful save, undo history is kept and the project is clean.

## Opening

- Opening replaces the project and starts a fresh undo history.
- If a file can't be opened, the current project stays open, unchanged, and an error names the
  file and the reason.

## Errors

Errors appear in the app's alert area and say which file and what went wrong, in plain words.
A cancelled file picker isn't an error.
