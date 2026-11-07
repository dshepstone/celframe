//! The view transform from `docs/specs/units-and-coordinates.md`.

/// Which part of the scene the canvas shows. UI state, not part of the document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// The scene point shown at the middle of the canvas.
    pub centre: [f64; 2],
    /// Screen pixels per scene unit. Greater than 0.
    pub zoom: f64,
    /// Radians, counter-clockwise in scene space.
    pub rotation: f64,
}

/// The furthest out and in the view can zoom, in screen pixels per scene unit.
pub const MIN_ZOOM: f64 = 0.01;
pub const MAX_ZOOM: f64 = 256.0;

impl Default for View {
    fn default() -> Self {
        Self {
            centre: [0.0, 0.0],
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

impl View {
    /// Maps a scene point to screen pixels on a canvas of `size`, with +Y down.
    pub fn scene_to_screen(&self, point: [f64; 2], size: [f64; 2]) -> [f64; 2] {
        let [x, y] = self.rotated_offset(point);
        [size[0] / 2.0 + x, size[1] / 2.0 - y]
    }

    /// The inverse of `scene_to_screen`, for pointer input.
    pub fn screen_to_scene(&self, point: [f64; 2], size: [f64; 2]) -> [f64; 2] {
        let x = (point[0] - size[0] / 2.0) / self.zoom;
        let y = (size[1] / 2.0 - point[1]) / self.zoom;
        let (sin, cos) = (-self.rotation).sin_cos();
        [
            self.centre[0] + x * cos - y * sin,
            self.centre[1] + x * sin + y * cos,
        ]
    }

    /// Steps 1 to 3 of the transform: subtract the centre, rotate, then scale.
    fn rotated_offset(&self, point: [f64; 2]) -> [f64; 2] {
        let (dx, dy) = (point[0] - self.centre[0], point[1] - self.centre[1]);
        let (sin, cos) = self.rotation.sin_cos();
        [
            (dx * cos - dy * sin) * self.zoom,
            (dx * sin + dy * cos) * self.zoom,
        ]
    }

    /// The same transform as a 2 × 3 matrix from scene space to clip space (-1 to 1, +Y up),
    /// computed in `f64` so a far-off centre doesn't lose precision. Rows are `[a, b, c]` with
    /// `clip = a * x + b * y + c`.
    pub fn clip_matrix(&self, size: [f64; 2]) -> [[f32; 3]; 2] {
        let (sin, cos) = self.rotation.sin_cos();
        let (sx, sy) = (2.0 * self.zoom / size[0], 2.0 * self.zoom / size[1]);
        let row = |scale: f64, a: f64, b: f64| {
            let c = -(a * self.centre[0] + b * self.centre[1]);
            [(scale * a) as f32, (scale * b) as f32, (scale * c) as f32]
        };
        [row(sx, cos, -sin), row(sy, sin, cos)]
    }

    /// A view that shows all of `rect` (`[min x, min y, max x, max y]` in scene units), centred
    /// on a canvas of `size` pixels, filling `fill` of the canvas along the tighter direction.
    pub fn fit(rect: [f64; 4], size: [f64; 2], fill: f64) -> Self {
        let (width, height) = (rect[2] - rect[0], rect[3] - rect[1]);
        let zoom = (size[0] / width).min(size[1] / height) * fill;
        Self {
            centre: [(rect[0] + rect[2]) / 2.0, (rect[1] + rect[3]) / 2.0],
            zoom: if zoom.is_finite() && zoom > 0.0 {
                zoom
            } else {
                1.0
            },
            rotation: 0.0,
        }
    }

    /// Zooms by `factor`, keeping the scene point under `anchor` (screen pixels) still. Zoom
    /// stays within `MIN_ZOOM` and `MAX_ZOOM`.
    pub fn zoomed_about(&self, anchor: [f64; 2], size: [f64; 2], factor: f64) -> Self {
        if !factor.is_finite() || factor <= 0.0 {
            return *self;
        }
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let fixed = self.screen_to_scene(anchor, size);
        let mut next = Self { zoom, ..*self };
        next.keep_at(fixed, anchor, size);
        next
    }

    /// Moves the scene with the pointer: `delta` is how far it moved, in screen pixels.
    pub fn panned(&self, delta: [f64; 2], size: [f64; 2]) -> Self {
        let middle = [size[0] / 2.0, size[1] / 2.0];
        let centre = self.screen_to_scene([middle[0] - delta[0], middle[1] - delta[1]], size);
        Self { centre, ..*self }
    }

    /// Turns the view by `angle` radians counter-clockwise, keeping the scene point under
    /// `anchor` still. Rotation stays within one turn.
    pub fn rotated_about(&self, anchor: [f64; 2], size: [f64; 2], angle: f64) -> Self {
        if !angle.is_finite() {
            return *self;
        }
        let fixed = self.screen_to_scene(anchor, size);
        let rotation = (self.rotation + angle).rem_euclid(std::f64::consts::TAU);
        let mut next = Self { rotation, ..*self };
        next.keep_at(fixed, anchor, size);
        next
    }

    /// Moves the centre so `scene_point` shows at `screen_point`.
    fn keep_at(&mut self, scene_point: [f64; 2], screen_point: [f64; 2], size: [f64; 2]) {
        let now = self.screen_to_scene(screen_point, size);
        self.centre = [
            self.centre[0] + scene_point[0] - now[0],
            self.centre[1] + scene_point[1] - now[1],
        ];
    }

    /// The largest curve error, in scene units, that stays under a quarter of a screen pixel.
    pub fn tolerance(&self) -> f32 {
        (0.25 / self.zoom) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: [f64; 2] = [200.0, 100.0];

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
    }

    fn view(centre: [f64; 2], zoom: f64, rotation: f64) -> View {
        View {
            centre,
            zoom,
            rotation,
        }
    }

    #[test]
    fn the_centre_is_the_middle_of_the_canvas_and_y_points_up() {
        let v = View::default();
        assert!(close(v.scene_to_screen([0.0, 0.0], SIZE), [100.0, 50.0]));
        assert!(close(v.scene_to_screen([10.0, 20.0], SIZE), [110.0, 30.0]));
    }

    #[test]
    fn zoom_pan_and_rotation_follow_the_spec() {
        let v = view([10.0, 0.0], 2.0, std::f64::consts::FRAC_PI_2);
        // (20, 0) is 10 right of the centre. A quarter turn counter-clockwise points it up, and
        // zoom 2 makes it 20 pixels.
        assert!(close(v.scene_to_screen([20.0, 0.0], SIZE), [100.0, 30.0]));
    }

    #[test]
    fn screen_to_scene_inverts_scene_to_screen() {
        let v = view([-3.5, 12.0], 1.7, 0.6);
        for point in [[0.0, 0.0], [12.5, -40.0], [-99.0, 7.25]] {
            let back = v.screen_to_scene(v.scene_to_screen(point, SIZE), SIZE);
            assert!(close(back, point), "{back:?} vs {point:?}");
        }
    }

    #[test]
    fn fit_shows_the_whole_rect_in_the_middle() {
        // A 1920 × 1080 scene on a 1000 × 1000 canvas is limited by its width.
        let v = View::fit([-960.0, -540.0, 960.0, 540.0], [1000.0, 1000.0], 0.9);
        assert!((v.zoom - 900.0 / 1920.0).abs() < 1e-12);
        assert!(close(
            v.scene_to_screen([960.0, 0.0], [1000.0, 1000.0]),
            [950.0, 500.0]
        ));

        // An empty canvas still gives a usable view.
        assert_eq!(View::fit([0.0, 0.0, 10.0, 10.0], [0.0, 0.0], 0.9).zoom, 1.0);
    }

    #[test]
    fn the_clip_matrix_matches_scene_to_screen() {
        let v = view([1000.0, -250.0], 3.0, -0.4);
        let m = v.clip_matrix(SIZE);
        for point in [[1000.0, -250.0], [1010.0, -240.0], [990.0, -260.5]] {
            let screen = v.scene_to_screen(point, SIZE);
            let expected = [
                screen[0] / SIZE[0] * 2.0 - 1.0,
                1.0 - screen[1] / SIZE[1] * 2.0,
            ];
            let apply = |r: [f32; 3]| {
                f64::from(r[0]) * point[0] + f64::from(r[1]) * point[1] + f64::from(r[2])
            };
            let clip = [apply(m[0]), apply(m[1])];
            assert!(
                (clip[0] - expected[0]).abs() < 1e-4,
                "{clip:?} vs {expected:?}"
            );
            assert!(
                (clip[1] - expected[1]).abs() < 1e-4,
                "{clip:?} vs {expected:?}"
            );
        }
    }

    #[test]
    fn zooming_keeps_the_point_under_the_anchor_still() {
        let v = view([5.0, -2.0], 1.5, 0.3);
        let anchor = [37.0, 81.0];
        let under = v.screen_to_scene(anchor, SIZE);
        let zoomed = v.zoomed_about(anchor, SIZE, 2.5);
        assert!((zoomed.zoom - 3.75).abs() < 1e-12);
        assert!(close(zoomed.screen_to_scene(anchor, SIZE), under));
    }

    #[test]
    fn zoom_is_clamped_and_bad_factors_are_ignored() {
        let v = View::default();
        assert_eq!(v.zoomed_about([0.0, 0.0], SIZE, 1e9).zoom, MAX_ZOOM);
        assert_eq!(v.zoomed_about([0.0, 0.0], SIZE, 1e-9).zoom, MIN_ZOOM);
        assert_eq!(v.zoomed_about([0.0, 0.0], SIZE, f64::NAN), v);
        assert_eq!(v.zoomed_about([0.0, 0.0], SIZE, -2.0), v);
    }

    #[test]
    fn panning_moves_the_scene_with_the_pointer() {
        let v = view([0.0, 0.0], 2.0, 0.7);
        let point = [3.0, 4.0];
        let before = v.scene_to_screen(point, SIZE);
        let after = v.panned([15.0, -6.0], SIZE).scene_to_screen(point, SIZE);
        assert!(close(after, [before[0] + 15.0, before[1] - 6.0]));
    }

    #[test]
    fn rotating_keeps_the_anchor_still_and_wraps() {
        let v = view([1.0, 1.0], 1.0, 6.0);
        let anchor = [150.0, 20.0];
        let under = v.screen_to_scene(anchor, SIZE);
        let turned = v.rotated_about(anchor, SIZE, 1.0);
        assert!(close(turned.screen_to_scene(anchor, SIZE), under));
        assert!((turned.rotation - (7.0 - std::f64::consts::TAU)).abs() < 1e-12);
    }
}
