//! Tauri shell. The Rust core owns the document, and the UI talks to it through these commands.

mod canvas;
mod eraser;
mod pen;
mod recovery;
mod select;
mod session;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use canvas::{CanvasLayout, Mapping, WindowCanvas};
use celframe_core::{Command, Project, StrokeSample};
use pen::PenOptions;
use recovery::{RecoveryInfo, RecoveryStore};
use serde::Deserialize;
use session::{AutosaveStep, DocumentView, Session};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, Window, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

type SharedSession = Mutex<Session>;

/// The native canvas, or `None` if the GPU couldn't start. The app still works without it.
#[derive(Default)]
struct CanvasState(Mutex<Option<WindowCanvas>>);

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

/// Draws the open scene into the window. Locks the session, then the canvas: always in that order.
fn redraw(app: &AppHandle) {
    let session = app.state::<SharedSession>();
    let Ok(session) = session.lock() else {
        return;
    };
    let Some(scene) = session.project().scenes.first() else {
        return;
    };
    if let Ok(mut canvas) = app.state::<CanvasState>().0.lock()
        && let Some(canvas) = canvas.as_mut()
    {
        let (Some(zoom), Some(layout)) = (canvas.zoom(scene), canvas.layout()) else {
            return;
        };
        let pixel = 1.0 / zoom;
        let mode = canvas.mode();
        // The frame's dimming and gate go under the selection's outlines and handles.
        let surround = canvas::surround(mode, &layout);
        let mut overlay = canvas::frame_overlay(scene, mode, pixel, surround);
        overlay.extend(session.overlay(pixel));
        let frame = session.current_frame();
        let layer = canvas.drawing_layer();
        let strokes = canvas::view_strokes(scene, mode, layer, frame);
        // Onion skin draws under the current drawing.
        let onion = canvas::onion_strokes(scene, layer, frame, &canvas.onion());
        canvas.render(
            scene,
            onion.iter().chain(strokes.iter().map(|s| &**s)),
            &overlay,
        );
    }
}

/// Redraws after a command that may have changed the document.
fn redrawn<T>(app: &AppHandle, result: Result<T, String>) -> Result<T, String> {
    if result.is_ok() {
        redraw(app);
    }
    result
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
    app: AppHandle,
    state: State<'_, SharedSession>,
    command: Command,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.execute(command).map_err(|e| e.to_string());
    redrawn(&app, result)
}

#[tauri::command]
fn add_layer(app: AppHandle, state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    let result = lock(&state)?.add_layer().map_err(|e| e.to_string());
    redrawn(&app, result)
}

/// A part of the current layer's drawing, as the UI chose it.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartChoice {
    layer: Option<celframe_core::LayerId>,
    art: celframe_core::Art,
}

#[tauri::command]
fn select_all(
    app: AppHandle,
    state: State<'_, SharedSession>,
    part: PartChoice,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.select_all(part.layer, part.art);
    redrawn(&app, result)
}

#[tauri::command]
fn clear_selection(
    app: AppHandle,
    state: State<'_, SharedSession>,
) -> Result<DocumentView, String> {
    let view = lock(&state)?.clear_selection();
    redrawn(&app, Ok(view))
}

#[tauri::command]
fn delete_selection(
    app: AppHandle,
    state: State<'_, SharedSession>,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.delete_selection();
    redrawn(&app, result)
}

#[tauri::command]
fn set_onion_skin(
    app: AppHandle,
    canvas: State<'_, CanvasState>,
    onion: canvas::OnionSkin,
) -> Result<(), String> {
    {
        let mut canvas = canvas.0.lock().map_err(|_| "the canvas is unavailable")?;
        if let Some(canvas) = canvas.as_mut() {
            canvas.set_onion(onion);
        }
    }
    redraw(&app);
    Ok(())
}

