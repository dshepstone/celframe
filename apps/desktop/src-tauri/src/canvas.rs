//! The native canvas: draws the open scene into the main window, under the transparent webview.
//! See ADR-0001 and `docs/specs/canvas-rendering.md`.

use std::borrow::Cow;

use celframe_core::{Art, LayerId, Scene, Stroke, StrokeId, StrokeSample};
use celframe_render::{Canvas, Paper, RenderError, Renderer, View};
use serde::Deserialize;
use tauri::WebviewWindow;

/// The scene frame is white, like the paper it stands for, in both themes.
const PAPER: [u8; 4] = [255, 255, 255, 255];
/// How much of the canvas area the scene frame fills when fitted.
const FIT: f64 = 0.9;

/// Which of the centre's views is showing. See `docs/specs/views.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    /// One drawing on its own.
    Drawing,
    /// The scene as it renders, with the resolution gate.
    #[default]
    Camera,
}

/// The gate's outline colour, and how strongly the area outside it is dimmed in the Camera view.
const GATE_COLOR: [u8; 4] = [14, 110, 104, 255];
/// The Camera view's light grey surround, in both themes. See `docs/specs/views.md`.
pub const CAMERA_SURROUND: [u8; 4] = [206, 208, 212, 255];

/// The colour around the paper in a view: light grey for the Camera view, the theme's
/// backdrop for the Drawing view.
pub fn surround(mode: ViewMode, layout: &CanvasLayout) -> [u8; 4] {
    match mode {
        ViewMode::Camera => CAMERA_SURROUND,
        ViewMode::Drawing => layout.backdrop,
    }
}

/// Onion skin settings. A viewing aid, never saved. See `docs/specs/onion-skin.md`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnionSkin {
    pub enabled: bool,
    pub before: u32,
    pub after: u32,
    /// Count neighbours every `step` frames instead of by exposure.
    pub by_frames: bool,
    pub step: u32,
    /// The nearest neighbour's opacity, from 0 to 1.
    pub opacity: f64,
}

impl Default for OnionSkin {
    fn default() -> Self {
        Self {
            enabled: false,
            before: 2,
            after: 1,
            by_frames: false,
            step: 1,
            opacity: 0.35,
        }
    }
}

const BEFORE_TINT: [u8; 3] = [205, 60, 50];
const AFTER_TINT: [u8; 3] = [40, 145, 80];

/// The layer a view works with: the chosen one, or the topmost visible one.
fn current_layer(scene: &Scene, layer: Option<LayerId>) -> Option<&celframe_core::Layer> {
    layer
        .and_then(|id| scene.layers.iter().find(|l| l.id == id))
        .or_else(|| scene.layers.iter().rev().find(|l| l.visible))
}

/// Tinted copies of the current layer's neighbouring drawings, furthest first, to draw under
/// the current drawing.
pub fn onion_strokes(
    scene: &Scene,
    layer: Option<LayerId>,
    frame: u32,
    onion: &OnionSkin,
) -> Vec<Stroke> {
    let Some(layer) = current_layer(scene, layer).filter(|_| onion.enabled) else {
        return Vec::new();
    };
    let current = layer.drawing_at(frame).map(|d| d.id);
    let drawing = |id| layer.drawings.iter().find(|d| d.id == id);
    let exposures = &layer.exposures;

    // Each side's neighbours, nearest first.
    let (before, after): (Vec<_>, Vec<_>) = if onion.by_frames {
        let step = onion.step.max(1);
        let at = |offset: i64| {
            u32::try_from(i64::from(frame) + offset)
                .ok()
                .and_then(|f| layer.drawing_at(f))
                .map(|d| d.id)
        };
        (
            (1..=i64::from(onion.before))
                .filter_map(|i| at(-i * i64::from(step)))
                .collect(),
            (1..=i64::from(onion.after))
                .filter_map(|i| at(i * i64::from(step)))
                .collect(),
        )
    } else {
        let earlier = exposures
            .iter()
            .rev()
            .filter(|e| e.start_frame + e.length <= frame)
            .map(|e| e.drawing);
        let later = exposures
            .iter()
            .filter(|e| e.start_frame > frame)
            .map(|e| e.drawing);
        (
            earlier.take(onion.before as usize).collect(),
            later.take(onion.after as usize).collect(),
        )
    };

    let mut strokes = Vec::new();
    for (side, tint) in [(before, BEFORE_TINT), (after, AFTER_TINT)] {
        let count = side.len().max(1) as f64;
        // Furthest first, so nearer neighbours draw on top.
        for (distance, id) in side.into_iter().enumerate().rev() {
            if Some(id) == current {
                continue;
            }
            let strength = onion.opacity.clamp(0.0, 1.0) * (count - distance as f64) / count;
            for stroke in drawing(id)
                .into_iter()
                .flat_map(|d| d.strokes_in_draw_order())
            {
                let alpha = (f64::from(stroke.color[3]) * strength).round() as u8;
                strokes.push(Stroke {
                    color: [tint[0], tint[1], tint[2], alpha],
                    ..stroke.clone()
                });
            }
        }
    }
    strokes
}
const GATE_WIDTH: f64 = 1.5;
const GUIDE_COLOR: [u8; 4] = [128, 128, 128, 140];
const OUTSIDE_DIM: f64 = 0.65;

