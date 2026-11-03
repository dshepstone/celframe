//! Rules a document must meet to be saved or loaded.

use std::collections::HashSet;

use celframe_core::{Layer, Project};

use crate::FormatError;

fn invalid(reason: impl Into<String>) -> FormatError {
    FormatError::InvalidDocument(reason.into())
}

pub(crate) fn validate(project: &Project) -> Result<(), FormatError> {
    if project.scenes.is_empty() {
        return Err(invalid("a project needs at least one scene"));
    }
    let mut scene_ids = HashSet::new();
    for scene in &project.scenes {
        if !scene_ids.insert(scene.id) {
            return Err(invalid("two scenes share an ID"));
        }
        if scene.width == 0 || scene.height == 0 || scene.frame_rate == 0 {
            return Err(invalid(
                "a scene's size and frame rate must be greater than 0",
            ));
        }
        let mut layer_ids = HashSet::new();
        for layer in &scene.layers {
            if !layer_ids.insert(layer.id) {
                return Err(invalid("two layers share an ID"));
            }
            validate_layer(layer)?;
        }
    }
    Ok(())
}

fn validate_layer(layer: &Layer) -> Result<(), FormatError> {
    let mut drawing_ids = HashSet::new();
    for drawing in &layer.drawings {
        if !drawing_ids.insert(drawing.id) {
            return Err(invalid("two drawings share an ID"));
        }
        let mut stroke_ids = HashSet::new();
        for stroke in &drawing.strokes {
            if !stroke_ids.insert(stroke.id) {
                return Err(invalid("two strokes share an ID"));
            }
            if !stroke.base_width.is_finite() || stroke.base_width < 0.0 {
                return Err(invalid("a stroke's width can't be negative"));
            }
            for sample in &stroke.samples {
                if !sample.x.is_finite() || !sample.y.is_finite() {
                    return Err(invalid("a stroke has an invalid position"));
                }
                if !(0.0..=1.0).contains(&sample.pressure) {
                    return Err(invalid("a stroke's pressure must be from 0 to 1"));
                }
            }
        }
    }

    // Re-inserting applies the same exposure rules as editing, and checks they stay sorted.
    let mut rebuilt = Layer {
        exposures: Vec::with_capacity(layer.exposures.len()),
        ..layer.clone()
    };
    for exposure in &layer.exposures {
        rebuilt
            .insert_exposure(*exposure)
            .map_err(|error| invalid(error.to_string()))?;
    }
    if rebuilt.exposures != layer.exposures {
        return Err(invalid("a layer's exposures aren't sorted"));
    }
    Ok(())
}