#[tauri::command]
fn set_current_frame(
    app: AppHandle,
    state: State<'_, SharedSession>,
    frame: u32,
) -> Result<DocumentView, String> {
    let view = lock(&state)?.set_current_frame(frame);
    redrawn(&app, Ok(view))
}

#[tauri::command]
fn edit_timing(
    app: AppHandle,
    state: State<'_, SharedSession>,
    edit: session::TimingEdit,
    layer: Option<celframe_core::LayerId>,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.edit_timing(edit, layer);
    redrawn(&app, result)
}

#[tauri::command]
fn set_handles_visible(
    app: AppHandle,
    state: State<'_, SharedSession>,
    visible: bool,
) -> Result<(), String> {
    lock(&state)?.set_handles_visible(visible);
    redraw(&app);
    Ok(())
}

#[tauri::command]
fn edit_selection(
    app: AppHandle,
    state: State<'_, SharedSession>,
    edit: select::SelectionEdit,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.edit_selection(edit);
    redrawn(&app, result)
}

#[tauri::command]
fn nudge_selection(
    app: AppHandle,
    state: State<'_, SharedSession>,
    dx: f64,
    dy: f64,
) -> Result<DocumentView, String> {
    let result = lock(&state)?.nudge_selection([dx, dy]);
    redrawn(&app, result)
}

#[tauri::command]
fn undo(app: AppHandle, state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    let result = lock(&state)?.undo().map_err(|e| e.to_string());
    redrawn(&app, result)
}

#[tauri::command]
fn redo(app: AppHandle, state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    let result = lock(&state)?.redo().map_err(|e| e.to_string());
    redrawn(&app, result)
}

/// The UI asks about unsaved changes before calling this.
#[tauri::command]
fn new_document(app: AppHandle, state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    let view = lock(&state)?.new_document();
    redrawn(&app, Ok(view))
}

/// A pointer sample in CSS pixels from the window's top left.
#[derive(Clone, Copy, Debug, Deserialize)]
struct PointerSample {
    x: f64,
    y: f64,
    pressure: f32,
    /// Shift was held. Shapes use it to constrain their proportions or angle.
    #[serde(default)]
    shift: bool,
    /// Alt was held. Shapes use it to draw from their centre.
    #[serde(default)]
    alt: bool,
}

impl PointerSample {
    fn constraints(&self) -> celframe_geom::Constraints {
        celframe_geom::Constraints {
            constrain: self.shift,
            from_centre: self.alt,
        }
    }
}

/// The current pointer-to-scene mapping. Call with the session locked, keeping the lock order.
fn mapping(session: &Session, canvas: &CanvasState) -> Result<Mapping, String> {
    let scene = session
        .project()
        .scenes
        .first()
        .ok_or("The project has no scene to draw in.")?;
    let canvas = canvas.0.lock().map_err(|_| "the canvas is unavailable")?;
    let canvas = canvas
        .as_ref()
        .ok_or("The canvas isn't ready to draw on.")?;
    let layout = canvas
        .layout()
        .ok_or("The canvas isn't ready to draw on.")?;
    Ok(Mapping::new(scene, &layout, canvas.view()))
}

fn to_scene(mapping: &Mapping, sample: PointerSample) -> StrokeSample {
    let [x, y] = mapping.scene_point(sample.x, sample.y);
    StrokeSample {
        x,
        y,
        pressure: sample.pressure,
    }
}

// The pen commands are synchronous, so they run in the order the UI sends them.

#[tauri::command]
fn pen_down(
    app: AppHandle,
    state: State<'_, SharedSession>,
    canvas: State<'_, CanvasState>,
    sample: PointerSample,
    options: Option<PenOptions>,
) -> Result<(), String> {
    {
        let mut session = lock(&state)?;
        let mapping = mapping(&session, &canvas)?;
        session.set_modifiers(sample.constraints());
        session.pen_down(
            to_scene(&mapping, sample),
            mapping.min_distance(),
            options.unwrap_or_default(),
        )?;
    }
    redraw(&app);
    Ok(())
}