/// The strokes a view shows at `frame`, bottom first, with each layer's opacity at that frame
/// applied. The Drawing view shows the drawing on `layer`, or on the topmost visible layer
/// when that's None or no longer exists.
pub fn view_strokes(
    scene: &Scene,
    mode: ViewMode,
    layer: Option<LayerId>,
    frame: u32,
) -> Vec<Cow<'_, Stroke>> {
    let layers: Vec<&celframe_core::Layer> = match mode {
        ViewMode::Camera => scene.layers.iter().filter(|l| l.visible).collect(),
        ViewMode::Drawing => {
            let chosen = layer.and_then(|id| scene.layers.iter().find(|l| l.id == id));
            chosen
                .or_else(|| scene.layers.iter().rev().find(|l| l.visible))
                .into_iter()
                .collect()
        }
    };
    let mut strokes = Vec::new();
    for layer in layers {
        let opacity = layer.opacity.at(frame).clamp(0.0, 1.0);
        if opacity <= 0.0 {
            continue;
        }
        let drawn = layer
            .drawing_at(frame)
            .into_iter()
            .flat_map(|d| d.strokes_in_draw_order());
        for stroke in drawn {
            strokes.push(if opacity >= 1.0 {
                Cow::Borrowed(stroke)
            } else {
                let mut faded = stroke.clone();
                faded.color[3] = (f64::from(stroke.color[3]) * opacity).round() as u8;
                Cow::Owned(faded)
            });
        }
    }
    strokes
}

fn stroke_through(points: &[[f64; 2]], width: f64, color: [u8; 4]) -> Stroke {
    Stroke {
        id: StrokeId::from_u128(0),
        base_width: width,
        color,
        art: Art::Line,
        samples: points
            .iter()
            .map(|&[x, y]| StrokeSample {
                x,
                y,
                pressure: 1.0,
            })
            .collect(),
    }
}

fn closed(rect: [f64; 4]) -> [[f64; 2]; 5] {
    [
        [rect[0], rect[1]],
        [rect[2], rect[1]],
        [rect[2], rect[3]],
        [rect[0], rect[3]],
        [rect[0], rect[1]],
    ]
}

/// What a view draws over the scene for the frame: in the Camera view, a dimmed surround and
/// the gate's outline; in the Drawing view, a faint guide. `pixel` is one device pixel in
/// scene units, and `backdrop` the canvas surround's colour.
pub fn frame_overlay(scene: &Scene, mode: ViewMode, pixel: f64, backdrop: [u8; 4]) -> Vec<Stroke> {
    let gate = Paper::scene_frame(scene.width, scene.height, PAPER).rect;
    match mode {
        ViewMode::Drawing => vec![stroke_through(&closed(gate), pixel, GUIDE_COLOR)],
        ViewMode::Camera => {
            // A very wide stroke around the gate, its inner edge on the gate. The inside of a
            // stroke's corners stays sharp, so the dimming meets the gate exactly.
            let reach = (f64::from(scene.width) + f64::from(scene.height)) * 50.0;
            let half = reach / 2.0;
            let around = [
                gate[0] - half,
                gate[1] - half,
                gate[2] + half,
                gate[3] + half,
            ];
            let dim = [
                backdrop[0],
                backdrop[1],
                backdrop[2],
                (OUTSIDE_DIM * 255.0).round() as u8,
            ];
            vec![
                stroke_through(&closed(around), reach, dim),
                stroke_through(&closed(gate), GATE_WIDTH * pixel, GATE_COLOR),
            ]
        }
    }
}

