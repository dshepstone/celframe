use celframe_core::{Command, History, Project};

fn rename(name: &str) -> Command {
    Command::RenameProject {
        name: name.to_owned(),
    }
}

#[test]
fn undo_and_redo_walk_through_edits() {
    let mut project = Project::new("A");
    let mut history = History::new();
    history.execute(&mut project, rename("B")).unwrap();
    history.execute(&mut project, rename("C")).unwrap();

    assert!(history.undo(&mut project).unwrap());
    assert_eq!(project.name, "B");
    assert!(history.undo(&mut project).unwrap());
    assert_eq!(project.name, "A");
    assert!(history.redo(&mut project).unwrap());
    assert!(history.redo(&mut project).unwrap());
    assert_eq!(project.name, "C");
}

#[test]
fn undo_and_redo_with_nothing_to_do_report_false() {
    let mut project = Project::new("A");
    let mut history = History::new();
    assert!(!history.can_undo() && !history.can_redo());
    assert!(!history.undo(&mut project).unwrap());
    assert!(!history.redo(&mut project).unwrap());
}

#[test]
fn a_new_edit_clears_redo() {
    let mut project = Project::new("A");
    let mut history = History::new();
    history.execute(&mut project, rename("B")).unwrap();
    history.undo(&mut project).unwrap();
    history.execute(&mut project, rename("C")).unwrap();
    assert!(!history.can_redo());
    assert!(!history.redo(&mut project).unwrap());
    assert_eq!(project.name, "C");
}

#[test]
fn dirty_state_follows_the_saved_point_through_undo_and_redo() {
    let mut project = Project::new("A");
    let mut history = History::new();
    assert!(!history.is_dirty());

    history.execute(&mut project, rename("B")).unwrap();
    assert!(history.is_dirty());
    history.mark_saved();
    assert!(!history.is_dirty());

    history.undo(&mut project).unwrap();
    assert!(history.is_dirty());
    history.redo(&mut project).unwrap();
    assert!(!history.is_dirty());
}

#[test]
fn a_saved_point_lost_to_a_new_branch_stays_dirty() {
    let mut project = Project::new("A");
    let mut history = History::new();
    history.execute(&mut project, rename("B")).unwrap();
    history.mark_saved();
    history.undo(&mut project).unwrap();
    history.execute(&mut project, rename("C")).unwrap();
    history.undo(&mut project).unwrap();
    history.execute(&mut project, rename("B")).unwrap();
    assert!(history.is_dirty());
}

#[test]
fn commands_cross_ipc_as_tagged_json() {
    let json = serde_json::to_string(&rename("B")).unwrap();
    assert_eq!(json, r#"{"type":"renameProject","name":"B"}"#);
    assert_eq!(serde_json::from_str::<Command>(&json).unwrap(), rename("B"));
}
