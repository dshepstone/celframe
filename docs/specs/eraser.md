# Eraser

The eraser removes parts of vector strokes, the way a real eraser removes ink, so lines can be cleaned up without redrawing them.

## Behaviour

- The eraser is a circle that follows the pointer. Its **size** is its diameter in scene units, set in the tool options bar.
- Where a stroke's centreline passes under the circle, that part of the stroke is removed. A stroke cut in the middle becomes two strokes, each keeping the original's colour, width, pressure and part.
- **Whole strokes** mode removes any stroke the eraser touches, instead of cutting it.
- The eraser only affects the **current part** of the drawing, line art or colour art, on the current layer at the current frame. See [line art and colour art](line-and-colour-art.md).
- Hidden and locked layers can't be erased, as with the pen.
- Pressure doesn't change the eraser's size.

## Undo

One drag of the eraser is one undo step, however many strokes it cut or removed.

## Accuracy

Cuts land within a quarter of the eraser's radius of its edge. Each remaining piece ends with the stroke's normal round cap, so a cut end can extend past the eraser's edge by up to half the stroke's width.

## Controls

| Action            | Shortcut | Elsewhere          |
| ----------------- | -------- | ------------------ |
| Eraser tool       | `E`      | Tool rail          |
| Size              | —        | Tool options bar   |
| Whole strokes     | —        | Tool options bar   |
| Pan while erasing | Space    | As with every tool |