#[tauri::command]
fn pen_move(
    app: AppHandle,
    state: State<'_, SharedSession>,
    canvas: State<'_, CanvasState>,
    samples: Vec<PointerSample>,
) -> Result<(), String> {
    {
        let mut session = lock(&state)?;
        let mapping = mapping(&session, &canvas)?;
        if let Some(last) = samples.last() {
            session.set_modifiers(last.constraints());
        }
        let samples: Vec<StrokeSample> =
            samples.into_iter().map(|s| to_scene(&mapping, s)).collect();
        session.pen_move(&samples)?;
    }
    redraw(&app);
    Ok(())
}

#[tauri::command]
fn pen_up(app: AppHandle, state: State<'_, SharedSession>) -> Result<DocumentView, String> {
    let result = lock(&state)?.pen_up();
    redrawn(&app, result)
}

/// Tells the canvas where to draw. Returns false when there is no native canvas, so the UI
/// keeps its own opaque backdrop instead of showing an undrawn window through it.
#[tauri::command]
fn set_canvas_layout(
    app: AppHandle,
    canvas: State<'_, CanvasState>,
    layout: CanvasLayout,
) -> Result<bool, String> {
    {
        let mut canvas = canvas.0.lock().map_err(|_| "the canvas is unavailable")?;
        let Some(canvas) = canvas.as_mut() else {
            return Ok(false);
        };
        canvas.set_layout(layout);
    }
    redraw(&app);
    Ok(true)
}

/// What the UI shows about the view: zoom as a percentage of one CSS pixel per scene unit,
/// and rotation in degrees.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ViewInfo {
    zoom_percent: f64,
    rotation_degrees: f64,
    fitted: bool,
}

/// A point in CSS pixels from the window's top left.
#[derive(Clone, Copy, Debug, Deserialize)]
struct ScreenPoint {
    x: f64,
    y: f64,
}

/// How the view changes. Anchors default to the middle of the canvas.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum ViewChange {
    Zoom {
        factor: f64,
        anchor: Option<ScreenPoint>,
    },
    Pan {
        dx: f64,
        dy: f64,
    },
    Rotate {
        degrees: f64,
        anchor: Option<ScreenPoint>,
    },
    Fit,
}

/// Changes the canvas view. The view is display state, so it isn't undoable or saved.
#[tauri::command]
fn change_view(
    app: AppHandle,
    state: State<'_, SharedSession>,
    canvas: State<'_, CanvasState>,
    change: ViewChange,
) -> Result<ViewInfo, String> {
    let info = {
        // Session first, then canvas: the same order as every other command.
        let session = lock(&state)?;
        let scene = session
            .project()
            .scenes
            .first()
            .ok_or("The project has no scene.")?;
        let mut canvas = canvas.0.lock().map_err(|_| "the canvas is unavailable")?;
        let canvas = canvas.as_mut().ok_or("The canvas isn't available.")?;
        let layout = canvas.layout().ok_or("The canvas isn't ready yet.")?;
        let size = canvas::viewport_size(&layout);
        let current = canvas
            .view()
            .unwrap_or_else(|| canvas::fitted_view(scene, &layout));
        // CSS pixels from the window's top left, to device pixels in the canvas area.
        let viewport = layout.viewport();
        let to_canvas = |p: Option<ScreenPoint>| match p {
            Some(p) => [
                p.x * layout.scale - f64::from(viewport[0]),
                p.y * layout.scale - f64::from(viewport[1]),
            ],
            None => [size[0] / 2.0, size[1] / 2.0],
        };
        let next = match change {
            ViewChange::Zoom { factor, anchor } => {
                Some(current.zoomed_about(to_canvas(anchor), size, factor))
            }
            ViewChange::Pan { dx, dy } => {
                Some(current.panned([dx * layout.scale, dy * layout.scale], size))
            }
            ViewChange::Rotate { degrees, anchor } => {
                Some(current.rotated_about(to_canvas(anchor), size, degrees.to_radians()))
            }
            ViewChange::Fit => None,
        };
        canvas.set_view(next);
        view_info(canvas, scene, &layout)
    };
    redraw(&app);
    Ok(info)
}

