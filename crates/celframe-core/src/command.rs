//! Edits to a document. Each command returns the command that reverses it.

use serde::{Deserialize, Serialize};

use crate::model::Project;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    RenameProject { name: String },
}

impl Command {
    /// Applies the command and returns its inverse. On error, the project is unchanged.
    pub fn apply(self, project: &mut Project) -> Result<Command, CommandError> {
        match self {
            Command::RenameProject { name } => {
                let name = std::mem::replace(&mut project.name, name);
                Ok(Command::RenameProject { name })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandError {}

impl std::fmt::Display for CommandError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for CommandError {}
