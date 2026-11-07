//! Golden-image test (issue #29): draw scripted strokes, save, reopen and render, then compare.
//!
//! Two checks:
//! - The render after reopening must match the render before saving, byte for byte.
//! - The render must match `tests/golden/<os>.png` within a small tolerance, because GPUs
//!   rasterise edges slightly differently. References come from the CI runners, so on another
//!   machine a mismatch is only a warning. In CI (`CELFRAME_REQUIRE_GPU`) it fails.
//!
//! On a mismatch the render is written to `target/golden/<os>-actual.png`. Set
//! `CELFRAME_UPDATE_GOLDEN=1` to write it as the new reference instead.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use celframe_core::Command;
use celframe_core::{
    Drawing, DrawingId, Exposure, History, Layer, LayerId, Project, ProjectId, Scene, SceneId,
    Stroke, StrokeId, StrokeSample,
};
use celframe_render::{Canvas, Image, Paper, Renderer, View};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const BACKDROP: [u8; 4] = [40, 42, 46, 255];
const PAPER: [u8; 4] = [255, 255, 255, 255];

/// A pixel counts as different when any channel differs by more than this.
const CHANNEL_TOLERANCE: u8 = 16;
/// At most this share of pixels may differ, which allows for anti-aliased edges.
const MAX_DIFFERENT_SHARE: f64 = 0.005;

fn renderer() -> Option<Renderer> {
    match Renderer::new_headless() {
        Ok(renderer) => Some(renderer),
        Err(error) if std::env::var_os("CELFRAME_REQUIRE_GPU").is_none() => {
            eprintln!("skipping: {error}");
            None
        }
        Err(error) => panic!("{error}"),
    }
}

/// Raw pen samples along a wave, with pressure rising and falling, as a pen would send them.
fn wave(offset_y: f64, amplitude: f64, turns: f64) -> Vec<StrokeSample> {
    (0..=240)
        .map(|i| {
            let t = f64::from(i) / 240.0;
            StrokeSample {
                x: -760.0 + 1520.0 * t,
                y: offset_y + amplitude * (t * turns * std::f64::consts::TAU).sin(),
                pressure: (0.2 + 0.8 * (t * std::f64::consts::PI).sin()) as f32,
            }
        })
        .collect()
}

/// Draws one stroke the way the app does: begin, append smoothed samples, end.
fn draw(
    project: &mut Project,
    history: &mut History,
    target: (SceneId, LayerId, DrawingId),
    stroke: (StrokeId, f64, [u8; 4]),
    raw: &[StrokeSample],
) {
    let smoothed = celframe_geom::smooth(raw, 0.5);
    let (scene, layer, drawing) = target;
    let (id, base_width, color) = stroke;
    let first = Stroke {
        id,
        base_width,
        color,
        art: celframe_core::Art::Line,
        samples: smoothed[..1].to_vec(),
    };
    history
        .begin_stroke(
            project,
            Command::AddStroke {
                scene,
                layer,
                drawing,
                stroke: first,
                index: 0,
            },
        )
        .unwrap();
    for chunk in smoothed[1..].chunks(7) {
        history
            .execute(
                project,
                Command::AppendStrokeSamples {
                    scene,
                    layer,
                    drawing,
                    stroke: id,
                    samples: chunk.to_vec(),
                },
            )
            .unwrap();
    }
    history.end_stroke();
}

/// A fixed project: two layers, a thick ink wave and a thinner red wave on top.
fn scripted_project() -> Project {
    let scene_id = SceneId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0002);
    let ink_id = LayerId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0003);
    let colour_id = LayerId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0004);
    let ink_drawing = DrawingId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0005);
    let colour_drawing = DrawingId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0006);

    let layer = |id, name: &str, drawing| {
        let mut layer = Layer {
            id,
            ..Layer::new(name)
        };
        layer.drawings.push(Drawing {
            id: drawing,
            strokes: Vec::new(),
        });
        layer.insert_exposure(Exposure::new(0, 1, drawing)).unwrap();
        layer
    };
    let mut project = Project {
        id: ProjectId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0001),
        name: "Golden".to_owned(),
        scenes: vec![Scene {
            id: scene_id,
            layers: vec![
                layer(ink_id, "Ink", ink_drawing),
                layer(colour_id, "Colour", colour_drawing),
            ],
            ..Scene::new("Shot")
        }],
    };

    let mut history = History::new();
    draw(
        &mut project,
        &mut history,
        (scene_id, ink_id, ink_drawing),
        (StrokeId::from_u128(0x10), 28.0, [22, 19, 15, 255]),
        &wave(-60.0, 260.0, 1.5),
    );
    draw(
        &mut project,
        &mut history,
        (scene_id, colour_id, colour_drawing),
        (StrokeId::from_u128(0x11), 10.0, [200, 50, 42, 255]),
        &wave(120.0, 140.0, 3.0),
    );
    project
}

