//! The native canvas: draws the open scene into the main window, under the transparent webview.
//! See ADR-0001 and `docs/specs/canvas-rendering.md`.

use celframe_core::{Scene, Stroke};
use celframe_render::{Canvas, Paper, RenderError, Renderer, View};
use serde::Deserialize;
use tauri::WebviewWindow;

/// The scene frame is white, like the paper it stands for, in both themes.
const PAPER: [u8; 4] = [255, 255, 255, 255];
/// How much of the canvas area the scene frame fills when fitted.
const FIT: f64 = 0.9;

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
    /// The person's zoom, pan and rotation. None shows the fitted scene.
    view: Option<View>,
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
            view: None,
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

    /// The view the canvas shows, or None for the fitted scene.
    pub fn view(&self) -> Option<View> {
        self.view
    }

    pub fn set_view(&mut self, view: Option<View>) {
        self.view = view;
    }

    /// Device pixels per scene unit in the view drawn now. None until the UI sends a layout.
    pub fn zoom(&self, scene: &Scene) -> Option<f64> {
        let layout = self.layout?;
        Some(
            self.view
                .unwrap_or_else(|| fitted_view(scene, &layout))
                .zoom,
        )
    }

    /// Draws a scene's frame 0 with `overlay` on top, and presents it. Does nothing until the
    /// UI has sent a layout.
    pub fn render(&mut self, scene: &Scene, overlay: &[Stroke]) {
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
            self.view,
        );
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
            scene.visible_strokes(0).chain(overlay),
            &view,
        );
        self.renderer.submit(encoder);
        self.renderer.present(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
