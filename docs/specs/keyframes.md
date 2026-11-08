# Keyframes

Values that change over time. Step 5 of the [production plan](../plans/production-features.md). The first keyable value was **layer opacity**; the [scene camera](scene-camera.md) uses the same system, and layer depth (step 7) and pegs (step 8) will too.

## Animated values

- An animated value has a **base value**, used while it has no keys, and a list of **keys**. Each key holds a value at a frame.
- With keys, the value before the first key is the first key's value, and after the last key it's the last key's value.
- Between two keys, the earlier key's **interpolation** decides how the value travels to the next:

| Interpolation | Between this key and the next                                      |
| ------------- | ------------------------------------------------------------------ |
| Linear        | Changes at an even rate                                            |
| Ease          | Starts slowly, speeds up, and slows into the next key (smoothstep) |
| Step          | Holds this key's value until the next key's frame                  |

- New keys use Ease.
- Values are worked out at whole frames.

## Layer opacity

- Every layer has an opacity from 0% to 100%, default 100%. It multiplies the opacity of everything the layer draws, in both views.
- Opacity is shown in the **Layers panel**, for the current layer, as a slider and a percentage.

## Editing keys

All key edits are undoable, one step each.

| Action               | How                                                                                                                            |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Set a key            | The hollow ◇ button beside opacity: keys the value shown at the current frame                                                  |
| Change a value       | Move the slider or type a percentage. With keys, this sets a key at the current frame; without keys, it changes the base value |
| Remove a key         | The filled ◆ button, which replaces ◇ when the current frame has a key                                                         |
| Change interpolation | The menu beside the key buttons, for the key at the current frame                                                              |
| Move a key           | Drag its diamond along the timeline. Dropping it on another key replaces that key                                              |
| Go to a key          | Click its diamond                                                                                                              |

- Each layer's keys appear as **diamonds** on its row in the timeline, at their frames.

## File format

Schema version 3 adds `opacity` to every layer: `{ "value": 1.0, "keys": [] }`, where each key is `{ "frame", "value", "interpolation" }` and interpolation is `"linear"`, `"ease"` or `"step"`. Keys are sorted by frame with no two at the same frame, and opacity values are from 0 to 1. Version 2 files are upgraded on open with every layer fully opaque and unkeyed.
