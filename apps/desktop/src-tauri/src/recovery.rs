//! The autosaved recovery copy, kept in the app's data folder. See
//! `docs/specs/autosave-and-recovery.md`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use celframe_core::Project;
use serde::{Deserialize, Serialize};

const PROJECT_FILE: &str = "recovery.celf";
const INFO_FILE: &str = "recovery.json";

/// What the UI shows when offering to restore.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryInfo {
    /// The file the project was last saved to, if any.
    pub original_path: Option<PathBuf>,
    /// When the copy was written, in milliseconds since the Unix epoch.
    pub saved_at_ms: u64,
}

pub struct RecoveryStore {
    dir: PathBuf,
}

impl RecoveryStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Writes the project first and the info last, so the info only exists beside a complete
    /// project. Both writes are atomic.
    pub fn write(&self, project: &Project, info: &RecoveryInfo) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        celframe_format::save_project(project, &self.dir.join(PROJECT_FILE))
            .map_err(|e| e.to_string())?;
        let json = serde_json::to_vec_pretty(info).map_err(|e| e.to_string())?;
        let temp = self.dir.join(format!("{INFO_FILE}.tmp"));
        fs::write(&temp, json).map_err(|e| e.to_string())?;
        fs::rename(&temp, self.dir.join(INFO_FILE)).map_err(|e| e.to_string())
    }

    /// Returns the info of a waiting recovery copy. Unreadable info counts as no copy.
    pub fn info(&self) -> Option<RecoveryInfo> {
        let json = fs::read(self.dir.join(INFO_FILE)).ok()?;
        serde_json::from_slice(&json).ok()
    }

    pub fn load(&self) -> Result<Project, String> {
        celframe_format::load_project(&self.dir.join(PROJECT_FILE))
            .map_err(|error| format!("Couldn't restore your unsaved work: {error}"))
    }

    /// Removes the info first, so a partial clear never leaves a copy that gets offered.
    pub fn clear(&self) -> io::Result<()> {
        remove_if_present(&self.dir.join(INFO_FILE))?;
        remove_if_present(&self.dir.join(PROJECT_FILE))
    }
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(original: Option<&str>) -> RecoveryInfo {
        RecoveryInfo {
            original_path: original.map(PathBuf::from),
            saved_at_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn writes_reads_and_clears_a_copy() {
        let dir = tempfile::tempdir().unwrap();
        let store = RecoveryStore::new(dir.path().join("recovery"));
        assert_eq!(store.info(), None);

        let project = Project::new("Ball");
        store.write(&project, &info(Some("ball.celf"))).unwrap();
        assert_eq!(store.info(), Some(info(Some("ball.celf"))));
        assert_eq!(store.load().unwrap(), project);

        // Each write replaces the last.
        store.write(&Project::new("Walk"), &info(None)).unwrap();
        assert_eq!(store.load().unwrap().name, "Walk");
        assert_eq!(store.info(), Some(info(None)));

        store.clear().unwrap();
        assert_eq!(store.info(), None);
        store.clear().unwrap();
    }

    #[test]
    fn a_damaged_copy_is_offered_but_fails_to_load_with_a_reason() {
        let dir = tempfile::tempdir().unwrap();
        let store = RecoveryStore::new(dir.path().to_owned());
        store.write(&Project::new("Ball"), &info(None)).unwrap();
        fs::write(dir.path().join(PROJECT_FILE), b"damaged").unwrap();

        assert!(store.info().is_some());
        let error = store.load().unwrap_err();
        assert!(
            error.starts_with("Couldn't restore your unsaved work"),
            "{error}"
        );
    }

    #[test]
    fn unreadable_info_counts_as_no_copy() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(INFO_FILE), b"{").unwrap();
        assert_eq!(RecoveryStore::new(dir.path().to_owned()).info(), None);
    }
}