fn render(renderer: &mut Renderer, scene: &Scene) -> Image {
    let paper = Paper::scene_frame(scene.width, scene.height, PAPER);
    let canvas = Canvas {
        paper: Some(paper),
        ..Canvas::full(WIDTH, HEIGHT, BACKDROP)
    };
    let view = View::fit(paper.rect, [f64::from(WIDTH), f64::from(HEIGHT)], 0.9);
    renderer
        .render_image(scene.visible_strokes(0), &view, &canvas)
        .unwrap()
}

fn write_png(path: &Path, image: &Image) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(path).unwrap()),
        image.width,
        image.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.pixels).unwrap();
}

fn read_png(path: &Path) -> Option<Image> {
    let decoder = png::Decoder::new(std::io::BufReader::new(File::open(path).ok()?));
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut pixels).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    pixels.truncate(info.buffer_size());
    Some(Image {
        width: info.width,
        height: info.height,
        pixels,
    })
}

/// The share of pixels where some channel differs by more than the tolerance.
fn different_share(a: &Image, b: &Image) -> f64 {
    let different = a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0)
        .filter(|(p, q)| {
            p.iter()
                .zip(*q)
                .any(|(x, y)| x.abs_diff(*y) > CHANNEL_TOLERANCE)
        })
        .count();
    different as f64 / f64::from(a.width * a.height)
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn actual_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/golden")
        .join(format!("{}-actual.png", std::env::consts::OS))
}

#[test]
fn a_saved_and_reopened_drawing_renders_identically_and_matches_the_reference() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let project = scripted_project();
    let before = render(&mut renderer, &project.scenes[0]);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("golden.celf");
    celframe_format::save_project(&project, &path).unwrap();
    let reopened = celframe_format::load_project(&path).unwrap();
    assert_eq!(reopened, project, "the file must reopen exactly");

    let after = render(&mut renderer, &reopened.scenes[0]);
    assert!(before == after, "reopening must not change a single pixel");

    // Sanity: something was drawn on the paper, and the backdrop is untouched.
    assert_eq!(after.pixel(2, 2), BACKDROP);
    assert!(after.pixels.as_chunks::<4>().0.contains(&[22, 19, 15, 255]));

    let reference = golden_dir().join(format!("{}.png", std::env::consts::OS));
    if std::env::var_os("CELFRAME_UPDATE_GOLDEN").is_some() {
        write_png(&reference, &after);
        eprintln!("wrote {}", reference.display());
        return;
    }
    let in_ci = std::env::var_os("CELFRAME_REQUIRE_GPU").is_some();
    let problem = match read_png(&reference) {
        None => Some(format!("no reference image at {}", reference.display())),
        Some(expected) if (expected.width, expected.height) != (WIDTH, HEIGHT) => {
            Some("the reference image is the wrong size".to_owned())
        }
        Some(expected) => {
            let share = different_share(&expected, &after);
            (share > MAX_DIFFERENT_SHARE).then(|| {
                format!(
                    "{:.3}% of pixels differ from {} (limit {:.3}%)",
                    share * 100.0,
                    reference.display(),
                    MAX_DIFFERENT_SHARE * 100.0
                )
            })
        }
    };
    if let Some(problem) = problem {
        write_png(&actual_path(), &after);
        let message = format!(
            "{problem}. The render is at {} on adapter {}.",
            actual_path().display(),
            renderer.adapter_name()
        );
        if in_ci {
            panic!("{message}");
        }
        eprintln!("warning (references come from CI runners): {message}");
    }
}
