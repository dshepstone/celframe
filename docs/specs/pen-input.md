# Pen input

This spec defines how pen, mouse and touch input becomes strokes. It builds on
[units and coordinates](units-and-coordinates.md) and [stroke shape](stroke-shape.md).

## Drawing a stroke

- Pressing a pen, the primary mouse button or a finger on the canvas starts a stroke. Moving
  adds to it, and lifting ends it.
- Pressure comes from the pen. A mouse always uses 1.0. Touch uses the pressure the device
  reports, or 1.0 if it reports none.
- Positions arrive in screen space and the core converts them to scene space with the current
  view, so a stroke lands where it was drawn at any zoom.
- Samples are smoothed as they arrive, with the stroke smoother in `celframe-geom`. Samples closer
  than half a screen pixel to the previous one are dropped.
- The first stroke on a new project, and every later one, is one undoable edit, however many
  samples it has.

## Where strokes go

- Strokes go to the topmost layer that is visible and unlocked, into the drawing it shows at the
  current frame. Phase 1 draws only at frame 0 of the first scene.
- If that layer shows no drawing at the frame, one is created there first. That costs extra undo
  steps, so new projects start with a drawing in place.
- If no layer is visible and unlocked, nothing is drawn and the app says why.

## Pen settings

Phase 1 has one pen: black, with a base width of 10 scene units. Pen options come later.

## Responsiveness

- The UI sends samples to the core at most once per displayed frame, batched, including
  the extra samples a pen reports between frames.
- The canvas is redrawn after each batch.
- The final two samples of a stroke in progress appear when the next samples arrive, or when the
  stroke ends, because smoothing needs them.
