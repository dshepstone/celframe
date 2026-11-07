use std::f64::consts::FRAC_PI_2;

use celframe_core::{Stroke, StrokeId, StrokeSample};
use celframe_render::{Canvas, Image, Paper, Renderer, View};

const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const SIZE: u32 = 64;

/// Returns a renderer, or `None` to skip when this machine has no adapter at all. CI sets
/// `CELFRAME_REQUIRE_GPU` so a missing adapter fails there instead of passing silently.
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

fn line(from: (f64, f64), to: (f64, f64), width: f64, color: [u8; 4]) -> Stroke {
    Stroke {
        id: StrokeId::new(),
        base_width: width,
        color,
        art: celframe_core::Art::Line,
        samples: vec![
            StrokeSample {
                x: from.0,
                y: from.1,
                pressure: 1.0,
            },
            StrokeSample {
                x: to.0,
                y: to.1,
                pressure: 1.0,
            },
        ],
    }
}

fn render(strokes: &[Stroke], view: View) -> Option<Image> {
    let mut renderer = renderer()?;
    Some(
        renderer
            .render_image(strokes, &view, &Canvas::full(SIZE, SIZE, WHITE))
            .unwrap(),
    )
}

/// True when every channel is within `slack` of `expected`.
fn near(actual: [u8; 4], expected: [u8; 4], slack: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, e)| a.abs_diff(e) <= slack)
}

/// The horizontal line y = 0 from x = -20 to 20, 10 units wide.
fn horizontal() -> Stroke {
    line((-20.0, 0.0), (20.0, 0.0), 10.0, BLACK)
}

#[test]
fn an_empty_canvas_is_the_background() {
    let Some(image) = render(&[], View::default()) else {
        return;
    };
    assert_eq!((image.width, image.height), (SIZE, SIZE));
    assert_eq!(image.pixels.len(), (SIZE * SIZE * 4) as usize);
    assert!(image.pixels.chunks(4).all(|p| p == WHITE));
}

#[test]
fn a_stroke_covers_the_pixels_under_it() {
    let Some(image) = render(&[horizontal()], View::default()) else {
        return;
    };
    // The canvas middle is scene (0, 0). The band covers rows 27 to 36 and columns 12 to 51,
    // plus round ends.
    assert_eq!(image.pixel(32, 32), BLACK);
    assert_eq!(image.pixel(14, 30), BLACK);
    assert_eq!(image.pixel(32, 24), WHITE);
    assert_eq!(image.pixel(32, 40), WHITE);
    assert_eq!(image.pixel(4, 32), WHITE);
    assert_eq!(image.pixel(60, 32), WHITE);
}

#[test]
fn edges_are_anti_aliased() {
    // Edges at screen rows 27.75 and 36.75, so the rows they cross are partly covered.
    let stroke = line((-20.0, 0.25), (20.0, 0.25), 9.0, BLACK);
    let Some(image) = render(&[stroke], View::default()) else {
        return;
    };
    let column: Vec<u8> = (20..44).map(|y| image.pixel(32, y)[0]).collect();
    assert!(column.iter().any(|&v| v > 40 && v < 215), "{column:?}");
}

#[test]
fn positive_y_is_up() {
    let stroke = line((-20.0, 15.0), (20.0, 15.0), 4.0, BLACK);
    let Some(image) = render(&[stroke], View::default()) else {
        return;
    };
    assert_eq!(image.pixel(32, 16), BLACK);
    assert_eq!(image.pixel(32, 48), WHITE);
}

#[test]
fn the_view_zooms_pans_and_rotates() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let mut draw = |view: View| {
        renderer
            .render_image(&[horizontal()], &view, &Canvas::full(SIZE, SIZE, WHITE))
            .unwrap()
    };

    // Zoom 2: the band is 20 pixels tall.
    let zoomed = draw(View {
        zoom: 2.0,
        ..View::default()
    });
    assert_eq!(zoomed.pixel(32, 24), BLACK);
    assert_eq!(zoomed.pixel(32, 20), WHITE);

    // Centre on (30, 0): the right end at x = 20 moves to column 22.
    let panned = draw(View {
        centre: [30.0, 0.0],
        ..View::default()
    });
    assert_eq!(panned.pixel(20, 32), BLACK);
    assert_eq!(panned.pixel(30, 32), WHITE);

    // A quarter turn makes the horizontal band vertical.
    let turned = draw(View {
        rotation: FRAC_PI_2,
        ..View::default()
    });
    assert_eq!(turned.pixel(32, 14), BLACK);
    assert_eq!(turned.pixel(14, 32), WHITE);
}

