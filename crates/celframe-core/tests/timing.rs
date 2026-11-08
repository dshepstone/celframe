use celframe_core::timing::{delete_frames, hold, insert_frames, scene_length, with_new_drawing};
use celframe_core::{DrawingId, Exposure, Layer, Scene};

/// Exposures as (start, length, which drawing).
fn spans(exposures: &[Exposure], ids: &[DrawingId]) -> Vec<(u32, u32, usize)> {
    exposures
        .iter()
        .map(|e| {
            let which = ids.iter().position(|d| *d == e.drawing).unwrap();
            (e.start_frame, e.length, which)
        })
        .collect()
}

/// Drawing A held for frames 0 to 3, a gap, then B for frames 6 and 7.
fn setup() -> (Vec<Exposure>, Vec<DrawingId>) {
    let ids = vec![DrawingId::new(), DrawingId::new(), DrawingId::new()];
    let exposures = vec![Exposure::new(0, 4, ids[0]), Exposure::new(6, 2, ids[1])];
    (exposures, ids)
}

#[test]
fn a_new_drawing_inside_a_hold_keeps_the_frames_around_it() {
    let (e, ids) = setup();
    let after = with_new_drawing(&e, 1, ids[2]);
    assert_eq!(
        spans(&after, &ids),
        [(0, 1, 0), (1, 1, 2), (2, 2, 0), (6, 2, 1)]
    );
    let at_start = with_new_drawing(&e, 0, ids[2]);
    assert_eq!(spans(&at_start, &ids), [(0, 1, 2), (1, 3, 0), (6, 2, 1)]);
    let in_gap = with_new_drawing(&e, 4, ids[2]);
    assert_eq!(spans(&in_gap, &ids), [(0, 4, 0), (4, 1, 2), (6, 2, 1)]);
}

#[test]
fn holds_lengthen_and_shorten_moving_later_exposures() {
    let (e, ids) = setup();
    assert_eq!(
        spans(&hold(&e, 2, 1).unwrap(), &ids),
        [(0, 5, 0), (7, 2, 1)]
    );
    assert_eq!(
        spans(&hold(&e, 2, -1).unwrap(), &ids),
        [(0, 3, 0), (5, 2, 1)]
    );
    let one = vec![Exposure::new(0, 1, ids[0])];
    assert!(hold(&one, 0, -1).is_none(), "never shorter than one frame");
    assert!(
        hold(&e, 5, 1).is_none(),
        "an empty frame has nothing to hold"
    );
}

#[test]
fn inserting_a_frame_splits_a_hold_and_moves_the_rest() {
    let (e, ids) = setup();
    assert_eq!(
        spans(&insert_frames(&e, 2, 1).unwrap(), &ids),
        [(0, 2, 0), (3, 2, 0), (7, 2, 1)]
    );
    assert_eq!(
        spans(&insert_frames(&e, 0, 2).unwrap(), &ids),
        [(2, 4, 0), (8, 2, 1)]
    );
    assert!(
        insert_frames(&e, 20, 1).is_none(),
        "past the end changes nothing"
    );
}

#[test]
fn deleting_frames_trims_holds_and_closes_the_gap() {
    let (e, ids) = setup();
    assert_eq!(
        spans(&delete_frames(&e, 2, 1).unwrap(), &ids),
        [(0, 3, 0), (5, 2, 1)]
    );
    // Cutting frames 3 to 6 removes the end of A, the gap, and the start of B.
    assert_eq!(
        spans(&delete_frames(&e, 3, 4).unwrap(), &ids),
        [(0, 3, 0), (3, 1, 1)]
    );
    let one = vec![Exposure::new(0, 1, ids[0]), Exposure::new(1, 1, ids[1])];
    assert_eq!(
        spans(&delete_frames(&one, 0, 1).unwrap(), &ids),
        [(0, 1, 1)]
    );
}

#[test]
fn the_scene_runs_to_its_last_exposure() {
    let mut scene = Scene::new("Shot");
    assert_eq!(scene_length(&scene), 1);
    let mut layer = Layer::new("Ink");
    let drawing = celframe_core::Drawing::new();
    let id = drawing.id;
    layer.drawings.push(drawing);
    layer.insert_exposure(Exposure::new(10, 5, id)).unwrap();
    scene.layers.push(layer);
    assert_eq!(scene_length(&scene), 15);
}
