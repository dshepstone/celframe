# Transforming and arranging a selection

These edits work on the strokes picked with the Select tool. See [selection](selection.md). Every edit here is one undo step and keeps stroke IDs, colours and parts.

## Tool properties panel

When the Select tool is active, the **Tool properties** panel at the top of the side panel shows three groups. Controls are disabled while nothing is selected.

### Transform

| Field               | Meaning                                                                                |
| ------------------- | -------------------------------------------------------------------------------------- |
| X, Y                | The centre of the selection's box, in scene units                                      |
| W, H                | The box's width and height, in scene units                                             |
| Lock                | Keeps W and H in proportion when either is changed                                     |
| Rotate              | An angle in degrees, applied with **Apply** or Enter. Positive turns counter-clockwise |
| Scale stroke widths | When on, scaling also scales stroke widths by the average of the two scale factors     |

- The box is measured around the strokes' centrelines, not their painted width, so typed sizes are exact.
- Changing X or Y moves the selection. Changing W or H scales it about the box's centre.
- A box with no width or no height, such as a single horizontal line, can't be scaled along that axis. That field is disabled.
- Fields apply their value on Enter or when they lose focus. Invalid values are ignored and the field shows the current value again.

### Flip and rotate

| Button            | Result                                       |
| ----------------- | -------------------------------------------- |
| Flip horizontally | Mirrors left to right about the box's centre |
| Flip vertically   | Mirrors top to bottom about the box's centre |
| Rotate 90° left   | Turns 90° counter-clockwise about the centre |
| Rotate 90° right  | Turns 90° clockwise about the centre         |

### Arrange

Changes the drawing order of the selected strokes within their drawing. Colour art still draws below line art.

| Button         | Shortcut       | Result                                                      |
| -------------- | -------------- | ----------------------------------------------------------- |
| Bring to front | `Ctrl+Shift+]` | Selected strokes draw above all others, keeping their order |
| Bring forward  | `Ctrl+]`       | Each selected stroke moves above the next stroke over it    |
| Send backward  | `Ctrl+[`       | Each selected stroke moves below the next stroke under it   |
| Send to back   | `Ctrl+Shift+[` | Selected strokes draw below all others, keeping their order |

## Shortcuts for transforms

| Action            | Shortcut  |
| ----------------- | --------- |
| Flip horizontally | `Shift+H` |
| Flip vertically   | `Shift+V` |

## On-canvas handles

While the Select tool is active and something is selected, the selection's box shows handles. The box is drawn around the strokes' painted area.

| Handle     | Where                          | Dragging it                                |
| ---------- | ------------------------------ | ------------------------------------------ |
| Corner (4) | The box's corners              | Scales both ways about the opposite corner |
| Edge (4)   | The middle of each side        | Scales one way about the opposite side     |
| Rotate     | 24 screen pixels above the top | Turns about the box's centre               |

- **Shift** keeps proportions on corner handles and snaps rotation to 15° steps.
- **Alt** scales about the box's centre instead of the opposite side.
- Modifier keys take effect on the next pointer movement.
- Handles are 8 screen pixels across and can be grabbed from 5 screen pixels away. Handles win over strokes under them.
- Scaling past the opposite side flips the selection, as a mirror would.
- **Scale stroke widths** in the Tool properties panel applies to handle scaling too.
- The strokes follow the pointer while dragging. Releasing applies the change as one undo step. A drag that ends where it started changes nothing.
- Handles are hidden with every other tool.
