# Scene camera

Every scene has one camera, and the Camera view looks through it. Camera moves are keys on its values. Step 6 of the [production plan](../plans/production-features.md).

## The camera's values

All five are keyable with the [keyframe](keyframes.md) system: linear, ease or step between keys.

| Value    | Meaning                                                                                    | Default | Range             |
| -------- | ------------------------------------------------------------------------------------------ | ------- | ----------------- |
| Pan X, Y | The scene point at the middle of the frame, in scene units, +Y up                          | 0, 0    | ±1,000,000        |
| Distance | How far the camera is from the scene, along Z. Smaller is closer.                          | 1000    | 1 to 100,000      |
| Rotation | Degrees, counter-clockwise. Turning the camera turns the picture the other way.            | 0       | ±3600 (ten turns) |
| Zoom     | The lens's magnification. 200% frames half as much of the scene without moving the camera. | 100%    | 1% to 10,000%     |

At the **standard distance** of 1000 with 100% zoom, the camera frames exactly the scene's width and height, so a scene with an untouched camera renders as it always did.

- **Truck in or out:** change the distance. Halving it frames half as much of the scene. Once layers have depth (step 7), trucking changes how near and far layers move against each other; zooming never does.
- **Pan and tilt:** change Pan X and Pan Y.
- **Rotate:** change the rotation.
- **Zoom:** change the zoom.

The camera frames `width × distance / (1000 × zoom)` by `height × distance / (1000 × zoom)` scene units.

## Camera view

- The **resolution gate** stays fixed and shows exactly what the camera frames. Artwork, onion skin and the selection move, turn and scale behind it as the camera moves, so playing back shows the shot as it will render.
- Drawing in the Camera view works through the camera: a stroke lands in the scene where the camera shows it, and a stroke's width stays in scene units.
- The view's own zoom, pan and rotation still move around the gate, so you can pull back to see what lies outside the frame.

## Drawing view

The camera never changes the Drawing view.

## Camera panel

A **Camera** tab, beside Tool properties by default, shows the camera at the current frame:

- One row for each value: its name, its number, and a key button.
  - **Drag a value's name** left or right to change it. Hold Shift to change it ten times faster.
  - Type a number, then press Enter or leave the field. Escape restores it.
  - With no keys on that value, a change sets its base value. With keys, a change keys it at the current frame.
  - **◇** keys the value at the current frame. **◆** shows a key is there; clicking it removes the key, and a menu beside it picks how the value travels to the next key.
- **‹◆ and ◆›** go to the previous and next frame with a camera key.
- **Key all** keys every value at the current frame, as one undoable edit.
- **Truck in** and **Truck out** move the camera 20% closer, or 25% farther, keying the distance if it has keys.
- **Reset** removes every camera key and returns to the standard camera, as one undoable edit.
- A line below says how much of the scene the camera frames.

## Timeline

When the camera has keys, a **Camera** row at the top of the timeline shows a diamond at each keyed frame. Click one to go to its frame; drag it to move every camera key at that frame together, as one undoable edit.

## File format

Schema version 4 adds `camera` to every scene: `x`, `y`, `z`, `rotation` and `zoom`, each `{ "value", "keys" }` as in [keyframes](keyframes.md). Files saved before version 4 open with the standard camera.
