# Plan: production features

Requested on 2026-11-07. Each step is built, tested and merged on its own, and **each needs the project owner's approval before work starts**. Steps are ordered so each builds on what comes before it. Later requests will be added here.

Everything is designed from general animation practice, in Celframe's own words and look. See [CLEANROOM.md](../../CLEANROOM.md).

## Overview

| Step | Feature                                                    | Depends on  | Owner decisions needed           |
| ---- | ---------------------------------------------------------- | ----------- | -------------------------------- |
| 1    | Dockable, tabbed panel areas                               | —           | —                                |
| 2    | Drawing view and Camera view tabs, resolution gate         | 1           | —                                |
| 3    | Timing basics: current frame, playhead, editable exposures | 1           | —                                |
| 4    | Exposure sheet                                             | 1, 3        | —                                |
| 5    | Keyframes for animated values                              | 3           | File format version 3            |
| 6    | Scene camera with Z depth and camera moves                 | 2, 5        | —                                |
| 7    | Multiplane: keyable layer depth                            | 5, 6        | —                                |
| 8    | Pegs and the node view                                     | 5, 7        | File format version 4            |
| 9    | Bitmap layers and image assets                             | —           | File format version 5            |
| 10   | Scanning                                                   | 9           | Scanner libraries on each system |
| 11   | Camera capture: webcams and DSLRs                          | 9           | DSLR support on Windows          |
| 12   | Peg-hole registration                                      | 9, 10 or 11 | —                                |

## 1. Dockable, tabbed panel areas

Every region of the window (left, right, bottom, and the centre) is a **dock area** that holds one or more **panel tabs**: Tool properties, Layers, Colour, Timeline, and later Exposure sheet, Camera and others.

- Tabs can be reordered within an area and dragged to another area, or onto an area's edge to split it.
- Areas can be resized by dragging the dividers between them. An area with no tabs closes.
- The arrangement is remembered, and **Window → Reset layout** restores the default.
- Workspaces from [workspaces.md](../specs/workspaces.md) become saved arrangements.
- Done when: every current panel lives in a tab, can be moved anywhere, and the layout survives a restart.

## 2. Drawing view and Camera view

The centre area gets two tabs:

- **Drawing view:** works on one drawing on its paper, as today. The camera and other layers don't move it.
- **Camera view:** shows the scene as the camera sees it, with the **resolution gate**, a frame outlining exactly what will be rendered. Outside the gate the scene stays visible, dimmed, so you can zoom out well beyond the frame or right in close.
- Both views keep their own zoom, pan and rotation. Drawing works in both.
- Done when: both tabs work side by side, and the gate matches the scene's resolution at every zoom.

## 3. Timing basics

The groundwork both the timeline and the exposure sheet need:

- A **current frame**, shown as a playhead, moved by clicking, dragging or the keyboard.
- New drawings, and extending, shortening, moving, inserting and deleting exposures, all as undoable commands.
- Playback at the scene frame rate, with looping.
- Done when: a simple animation can be drawn frame by frame, retimed and played.

## 4. Exposure sheet

A **column per layer, row per frame** panel that sits alongside the timeline. Both show the same data, so a change in one appears in the other at once.

- Each cell shows the drawing number where a drawing starts and a hold line while it continues.
- Type a drawing number into a cell to expose that drawing; select cells to hold, insert, delete or reverse frames.
- Rows are marked every second and half second, with optional notes and dialogue columns.
- Done when: a scene can be timed entirely from the exposure sheet.

## 5. Keyframes

A general system for values that change over time: positions, scales, rotations, depths, opacities.

- A keyable value stores keys at frames, with linear, eased or stepped interpolation between them.
- Keys appear in the timeline and can be added, moved and deleted as undoable commands.
- This changes the file format to **version 3**, with a migration from version 2.
- Done when: a test value can be keyed, interpolated, saved and reopened.

## 6. Scene camera

A camera object belonging to each scene:

- Position, **Z depth** (distance from the scene), rotation and field size, all keyable.
- Camera moves are keys on these values: trucks in and out (Z), pans, tilts, rotations and zooms.
- The Camera view shows its gate; the Drawing view ignores it. Camera controls get their own panel tab.
- Done when: a truck-in with an eased pan can be keyed and played back in the Camera view.

## 7. Multiplane

Layers placed at different distances from the camera, for parallax:

- Each layer has a keyable **Z depth**. Moving the camera makes nearer layers move more than farther ones.
- A side view in the Camera panel shows layers by depth, so they can be dragged nearer or farther.
- Done when: a camera pan over three layers at different depths shows correct parallax.

## 8. Pegs and the node view

The structure of a scene shown and edited as a graph of connected nodes, alongside the Layers panel and timeline:

- **Pegs:** transform nodes with keyable position, rotation, scale, skew and pivot. Layers and other pegs hang from a peg and move with it, so a character, a prop or a whole group can be animated together. Pegs nest.
- **Node view:** a dock panel where every layer, peg, group, composite and the scene's output is a node, connected by wires that show what feeds what. Nodes can be added, wired, rewired, grouped and arranged; the graph scrolls and zooms.
- The Layers panel, timeline and exposure sheet show the same structure, so a change in one shows in the others.
- Later node types such as effects, masks and deformers plug into the same view.
- This changes the file format to **version 4**, with a migration that connects existing layers straight to the scene's output.
- Done when: a character's layers under a peg can be rewired in the node view and animated by keying the peg.

## 9. Bitmap layers and image assets

The base that scanning and capture need:

- Layers can hold bitmap drawings as well as vector ones. Images are stored in the `.celf` file's `assets/` folder.
- Import images and numbered image sequences, one image per drawing.
- This changes the file format to **version 5**.
- Done when: an image sequence can be imported, timed, saved and reopened.

## 10. Scanning

Bring paper drawings in from a scanner:

- **Import scans** from files or a watched folder, which works with any scanner and its own software.
- **Scan directly** from a scanner through each system's own scanning service: WIA on Windows, Image Capture on macOS, SANE on Linux. Batch scanning feeds one drawing per sheet.
- **Decision needed:** linking to SANE on Linux, which is GPL-licensed with an exception for programs that use it. Windows and macOS use built-in system services.
- Done when: a stack of sheets can be scanned into a layer, one drawing per sheet.

## 11. Camera capture

Shoot paper animation, or stop motion, with a camera on a stand:

- **Webcams** and capture devices through the system's camera support, with no extra libraries.
- **DSLR and mirrorless cameras**: live view, focus check and full-resolution capture, through **libgphoto2** (LGPL) on macOS and Linux.
- A capture view shows live video with onion skin of the previous frames, so each new drawing lines up.
- **Decision needed:** DSLR support on Windows. Camera makers' own control kits are proprietary and can't ship in GPL software. The options are libgphoto2 on Windows (less mature there), or the camera's own webcam mode.
- Done when: a sequence can be shot from a webcam and from a DSLR into a layer.

## 12. Peg-hole registration

Paper animation stays in register through the punched peg holes along each sheet's edge. Celframe finds them and lines every frame up:

- Detects the **Acme punch**, the standard animation peg layout: a round centre hole with a long slot on each side, at the top or bottom of the sheet. Other layouts can be added later.
- Works on scans and camera captures, and corrects shift, rotation and small scale changes so every drawing sits exactly in place.
- Shows the detected holes for checking, with manual adjustment when a hole is damaged or hidden.
- Done when: a scanned or shot sequence on punched paper plays back without any jitter from paper movement.
