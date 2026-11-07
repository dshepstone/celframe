# Shelves

Shelves are the bar under the menus. Each shelf is a named row of buttons the user puts together for their own way of working: favourite actions, specialty tools such as one-click shape presets, and, once scripting exists, scripts. Tool settings never go here; they live in the Tool properties panel.

## Actions

- Every button runs an **action** from Celframe's action list. The command palette (`Ctrl+K`) searches the same list, so anything that can be found there can be put on a shelf.
- Actions are grouped: Tools, Shapes, Edit, Selection, Art, View, Layer and File.
- A button shows the action's icon, or its name when it has none. Its tooltip gives the name and shortcut.
- A button is disabled when its action can't run now, such as Flip with nothing selected.
- Script buttons arrive with scripting, in a later phase.

## Shelves

- Shelves appear as tabs at the left of the bar. Clicking a tab shows its buttons.
- **+** adds a shelf called "Shelf N" and shows it.
- Double-clicking a tab, or pressing F2 on it, renames it. Enter keeps the name, Escape cancels, and an empty name is ignored.
- **Customise** switches the bar into editing:
  - **Add action** opens a searchable list of every action. Choosing one adds it to the end of the shelf. An action can be on a shelf once.
  - Each button gets **move left**, **move right** and **remove** controls.
  - **Delete shelf** removes the current shelf. The last shelf can't be deleted.
  - **Done** leaves editing.

## Defaults and storage

- New installs start with two shelves:
  - **Quick:** Undo, Redo, Flip horizontally, Flip vertically, Rotate 90° left, Rotate 90° right, Bring to front, Send to back.
  - **Shapes:** Line, Rectangle, Ellipse and Polygon, each switching to the Shape tool with that shape.
- Shelves are a preference of the person using the computer, not part of a project. They're kept in the app's local storage and aren't saved in `.celf` files.
- If the stored shelves are missing or unreadable, the defaults are used. Actions that no longer exist are dropped when shelves load.
