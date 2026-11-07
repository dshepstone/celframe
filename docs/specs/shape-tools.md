# Shape tools

The Shape tool draws clean geometric strokes: lines, rectangles, ellipses and regular polygons. Each shape is an ordinary stroke, so the eraser, undo, line and colour art, and saving all work on it as on a pen stroke.

## Shapes

| Shape     | Dragging from A to B draws                                                 |
| --------- | -------------------------------------------------------------------------- |
| Line      | A straight line from A to B                                                |
| Rectangle | A rectangle with opposite corners A and B                                  |
| Ellipse   | An ellipse that fits the rectangle from A to B                             |
| Polygon   | A regular polygon that fits the rectangle from A to B, a corner at the top |

- Polygons have 3 to 12 sides, set in the Tool properties panel. The default is 5.
- Ellipses are made of straight segments about 4 scene units long, with at least 32 and at most 256.
- Shapes use the pen's size and colour, at full pressure, so their width is even.
- Closed shapes end where they start.

## Modifier keys

Hold these while dragging. They take effect on the next pointer movement.

| Key   | Line                        | Rectangle, ellipse, polygon   |
| ----- | --------------------------- | ----------------------------- |
| Shift | Angle snaps to steps of 15° | Width and height become equal |
| Alt   | A is the middle of the line | A is the centre of the shape  |

## Drawing

- The shape shows while dragging and is final when the pointer is released.
- A drag that hasn't moved from where it started draws nothing and adds no undo step.
- One drag is one undo step.
- Shapes go on the current part and layer, like the pen. See [line art and colour art](line-and-colour-art.md).

## Controls

| Action     | Shortcut | Elsewhere                                                |
| ---------- | -------- | -------------------------------------------------------- |
| Shape tool | `U`      | Tool rail                                                |
| Shape      | —        | Tool properties panel: Line, Rectangle, Ellipse, Polygon |
| Sides      | —        | Tool properties panel, for polygons                      |
