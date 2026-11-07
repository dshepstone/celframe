# Workspaces and panels

How Celframe's interface grows to hold every feature in the inventory without crowding. It is designed from general interface practice for creative tools. The arrangement, look and names are our own.

## Principles

- **One mode, one workspace.** Draw, Animate, Rig and Composite each have their own default arrangement of panels, suited to the task. The mode switcher changes workspace.
- **Panels, not windows.** Every tool area is a panel that can be docked on any side, stacked as tabs, resized, collapsed or floated. Layouts are saved per workspace and can be reset.
- **The canvas stays central.** Panels never cover it by default. A focus mode (Tab) hides every panel.
- **Every control works or says when.** Panels for future phases appear in the Window menu marked with their phase, not as dead controls.
- **Discoverable and fast.** Every action is in a menu, has a palette entry and can take a shortcut.

## Default workspaces

| Workspace | Left            | Centre                  | Right                         | Bottom                 |
| --------- | --------------- | ----------------------- | ----------------------------- | ---------------------- |
| Draw      | Tool rail       | Canvas                  | Layers, Colour, Tool presets  | Timeline (compact)     |
| Animate   | Tool rail       | Canvas, Onion skin bar  | Layers, Exposure sheet        | Timeline (full), Audio |
| Rig       | Rig tools       | Canvas                  | Hierarchy, Deformers, Library | Timeline, Curve editor |
| Composite | Effects library | Node view, Canvas split | Properties                    | Timeline               |

## Panels

| Panel           | Phase | Purpose                                              |
| --------------- | ----- | ---------------------------------------------------- |
| Tool rail       | 1     | Pick the drawing tool                                |
| Tool properties | 2     | Every setting of the current tool                    |
| Layers          | 1     | Show, lock, order, rename and choose layers          |
| Colour          | 1     | Swatches and custom colours                          |
| Timeline        | 1, 3  | Read-only in Phase 1; full timing editing in Phase 3 |
| Palettes        | 2     | Named, linked project colours                        |
| Tool presets    | 2     | Saved brush and tool settings                        |
| Exposure sheet  | 3     | Timing by drawing number, frame by frame             |
| Onion skin      | 3     | Neighbouring-drawing display settings                |
| Audio           | 3     | Sound tracks and waveforms                           |
| Library         | 6     | Reusable drawings, parts, poses and templates        |
| Hierarchy       | 6     | The parent–child structure of a rig                  |
| Curve editor    | 6     | Keyframe timing and easing                           |
| Node view       | 7     | How layers, effects and outputs connect              |
| Properties      | 6, 7  | Values of the selected layer, node or keyframe       |
| Scripts         | 8     | Run and manage scripts                               |
| Review          | 9     | Comments and drawn notes                             |

## Look

- Neutral dark and light themes from our own design tokens, with one teal accent.
- Compact density by default, with a comfortable option for touch screens.
- Our own line icons on a 20-pixel grid.
- Panel headers in small capitals, with actions as icon buttons on the right.
