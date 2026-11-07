# Canvas navigation

Zoom, pan and rotate change how the canvas shows the scene. They never change the document: they aren't undoable, aren't saved and don't mark the project as changed. The maths is the view transform in [units and coordinates](units-and-coordinates.md).

## Controls

| Action    | Mouse or pen                                   | Keyboard  | Elsewhere                       |
| --------- | ---------------------------------------------- | --------- | ------------------------------- |
| Zoom      | Wheel, anchored at the pointer                 | `+` / `-` | View menu, status bar `+` / `−` |
| Zoom tool | Click to zoom in at the pointer, Alt-click out | `Z`       | Tool rail                       |
| Pan       | Middle-drag, or drag while holding Space       | Space     | Pan tool                        |
| Pan tool  | Drag                                           | `H`       | Tool rail                       |
| Rotate    | —                                              | `,` / `.` | View menu, status bar ⟲ / ⟳     |
| Fit       | —                                              | `0`       | View menu, status bar Fit       |
| Pen       | Draw                                           | `P`       | Tool rail                       |

- Zoom steps are 25% per key press and 15% per wheel notch.
- Rotation steps are 15°. Rotation is shown in degrees from 0 to 359.
- Zoom and rotation keep the scene point under the anchor still. Keyboard and button changes anchor at the middle of the canvas.
- Zoom is limited to 0.01 to 256 device pixels per scene unit.
- Single-key shortcuts are ignored while typing in a text field.
- Releasing Space, or the window losing focus, ends Space-panning.

## Behaviour

- The view starts fitted: the whole scene frame fills 90% of the canvas area. Fit returns to this, and resets rotation.
- Once the view has been changed, it stays put when the window or panels resize. Only Fit refits it.
- Pen strokes go through the same view, so a stroke lands where it's drawn at any zoom or rotation.
- The status bar shows zoom as a percentage of one screen pixel per scene unit, and the rotation.
