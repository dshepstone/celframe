# Autosave and crash recovery

This spec defines how Celframe protects unsaved work if the app closes unexpectedly.

## Autosave

- Every 30 seconds, if the project has unsaved changes that haven't been autosaved yet, the app
  writes a copy to a recovery folder inside the app's data folder.
- Autosave never writes to the person's own file. Only Save and Save As do that.
- Each autosave replaces the previous one. Writes are atomic, as `docs/file-format.md` describes,
  so a crash during an autosave leaves the previous autosave intact.
- A failed autosave is retried at the next interval. It doesn't interrupt the person.

## When the recovery copy is removed

- When the project has no unsaved changes at an autosave check, for example after a save or after
  undoing back to the saved state.
- When the app exits with no unsaved changes, or after choosing **Don't save** when closing. An
  exit that skips the close prompt, such as quitting from the macOS app menu, keeps the copy.

So a recovery copy left at launch means the app didn't exit normally while there was unsaved
work.

## Recovery at launch

If a recovery copy exists at launch, the app asks before anything else:

> Celframe closed before your changes were saved. Restore your unsaved work from (time)?

- **Restore** opens the copy as unsaved work. Its file is the file it was last saved to, if any,
  so Save writes there and Save As works as usual. Undo history starts fresh.
- **Discard** deletes the copy and keeps the new empty project.

There is no other way to dismiss the prompt. Autosave doesn't run until the person chooses, so
the copy can't be overwritten before they decide.

If the copy can't be opened, the app says why and the prompt stays open. **Discard** deletes the
copy. Closing Celframe instead keeps it, so a later version can try again.

## Limits

- One recovery copy is kept, for the one open project.
- Running two copies of Celframe at once isn't supported yet. They would share the recovery copy.
