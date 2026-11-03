//! The open document and its history. Tauri handlers only wrap these methods.

use celframe_core::{Command, CommandError, History, Project};
use serde::Serialize;

pub struct Session {
    project: Project,
    history: History,
}

/// What the UI renders. It is always replaced as a whole, never edited in the UI.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub project: Project,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
}

impl Session {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            history: History::new(),
        }
    }

    pub fn view(&self) -> DocumentView {
        DocumentView {
            project: self.project.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            dirty: self.history.is_dirty(),
        }
    }

    pub fn execute(&mut self, command: Command) -> Result<DocumentView, CommandError> {
        self.history.execute(&mut self.project, command)?;
        Ok(self.view())
    }

    pub fn undo(&mut self) -> Result<DocumentView, CommandError> {
        self.history.undo(&mut self.project)?;
        Ok(self.view())
    }

    pub fn redo(&mut self) -> Result<DocumentView, CommandError> {
        self.history.redo(&mut self.project)?;
        Ok(self.view())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename(name: &str) -> Command {
        Command::RenameProject {
            name: name.to_owned(),
        }
    }

    #[test]
    fn views_track_edits_undo_and_redo() {
        let mut session = Session::new(Project::new("A"));
        let view = session.view();
        assert!(!view.can_undo && !view.can_redo && !view.dirty);

        let view = session.execute(rename("B")).unwrap();
        assert_eq!(view.project.name, "B");
        assert!(view.can_undo && view.dirty);

        let view = session.undo().unwrap();
        assert_eq!(view.project.name, "A");
        assert!(view.can_redo && !view.dirty);

        let view = session.redo().unwrap();
        assert_eq!(view.project.name, "B");
    }

    #[test]
    fn views_serialize_for_the_ui_in_camel_case() {
        let json = serde_json::to_value(Session::new(Project::new("A")).view()).unwrap();
        assert_eq!(json["canUndo"], false);
        assert_eq!(json["project"]["name"], "A");
    }
}