/// Where the webview's canvas area is, and what colour surrounds the paper. Sent by the UI.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasLayout {
    /// The canvas area in CSS pixels from the window's top left.
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Device pixels per CSS pixel.
    pub scale: f64,
    pub backdrop: [u8; 4],
}

impl CanvasLayout {
    /// The canvas area in device pixels, rounded outwards so no edge is left undrawn.
    pub fn viewport(&self) -> [u32; 4] {
        let px = |v: f64| (v * self.scale).max(0.0);
        let (x0, y0) = (px(self.x).floor(), px(self.y).floor());
        let (x1, y1) = (
            px(self.x + self.width).ceil(),
            px(self.y + self.height).ceil(),
        );
        [x0, y0, x1 - x0, y1 - y0].map(|v| v.min(f64::from(u32::MAX)) as u32)
    }
}

/// The view that shows the whole scene frame in the canvas area.
pub fn fitted_view(scene: &Scene, layout: &CanvasLayout) -> View {
    let paper = Paper::scene_frame(scene.width, scene.height, PAPER);
    View::fit(paper.rect, viewport_size(layout), FIT)
}

/// The canvas area's size in device pixels.
pub fn viewport_size(layout: &CanvasLayout) -> [f64; 2] {
    let viewport = layout.viewport();
    [f64::from(viewport[2]), f64::from(viewport[3])]
}

/// The frame drawn for a scene: its paper, shown with `view`, or fitted when there is none.
pub fn frame_for(
    scene: &Scene,
    layout: &CanvasLayout,
    target_size: [u32; 2],
    view: Option<View>,
) -> (Canvas, View) {
    let paper = Paper::scene_frame(scene.width, scene.height, PAPER);
    let viewport = layout.viewport();
    let view = view.unwrap_or_else(|| fitted_view(scene, layout));
    let canvas = Canvas {
        target_size,
        viewport,
        backdrop: layout.backdrop,
        paper: Some(paper),
    };
    (canvas, view)
}

/// Converts pointer positions on the canvas to scene space, with the view the canvas draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mapping {
    layout: CanvasLayout,
    viewport: [u32; 4],
    view: View,
}

impl Mapping {
    pub fn new(scene: &Scene, layout: &CanvasLayout, view: Option<View>) -> Self {
        // The target size doesn't affect the view, only where the viewport sits.
        let (canvas, view) = frame_for(scene, layout, [u32::MAX, u32::MAX], view);
        Self {
            layout: *layout,
            viewport: canvas.viewport,
            view,
        }
    }

    /// A point in CSS pixels from the window's top left, as pointer events report it.
    pub fn scene_point(&self, x: f64, y: f64) -> [f64; 2] {
        let [vx, vy, vw, vh] = self.viewport.map(f64::from);
        let device = [x * self.layout.scale - vx, y * self.layout.scale - vy];
        self.view.screen_to_scene(device, [vw, vh])
    }

    /// Half a device pixel, in scene units: samples closer than this are dropped.
    pub fn min_distance(&self) -> f64 {
        0.5 / self.view.zoom
    }
}

pub struct WindowCanvas {
    renderer: Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    layout: Option<CanvasLayout>,
    /// Each view's zoom, pan and rotation, Drawing then Camera. None shows the fitted scene.
    views: [Option<View>; 2],
    mode: ViewMode,
    /// The layer the Drawing view shows, as the UI chose it.
    drawing_layer: Option<LayerId>,
    onion: OnionSkin,
}