/// The current view's zoom and rotation, as the UI shows them.
fn view_info(
    canvas: &canvas::WindowCanvas,
    scene: &celframe_core::Scene,
    layout: &canvas::CanvasLayout,
) -> ViewInfo {
    let view = canvas.view();
    let shown = view.unwrap_or_else(|| canvas::fitted_view(scene, layout));
    ViewInfo {
        zoom_percent: shown.zoom / layout.scale * 100.0,
        rotation_degrees: shown.rotation.to_degrees(),
        fitted: view.is_none(),
    }
}

/// Shows the Drawing or Camera view. Returns that view's zoom and rotation, or None while the
/// canvas isn't ready.
#[tauri::command]
fn set_view_mode(
    app: AppHandle,
    state: State<'_, SharedSession>,
    canvas: State<'_, CanvasState>,
    mode: canvas::ViewMode,
    layer: Option<celframe_core::LayerId>,
) -> Result<Option<ViewInfo>, String> {
    let info = {
        let session = lock(&state)?;
        let mut canvas = canvas.0.lock().map_err(|_| "the canvas is unavailable")?;
        let Some(canvas) = canvas.as_mut() else {
            return Ok(None);
        };
        canvas.set_mode(mode, layer);
        let scene = session.project().scenes.first();
        match (scene, canvas.layout()) {
            (Some(scene), Some(layout)) => Some(view_info(canvas, scene, &layout)),
            _ => None,
        }
    };
    redraw(&app);
    Ok(info)
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
    let result = lock(&state)?.open(&path).map(Some);
    redrawn(&app, result)
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
    app: AppHandle,
    state: State<'_, SharedSession>,
    store: State<'_, RecoveryStore>,
) -> Result<DocumentView, String> {
    let project = store.load()?;
    let original_path = store.info().and_then(|info| info.original_path);
    let view = lock(&state)?.restore(project, original_path);
    redrawn(&app, Ok(view))
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
    let mut session = Session::new(Project::starter("Untitled"));
    session.set_recovery_pending(store.info().is_some());
    app.manage(Mutex::new(session));
    app.manage(store);

    // Without a GPU the app still opens: the UI keeps an opaque backdrop over the canvas area.
    let canvas = app
        .get_webview_window("main")
        .ok_or_else(|| "the main window is missing".to_owned())
        .and_then(|window| WindowCanvas::new(&window));
    match &canvas {
        Ok(canvas) => eprintln!("canvas: drawing with {}", canvas.adapter_name()),
        Err(error) => eprintln!("canvas: unavailable: {error}"),
    }
    app.manage(CanvasState(Mutex::new(canvas.ok())));

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
    if let WindowEvent::Resized(size) = event {
        if let Ok(mut canvas) = window.state::<CanvasState>().0.lock()
            && let Some(canvas) = canvas.as_mut()
        {
            canvas.resize(size.width, size.height);
        }
        redraw(window.app_handle());
        return;
    }
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
            add_layer,
            undo,
            redo,
            new_document,
            open_document,
            save_document,
            save_document_as,
            close_window,
            get_recovery,
            restore_recovery,
            discard_recovery,
            set_canvas_layout,
            change_view,
            pen_down,
            pen_move,
            pen_up,
            select_all,
            clear_selection,
            delete_selection,
            nudge_selection,
            edit_selection,
            set_handles_visible,
            set_view_mode,
            set_current_frame,
            edit_timing,
            set_onion_skin
        ])
        .build(tauri::generate_context!())
        .expect("the app failed to start")
        .run(on_run_event);
}
