//! Tauri shell. The Rust core owns the document, and the UI talks to it through these commands.

mod session;

use std::sync::{Mutex, MutexGuard};

use celframe_core::{Command, Project};
use session::{DocumentView, Session};
use tauri::State;

type SharedSession = Mutex<Session>;

fn lock<'a>(state: &'a State<'_, SharedSession>) -> Result<MutexGuard<'a, Session>, String> {
    state
        .lock()
        .map_err(|_| "the document is unavailable after an earlier error".to_owned())
}

#[tauri::command]
fn get_document(state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    Ok(lock(&state)?.view())
}

#[tauri::command]
fn execute_command(
    state: State<'_, SharedSession>,
    command: Command,
) -> Result<DocumentView, String> {
    lock(&state)?.execute(command).map_err(|e| e.to_string())
}

#[tauri::command]
fn undo(state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    lock(&state)?.undo().map_err(|e| e.to_string())
}

#[tauri::command]
fn redo(state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    lock(&state)?.redo().map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Session::new(Project::new("Untitled"))))
        .invoke_handler(tauri::generate_handler![
            get_document,
            execute_command,
            undo,
            redo
        ])
        .run(tauri::generate_context!())
        .expect("the app failed to start");
}
