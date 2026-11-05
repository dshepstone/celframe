//! The open document, its history and its file. Tauri handlers only wrap these methods.

use std::path::{Path, PathBuf};

use celframe_core::{Command, CommandError, History, Project};
use serde::Serialize;

pub struct Session {
    project: Project,
    history: History,
    path: Option<PathBuf>,
    autosave: AutosaveState,
    /// A recovery copy from an earlier run is waiting for the person to restore or discard it.
    recovery_pending: bool,
}

/// What the recovery copy on disk holds, as far as this session knows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum AutosaveState {
    /// No copy written by this session.
    None,
    /// A copy of this document at this history revision.
    Written(u64),
    /// A copy that is out of date or belongs to a replaced document.
    Stale,
}

/// What the autosave timer should do next.
#[derive(Debug, PartialEq)]
pub enum AutosaveStep {
    Nothing,
    /// Write this project as the recovery copy. The path is its own file, if any.
    Write(Project, Option<PathBuf>),
    /// Remove the recovery copy.
    Clear,
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
            autosave: AutosaveState::None,
            recovery_pending: false,
        }
    }

    /// Swaps in another project with a fresh history, keeping track of the recovery copy.
    fn replace(&mut self, project: Project, path: Option<PathBuf>) {
        self.project = project;
        self.history = History::new();
        self.path = path;
        if self.autosave != AutosaveState::None {
            self.autosave = AutosaveState::Stale;
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

    pub fn project(&self) -> &Project {
        &self.project
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
        self.replace(Project::new("Untitled"), None);
        self.view()
    }

    /// Opens a file. On failure the current project stays open, unchanged.
    pub fn open(&mut self, path: &Path) -> Result<DocumentView, String> {
        let project = celframe_format::load_project(path)
            .map_err(|error| format!("Couldn't open {}: {error}", display_name(path)))?;
        self.replace(project, Some(path.to_owned()));
        Ok(self.view())
    }

    /// Saves to exactly `path`, which becomes the project's file. History is kept.
    pub fn save_to(&mut self, path: &Path) -> Result<DocumentView, String> {
        celframe_format::save_project(&self.project, path)
            .map_err(|error| format!("Couldn't save {}: {error}", display_name(path)))?;
        self.history.mark_saved();
        self.path = Some(path.to_owned());
        Ok(self.view())
    }

    /// Saves to a path chosen in Save As, adding `.celf` if it has no extension.
    pub fn save_as(&mut self, chosen: &Path) -> Result<DocumentView, String> {
        let path = with_celf_extension(chosen);
        // The picker only confirmed replacing `chosen`, not a file we named for it.
        if path != chosen && path.exists() {
            return Err(format!(
                "Couldn't save {}: a file with that name already exists. Choose it in Save As to replace it.",
                display_name(&path)
            ));
        }
        self.save_to(&path)
    }
}

impl Session {
    pub fn set_recovery_pending(&mut self, pending: bool) {
        self.recovery_pending = pending;
    }

    pub fn recovery_pending(&self) -> bool {
        self.recovery_pending
    }

    /// Opens restored work as unsaved changes to its original file, if it had one.
    pub fn restore(&mut self, project: Project, original_path: Option<PathBuf>) -> DocumentView {
        self.replace(project, original_path);
        self.history.mark_unsaved();
        // The copy on disk is the one being restored, so rewrite it at the next step.
        self.autosave = AutosaveState::Stale;
        self.recovery_pending = false;
        self.view()
    }

    /// Decides what autosave does now. Call `autosave_failed` if a write fails.
    pub fn autosave_step(&mut self) -> AutosaveStep {
        if self.recovery_pending {
            return AutosaveStep::Nothing;
        }
        let revision = self.history.revision();
        match (self.history.is_dirty(), self.autosave) {
            (true, AutosaveState::Written(written)) if written == revision => AutosaveStep::Nothing,
            (true, _) => {
                self.autosave = AutosaveState::Written(revision);
                AutosaveStep::Write(self.project.clone(), self.path.clone())
            }
            (false, AutosaveState::None) => AutosaveStep::Nothing,
            (false, _) => {
                self.autosave = AutosaveState::None;
                AutosaveStep::Clear
            }
        }
    }

    pub fn autosave_failed(&mut self) {
        self.autosave = AutosaveState::Stale;
    }

    /// True when exiting should remove the recovery copy.
    pub fn should_clear_recovery_on_exit(&self) -> bool {
        !self.recovery_pending
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

        let view = session.save_as(&dir.path().join("plain")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("plain.celf")));
        assert!(dir.path().join("plain.celf").is_file());

        let view = session.save_as(&dir.path().join("kept.backup")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("kept.backup")));
    }

    #[test]
    fn save_as_never_replaces_a_file_the_picker_did_not_confirm() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("plain.celf");
        std::fs::write(&existing, b"someone else's file").unwrap();
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();

        let error = session.save_as(&dir.path().join("plain")).unwrap_err();
        assert!(error.contains("plain.celf"), "{error}");
        assert_eq!(std::fs::read(&existing).unwrap(), b"someone else's file");
        assert!(session.view().dirty);

        // Choosing the existing file itself is a confirmed replace.
        assert!(!session.save_as(&existing).unwrap().dirty);
    }

    #[test]
    fn save_keeps_the_exact_path_of_a_file_with_no_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-extension");
        let mut session = Session::new(Project::new("A"));
        session.save_to(&path).unwrap();
        session.open(&path).unwrap();
        session.execute(rename("B")).unwrap();

        let view = session.save_to(&path).unwrap();
        assert_eq!(view.path, Some(path.clone()));
        assert!(!dir.path().join("no-extension.celf").exists());
        let mut other = Session::new(Project::new("Other"));
        assert_eq!(other.open(&path).unwrap().project.name, "B");
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

    fn write_step(session: &mut Session) -> Project {
        match session.autosave_step() {
            AutosaveStep::Write(project, _) => project,
            other => panic!("expected a write, got {other:?}"),
        }
    }

    #[test]
    fn autosave_writes_each_new_revision_once_and_clears_when_clean() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        session.execute(rename("B")).unwrap();
        assert_eq!(write_step(&mut session).name, "B");
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        session.execute(rename("C")).unwrap();
        assert_eq!(write_step(&mut session).name, "C");

        session.save_to(&dir.path().join("a.celf")).unwrap();
        assert_eq!(session.autosave_step(), AutosaveStep::Clear);
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        // Undoing past the save is unsaved work again.
        session.undo().unwrap();
        assert_eq!(write_step(&mut session).name, "B");
    }

    #[test]
    fn autosave_retries_after_a_failed_write() {
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();
        write_step(&mut session);
        session.autosave_failed();
        assert_eq!(write_step(&mut session).name, "B");
    }

    #[test]
    fn replacing_the_document_clears_its_copy_even_at_a_matching_revision() {
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();
        write_step(&mut session);

        session.new_document();
        assert_eq!(session.autosave_step(), AutosaveStep::Clear);

        // A new document's revisions start again, so they must not match the old copy.
        session.execute(rename("Fresh")).unwrap();
        assert_eq!(write_step(&mut session).name, "Fresh");
    }

    #[test]
    fn a_pending_recovery_blocks_autosave_until_restored() {
        let mut session = Session::new(Project::new("Untitled"));
        session.set_recovery_pending(true);
        session.execute(rename("B")).unwrap();
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);
        assert!(!session.should_clear_recovery_on_exit());

        let view = session.restore(Project::new("Recovered"), Some("ball.celf".into()));
        assert!(view.dirty && !view.can_undo);
        assert_eq!(view.path, Some(PathBuf::from("ball.celf")));
        assert!(session.should_clear_recovery_on_exit());
        match session.autosave_step() {
            AutosaveStep::Write(project, path) => {
                assert_eq!(project.name, "Recovered");
                assert_eq!(path, Some(PathBuf::from("ball.celf")));
            }
            other => panic!("expected a write, got {other:?}"),
        }
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
