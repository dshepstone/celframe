# ADR-0001: Rendering approach

Status: accepted, pending a manual check on macOS and Linux (see Consequences)

## Context

Celframe draws strokes with wgpu. The UI is a Tauri webview. There are two ways to put the
canvas on screen:

- **(a) In the webview.** The renderer is compiled to WebAssembly and draws to a `<canvas>`
  through the webview's WebGPU, falling back to WebGL2.
- **(b) Native, under a transparent webview.** The Rust process draws straight into the window
  with native wgpu. The webview sits on top with a transparent background, and the canvas area
  shows through.

Throwaway prototypes of both, sharing one stroke renderer, are on the `spike/14-15-rendering`
branch (PR #48). Each prototype measured the time from a pen sample to the GPU finishing the
frame, and to the next displayed frame, with 100, 1,000 and 10,000 strokes on the canvas.

## Results

Windows, with the same Intel UHD integrated GPU for both prototypes. Median time from sample
to next frame, three runs each:

| Strokes | (a) WebGPU in webview   | (b) Native, DX12 |
| ------- | ----------------------- | ---------------- |
| 100     | 7.1–7.6 ms              | 4.1–4.3 ms       |
| 1,000   | 7.0–8.9 ms              | 4.8–5.0 ms       |
| 10,000  | 9.2–9.4 ms, 332 ms once | about 890 ms     |

- On an RTX 3070 laptop GPU, (b) stays at 3–4 ms up to 10,000 strokes on both DX12 and Vulkan.
- The webview always chose the integrated GPU, even with a discrete GPU present. Only (b) can
  choose the high-performance GPU.
- (b)'s collapse at 10,000 strokes on the integrated GPU also happens with (a)'s WebGL2 fallback,
  but not with (a)'s WebGPU path, which runs the same renderer code. It points to a fault in the
  spike renderer rather than in the approach, and is to be fixed in the real renderer.
- Tauri IPC adds about 1.3 ms per round trip. (b) pays it on every sample and is still faster.

macOS and Linux ran in CI only, on a virtual Apple GPU and on llvmpipe software rendering, so their
timings aren't meaningful and are treated as unmeasured. They did show that:

- (a) found no WebGPU adapter on macOS CI and had to use WebGL on Linux (WebKitGTK).
- (b) ran without errors on both, but nobody has yet checked by eye that the webview composites
  correctly over the native surface.

## Decision

Use **(b): native wgpu under a transparent webview.**

- It's faster on the same hardware and can use the discrete GPU.
- It keeps the Rust core as the single source of truth. The renderer reads strokes from the core
  directly, instead of keeping a second copy in the webview in sync over IPC.
- The golden-image test (#29) can render headlessly through the same code the app uses.
- It uses the same native graphics path on every OS, instead of per-OS WebGPU or WebGL fallbacks.

## Consequences

- **Manual check before Release 0.1:** on a real Mac and a real Linux desktop, confirm that strokes
  draw under the toolbar and panels, that the canvas area is transparent, and that resizing and
  moving between displays with different scale factors stay correct. If Linux compositing fails,
  revisit this decision for Linux before more rendering work builds on it.
- The renderer owns a window surface, so it has to handle resize, scale factor changes and surface
  loss itself.
- The webview must keep a transparent background over the canvas area. Opaque UI is drawn only
  outside it, or on top as panels.
- Pen input still arrives in the webview and reaches the core over IPC, about 1.3 ms per call.
- The 10,000-stroke slowdown seen in the spike must be found and fixed in `celframe-render`, with a
  benchmark that would catch it again.
