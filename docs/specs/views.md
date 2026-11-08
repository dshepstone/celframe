# Drawing and Camera views

The centre of the window has two tabs. Step 2 of the [production plan](../plans/production-features.md).

## Camera view

Shows the scene as it will be rendered:

- Every visible layer at the current frame, composited in layer order.
- A **light grey surround** around the paper, in both themes, so the frame reads clearly.
- The **resolution gate**: a 1.5-pixel dark teal outline around exactly the area that will be rendered, the scene's width by height.
- Outside the gate the scene stays visible but **dimmed**, covered by the light grey surround at 65% opacity, so artwork that runs past the frame can still be seen and worked on.
- Zoom reaches from 1% to 25,600%, so you can pull far back from the gate or move right in.
- Fit shows the whole gate filling 90% of the view.
- Once the scene camera arrives (step 6), the gate follows the camera's framing, and moving the camera in and out along Z changes what it frames.

## Drawing view

Shows one drawing on its own, for drawing and cleaning up:

- Only the drawing exposed at the current frame on the **current layer**: the layer chosen in the Layers panel, or with none chosen, the topmost visible layer.
- The frame's outline is drawn as a faint 1-pixel grey guide; nothing is dimmed outside it.
- Camera moves and layer depth (steps 6 and 7) never change this view.

## Both views

- Each view keeps its own zoom, pan and rotation, so switching between them never loses your place.
- Every tool works in both views. Strokes always go to the current layer.
- The selection and its handles show in both.
- The views are tabs above the canvas. `Ctrl+1` shows Drawing and `Ctrl+2` shows Camera. The View menu has both.
- Which view is showing is remembered for the session, not saved in the project.
- New windows open in the Camera view.
