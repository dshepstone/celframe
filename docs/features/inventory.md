# Feature inventory

Everything Celframe aims to cover, grouped by roadmap phase. It is written from general knowledge of how 2D animation is made, in our own words, and describes what animators need, not how any other product does it. Each item becomes a spec in `docs/specs/` before it is built.

Status: ✅ done · 🚧 in progress · ⬜ planned

## Phase 1: Foundation

| Feature                                                                         | Status |
| ------------------------------------------------------------------------------- | ------ |
| Document model: project, scenes, layers, drawings, strokes, exposures           | ✅     |
| Every edit undoable; continuous strokes undo as one step                        | ✅     |
| Open `.celf` file format with versioning and migrations                         | ✅     |
| New, Open, Save, Save As; autosave and crash recovery                           | ✅     |
| Pressure-sensitive vector pen with smoothing                                    | ✅     |
| GPU rendering of strokes                                                        | ✅     |
| Studio layout: menus, tool rail, tool options, layers, colour, timeline, status | ✅     |
| Command palette and keyboard shortcuts                                          | ✅     |
| Light and dark themes meeting WCAG AA                                           | ✅     |
| Canvas zoom, pan and rotate                                                     | ⬜     |
| Golden-image rendering test                                                     | ⬜     |

## Phase 2: Drawing tools

**Line work**

- Pen and brush with pressure, tilt and velocity affecting width and opacity.
- Pencil lines whose thickness profile stays editable after drawing.
- Eraser that cuts strokes, with a stroke-erase option that removes whole strokes.
- Shape tools: line, rectangle, ellipse, polyline and polygon, with snapping.
- Point editing: move points and Bézier handles, add, delete and smooth points.
- Stroke smoothing strength, and "stabiliser" lag for steady curves.
- Cut and join strokes; close gaps between strokes that nearly touch.

**Colour and fill**

- Fill tool that fills areas enclosed by lines, with gap closing tolerance.
- Separate line art and colour art within a layer, so colour can be painted under lines without disturbing them.
- Paint modes: paint everywhere, paint only empty areas, recolour only existing fills, and paint without crossing lines.
- Palettes: named colour swatches stored with the project, shareable between scenes.
- Linked colours: changing a palette colour updates everything painted with it.
- Linear and radial gradients; colour picker and eyedropper.

**Selection and transform**

- Select, lasso and marquee selection of strokes and fills.
- Move, rotate, scale, skew and flip, with a movable pivot.
- Copy, paste, duplicate, group and align.

**Drawing aids**

- Rulers, guides, grids and safe-area frames.
- Perspective guides with one, two and three vanishing points.
- Symmetry drawing across one or more axes.
- Reference image layers for model sheets and layouts.

## Phase 3: Timing

- **Timeline:** layers as tracks, frames as columns, exposure blocks that can be extended, shortened, moved and swapped.
- **Exposure sheet:** a column-per-layer, row-per-frame view for timing drawings by number, with annotations.
- Hold, insert, delete and reverse frames; fill empty frames; change the timing of a range.
- Drawing substitution: choose which drawing a layer shows at each frame.
- **Onion skin:** previous and next drawings in adjustable colours, opacity and range.
- **Light table:** see other layers faintly while drawing on one.
- Playback at the scene frame rate, looping, ranges and frame-by-frame scrubbing.
- Audio tracks with a waveform, scrubbing and sync.
- Mouth-shape charts from dialogue, for lip sync.
- Markers and notes on frames; scenes and sequences in one project.

## Phase 4: Export and Release 0.1

- Image sequences (PNG, with transparency) and video (MP4) and animated GIF.
- Render settings: resolution, frame range, background and transparency.
- Installers for Windows, macOS and Linux.
- First public release, with getting-started documentation for students.

## Phase 5: Raster and textures

- Bitmap layers alongside vector layers.
- Textured brushes with grain, scatter and paper texture.
- Layer opacity and blending modes.
- Import images and image sequences as references or artwork.

## Phase 6: Cut-out rigging

- Characters built from separate drawn parts in a parent–child hierarchy.
- Pivots, and transform nodes that move groups of parts together.
- Keyframes with interpolation: linear, eased and stepped.
- Motion paths shown and edited on the canvas.
- Curve editor for timing and spacing.
- Deformers: bones, curve deformers and envelope deformers that bend drawings.
- Inverse kinematics for limbs.
- Drawing swaps for hands, mouths and other replaceable parts.
- Controllers that drive many parameters from one handle.
- Pose and part libraries, and reusable character templates.

## Phase 7: Compositing and camera

- Node-based compositing view of how layers, effects and outputs connect.
- Effects: blur, glow, drop shadow, colour correction, tone and highlight shading, masks and cut-outs.
- Multiplane camera: layers at different depths for parallax.
- Camera moves, zooms and shakes, with keyframes.
- Render preview of effects in the scene.

## Phase 8: Scripting and automation

- A Python API covering the document, commands and rendering.
- Plugins that add tools, panels, effects and importers.
- Batch rendering and a command-line interface.

## Phase 9: Collaboration and classroom

- Real-time co-editing of a project.
- Review: comments and drawn notes on frames.
- Version history with compare and restore.
- Classroom tools: assignment templates, submissions and feedback.

## Across every phase

- Dockable panels and saved workspaces for each task.
- Customisable keyboard shortcuts.
- Tool presets.
- Accessibility: keyboard access, screen reader labels and sufficient contrast.
- Localisation of the interface.
