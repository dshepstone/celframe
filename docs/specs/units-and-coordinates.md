# Units and coordinate system

This spec defines how positions, sizes and time are measured in a Celframe document.

## Scene space

Everything in a document is stored in scene space.

- **Unit:** one scene unit equals one output pixel at the scene's own resolution. A 1920 × 1080 scene is 1920 units wide and 1080 units tall.
- **Origin:** (0, 0) is the centre of the scene frame.
- **Axes:** +X points right and +Y points up.
- **Scene frame:** runs from `-width / 2` to `+width / 2` on X and from `-height / 2` to `+height / 2` on Y. Artwork may extend past it.
- **Precision:** positions are stored as `f64`.

Centring the origin means a scene can change resolution or aspect ratio without moving its artwork. A Y-up axis matches how later rigging and camera work is usually reasoned about.

## Strokes

- A stroke is an ordered list of samples, each with a position and a pressure.
- **Pressure** is an `f32` from 0.0 to 1.0. Input without pressure, such as a mouse, uses 1.0.
- **Width:** a stroke's width at a sample is `base_width × pressure`, in scene units.
- Stored samples never depend on the view. Zooming doesn't change them.

## Screen space

Screen space is what the pointer reports: CSS pixels with the origin at the canvas's top-left corner and +Y pointing down.

## View transform

The view maps scene space to screen space. It has a centre (the scene point shown at the middle of the canvas), a zoom (screen pixels per scene unit, greater than 0) and a rotation (radians, counter-clockwise in scene space).

To go from a scene point `s` to a screen point `p`, on a canvas of size `(w, h)`:

1. Subtract the centre: `d = s - centre`.
2. Rotate `d` by the rotation angle.
3. Multiply by zoom.
4. Flip Y and move to the canvas middle: `p = (w / 2 + d.x, h / 2 - d.y)`.

Pointer input uses the inverse. The view is UI state: it isn't part of the document, isn't saved in scene data and isn't undoable.

## Time

- Time is counted in whole frames, stored as `u32`.
- The first frame of a scene is frame 0 in data. The UI shows it as frame 1.
- A scene's frame rate is stored with the scene as frames per second.
- An exposure holds one drawing from a start frame for a length of at least 1 frame.
