# Onion skin

Faint copies of the drawings before and after the current one, so each new drawing can be placed and spaced against its neighbours. It's a viewing aid: it isn't saved in the project, isn't undoable and is never rendered out.

## What shows

- Onion skin applies to the **current layer**: the layer chosen in the Layers panel, or with none chosen, the topmost visible layer.
- **Before** drawings are tinted red, **after** drawings green, both drawn under the current drawing.
- The nearest neighbour is drawn at the chosen opacity; each one further away fades in equal steps, so the furthest of four is a quarter as strong.
- A neighbour showing the same drawing as the current frame is skipped.
- It shows in both the Drawing and Camera views.

## Settings

The settings sit in a bar at the top of the exposure sheet.

| Setting    | Range              | Default  | Meaning                                                                                                                                                                           |
| ---------- | ------------------ | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Onion skin | On or off          | Off      | `O` switches it too, and it's in the command palette and shelves                                                                                                                  |
| Before     | 0 to 5             | 2        | How many earlier neighbours show                                                                                                                                                  |
| After      | 0 to 5             | 1        | How many later neighbours show                                                                                                                                                    |
| Count by   | Drawings or Frames | Drawings | **Drawings:** the neighbours are the previous and next exposures, however long they hold. **Frames:** the neighbours are whatever is exposed every _Step_ frames before and after |
| Step       | 1 to 24 frames     | 1        | Used when counting by frames, such as every 2 frames for animation on twos                                                                                                        |
| Opacity    | 10% to 80%         | 35%      | The nearest neighbour's strength                                                                                                                                                  |

The settings are remembered for the session.
