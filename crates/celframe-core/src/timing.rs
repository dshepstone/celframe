//! Timing edits on one layer's exposures. See `docs/specs/timing.md`.
//!
//! Each function returns the layer's exposures after the edit, or None when it changes
//! nothing. `replace_exposures` turns a new list into undoable commands.

use crate::command::Command;
use crate::ids::{DrawingId, LayerId, SceneId};
use crate::model::{Exposure, Layer, Scene};

fn end(e: &Exposure) -> u32 {
    e.start_frame.saturating_add(e.length)
}

fn covering(exposures: &[Exposure], frame: u32) -> Option<usize> {
    exposures
        .iter()
        .position(|e| e.start_frame <= frame && frame < end(e))
}

fn changed(before: &[Exposure], after: Vec<Exposure>) -> Option<Vec<Exposure>> {
    (after != before).then_some(after)
}

/// The scene's length in frames: to the end of its last exposure, and at least 1.
pub fn scene_length(scene: &Scene) -> u32 {
    scene
        .layers
        .iter()
        .flat_map(|l| l.exposures.iter().map(end))
        .max()
        .unwrap_or(0)
        .max(1)
}

/// Exposes `drawing` at `frame` for one frame. A hold across the frame keeps the frames before
/// and after it.
pub fn with_new_drawing(exposures: &[Exposure], frame: u32, drawing: DrawingId) -> Vec<Exposure> {
    let mut out = Vec::with_capacity(exposures.len() + 2);
    for e in exposures {
        if e.start_frame <= frame && frame < end(e) {
            if e.start_frame < frame {
                out.push(Exposure::new(
                    e.start_frame,
                    frame - e.start_frame,
                    e.drawing,
                ));
            }
            if frame + 1 < end(e) {
                out.push(Exposure::new(frame + 1, end(e) - frame - 1, e.drawing));
            }
        } else {
            out.push(*e);
        }
    }
    out.push(Exposure::new(frame, 1, drawing));
    out.sort_by_key(|e| e.start_frame);
    out
}

/// Exposes `drawing` from `frame`, as typing its number into an exposure sheet does. Inside a
/// hold, the hold ends just before the frame and the drawing holds for the rest of it; on an
/// empty frame it lasts one frame.
pub fn expose(exposures: &[Exposure], frame: u32, drawing: DrawingId) -> Option<Vec<Exposure>> {
    let mut out = Vec::with_capacity(exposures.len() + 1);
    match covering(exposures, frame) {
        Some(index) => {
            let e = exposures[index];
            if e.drawing == drawing && e.start_frame == frame {
                return None;
            }
            for (i, other) in exposures.iter().enumerate() {
                if i != index {
                    out.push(*other);
                } else if e.start_frame < frame {
                    out.push(Exposure::new(
                        e.start_frame,
                        frame - e.start_frame,
                        e.drawing,
                    ));
                    out.push(Exposure::new(frame, end(&e) - frame, drawing));
                } else {
                    out.push(Exposure::new(frame, e.length, drawing));
                }
            }
        }
        None => {
            out.extend_from_slice(exposures);
            out.push(Exposure::new(frame, 1, drawing));
            out.sort_by_key(|e| e.start_frame);
        }
    }
    changed(exposures, out)
}

/// Empties one frame without moving anything: a hold across it keeps its frames on either side.
pub fn clear_frame(exposures: &[Exposure], frame: u32) -> Option<Vec<Exposure>> {
    let index = covering(exposures, frame)?;
    let e = exposures[index];
    let mut out = Vec::with_capacity(exposures.len() + 1);
    for (i, other) in exposures.iter().enumerate() {
        if i != index {
            out.push(*other);
            continue;
        }
        if e.start_frame < frame {
            out.push(Exposure::new(
                e.start_frame,
                frame - e.start_frame,
                e.drawing,
            ));
        }
        if frame + 1 < end(&e) {
            out.push(Exposure::new(frame + 1, end(&e) - frame - 1, e.drawing));
        }
    }
    changed(exposures, out)
}

/// Lengthens or shortens the exposure at `frame` by `by` frames, moving later exposures with
/// it. Never shortens below one frame.
pub fn hold(exposures: &[Exposure], frame: u32, by: i64) -> Option<Vec<Exposure>> {
    let index = covering(exposures, frame)?;
    let target = exposures[index];
    let length = (i64::from(target.length) + by).max(1);
    let shift = length - i64::from(target.length);
    if shift == 0 {
        return None;
    }
    let after = exposures
        .iter()
        .map(|e| {
            if e.start_frame == target.start_frame {
                Exposure::new(e.start_frame, length as u32, e.drawing)
            } else if e.start_frame > target.start_frame {
                Exposure::new(
                    (i64::from(e.start_frame) + shift) as u32,
                    e.length,
                    e.drawing,
                )
            } else {
                *e
            }
        })
        .collect();
    changed(exposures, after)
}

/// Inserts `count` empty frames at `frame`, splitting a hold across it.
pub fn insert_frames(exposures: &[Exposure], frame: u32, count: u32) -> Option<Vec<Exposure>> {
    if count == 0 {
        return None;
    }
    let mut out = Vec::with_capacity(exposures.len() + 1);
    for e in exposures {
        if end(e) <= frame {
            out.push(*e);
        } else if e.start_frame >= frame {
            out.push(Exposure::new(e.start_frame + count, e.length, e.drawing));
        } else {
            out.push(Exposure::new(
                e.start_frame,
                frame - e.start_frame,
                e.drawing,
            ));
            out.push(Exposure::new(frame + count, end(e) - frame, e.drawing));
        }
    }
    changed(exposures, out)
}

/// Removes `count` frames starting at `frame`, moving everything after them earlier.
pub fn delete_frames(exposures: &[Exposure], frame: u32, count: u32) -> Option<Vec<Exposure>> {
    if count == 0 {
        return None;
    }
    let cut_end = frame.saturating_add(count);
    let mut out: Vec<Exposure> = Vec::with_capacity(exposures.len());
    for e in exposures {
        // Frames of this exposure before the cut, and after it.
        let before = end(e).min(frame).saturating_sub(e.start_frame);
        let after_start = e.start_frame.max(cut_end);
        let after = end(e).saturating_sub(after_start);
        let length = before + after;
        if length == 0 {
            continue;
        }
        let start = if e.start_frame < frame {
            e.start_frame
        } else {
            after_start - count
        };
        out.push(Exposure::new(start, length, e.drawing));
    }
    changed(exposures, out)
}

/// Commands that change a layer's exposures to `after`, as one edit when batched.
pub fn replace_exposures(scene: SceneId, layer: &Layer, after: &[Exposure]) -> Vec<Command> {
    let removals = layer.exposures.iter().map(|e| Command::RemoveExposure {
        scene,
        layer: layer.id,
        start_frame: e.start_frame,
    });
    let insertions = after.iter().map(|e| Command::InsertExposure {
        scene,
        layer: layer.id,
        exposure: *e,
    });
    removals.chain(insertions).collect()
}

/// The layer with this ID, for building timing commands.
pub fn layer_in(scene: &Scene, id: LayerId) -> Option<&Layer> {
    scene.layers.iter().find(|l| l.id == id)
}
