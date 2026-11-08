# Timing basics

The current frame, the playhead, playback and the edits that change when drawings appear. Step 3 of the [production plan](../plans/production-features.md). The exposure sheet (step 4) uses the same frame and edits.

Frames are numbered from 1 in the interface; the document stores them from 0.

## Current frame

- The project has one **current frame**. Every view, tool and edit works at it: the views show what is exposed there, and a new stroke goes into the drawing exposed there on the current layer.
- Drawing on an empty frame creates a new drawing exposed for one frame, in the same undo step as the stroke.
- The current frame isn't saved in the project and changing it isn't undoable.
- The **scene length** runs to the end of the last exposure on any layer, and is at least 1 frame. The current frame can be anywhere from frame 1 to the scene length plus 100 frames, so there's room to keep drawing.

## Moving through frames

| Action         | Shortcut      | Elsewhere                                  |
| -------------- | ------------- | ------------------------------------------ |
| Previous frame | `Alt+←`       | Timeline transport                         |
| Next frame     | `Alt+→`       | Timeline transport                         |
| First frame    | `Home`        | Timeline transport                         |
| Last frame     | `End`         | Timeline transport: the scene's last frame |
| Go to a frame  | —             | Click a frame number in the timeline       |
| Play or stop   | `Shift+Space` | Timeline transport                         |

- Playback runs at the scene's frame rate from the current frame. With **Loop** on (the default) it wraps from the last frame to the first; with Loop off it stops on the last frame.
- Any edit, or choosing a tool on the canvas, stops playback.

## Timing edits

Each works on the current layer at the current frame and is one undo step. A hidden or locked layer can't be edited.

| Edit         | Shortcut | Result                                                                                                                          |
| ------------ | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| New drawing  | `Alt+N`  | An empty drawing exposed at the current frame for 1 frame. A drawing held over this frame keeps the frames before and after it. |
| Hold longer  | `Alt+=`  | The exposure at the current frame lasts one frame more; later exposures move one frame later.                                   |
| Hold shorter | `Alt+-`  | The exposure at the current frame lasts one frame less, never less than 1; later exposures move one frame earlier.              |
| Insert frame | `Alt+I`  | Adds an empty frame at the current frame; everything from here moves one frame later. A hold across it is split around the gap. |
| Delete frame | `Alt+D`  | Removes the current frame from the layer; everything after it moves one frame earlier.                                          |

- Holding longer or shorter on an empty frame changes nothing.
- Drawings no exposure uses any more stay in the layer, so undoing brings everything back exactly.

## Timeline

- Frame numbers along the top are clickable and show the **playhead**: the current frame's column is highlighted with an accent line through every row.
- A transport bar above the rows holds first, previous, play or stop, next and last, the frame counter, **Loop**, and buttons for the five timing edits.
- The status bar shows the current frame and the scene length.
