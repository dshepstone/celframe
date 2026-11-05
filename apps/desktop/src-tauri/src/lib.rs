//! Tauri shell. The Rust core owns the document, and the UI talks to it through these commands.

mod session;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use celframe_core::{Command, Project};
use session::{DocumentView, Session};
use tauri::{AppHandle, Emitter, Manager, State, Window, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

type SharedSession = Mutex<Session>;

/// Set once the person has chosen to close despite unsaved changes.
#[derive(Default)]
struct CloseConfirmed(AtomicBool);

/// Sent to the UI when closing the window would discard unsaved changes.
const CLOSE_REQUESTED: &str = "close-requested";

fn lock<'a>(state: &'a State<'_, SharedSession>) -> Result<MutexGuard<'a, Session>, String> {
    state
        .lock()
        .map_err(|_| "the document is unavailable after an earlier error".to_owned())
}

fn celf_dialog(app: &AppHandle) -> FileDialogBuilder<tauri::Wry> {
    app.dialog()
        .file()
        .add_filter("Celframe project", &["celf"])
}

fn into_path(file: FilePath) -> Result<PathBuf, String> {
    file.into_path()
        .map_err(|error| format!("Couldn't use the chosen file: {error}"))
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

/// The UI asks about unsaved changes before calling this.
#[tauri::command]
fn new_document(state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    Ok(lock(&state)?.new_document())
}

// File pickers block, so the commands that show one are async and run off the main thread.
// Each returns `None` if the person cancels the picker.

#[tauri::command]
async fn open_document(
    app: AppHandle,
    state: State<'_, SharedSession>,
) -> Result<Option<DocumentView>, String> {
    let Some(file) = celf_dialog(&app).blocking_pick_file() else {
        return Ok(None);
    };
    let path = into_path(file)?;
    lock(&state)?.open(&path).map(Some)
}

/// Saves to the project's file, or asks for one if it has never been saved.
#[tauri::command]
async fn save_document(
    app: AppHandle,
    state: State<'_, SharedSession>,
) -> Result<Option<DocumentView>, String> {
    let path = lock(&state)?.path().map(ToOwned::to_owned);
    match path {
        Some(path) => lock(&state)?.save_to(&path).map(Some),
        None => save_document_as(app, state).await,
    }
}

#[tauri::command]
async fn save_document_as(
    app: AppHandle,
    state: State<'_, SharedSession>,
) -> Result<Option<DocumentView>, String> {
    let (name, folder) = {
        let session = lock(&state)?;
        let folder = session
            .path()
            .and_then(|p| p.parent())
            .map(ToOwned::to_owned);
        (format!("{}.celf", session.project_name()), folder)
    };
    let mut dialog = celf_dialog(&app).set_file_name(name);
    if let Some(folder) = folder {
        dialog = dialog.set_directory(folder);
    }
    let Some(file) = dialog.blocking_save_file() else {
        return Ok(None);
    };
    let path = into_path(file)?;
    lock(&state)?.save_to(&path).map(Some)
}

/// Closes the window, discarding unsaved changes. The UI asks first.
#[tauri::command]
fn close_window(window: Window, confirmed: State<'_, CloseConfirmed>) -> Result<(), String> {
    confirmed.0.store(true, Ordering::SeqCst);
    window.close().map_err(|e| e.to_string())
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    if window.state::<CloseConfirmed>().0.load(Ordering::SeqCst) {
        return;
    }
    let dirty = window
        .state::<SharedSession>()
        .lock()
        .is_ok_and(|session| session.is_dirty());
    if dirty {
        api.prevent_close();
        let _ = window.emit(CLOSE_REQUESTED, ());
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(Session::new(Project::new("Untitled"))))
        .manage(CloseConfirmed::default())
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            get_document,
            execute_command,
            undo,
            redo,
            new_document,
            open_document,
            save_document,
            save_document_as,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("the app failed to start");
}
