//! The open document, its history and its file. Tauri handlers only wrap these methods.

use std::path::{Path, PathBuf};

use celframe_core::{Command, CommandError, History, Project};
use serde::Serialize;

pub struct Session {
    project: Project,
    history: History,
    path: Option<PathBuf>,
}

/// What the UI renders. It is always replaced as a whole, never edited in the UI.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub project: Project,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    /// The file the project was opened from or last saved to.
    pub path: Option<PathBuf>,
}

impl Session {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            history: History::new(),
            path: None,
        }
    }

    pub fn view(&self) -> DocumentView {
        DocumentView {
            project: self.project.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            dirty: self.history.is_dirty(),
            path: self.path.clone(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
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

    /// Replaces the project with an empty, unsaved one.
    pub fn new_document(&mut self) -> DocumentView {
        *self = Self::new(Project::new("Untitled"));
        self.view()
    }

    /// Opens a file. On failure the current project stays open, unchanged.
    pub fn open(&mut self, path: &Path) -> Result<DocumentView, String> {
        let project = celframe_format::load_project(path)
            .map_err(|error| format!("Couldn't open {}: {error}", display_name(path)))?;
        *self = Self::new(project);
        self.path = Some(path.to_owned());
        Ok(self.view())
    }

    /// Saves to `path`, which becomes the project's file. History is kept.
    pub fn save_to(&mut self, path: &Path) -> Result<DocumentView, String> {
        let path = with_celf_extension(path);
        celframe_format::save_project(&self.project, &path)
            .map_err(|error| format!("Couldn't save {}: {error}", display_name(&path)))?;
        self.history.mark_saved();
        self.path = Some(path);
        Ok(self.view())
    }
}

/// Adds `.celf` to a path with no extension.
fn with_celf_extension(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        path.to_owned()
    } else {
        path.with_extension("celf")
    }
}

fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
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
        assert!(json["path"].is_null());
    }

    #[test]
    fn new_document_discards_the_project_history_and_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));
        session.save_to(&dir.path().join("a.celf")).unwrap();
        session.execute(rename("B")).unwrap();

        let view = session.new_document();
        assert_eq!(view.project.name, "Untitled");
        assert!(!view.can_undo && !view.dirty && view.path.is_none());
    }

    #[test]
    fn save_marks_clean_keeps_history_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ball.celf");
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("Ball")).unwrap();

        let view = session.save_to(&path).unwrap();
        assert!(!view.dirty && view.can_undo);
        assert_eq!(view.path.as_deref(), Some(path.as_path()));

        // Undoing past the save makes the project dirty again.
        assert!(session.undo().unwrap().dirty);

        let mut other = Session::new(Project::new("Other"));
        let opened = other.open(&path).unwrap();
        assert_eq!(opened.project, view.project);
        assert!(!opened.dirty && !opened.can_undo);
        assert_eq!(opened.path.as_deref(), Some(path.as_path()));
    }

    #[test]
    fn save_as_adds_the_extension_only_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));

        let view = session.save_to(&dir.path().join("plain")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("plain.celf")));
        assert!(dir.path().join("plain.celf").is_file());

        let view = session.save_to(&dir.path().join("kept.backup")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("kept.backup")));
    }

    #[test]
    fn failed_open_leaves_the_project_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("broken.celf");
        std::fs::write(&bad, b"not a zip").unwrap();
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("Edited")).unwrap();
        let before = session.view();

        let error = session.open(&bad).unwrap_err();
        assert!(error.starts_with("Couldn't open broken.celf: "), "{error}");
        assert_eq!(session.view(), before);
    }

    #[test]
    fn failed_save_keeps_the_project_dirty_and_unsaved() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("no-such-folder").join("a.celf");
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();

        let error = session.save_to(&missing).unwrap_err();
        assert!(error.starts_with("Couldn't save a.celf: "), "{error}");
        let view = session.view();
        assert!(view.dirty && view.path.is_none());
    }
}
