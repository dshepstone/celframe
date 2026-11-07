# Selection and moving

The Select tool picks strokes so they can be moved or deleted. Scaling, rotating, flipping and the clipboard build on this selection in later changes.

## What can be selected

- Strokes in the **current part**, line art or colour art, of the drawing shown on the current layer at the current frame. See [line art and colour art](line-and-colour-art.md).
- Hidden and locked layers can't be selected from.
- The selection isn't part of the document: it isn't saved and selecting isn't undoable. Strokes that stop existing, for example after an undo, leave the selection.
- Changing layer or part clears the selection.

## Picking

| Action                      | Result                                                      |
| --------------------------- | ----------------------------------------------------------- |
| Click a stroke              | Selects only that stroke                                    |
| Shift-click a stroke        | Adds it to the selection, or removes it if already selected |
| Drag from empty space       | Draws a box. Strokes with any point inside it are selected  |
| Shift-drag from empty space | Adds the strokes in the box to the selection                |
| Click empty space           | Clears the selection                                        |
| `Ctrl+A` / `Cmd+A`          | Selects every stroke in the current part                    |
| `Escape`                    | Clears the selection                                        |

- A click hits a stroke when it lands on the stroke or within 4 screen pixels of its edge. Where strokes overlap, the one drawn on top wins.

## Moving and deleting

- Dragging a selected stroke, or anywhere inside the selection's box, moves the whole selection. Dragging an unselected stroke selects it and moves it.
- The strokes follow the pointer while dragging. Releasing ends the move as one undo step. A drag that ends where it started changes nothing.
- Arrow keys move the selection by 1 scene unit, or 10 with Shift. Each press is one undo step.
- `Delete` or `Backspace` deletes the selected strokes as one undo step.

## Display

- The selection's bounding box, including stroke width, is outlined on the canvas in a blue line 1.5 screen pixels wide.
- A box being dragged out is outlined in the same blue, 1 screen pixel wide.
- The status bar shows how many strokes are selected.

## Controls

| Action      | Shortcut              | Elsewhere |
| ----------- | --------------------- | --------- |
| Select tool | `V`                   | Tool rail |
| Select all  | `Ctrl+A` / `Cmd+A`    | Edit menu |
| Deselect    | `Escape`              | Edit menu |
| Delete      | `Delete`, `Backspace` | Edit menu |
| Nudge       | Arrow keys (+ Shift)  | —         |
