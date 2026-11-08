# Exposure sheet

A sheet of **rows for frames and columns for layers**, the traditional way to plan and time hand-drawn animation. It shows the same exposures as the timeline, so a change in one appears in the other at once. Step 4 of the [production plan](../plans/production-features.md).

## Layout

- The sheet is a dock panel, **Exposure sheet**, opened by default as a tab beside the Timeline. Like any panel it can be dragged anywhere (see [docking](docking.md)).
- The first column is the **frame number**. Then there's one column per layer, top layer first, matching the Layers panel. Hidden layers' columns are dimmed.
- Rows run from frame 1 to 24 frames past the scene's end, or past the current frame if that's later.
- Every 12th row is ruled with a lighter line and every 24th row with a heavier one, so seconds and half-seconds stand out at the scene's usual frame rates.
- The **current frame's row** is highlighted, and the **current cell** (current layer at the current frame) has an accent outline.

## Cells

- A cell where an exposure starts shows its **drawing number**.
- Following cells of the same exposure show a **hold line** down the middle of the cell.
- Empty cells are blank.

## Moving

- Clicking a cell makes its layer current and its frame current. Clicking a frame number only changes the frame.
- With the sheet focused, the arrow keys move the current cell: up and down change the frame, left and right change the layer.

## Editing

All edits work on the current cell, are one undo step each, and refuse hidden or locked layers.

| Action                | How                                                  | Result                                                                                                                                                                                    |
| --------------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Expose a drawing      | Type its number, then Enter                          | The drawing starts at this frame. If a hold was running through it, the hold ends just before, and the new drawing holds for the rest of that hold. On an empty frame it lasts one frame. |
| New drawing by number | Type the next unused number, then Enter              | Creates a new empty drawing and exposes it as above.                                                                                                                                      |
| Clear the frame       | `Delete` or `Backspace`                              | The frame becomes empty. Frames before and after keep their drawings and nothing moves.                                                                                                   |
| Timing edits          | The timeline's transport buttons and `Alt` shortcuts | Same as in the timeline: new drawing, hold longer or shorter, insert or delete frame.                                                                                                     |

- Typing a number above the next unused one, or anything that isn't a whole number, is refused with a message and changes nothing.
- `Escape` abandons what's been typed.
- After Enter, the current cell moves down to the next frame, so a sequence can be typed quickly.

## Not yet

Selecting ranges of cells, reversing, and notes and dialogue columns come in a later step.
