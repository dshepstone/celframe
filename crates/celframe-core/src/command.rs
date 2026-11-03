//! Edits to a document. Each command returns the command that reverses it.

use serde::{Deserialize, Serialize};

use crate::ids::{DrawingId, LayerId, SceneId, StrokeId};
use crate::model::{Drawing, Exposure, ExposureError, Layer, Project, Scene, Stroke};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    RenameProject {
        name: String,
    },
    AddScene {
        scene: Scene,
        index: usize,
    },
    RemoveScene {
        scene: SceneId,
    },
    RenameScene {
        scene: SceneId,
        name: String,
    },
    AddLayer {
        scene: SceneId,
        layer: Layer,
        index: usize,
    },
    RemoveLayer {
        scene: SceneId,
        layer: LayerId,
    },
    MoveLayer {
        scene: SceneId,
        layer: LayerId,
        index: usize,
    },
    RenameLayer {
        scene: SceneId,
        layer: LayerId,
        name: String,
    },
    SetLayerVisible {
        scene: SceneId,
        layer: LayerId,
        visible: bool,
    },
    SetLayerLocked {
        scene: SceneId,
        layer: LayerId,
        locked: bool,
    },
    AddDrawing {
        scene: SceneId,
        layer: LayerId,
        drawing: Drawing,
        index: usize,
    },
    RemoveDrawing {
        scene: SceneId,
        layer: LayerId,
        drawing: DrawingId,
    },
    InsertExposure {
        scene: SceneId,
        layer: LayerId,
        exposure: Exposure,
    },
    RemoveExposure {
        scene: SceneId,
        layer: LayerId,
        start_frame: u32,
    },
    AddStroke {
        scene: SceneId,
        layer: LayerId,
        drawing: DrawingId,
        stroke: Stroke,
        index: usize,
    },
    RemoveStroke {
        scene: SceneId,
        layer: LayerId,
        drawing: DrawingId,
        stroke: StrokeId,
    },
}

