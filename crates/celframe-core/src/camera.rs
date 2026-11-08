//! The scene camera. See `docs/specs/scene-camera.md`.

use serde::{Deserialize, Serialize};

use crate::animated::Animated;
use crate::model::Stroke;

/// The camera's distance from the scene, in scene units, at which it frames exactly the
/// scene's width and height with a zoom of 1.
pub const STANDARD_DISTANCE: f64 = 1000.0;

/// Where the camera is and how it frames the scene. Every value is keyable.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    /// The scene point at the middle of the frame, in scene units, +Y up.
    pub x: Animated,
    pub y: Animated,
    /// Distance from the scene in scene units. Smaller is closer: a truck in.
    pub z: Animated,
    /// Degrees, counter-clockwise. Turning the camera turns the picture the other way.
    pub rotation: Animated,
    /// The lens's magnification. 2 frames half as much of the scene, with no change in depth.
    pub zoom: Animated,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: Animated::constant(0.0),
            y: Animated::constant(0.0),
            z: Animated::constant(STANDARD_DISTANCE),
            rotation: Animated::constant(0.0),
            zoom: Animated::constant(1.0),
        }
    }
}

impl Camera {
    /// How the camera frames the scene at a frame.
    pub fn framing(&self, frame: u32) -> Framing {
        let z = self.z.at(frame).max(*CameraProperty::Z.range().start());
        let zoom = self
            .zoom
            .at(frame)
            .max(*CameraProperty::Zoom.range().start());
        Framing {
            centre: [self.x.at(frame), self.y.at(frame)],
            rotation: self.rotation.at(frame).to_radians(),
            scale: zoom * STANDARD_DISTANCE / z,
        }
    }

    /// True when every value and key is in its property's range, with keys in frame order.
    pub fn is_valid(&self) -> bool {
        CameraProperty::ALL
            .iter()
            .all(|&p| p.of(self).is_valid(p.range()))
    }
}

/// A keyable value on the scene camera.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CameraProperty {
    X,
    Y,
    Z,
    Rotation,
    Zoom,
}

impl CameraProperty {
    pub const ALL: [Self; 5] = [Self::X, Self::Y, Self::Z, Self::Rotation, Self::Zoom];

    /// The values this property accepts.
    pub fn range(self) -> std::ops::RangeInclusive<f64> {
        match self {
            Self::X | Self::Y => -1.0e6..=1.0e6,
            Self::Z => 1.0..=1.0e5,
            Self::Rotation => -3600.0..=3600.0,
            Self::Zoom => 0.01..=100.0,
        }
    }

    pub fn of(self, camera: &Camera) -> &Animated {
        match self {
            Self::X => &camera.x,
            Self::Y => &camera.y,
            Self::Z => &camera.z,
            Self::Rotation => &camera.rotation,
            Self::Zoom => &camera.zoom,
        }
    }

    pub(crate) fn of_mut(self, camera: &mut Camera) -> &mut Animated {
        match self {
            Self::X => &mut camera.x,
            Self::Y => &mut camera.y,
            Self::Z => &mut camera.z,
            Self::Rotation => &mut camera.rotation,
            Self::Zoom => &mut camera.zoom,
        }
    }
}

/// Maps scene space to gate space: the camera's picture, where the gate is the scene's frame
/// centred on the origin. A turn, a uniform scale and a shift, so strokes keep their shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    /// The scene point shown at the middle of the gate.
    pub centre: [f64; 2],
    /// Radians, counter-clockwise: the camera's turn.
    pub rotation: f64,
    /// Gate units per scene unit. Greater than 0.
    pub scale: f64,
}

impl Framing {
    /// The standard camera: gate space is scene space.
    pub const IDENTITY: Self = Self {
        centre: [0.0, 0.0],
        rotation: 0.0,
        scale: 1.0,
    };

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    pub fn to_gate(&self, point: [f64; 2]) -> [f64; 2] {
        let (dx, dy) = (point[0] - self.centre[0], point[1] - self.centre[1]);
        let (sin, cos) = (-self.rotation).sin_cos();
        [
            (dx * cos - dy * sin) * self.scale,
            (dx * sin + dy * cos) * self.scale,
        ]
    }

    /// The inverse of `to_gate`, for pointer input.
    pub fn to_scene(&self, point: [f64; 2]) -> [f64; 2] {
        let (x, y) = (point[0] / self.scale, point[1] / self.scale);
        let (sin, cos) = self.rotation.sin_cos();
        [
            self.centre[0] + x * cos - y * sin,
            self.centre[1] + x * sin + y * cos,
        ]
    }

    /// A copy of a stroke as the camera shows it, its width scaled with it.
    pub fn stroke(&self, stroke: &Stroke) -> Stroke {
        let mut framed = stroke.clone();
        framed.base_width *= self.scale;
        for sample in &mut framed.samples {
            [sample.x, sample.y] = self.to_gate([sample.x, sample.y]);
        }
        framed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animated::{Interpolation, Key};

    fn close(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
    }

    #[test]
    fn the_standard_camera_frames_the_scene_unchanged() {
        let framing = Camera::default().framing(0);
        assert!(framing.is_identity());
        assert_eq!(framing.to_gate([12.0, -3.0]), [12.0, -3.0]);
    }

    #[test]
    fn trucking_in_halfway_doubles_the_picture() {
        let camera = Camera {
            z: Animated::constant(STANDARD_DISTANCE / 2.0),
            x: Animated::constant(100.0),
            ..Camera::default()
        };
        let framing = camera.framing(0);
        assert_eq!(framing.scale, 2.0);
        assert!(close(framing.to_gate([110.0, 5.0]), [20.0, 10.0]));
    }

    #[test]
    fn turning_the_camera_turns_the_picture_the_other_way() {
        let camera = Camera {
            rotation: Animated::constant(90.0),
            ..Camera::default()
        };
        let framing = camera.framing(0);
        assert!(close(framing.to_gate([0.0, 10.0]), [10.0, 0.0]));
    }

    #[test]
    fn to_scene_reverses_to_gate() {
        let framing = Framing {
            centre: [40.0, -7.0],
            rotation: 0.6,
            scale: 1.7,
        };
        let point = [-123.0, 456.0];
        assert!(close(framing.to_scene(framing.to_gate(point)), point));
    }

    #[test]
    fn keys_move_the_camera_over_time() {
        let mut camera = Camera::default();
        let key = |frame, value| Key {
            frame,
            value,
            interpolation: Interpolation::Linear,
        };
        camera.z.set_key(key(0, 1000.0));
        camera.z.set_key(key(10, 500.0));
        assert_eq!(camera.framing(0).scale, 1.0);
        assert_eq!(camera.framing(10).scale, 2.0);
        assert!(camera.framing(5).scale > 1.0 && camera.framing(5).scale < 2.0);
    }

    #[test]
    fn out_of_range_values_are_invalid() {
        let mut camera = Camera::default();
        assert!(camera.is_valid());
        camera.z = Animated::constant(0.0);
        assert!(!camera.is_valid());
    }
}
