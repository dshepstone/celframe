//! Rendering spike for ADR-0001. `SPIKE_MODE` picks the prototype:
//! `webview` = (a) wgpu to a canvas in the webview, `native` = (b) native wgpu under a transparent
//! webview. With `SPIKE_OUT` set, the page runs the scripted test, writes JSON and exits.

use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use spike_renderer::{Renderer, Sample};
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

struct Native(Mutex<Option<Renderer>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeInfo {
    backend: String,
    adapter_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Timing {
    /// Tessellate, draw, present and wait for the GPU, in Rust.
    rust_ms: f64,
}

#[tauri::command]
fn spike_config() -> serde_json::Value {
    serde_json::json!({
        "mode": std::env::var("SPIKE_MODE").unwrap_or_else(|_| "webview".to_owned()),
        "api": std::env::var("SPIKE_API").unwrap_or_else(|_| "webgpu".to_owned()),
        "scripted": std::env::var("SPIKE_OUT").is_ok(),
    })
}

#[tauri::command(async)]
fn native_info(state: State<'_, Native>) -> Result<NativeInfo, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    let renderer = guard.as_ref().ok_or("no native renderer")?;
    Ok(NativeInfo {
        backend: renderer.backend.clone(),
        adapter_name: renderer.adapter_name.clone(),
    })
}

#[tauri::command(async)]
fn native_background(state: State<'_, Native>, count: u32) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let renderer = guard.as_mut().ok_or("no native renderer")?;
    renderer.load_background(count);
    renderer.render();
    renderer.wait_for_gpu();
    Ok(())
}

#[tauri::command(async)]
fn native_sample(
    state: State<'_, Native>,
    x: f32,
    y: f32,
    pressure: f32,
) -> Result<Timing, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let renderer = guard.as_mut().ok_or("no native renderer")?;
    let start = Instant::now();
    renderer.add_sample(Sample { x, y, pressure });
    renderer.render();
    renderer.wait_for_gpu();
    Ok(Timing {
        rust_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

#[tauri::command(async)]
fn native_pen_up(state: State<'_, Native>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    guard.as_mut().ok_or("no native renderer")?.pen_up();
    Ok(())
}

/// Measures IPC alone: one sample's worth of data there and back.
#[tauri::command(async)]
fn ipc_echo(x: f32, y: f32, pressure: f32) -> (f32, f32, f32) {
    (x, y, pressure)
}

#[tauri::command]
fn write_results(app: tauri::AppHandle, results: serde_json::Value) -> Result<(), String> {
    if let Ok(path) = std::env::var("SPIKE_OUT") {
        let text = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| e.to_string())?;
    }
    app.exit(0);
    Ok(())
}

fn main() {
    let native = std::env::var("SPIKE_MODE").as_deref() == Ok("native");
    tauri::Builder::default()
        .manage(Native(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            spike_config,
            native_info,
            native_background,
            native_sample,
            native_pen_up,
            ipc_echo,
            write_results
        ])
        .setup(move |app| {
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Celframe rendering spike")
                .inner_size(1280.0, 800.0)
                .transparent(native)
                .build()?;
            if native {
                let size = window.inner_size()?;
                let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
                descriptor.backends = match std::env::var("SPIKE_BACKEND").as_deref() {
                    Ok("dx12") => wgpu::Backends::DX12,
                    Ok("vulkan") => wgpu::Backends::VULKAN,
                    _ => wgpu::Backends::all(),
                };
                let instance = wgpu::Instance::new(descriptor);
                let surface = instance
                    .create_surface(window.clone())
                    .map_err(|e| e.to_string())?;
                let power = if std::env::var("SPIKE_POWER").as_deref() == Ok("low") {
                    wgpu::PowerPreference::LowPower
                } else {
                    wgpu::PowerPreference::HighPerformance
                };
                let renderer = pollster::block_on(Renderer::new(
                    instance,
                    surface,
                    size.width,
                    size.height,
                    power,
                ))?;
                *app.state::<Native>().0.lock().map_err(|e| e.to_string())? = Some(renderer);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the spike failed to start");
}