#[test]
fn colours_blend_and_later_strokes_are_on_top() {
    let red = line((-20.0, 0.0), (20.0, 0.0), 10.0, [255, 0, 0, 255]);
    let blue_on_top = line((0.0, -20.0), (0.0, 20.0), 10.0, [0, 0, 255, 255]);
    let half_black = line((-20.0, 20.0), (20.0, 20.0), 6.0, [0, 0, 0, 128]);
    let Some(image) = render(&[red, blue_on_top, half_black], View::default()) else {
        return;
    };
    assert_eq!(image.pixel(20, 32), [255, 0, 0, 255]);
    assert_eq!(image.pixel(32, 32), [0, 0, 255, 255]);
    // Half-transparent black over white is mid grey.
    assert!(
        near(image.pixel(20, 12), [127, 127, 127, 255], 2),
        "{:?}",
        image.pixel(20, 12)
    );
}

#[test]
fn rendering_is_repeatable() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let strokes = [
        horizontal(),
        line((-10.0, -25.0), (25.0, 10.0), 3.0, [20, 120, 60, 200]),
    ];
    let view = View {
        centre: [2.0, 3.0],
        zoom: 1.3,
        rotation: 0.2,
    };
    let first = renderer
        .render_image(&strokes, &view, &Canvas::full(SIZE, SIZE, WHITE))
        .unwrap();
    let second = renderer
        .render_image(&strokes, &view, &Canvas::full(SIZE, SIZE, WHITE))
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn odd_sizes_read_back_correctly() {
    // 37 × 5 pixels is 148 bytes a row, padded to 256 in the copy. Padding must not leak in.
    let Some(mut renderer) = renderer() else {
        return;
    };
    let image = renderer
        .render_image(
            &[],
            &View::default(),
            &Canvas::full(37, 5, [10, 20, 30, 255]),
        )
        .unwrap();
    assert_eq!(image.pixels.len(), 37 * 5 * 4);
    assert!(image.pixels.chunks(4).all(|p| p == [10, 20, 30, 255]));
}

#[test]
fn zero_sized_images_are_refused() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    assert!(
        renderer
            .render_image(&[], &View::default(), &Canvas::full(0, 10, WHITE))
            .is_err()
    );
}

const GREY: [u8; 4] = [200, 200, 200, 255];

#[test]
fn paper_is_drawn_behind_strokes_with_the_backdrop_around_it() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let canvas = Canvas {
        paper: Some(Paper::scene_frame(40, 30, WHITE)),
        ..Canvas::full(SIZE, SIZE, GREY)
    };
    let image = renderer
        .render_image(&[horizontal()], &View::default(), &canvas)
        .unwrap();
    // The paper covers columns 12 to 51 and rows 17 to 46.
    assert_eq!(image.pixel(2, 2), GREY);
    assert_eq!(image.pixel(32, 20), WHITE);
    assert_eq!(image.pixel(32, 32), BLACK);
    assert_eq!(image.pixel(32, 50), GREY);
}

#[test]
fn only_the_viewport_shows_the_scene() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    // The right half of a 128 × 64 target.
    let canvas = Canvas {
        target_size: [128, 64],
        viewport: [64, 0, 64, 64],
        backdrop: GREY,
        paper: Some(Paper::scene_frame(1000, 1000, WHITE)),
    };
    let image = renderer
        .render_image(&[horizontal()], &View::default(), &canvas)
        .unwrap();
    // The view centre is the middle of the viewport, column 96.
    assert_eq!(image.pixel(96, 32), BLACK);
    assert_eq!(image.pixel(66, 10), WHITE);
    // The paper is clipped to the viewport, so the left half stays backdrop.
    assert_eq!(image.pixel(32, 32), GREY);
    assert_eq!(image.pixel(60, 10), GREY);
}

#[test]
fn a_viewport_outside_the_target_draws_only_the_backdrop() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let canvas = Canvas {
        viewport: [500, 500, 10, 10],
        ..Canvas::full(SIZE, SIZE, GREY)
    };
    let image = renderer
        .render_image(&[horizontal()], &View::default(), &canvas)
        .unwrap();
    assert!(image.pixels.chunks(4).all(|p| p == GREY));
}