impl WindowCanvas {
    pub fn new(window: &WebviewWindow) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("couldn't draw into the window: {e}"))?;
        let adapter =
            pollster::block_on(celframe_render::request_adapter(&instance, Some(&surface)))
                .map_err(|e: RenderError| e.to_string())?;
        let renderer =
            pollster::block_on(Renderer::from_adapter(&adapter)).map_err(|e| e.to_string())?;

        let caps = surface.get_capabilities(&adapter);
        // The spec blends in sRGB space, which a non-sRGB format gives directly.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| caps.formats.first().copied())
            .ok_or("the window can't be drawn into")?;
        let alpha_mode = if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            caps.alpha_modes[0]
        };
        let size = window.inner_size().map_err(|e| e.to_string())?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(renderer.device(), &config);
        Ok(Self {
            renderer,
            surface,
            config,
            layout: None,
            views: [None, None],
            mode: ViewMode::Camera,
            drawing_layer: None,
            onion: OnionSkin::default(),
        })
    }

    pub fn adapter_name(&self) -> &str {
        self.renderer.adapter_name()
    }

    /// Call when the window's inner size changes, in device pixels.
    pub fn resize(&mut self, width: u32, height: u32) {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) != (self.config.width, self.config.height) {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(self.renderer.device(), &self.config);
        }
    }

    pub fn set_layout(&mut self, layout: CanvasLayout) {
        self.layout = Some(layout);
    }

    pub fn layout(&self) -> Option<CanvasLayout> {
        self.layout
    }

    fn slot(mode: ViewMode) -> usize {
        match mode {
            ViewMode::Drawing => 0,
            ViewMode::Camera => 1,
        }
    }

    /// The current view's zoom, pan and rotation, or None for the fitted scene.
    pub fn view(&self) -> Option<View> {
        self.views[Self::slot(self.mode)]
    }

    pub fn set_view(&mut self, view: Option<View>) {
        self.views[Self::slot(self.mode)] = view;
    }

    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    pub fn drawing_layer(&self) -> Option<LayerId> {
        self.drawing_layer
    }

    pub fn onion(&self) -> OnionSkin {
        self.onion
    }

    pub fn set_onion(&mut self, onion: OnionSkin) {
        self.onion = onion;
    }

    /// Shows a view. Each view keeps its own navigation.
    pub fn set_mode(&mut self, mode: ViewMode, drawing_layer: Option<LayerId>) {
        self.mode = mode;
        self.drawing_layer = drawing_layer;
    }

    /// Device pixels per scene unit in the view drawn now. None until the UI sends a layout.
    pub fn zoom(&self, scene: &Scene) -> Option<f64> {
        let layout = self.layout?;
        Some(
            self.view()
                .unwrap_or_else(|| fitted_view(scene, &layout))
                .zoom,
        )
    }

    /// Draws `strokes` from a scene's frame 0 with `overlay` on top, and presents it. Does
    /// nothing until the UI has sent a layout.
    pub fn render<'a>(
        &mut self,
        scene: &Scene,
        strokes: impl Iterator<Item = &'a Stroke>,
        overlay: &'a [Stroke],
    ) {
        let Some(layout) = self.layout else {
            return;
        };
        use wgpu::CurrentSurfaceTexture as Frame;
        let frame = match self.surface.get_current_texture() {
            Frame::Success(frame) | Frame::Suboptimal(frame) => frame,
            Frame::Outdated | Frame::Lost => {
                // The window changed under us. Reconfigure, and the next redraw will draw.
                self.surface.configure(self.renderer.device(), &self.config);
                return;
            }
            _ => return,
        };
        let (canvas, view) = frame_for(
            scene,
            &layout,
            [self.config.width, self.config.height],
            self.view(),
        );
        let canvas = Canvas {
            backdrop: surround(self.mode, &layout),
            ..canvas
        };
        let target = frame.texture.create_view(&Default::default());
        let mut encoder = self
            .renderer
            .device()
            .create_command_encoder(&Default::default());
        self.renderer.draw(
            &mut encoder,
            &target,
            self.config.format,
            &canvas,
            strokes.chain(overlay),
            &view,
        );
        self.renderer.submit(encoder);
        self.renderer.present(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(x: f64) -> Stroke {
        Stroke {
            samples: vec![
                StrokeSample {
                    x,
                    y: 0.0,
                    pressure: 1.0,
                },
                StrokeSample {
                    x,
                    y: 10.0,
                    pressure: 1.0,
                },
            ],
            ..stroke_through(&[], 2.0, [0, 0, 0, 255])
        }
    }

    /// A scene with a bottom layer holding a stroke at x = 1 and a top layer at x = 2.
    fn two_layers() -> Scene {
        let mut project = celframe_core::Project::starter("A");
        let scene = &mut project.scenes[0];
        scene.layers[0].drawings[0].strokes.push(line(1.0));
        let mut top = scene.layers[0].clone();
        top.id = LayerId::new();
        top.drawings[0].strokes = vec![line(2.0)];
        scene.layers.push(top);
        project.scenes.remove(0)
    }

    fn xs(strokes: &[Cow<'_, Stroke>]) -> Vec<f64> {
        strokes.iter().map(|s| s.samples[0].x).collect()
    }

    #[test]
    fn the_camera_view_shows_every_visible_layer_and_the_drawing_view_one() {
        let mut scene = two_layers();
        let bottom = scene.layers[0].id;
        assert_eq!(
            xs(&view_strokes(&scene, ViewMode::Camera, None, 0)),
            [1.0, 2.0]
        );
        assert_eq!(
            xs(&view_strokes(&scene, ViewMode::Drawing, Some(bottom), 0)),
            [1.0]
        );
        assert_eq!(
            xs(&view_strokes(&scene, ViewMode::Drawing, None, 0)),
            [2.0],
            "topmost"
        );
        scene.layers[1].visible = false;
        assert_eq!(
            xs(&view_strokes(&scene, ViewMode::Drawing, None, 0)),
            [1.0],
            "visible only"
        );
        let gone = Some(LayerId::new());
        assert_eq!(xs(&view_strokes(&scene, ViewMode::Drawing, gone, 0)), [1.0]);
    }

    /// One layer with drawings A to E, each exposed for two frames from frame 0.
    fn five_drawings() -> (Scene, Vec<f64>) {
        let mut project = celframe_core::Project::starter("A");
        let layer = &mut project.scenes[0].layers[0];
        layer.drawings.clear();
        layer.exposures.clear();
        let xs = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        for (i, &x) in xs.iter().enumerate() {
            let mut drawing = celframe_core::Drawing::new();
            drawing.strokes.push(line(x));
            let id = drawing.id;
            layer.drawings.push(drawing);
            layer
                .insert_exposure(celframe_core::Exposure::new(i as u32 * 2, 2, id))
                .unwrap();
        }
        (project.scenes.remove(0), xs)
    }

    #[test]
    fn layer_opacity_at_the_frame_fades_what_the_layer_draws() {
        let mut scene = two_layers();
        // Both drawings hold for 20 frames.
        for layer in &mut scene.layers {
            layer.exposures[0].length = 20;
        }
        scene.layers[0].opacity.set_key(celframe_core::Key {
            frame: 0,
            value: 0.0,
            interpolation: celframe_core::Interpolation::Linear,
        });
        scene.layers[0].opacity.set_key(celframe_core::Key {
            frame: 10,
            value: 1.0,
            interpolation: celframe_core::Interpolation::Linear,
        });
        let at = |frame| -> Vec<(f64, u8)> {
            view_strokes(&scene, ViewMode::Camera, None, frame)
                .iter()
                .map(|s| (s.samples[0].x, s.color[3]))
                .collect()
        };
        assert_eq!(at(0), [(2.0, 255)], "fully transparent layers draw nothing");
        assert_eq!(at(5), [(1.0, 128), (2.0, 255)]);
        assert_eq!(at(10), [(1.0, 255), (2.0, 255)]);
    }

    #[test]
    fn onion_skin_shows_neighbouring_drawings_tinted_and_fading() {
        let (scene, _) = five_drawings();
        let onion = OnionSkin {
            enabled: true,
            before: 2,
            after: 1,
            ..OnionSkin::default()
        };
        // Frame 4 shows drawing C; before are B and A, after is D.
        let strokes = onion_strokes(&scene, None, 4, &onion);
        let shown: Vec<(f64, u8, u8)> = strokes
            .iter()
            .map(|s| (s.samples[0].x, s.color[0], s.color[3]))
            .collect();
        // Furthest first: A at half strength, then B, then D.
        assert_eq!(shown, [(10.0, 205, 45), (20.0, 205, 89), (40.0, 40, 89)]);
        let off = OnionSkin {
            enabled: false,
            ..onion
        };
        assert!(onion_strokes(&scene, None, 4, &off).is_empty());
    }

    #[test]
    fn onion_skin_can_count_by_frames() {
        let (scene, _) = five_drawings();
        let onion = OnionSkin {
            enabled: true,
            before: 2,
            after: 2,
            by_frames: true,
            step: 3,
            ..OnionSkin::default()
        };
        // From frame 4 (C): frames 1 and -2 before, 7 and 10 after: A, D, and nothing.
        let xs: Vec<f64> = onion_strokes(&scene, None, 4, &onion)
            .iter()
            .map(|s| s.samples[0].x)
            .collect();
        assert_eq!(xs, [10.0, 40.0]);
    }

    #[test]
    fn the_camera_view_has_a_light_grey_surround() {
        let layout = CanvasLayout {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            scale: 1.0,
            backdrop: [1, 2, 3, 255],
        };
        assert_eq!(surround(ViewMode::Camera, &layout), CAMERA_SURROUND);
        assert_eq!(surround(ViewMode::Drawing, &layout), [1, 2, 3, 255]);
    }

    #[test]
    fn the_camera_view_dims_outside_the_gate_and_outlines_it() {
        let scene = Scene::new("Shot");
        let [dim, gate] = &frame_overlay(&scene, ViewMode::Camera, 0.5, [10, 20, 30, 255])[..]
        else {
            panic!("two strokes");
        };
        assert_eq!(dim.color, [10, 20, 30, 166]);
        // The dimming stroke's inner edge sits exactly on the gate.
        let inner = dim.samples[0].x + dim.base_width / 2.0;
        assert_eq!(inner, -960.0);
        assert_eq!((gate.samples[0].x, gate.samples[0].y), (-960.0, -540.0));
        assert_eq!(gate.base_width, 0.75);
        let guide = frame_overlay(&scene, ViewMode::Drawing, 0.5, [0; 4]);
        assert_eq!(guide.len(), 1, "the Drawing view only gets a guide");
    }

    #[test]
    fn rendering_the_camera_view_leaves_the_paper_clear_and_dims_beyond_it() {
        let Ok(mut renderer) = celframe_render::Renderer::new_headless() else {
            return;
        };
        let scene = Scene::new("Shot");
        let backdrop = [40, 40, 40, 255];
        let layout = CanvasLayout {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 200.0,
            scale: 1.0,
            backdrop,
        };
        let (canvas, view) = frame_for(&scene, &layout, [200, 200], None);
        // A wide stroke crossing the gate's left edge, half inside and half outside.
        let crossing = Stroke {
            base_width: 400.0,
            color: [0, 0, 255, 255],
            samples: vec![
                StrokeSample {
                    x: -960.0,
                    y: -500.0,
                    pressure: 1.0,
                },
                StrokeSample {
                    x: -960.0,
                    y: 500.0,
                    pressure: 1.0,
                },
            ],
            ..stroke_through(&[], 1.0, [0; 4])
        };
        let pixel = 1.0 / view.zoom;
        let overlay = frame_overlay(&scene, ViewMode::Camera, pixel, backdrop);
        let image = renderer
            .render_image(std::iter::once(&crossing).chain(&overlay), &view, &canvas)
            .unwrap();
        // The gate fills 90% of 200 px across: x from 10 to 190, the middle row at y = 100.
        let inside = image.pixel(14, 100);
        let outside = image.pixel(6, 100);
        assert_eq!(
            inside,
            [0, 0, 255, 255],
            "the stroke inside the gate is untouched"
        );
        assert!(
            outside[2] < 140 && outside[0] > 15,
            "dimmed outside: {outside:?}"
        );
        assert_eq!(image.pixel(100, 100), [255, 255, 255, 255], "paper inside");
    }

    fn layout(scale: f64) -> CanvasLayout {
        CanvasLayout {
            x: 10.5,
            y: 20.0,
            width: 300.25,
            height: 200.0,
            scale,
            backdrop: [1, 2, 3, 255],
        }
    }

    #[test]
    fn the_viewport_is_in_device_pixels_and_covers_the_whole_area() {
        assert_eq!(layout(1.0).viewport(), [10, 20, 301, 200]);
        // 125% scaling: 13.125 to 388.4375 across, 25 to 275 down.
        assert_eq!(layout(1.25).viewport(), [13, 25, 376, 250]);
    }

    #[test]
    fn pointer_positions_map_to_where_the_canvas_draws_them() {
        let scene = Scene::new("Shot");
        let layout = CanvasLayout {
            x: 0.0,
            y: 100.0,
            width: 1000.0,
            height: 600.0,
            scale: 2.0,
            backdrop: [0, 0, 0, 255],
        };
        let mapping = Mapping::new(&scene, &layout, None);
        let (canvas, view) = frame_for(&scene, &layout, [4000, 4000], None);
        let [vx, vy, vw, vh] = canvas.viewport.map(f64::from);

        // The middle of the canvas area is the scene origin.
        let middle = mapping.scene_point(500.0, 400.0);
        assert!(
            middle[0].abs() < 1e-9 && middle[1].abs() < 1e-9,
            "{middle:?}"
        );

        // Drawing a point back through the view lands on the same pointer position.
        let point = mapping.scene_point(123.0, 456.5);
        let screen = view.scene_to_screen(point, [vw, vh]);
        assert!((screen[0] + vx - 246.0).abs() < 1e-9);
        assert!((screen[1] + vy - 913.0).abs() < 1e-9);

        assert!((mapping.min_distance() - 0.5 / view.zoom).abs() < 1e-12);
    }

    #[test]
    fn a_zoomed_and_turned_view_maps_pointers_through_the_same_view() {
        let scene = Scene::new("Shot");
        let layout = layout(2.0);
        let size = viewport_size(&layout);
        let view = fitted_view(&scene, &layout)
            .zoomed_about([40.0, 30.0], size, 3.0)
            .rotated_about([100.0, 80.0], size, 0.5)
            .panned([12.0, -7.0], size);
        let mapping = Mapping::new(&scene, &layout, Some(view));
        let (canvas, drawn) = frame_for(&scene, &layout, [800, 600], Some(view));
        assert_eq!(drawn, view);
        let [vx, vy, ..] = canvas.viewport.map(f64::from);
        let point = mapping.scene_point(70.0, 55.0);
        let screen = view.scene_to_screen(point, size);
        assert!((screen[0] + vx - 140.0).abs() < 1e-9);
        assert!((screen[1] + vy - 110.0).abs() < 1e-9);
        assert!((mapping.min_distance() - 0.5 / view.zoom).abs() < 1e-12);
    }

    #[test]
    fn the_scene_frame_is_fitted_into_the_canvas_area() {
        let scene = Scene::new("Shot");
        let (canvas, view) = frame_for(&scene, &layout(1.0), [800, 600], None);
        assert_eq!(canvas.target_size, [800, 600]);
        assert_eq!(canvas.viewport, [10, 20, 301, 200]);
        assert_eq!(canvas.backdrop, [1, 2, 3, 255]);
        assert_eq!(canvas.paper.unwrap().rect, [-960.0, -540.0, 960.0, 540.0]);
        assert_eq!(view.centre, [0.0, 0.0]);
        // Limited by the width: 301 × 0.9 / 1920.
        assert!((view.zoom - 301.0 * 0.9 / 1920.0).abs() < 1e-12);
    }
}
