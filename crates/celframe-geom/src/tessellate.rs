//! Turns a stroke into triangles. The shape is specified in `docs/specs/stroke-shape.md`.

use celframe_core::Stroke;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions,
    StrokeTessellator, StrokeVertex, TessellationError, VertexBuffers,
};

/// lyon produces invalid vertices for zero width, so narrower points are drawn this wide. It is
/// far below anything visible.
const MIN_WIDTH: f32 = 1e-3;

/// Triangles in scene space. Every three indices form one triangle.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StrokeMesh {
    pub positions: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl StrokeMesh {
    pub fn clear(&mut self) {
        self.positions.clear();
        self.indices.clear();
    }
}

/// Appends a stroke's triangles to `mesh`. `tolerance` is in scene units.
pub fn tessellate_stroke(stroke: &Stroke, tolerance: f32, mesh: &mut StrokeMesh) {
    let Some(first) = stroke.samples.first() else {
        return;
    };
    let width = |pressure: f32| stroke.base_width as f32 * pressure;
    let widest = stroke
        .samples
        .iter()
        .map(|s| width(s.pressure))
        .fold(0.0_f32, f32::max);
    // f32::max skips NaN, so `widest` is never NaN.
    if widest <= 0.0 {
        return;
    }

    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let all_in_one_place = stroke
        .samples
        .iter()
        .all(|s| s.x == first.x && s.y == first.y);
    let result = if all_in_one_place {
        // lyon draws nothing for a zero-length line, so a dot is a filled circle.
        dot(
            point(first.x as f32, first.y as f32),
            widest / 2.0,
            tolerance,
            &mut buffers,
        )
    } else {
        band(stroke, width, tolerance, &mut buffers)
    };

    // A stroke lyon can't handle, such as one with huge coordinates, is skipped rather than drawn
    // half-finished or with invalid vertices.
    let valid = buffers.vertices.iter().flatten().all(|v| v.is_finite());
    if result.is_err() || !valid {
        return;
    }
    // lyon numbers vertices from zero, so offset them past what the mesh already holds.
    let offset = u32::try_from(mesh.positions.len()).expect("the mesh has too many vertices");
    mesh.positions.extend_from_slice(&buffers.vertices);
    mesh.indices
        .extend(buffers.indices.iter().map(|index| index + offset));
}

fn band(
    stroke: &Stroke,
    width: impl Fn(f32) -> f32,
    tolerance: f32,
    buffers: &mut VertexBuffers<[f32; 2], u32>,
) -> Result<(), TessellationError> {
    let width = |pressure: f32| width(pressure).max(MIN_WIDTH);
    // The only attribute is the line width at each point.
    let mut builder = Path::builder_with_attributes(1);
    let mut samples = stroke.samples.iter();
    if let Some(first) = samples.next() {
        builder.begin(
            point(first.x as f32, first.y as f32),
            &[width(first.pressure)],
        );
    }
    for sample in samples {
        builder.line_to(
            point(sample.x as f32, sample.y as f32),
            &[width(sample.pressure)],
        );
    }
    builder.end(false);

    let options = StrokeOptions::default()
        .with_variable_line_width(0)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round)
        .with_tolerance(tolerance);
    StrokeTessellator::new().tessellate_path(
        &builder.build(),
        &options,
        &mut BuffersBuilder::new(buffers, |v: StrokeVertex| v.position().to_array()),
    )
}

fn dot(
    centre: lyon::math::Point,
    radius: f32,
    tolerance: f32,
    buffers: &mut VertexBuffers<[f32; 2], u32>,
) -> Result<(), TessellationError> {
    FillTessellator::new().tessellate_circle(
        centre,
        radius,
        &FillOptions::tolerance(tolerance),
        &mut BuffersBuilder::new(buffers, |v: FillVertex| v.position().to_array()),
    )
}
