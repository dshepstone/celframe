# Line art and colour art

Animators usually ink lines first and paint colour underneath them. Keeping the two apart on every drawing means colour can be repainted, filled or erased without damaging the lines, and lines can be redrawn without touching the colour.

## Model

- Every stroke belongs to one of two parts of its drawing: **line art** or **colour art**.
- The part is stored on the stroke as `art`, either `"line"` or `"colour"`.
- A drawing still keeps one list of strokes. Each part's strokes keep their order within that list.

## Drawing order

- Within a drawing, all colour art draws below all line art, whatever the list order.
- Within a part, strokes draw in list order, first stroke at the bottom.
- Layers still stack as before: a higher layer's colour art draws above a lower layer's line art.

## Tools

- Drawing tools draw on the **current part**, chosen in the tool options bar. The default is line art.
- `L` switches the current part. The status bar shows which part is current.
- The current part is a tool setting, not part of the document, so switching it isn't undoable.
- Later tools follow these rules:
  - The eraser only affects the current part.
  - Fill puts its result on colour art, and finds the edges of areas from line art.
  - Selection works on the current part, with an option to select both.

## File format

Schema version 2 adds `art` to every stroke. Version 1 files are upgraded on open, and every existing stroke becomes line art, so old files look exactly as before.
