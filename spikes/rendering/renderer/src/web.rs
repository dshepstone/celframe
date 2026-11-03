//! Prototype (a): wgpu compiled to WebAssembly, drawing to a canvas in the webview.

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{Renderer, Sample};

#[wasm_bindgen]
pub struct WebRenderer(Renderer);

#[wasm_bindgen]
impl WebRenderer {
    /// `api` is "webgpu" or "webgl2".
    pub async fn create(canvas: HtmlCanvasElement, api: String) -> Result<WebRenderer, JsError> {
        let (width, height) = (canvas.width(), canvas.height());
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = if api == "webgpu" {
            wgpu::Backends::BROWSER_WEBGPU
        } else {
            wgpu::Backends::GL
        };
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let renderer = Renderer::new(instance, surface, width, height, wgpu::PowerPreference::HighPerformance)
            .await
            .map_err(|e| JsError::new(&e))?;
        Ok(Self(renderer))
    }

    #[wasm_bindgen(getter)]
    pub fn backend(&self) -> String {
        self.0.backend.clone()
    }

    #[wasm_bindgen(getter, js_name = adapterName)]
    pub fn adapter_name(&self) -> String {
        self.0.adapter_name.clone()
    }

    #[wasm_bindgen(js_name = loadBackground)]
    pub fn load_background(&mut self, count: u32) {
        self.0.load_background(count);
    }

    #[wasm_bindgen(js_name = addSample)]
    pub fn add_sample(&mut self, x: f32, y: f32, pressure: f32) {
        self.0.add_sample(Sample { x, y, pressure });
    }

    #[wasm_bindgen(js_name = penUp)]
    pub fn pen_up(&mut self) {
        self.0.pen_up();
    }

    pub fn render(&mut self) -> bool {
        self.0.render()
    }

    /// Runs completion callbacks. The WebGL path needs this called until `gpuDone` resolves.
    pub fn poll(&self) {
        self.0.poll();
    }

    /// Resolves when the GPU has finished submitted work.
    #[wasm_bindgen(js_name = gpuDone)]
    pub fn gpu_done(&self) -> js_sys::Promise {
        let queue_done = js_sys::Promise::new(&mut |resolve, _| {
            let resolve = send_wrapper(resolve);
            self.0.on_done(move || {
                let _ = resolve.0.call0(&JsValue::NULL);
            });
        });
        queue_done
    }
}

// Browsers run this on one thread, so the callback can't actually change threads.
struct AssertSend(js_sys::Function);
unsafe impl Send for AssertSend {}
fn send_wrapper(f: js_sys::Function) -> AssertSend {
    AssertSend(f)
}
