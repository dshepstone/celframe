# Stroke shape

This spec defines the filled shape drawn for a stroke. It builds on the stroke rules in
[units and coordinates](units-and-coordinates.md).

## Outline

- A stroke is drawn as a filled band centred on its smoothed centreline, the samples stored in the
  document.
- The band's width at each sample is `base_width × pressure` scene units, and it changes linearly
  between samples.
- Both ends are round. Joins between segments are round, so sharp turns never form spikes.
- A stroke with one sample is drawn as a round dot with the width of that sample.
- A stroke with no samples, or whose samples all have zero width, draws nothing.

## Mesh

- The shape is turned into triangles in scene space, independent of the view. Zooming doesn't
  change the mesh.
- Round ends and joins are approximated with straight edges. The **tolerance** is the largest
  distance, in scene units, between the true curve and its approximation. The renderer chooses it.
- Mesh positions are `f32`. Scene coordinates are `f64`, which keeps full precision within the
  range of any practical scene.
- Triangles carry no colour. The renderer applies each stroke's colour.
