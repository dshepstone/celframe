# Studio layout

The Draw mode window, designed from general animation practice: drawing tools on one side, the scene in the middle, layer and colour controls on the other side, and timing along the bottom. The arrangement and look are our own.

## Regions

| Region      | Position        | Phase 1 contents                                                                                  |
| ----------- | --------------- | ------------------------------------------------------------------------------------------------- |
| Menu bar    | Top             | File, Edit, Layer and View menus, project name, mode switcher, undo and redo, command search      |
| Shelves     | Under the menus | The user's own shelves of actions, specialty tools and, later, scripts. See [shelves](shelves.md) |
| Tool rail   | Left            | Pen. Eraser, Fill, Select, Pan and Zoom are shown but disabled, each naming when it arrives       |
| Canvas      | Centre          | The native canvas, which must stay see-through in the page (ADR-0001)                             |
| Side panels | Right           | Tool properties (every setting of the current tool), Layers, and Colour (swatches, custom)        |
| Timeline    | Bottom          | Layers against frames, showing each exposure and its drawing number. Read-only until Phase 3      |
| Status bar  | Very bottom     | Scene size, frame rate, current frame, target layer, stroke count, file and save state            |

## Rules

- Every control either works or says when it will. No silent placeholders.
- Every setting of a tool lives in the Tool properties panel. The bar under the menus is kept for shelves, not tool settings.
- The side panels and the timeline can be hidden from the View menu, giving the canvas the space.
- Layers are listed top layer first in both the Layers panel and the timeline.
- The layer to draw on is the one chosen in the Layers panel. With none chosen, it's the topmost visible, unlocked layer. Drawing on a hidden or locked chosen layer is refused with a message.
- New layers get their IDs from the core, never from the UI.
- Pen size is in scene units at full pressure, from 0.5 to 200.
- All menus, panels and controls work from the keyboard. Layer rows rename with F2 or a double-click.