impl Command {
    /// Applies the command and returns its inverse. On error, the project is unchanged.
    pub fn apply(self, project: &mut Project) -> Result<Command, CommandError> {
        match self {
            Command::RenameProject { name } => {
                let name = std::mem::replace(&mut project.name, name);
                Ok(Command::RenameProject { name })
            }
            Command::AddScene { scene, index } => {
                check_insert(project.scenes.len(), index)?;
                if project.scenes.iter().any(|s| s.id == scene.id) {
                    return Err(CommandError::DuplicateId);
                }
                let id = scene.id;
                project.scenes.insert(index, scene);
                Ok(Command::RemoveScene { scene: id })
            }
            Command::RemoveScene { scene } => {
                if project.scenes.len() == 1 && project.scenes[0].id == scene {
                    return Err(CommandError::LastScene);
                }
                let index = position(&project.scenes, |s| s.id == scene)?;
                let scene = project.scenes.remove(index);
                Ok(Command::AddScene { scene, index })
            }
            Command::RenameScene { scene, name } => {
                let name = std::mem::replace(&mut find_scene(project, scene)?.name, name);
                Ok(Command::RenameScene { scene, name })
            }
            Command::AddLayer {
                scene,
                layer,
                index,
            } => {
                let layers = &mut find_scene(project, scene)?.layers;
                check_insert(layers.len(), index)?;
                if layers.iter().any(|l| l.id == layer.id) {
                    return Err(CommandError::DuplicateId);
                }
                let id = layer.id;
                layers.insert(index, layer);
                Ok(Command::RemoveLayer { scene, layer: id })
            }
            Command::RemoveLayer { scene, layer } => {
                let layers = &mut find_scene(project, scene)?.layers;
                let index = position(layers, |l| l.id == layer)?;
                let layer = layers.remove(index);
                Ok(Command::AddLayer {
                    scene,
                    layer,
                    index,
                })
            }
            Command::MoveLayer {
                scene,
                layer,
                index,
            } => {
                let layers = &mut find_scene(project, scene)?.layers;
                let from = position(layers, |l| l.id == layer)?;
                if index >= layers.len() {
                    return Err(CommandError::InvalidIndex);
                }
                let moved = layers.remove(from);
                layers.insert(index, moved);
                Ok(Command::MoveLayer {
                    scene,
                    layer,
                    index: from,
                })
            }
            Command::RenameLayer { scene, layer, name } => {
                let target = find_layer(project, scene, layer)?;
                let name = std::mem::replace(&mut target.name, name);
                Ok(Command::RenameLayer { scene, layer, name })
            }
            Command::SetLayerVisible {
                scene,
                layer,
                visible,
            } => {
                let target = find_layer(project, scene, layer)?;
                let visible = std::mem::replace(&mut target.visible, visible);
                Ok(Command::SetLayerVisible {
                    scene,
                    layer,
                    visible,
                })
            }
            Command::SetLayerLocked {
                scene,
                layer,
                locked,
            } => {
                let target = find_layer(project, scene, layer)?;
                let locked = std::mem::replace(&mut target.locked, locked);
                Ok(Command::SetLayerLocked {
                    scene,
                    layer,
                    locked,
                })
            }
            Command::AddDrawing {
                scene,
                layer,
                drawing,
                index,
            } => {
                let target = find_layer(project, scene, layer)?;
                check_insert(target.drawings.len(), index)?;
                if target.drawings.iter().any(|d| d.id == drawing.id) {
                    return Err(CommandError::DuplicateId);
                }
                let id = drawing.id;
                target.drawings.insert(index, drawing);
                Ok(Command::RemoveDrawing {
                    scene,
                    layer,
                    drawing: id,
                })
            }
            Command::RemoveDrawing {
                scene,
                layer,
                drawing,
            } => {
                let target = find_layer(project, scene, layer)?;
                if target.exposures.iter().any(|e| e.drawing == drawing) {
                    return Err(CommandError::DrawingInUse);
                }
                let index = position(&target.drawings, |d| d.id == drawing)?;
                let drawing = target.drawings.remove(index);
                Ok(Command::AddDrawing {
                    scene,
                    layer,
                    drawing,
                    index,
                })
            }
            Command::InsertExposure {
                scene,
                layer,
                exposure,
            } => {
                find_layer(project, scene, layer)?
                    .insert_exposure(exposure)
                    .map_err(CommandError::Exposure)?;
                Ok(Command::RemoveExposure {
                    scene,
                    layer,
                    start_frame: exposure.start_frame,
                })
            }
            Command::RemoveExposure {
                scene,
                layer,
                start_frame,
            } => {
                let exposures = &mut find_layer(project, scene, layer)?.exposures;
                let index = position(exposures, |e| e.start_frame == start_frame)?;
                let exposure = exposures.remove(index);
                Ok(Command::InsertExposure {
                    scene,
                    layer,
                    exposure,
                })
            }
            Command::AddStroke {
                scene,
                layer,
                drawing,
                stroke,
                index,
            } => {
                let strokes = &mut find_drawing(project, scene, layer, drawing)?.strokes;
                check_insert(strokes.len(), index)?;
                if strokes.iter().any(|s| s.id == stroke.id) {
                    return Err(CommandError::DuplicateId);
                }
                let id = stroke.id;
                strokes.insert(index, stroke);
                Ok(Command::RemoveStroke {
                    scene,
                    layer,
                    drawing,
                    stroke: id,
                })
            }
            Command::RemoveStroke {
                scene,
                layer,
                drawing,
                stroke,
            } => {
                let strokes = &mut find_drawing(project, scene, layer, drawing)?.strokes;
                let index = position(strokes, |s| s.id == stroke)?;
                let stroke = strokes.remove(index);
                Ok(Command::AddStroke {
                    scene,
                    layer,
                    drawing,
                    stroke,
                    index,
                })
            }
        }
    }
}

fn check_insert(len: usize, index: usize) -> Result<(), CommandError> {
    if index > len {
        return Err(CommandError::InvalidIndex);
    }
    Ok(())
}

fn position<T>(items: &[T], matches: impl FnMut(&T) -> bool) -> Result<usize, CommandError> {
    items.iter().position(matches).ok_or(CommandError::NotFound)
}

fn find_scene(project: &mut Project, id: SceneId) -> Result<&mut Scene, CommandError> {
    project
        .scenes
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or(CommandError::NotFound)
}

fn find_layer(
    project: &mut Project,
    scene: SceneId,
    id: LayerId,
) -> Result<&mut Layer, CommandError> {
    find_scene(project, scene)?
        .layers
        .iter_mut()
        .find(|l| l.id == id)
        .ok_or(CommandError::NotFound)
}

fn find_drawing(
    project: &mut Project,
    scene: SceneId,
    layer: LayerId,
    id: DrawingId,
) -> Result<&mut Drawing, CommandError> {
    find_layer(project, scene, layer)?
        .drawings
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or(CommandError::NotFound)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandError {
    NotFound,
    DuplicateId,
    InvalidIndex,
    LastScene,
    DrawingInUse,
    Exposure(ExposureError),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("the item no longer exists"),
            Self::DuplicateId => f.write_str("an item with this ID already exists"),
            Self::InvalidIndex => f.write_str("the position is out of range"),
            Self::LastScene => f.write_str("a project must keep at least one scene"),
            Self::DrawingInUse => f.write_str("the drawing is still used by an exposure"),
            Self::Exposure(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CommandError {}
