# Dockable panels

Every panel lives in a **tab**, and tabs live in **dock areas** around the canvas. People arrange them to suit their work, and Celframe remembers the arrangement. Step 1 of the [production plan](../plans/production-features.md).

## Areas and stacks

- There are three dock areas: **left** (between the tool rail and the canvas), **right**, and **bottom** (under the canvas).
- Each area holds one or more **stacks**. Stacks in the left and right areas sit one above another; stacks in the bottom area sit side by side.
- A stack has a **tab strip** and shows its active tab's panel below it.
- An area with no stacks takes no space. A stack with no tabs is removed.
- The centre is the canvas. Its own tabs (Drawing and Camera views) arrive in step 2.

## Panels

| Panel           | Default place        |
| --------------- | -------------------- |
| Tool properties | Right, top stack     |
| Colour          | Right, middle stack  |
| Layers          | Right, bottom stack  |
| Exposure sheet  | Right, beside Layers |
| Timeline        | Bottom               |

Each panel appears once. Later panels, such as the exposure sheet and camera, join this list.

## Moving tabs

Tabs move by dragging with the mouse or pen:

- Dragging starts once the pointer has moved 4 pixels, so clicks still just select a tab.
- While dragging, the tab follows the pointer and the drop target is highlighted:
  - **A tab strip:** the tab joins that stack, at the position shown by an insertion line.
  - **A stack's panel:** the half nearest the pointer is highlighted. Dropping there splits the stack, putting a new stack before or after it: above or below in side areas, left or right in the bottom area.
  - **An empty area's edge:** while dragging, a drop strip appears along each empty area. Dropping there opens the area with a new stack.
- **Escape** cancels a drag. Dropping anywhere else leaves the tab where it was.
- Moving tabs isn't part of the document, so it isn't undoable; **Window → Reset layout** restores the default.

## Sizes

- The left and right areas' widths and the bottom area's height are changed by dragging their inner edge, or with arrow keys when the edge has focus. Side areas are 200 to 520 pixels wide; the bottom area 100 to 600 pixels high.
- The divider between two stacks in an area resizes them against each other. No stack gets smaller than 80 pixels.

## Window menu and keyboard

- **Window** lists every panel; choosing one shows it, opening it in the right area if it isn't in the layout. **Reset layout** restores the default arrangement.
- **View → Hide side panels / Hide timeline** hide the right and bottom areas without changing their contents.
- Tabs are reachable with Tab; arrow keys move between tabs in a strip, and Enter or Space shows one.

## Remembering

When a new version adds a panel or changes where one belongs, stored layouts gain it at its default place. The arrangement, sizes and active tabs are kept in local storage as a preference of the person using the computer, not in the project. Anything unreadable falls back to the default layout, and unknown panels are dropped.
