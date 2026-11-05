//! Tauri shell. The Rust core owns the document, and the UI talks to it through these commands.

mod recovery;
mod session;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use celframe_core::{Command, Project};
use recovery::{RecoveryInfo, RecoveryStore};
use session::{AutosaveStep, DocumentView, Session};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, Window, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

type SharedSession = Mutex<Session>;

/// Set once the person has chosen to close despite unsaved changes.
#[derive(Default)]
struct CloseConfirmed(AtomicBool);

/// Sent to the UI when closing the window would discard unsaved changes.
const CLOSE_REQUESTED: &str = "close-requested";

const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(30);

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
    lock(&state)?.save_as(&path).map(Some)
}

/// Closes the window, discarding unsaved changes. The UI asks first.
#[tauri::command]
fn close_window(window: Window, confirmed: State<'_, CloseConfirmed>) -> Result<(), String> {
    confirmed.0.store(true, Ordering::SeqCst);
    window.close().map_err(|e| e.to_string())
}

/// The recovery copy left by an earlier run, while the person hasn't chosen what to do with it.
#[tauri::command]
fn get_recovery(
    state: State<'_, SharedSession>,
    store: State<'_, RecoveryStore>,
) -> Result<Option<RecoveryInfo>, String> {
    let pending = lock(&state)?.recovery_pending();
    Ok(if pending { store.info() } else { None })
}

/// On failure the copy is kept and the choice stays open.
#[tauri::command]
fn restore_recovery(
    state: State<'_, SharedSession>,
    store: State<'_, RecoveryStore>,
) -> Result<DocumentView, String> {
    let project = store.load()?;
    let original_path = store.info().and_then(|info| info.original_path);
    Ok(lock(&state)?.restore(project, original_path))
}

#[tauri::command]
fn discard_recovery(
    state: State<'_, SharedSession>,
    store: State<'_, RecoveryStore>,
) -> Result<(), String> {
    store
        .clear()
        .map_err(|error| format!("Couldn't delete the unsaved work: {error}"))?;
    lock(&state)?.set_recovery_pending(false);
    Ok(())
}

/// One autosave check. The project is written outside the lock so editing never waits on disk.
fn autosave_tick(app: &AppHandle) {
    let session = app.state::<SharedSession>();
    let store = app.state::<RecoveryStore>();
    let Ok(step) = session.lock().map(|mut s| s.autosave_step()) else {
        return;
    };
    match step {
        AutosaveStep::Nothing => {}
        AutosaveStep::Write(project, original_path) => {
            let info = RecoveryInfo {
                original_path,
                saved_at_ms: now_ms(),
            };
            if let Err(error) = store.write(&project, &info) {
                eprintln!("autosave failed: {error}");
                if let Ok(mut session) = session.lock() {
                    session.autosave_failed();
                }
            }
        }
        AutosaveStep::Clear => {
            if let Err(error) = store.clear() {
                eprintln!("couldn't remove the recovery copy: {error}");
            }
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let store = RecoveryStore::new(app.path().app_data_dir()?.join("recovery"));
    let mut session = Session::new(Project::new("Untitled"));
    session.set_recovery_pending(store.info().is_some());
    app.manage(Mutex::new(session));
    app.manage(store);

    let handle = app.handle().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(AUTOSAVE_INTERVAL);
            autosave_tick(&handle);
        }
    });
    Ok(())
}

fn on_run_event(app: &AppHandle, event: RunEvent) {
    if !matches!(event, RunEvent::Exit) {
        return;
    }
    // Some exits skip the close prompt, such as quitting from the macOS menu. Unsaved work
    // then keeps its recovery copy unless the person chose to discard it.
    let discarded = app.state::<CloseConfirmed>().0.load(Ordering::SeqCst);
    let clear = app.state::<SharedSession>().lock().is_ok_and(|session| {
        session.should_clear_recovery_on_exit() && (discarded || !session.is_dirty())
    });
    if clear && let Err(error) = app.state::<RecoveryStore>().clear() {
        eprintln!("couldn't remove the recovery copy: {error}");
    }
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
        .manage(CloseConfirmed::default())
        .setup(setup)
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
            close_window,
            get_recovery,
            restore_recovery,
            discard_recovery
        ])
        .build(tauri::generate_context!())
        .expect("the app failed to start")
        .run(on_run_event);
}
