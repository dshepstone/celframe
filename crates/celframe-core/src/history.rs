//! Undo and redo stacks. See ADR-0003.

use crate::command::{Command, CommandError};
use crate::model::Project;

/// Tracks which commands reverse the edits made so far.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    next_state: u64,
    current_state: u64,
    saved_state: u64,
}

/// A reversing command, and the document state that exists before it runs.
#[derive(Debug)]
struct Entry {
    command: Command,
    target_state: u64,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// True when the document differs from the last saved state.
    pub fn is_dirty(&self) -> bool {
        self.current_state != self.saved_state
    }

    pub fn mark_saved(&mut self) {
        self.saved_state = self.current_state;
    }

    /// Applies a new edit. Redo is cleared only if the edit succeeds.
    pub fn execute(&mut self, project: &mut Project, command: Command) -> Result<(), CommandError> {
        let inverse = command.apply(project)?;
        self.undo.push(Entry {
            command: inverse,
            target_state: self.current_state,
        });
        self.redo.clear();
        self.next_state += 1;
        self.current_state = self.next_state;
        Ok(())
    }

    /// Returns false when there is nothing to undo.
    pub fn undo(&mut self, project: &mut Project) -> Result<bool, CommandError> {
        Self::step(
            &mut self.undo,
            &mut self.redo,
            &mut self.current_state,
            project,
        )
    }

    /// Returns false when there is nothing to redo.
    pub fn redo(&mut self, project: &mut Project) -> Result<bool, CommandError> {
        Self::step(
            &mut self.redo,
            &mut self.undo,
            &mut self.current_state,
            project,
        )
    }

    fn step(
        from: &mut Vec<Entry>,
        to: &mut Vec<Entry>,
        current_state: &mut u64,
        project: &mut Project,
    ) -> Result<bool, CommandError> {
        let Some(entry) = from.pop() else {
            return Ok(false);
        };
        let target_state = entry.target_state;
        match entry.command.clone().apply(project) {
            Ok(inverse) => {
                to.push(Entry {
                    command: inverse,
                    target_state: *current_state,
                });
                *current_state = target_state;
                Ok(true)
            }
            Err(error) => {
                from.push(entry);
                Err(error)
            }
        }
    }
}
