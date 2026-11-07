//! Undo and redo stacks. See ADR-0003.

use crate::command::{Command, CommandError};
use crate::ids::StrokeId;
use crate::model::Project;

/// Tracks which commands reverse the edits made so far.
#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    next_state: u64,
    current_state: u64,
    saved_state: u64,
    /// The stroke whose appended samples merge into its original undo entry.
    open_stroke: Option<StrokeId>,
    /// Edits merge into one undo entry until `end_group`. True once the group has an entry.
    open_group: Option<bool>,
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

    /// Marks the document as differing from every saved file, such as after restoring
    /// autosaved work. Only a later save makes it clean again.
    pub fn mark_unsaved(&mut self) {
        // State numbers count up from 0, so this one is never reached.
        self.saved_state = u64::MAX;
    }

    /// Identifies the current document state. It changes with every edit, undo and redo,
    /// and returns to an earlier value only when undo or redo returns to that state.
    pub fn revision(&self) -> u64 {
        self.current_state
    }

    /// Applies a new edit. Redo is cleared only if the edit succeeds.
    pub fn execute(&mut self, project: &mut Project, command: Command) -> Result<(), CommandError> {
        let merges = matches!(
            &command,
            Command::AppendStrokeSamples { stroke, .. } if self.open_stroke == Some(*stroke)
        );
        let inverse = command.apply(project)?;
        if merges {
            // Undoing the entry that added the stroke already removes these samples.
            self.finish_edit();
            return Ok(());
        }
        self.open_stroke = None;
        if self.open_group == Some(true)
            && let Some(entry) = self.undo.last_mut()
        {
            // Undo reverses the newest edit first, then everything before it in the group.
            let earlier = std::mem::replace(
                &mut entry.command,
                Command::Batch {
                    commands: Vec::new(),
                },
            );
            let mut commands = vec![inverse];
            match earlier {
                Command::Batch { commands: rest } => commands.extend(rest),
                other => commands.push(other),
            }
            entry.command = Command::Batch { commands };
        } else {
            self.undo.push(Entry {
                command: inverse,
                target_state: self.current_state,
            });
            if self.open_group.is_some() {
                self.open_group = Some(true);
            }
        }
        self.finish_edit();
        Ok(())
    }

    /// Merges the edits that follow into one undo step, until `end_group`, undo or redo.
    /// Used for edits made by one drag, such as erasing.
    pub fn begin_group(&mut self) {
        self.open_stroke = None;
        self.open_group = Some(false);
    }

    pub fn end_group(&mut self) {
        self.open_group = None;
    }

    /// Adds a stroke and merges later samples for it, until `end_stroke` or another edit.
    pub fn begin_stroke(
        &mut self,
        project: &mut Project,
        command: Command,
    ) -> Result<(), CommandError> {
        let stroke = match &command {
            Command::AddStroke { stroke, .. } => Some(stroke.id),
            _ => None,
        };
        self.execute(project, command)?;
        self.open_stroke = stroke;
        Ok(())
    }

    pub fn end_stroke(&mut self) {
        self.open_stroke = None;
    }

    fn finish_edit(&mut self) {
        self.redo.clear();
        self.next_state += 1;
        self.current_state = self.next_state;
    }

    /// Returns false when there is nothing to undo.
    pub fn undo(&mut self, project: &mut Project) -> Result<bool, CommandError> {
        self.open_stroke = None;
        self.open_group = None;
        Self::step(
            &mut self.undo,
            &mut self.redo,
            &mut self.current_state,
            project,
        )
    }

    /// Returns false when there is nothing to redo.
    pub fn redo(&mut self, project: &mut Project) -> Result<bool, CommandError> {
        self.open_stroke = None;
        self.open_group = None;
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
