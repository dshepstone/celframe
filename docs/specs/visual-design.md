# Visual design

Celframe should feel like a precise working instrument: calm, dense and fast to read, so the artwork is the brightest and most colourful thing on screen. This spec sets the rules every part of the interface follows. The design is our own, worked out from general principles of tool design, not from any other application.

## Principles

1. **The artwork leads.** The interface is neutral graphite. Colour is used only to show state: the current tool, the selection, the active layer, focus.
2. **One accent, used sparingly.** Teal marks what is active. Large blocks of accent colour are avoided; an active item gets a tinted background and a thin accent edge instead.
3. **Dense but legible.** Controls are compact, so more of the screen goes to the canvas. Text never drops below 11 px.
4. **Quiet structure.** Regions are separated by 1 px dividers and small changes in surface tone, not boxes and shadows.
5. **Numbers are precise.** Numeric values use tabular figures, align to the right, and can be changed by dragging as well as typing.

## Colour

Tokens live in `packages/ui/src/tokens.ts`. Both themes must pass the WCAG AA tests there: 4.5:1 for text and 3:1 for controls.

| Token                | Role                                                        |
| -------------------- | ----------------------------------------------------------- |
| `surface`            | Panels and bars                                             |
| `surface-raised`     | Panel headers, menus, popovers                              |
| `surface-sunken`     | Fields, the tool rail, wells                                |
| `surface-hover`      | Hovered rows and buttons                                    |
| `canvas-backdrop`    | Around the paper. Mid-dark, so the paper's edge stays clear |
| `accent`             | Active edges, check marks, focused slider thumbs            |
| `accent-muted`       | Background of active and selected items                     |
| `text`, `text-muted` | Labels and values; secondary labels                         |
| `border`             | Dividers between regions                                    |
| `control-border`     | Edges of fields and buttons                                 |

Dark is the default theme. Light uses the same structure with light greys.

## Type

- The system UI font, at 12 px for labels and values and 11 px for panel titles, hints and the status bar.
- Panel titles are 11 px, semibold, uppercase, with slight letter spacing.
- Numbers use tabular figures everywhere.
- Interface text can't be selected by dragging, except inside fields.

## Size and spacing

- Spacing follows a 4 px grid.
- Bars: the menu row is 32 px tall, the shelf bar 34 px, the status bar 22 px.
- Controls are 24 px tall; icon buttons are 28 px square on bars and 24 px in panels; tool rail buttons are 32 px.
- Corners are 3 px on controls and 4 px on popovers. Nothing is pill-shaped.
- Icons are 16 px line drawings with a consistent stroke, drawn for Celframe.

## Components

- **Panels** have a 26 px header with a title and a chevron that collapses the panel. A collapsed panel keeps only its header. Which panels are collapsed is remembered.
- **The side panel** can be resized by dragging its left edge, from 220 to 420 px. Its width is remembered.
- **Tool rail buttons** show the active tool with an `accent-muted` background and a 2 px accent bar on the left. Tooltips give the tool's name and shortcut.
- **Tabs** (mode switcher, shelves) are flat text. The active tab has `text` colour and a 2 px accent underline; others are muted.
- **Fields** are 24 px tall on `surface-sunken` with a `control-border` edge, text aligned right for numbers.
- **Scrub labels:** dragging a numeric field's label left or right changes its value, 1 unit per pixel, 10 with Shift and 0.1 with Alt. The value is applied once, when the drag ends, so it's one undo step.
- **Menus** use 24 px rows, with shortcuts right-aligned in muted text. The highlighted row uses `accent-muted`.
- **Lists** (layers) use 26 px rows. The selected row has `accent-muted` and a 2 px accent bar on the left.
- **The timeline** draws exposures as muted accent blocks with the drawing number, and marks the current frame with an accent line.

## Motion

Hover and press changes are instant or under 100 ms. Nothing bounces or slides.
