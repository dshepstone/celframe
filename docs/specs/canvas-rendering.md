# Canvas rendering

This spec defines how strokes become pixels. It builds on
[units and coordinates](units-and-coordinates.md) and [stroke shape](stroke-shape.md). The
approach is recorded in [ADR-0001](../adr/0001-rendering.md).

## What is drawn

- Strokes are drawn in document order, bottom stroke first, so later strokes cover earlier ones.
- Each stroke is filled with its own colour, given as straight (not premultiplied) 8-bit RGBA.
- Colours blend with the standard "over" rule, in sRGB space, as most 2D drawing apps do.
- The canvas is first cleared to a background colour chosen by the caller.

## View

- Scene space maps to the canvas with the view transform in the units spec: a centre, a zoom and
  a rotation.
- A canvas pixel `(px, py)` covers screen space from `(px, py)` to `(px + 1, py + 1)`.
- Curves are approximated finely enough that the error stays under a quarter of a screen pixel at
  the current zoom.

## Quality

- Edges are anti-aliased with 4× multisampling.
- Rendering is deterministic for a given GPU and driver. Different GPUs may differ slightly at
  edges, so image comparisons allow a small per-pixel tolerance.

## Known limits

- Where one semi-transparent stroke overlaps itself, the overlap is darker. Strokes are usually
  opaque in Phase 1. Flattening each stroke before blending can come later.

## Headless rendering

The renderer can draw to an offscreen image and read the pixels back, with no window. Tests use
this, including the golden-image test. The image is RGBA, 8 bits per channel, rows top to bottom,
with premultiplied alpha. For an opaque background that is the same as straight alpha.
