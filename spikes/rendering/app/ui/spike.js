// Runs the same input path for both prototypes and records event-to-GPU-done time.
const { invoke } = window.__TAURI_INTERNALS__;
const surface = document.getElementById("surface");
const panel = document.getElementById("panel");
const config = await invoke("spike_config");
const LOADS = [100, 1000, 10000];
const SAMPLES_PER_RUN = 300;
const SAMPLE_INTERVAL_MS = 8;

const penStats = { pointerTypes: {}, maxPressure: 0, coalesced: 0, events: 0 };
const log = (text) => (panel.textContent = text);
const nextFrame = () => new Promise(requestAnimationFrame);
const sleepUntil = (time) =>
  new Promise((resolve) =>
    setTimeout(resolve, Math.max(0, time - performance.now())),
  );

async function webBackend() {
  const init = await import("./pkg/spike_renderer.js");
  await init.default();
  const ratio = devicePixelRatio;
  surface.width = innerWidth * ratio;
  surface.height = innerHeight * ratio;
  const hasWebGpu = "gpu" in navigator;
  const api = config.api === "webgpu" && hasWebGpu ? "webgpu" : "webgl2";
  const renderer = await init.WebRenderer.create(surface, api);
  const gpu = hasWebGpu
    ? await navigator.gpu.requestAdapter({
        powerPreference: "high-performance",
      })
    : null;
  const adapterInfo = gpu?.info
    ? {
        vendor: gpu.info.vendor,
        architecture: gpu.info.architecture,
        description: gpu.info.description,
      }
    : null;
  const gpuDone = async () => {
    // WebGPU completes by itself. WebGL only completes when polled.
    const timer =
      api === "webgl2" ? setInterval(() => renderer.poll(), 0) : null;
    await renderer.gpuDone();
    if (timer !== null) clearInterval(timer);
  };
  return {
    info: {
      backend: renderer.backend,
      adapterName: renderer.adapterName,
      hasWebGpu,
      api,
      adapterInfo,
    },
    background: async (count) => {
      renderer.loadBackground(count);
      renderer.render();
      await gpuDone();
    },
    sample: async (x, y, pressure) => {
      renderer.addSample(x, y, pressure);
      renderer.render();
      await gpuDone();
      return {};
    },
    penUp: async () => renderer.penUp(),
  };
}

async function nativeBackend() {
  surface.style.background = "transparent";
  const info = await invoke("native_info");
  return {
    info: { ...info, hasWebGpu: "gpu" in navigator },
    background: (count) => invoke("native_background", { count }),
    sample: (x, y, pressure) => invoke("native_sample", { x, y, pressure }),
    penUp: () => invoke("native_pen_up"),
  };
}

function percentile(sorted, p) {
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))];
}

function summarize(values) {
  const sorted = [...values].sort((a, b) => a - b);
  return {
    n: sorted.length,
    p50: percentile(sorted, 0.5),
    p95: percentile(sorted, 0.95),
    p99: percentile(sorted, 0.99),
    max: sorted[sorted.length - 1],
  };
}

async function measureIpc() {
  const times = [];
  for (let i = 0; i < 500; i++) {
    const start = performance.now();
    await invoke("ipc_echo", { x: 0.5, y: 0.5, pressure: 0.5 });
    times.push(performance.now() - start);
  }
  return summarize(times);
}

// Dispatches real pointer events, so the browser event path is part of the measurement.
async function measureLoad(backend, count) {
  log(`Loading ${count} strokes…`);
  const loadStart = performance.now();
  await backend.background(count);
  const loadMs = performance.now() - loadStart;
  await nextFrame();

  const toGpu = [];
  const toFrame = [];
  const rust = [];
  let pending = Promise.resolve();
  const onMove = (event) => {
    const start = performance.now();
    const x = (event.clientX / innerWidth) * 2 - 1;
    const y = 1 - (event.clientY / innerHeight) * 2;
    pending = pending.then(async () => {
      const timing = await backend.sample(x, y, event.pressure);
      toGpu.push(performance.now() - start);
      if (timing.rustMs !== undefined) rust.push(timing.rustMs);
      await nextFrame();
      toFrame.push(performance.now() - start);
    });
  };
  surface.addEventListener("pointermove", onMove);

  const begin = performance.now();
  for (let i = 0; i < SAMPLES_PER_RUN; i++) {
    const t = i / SAMPLES_PER_RUN;
    surface.dispatchEvent(
      new PointerEvent("pointermove", {
        clientX: innerWidth * (0.1 + 0.8 * t),
        clientY: innerHeight * (0.5 + 0.3 * Math.sin(t * 25)),
        pressure: 0.3 + 0.7 * Math.abs(Math.sin(t * 9)),
        bubbles: true,
      }),
    );
    await sleepUntil(begin + (i + 1) * SAMPLE_INTERVAL_MS);
  }
  await pending;
  surface.removeEventListener("pointermove", onMove);
  await backend.penUp();

  return {
    strokes: count,
    loadMs,
    eventToGpuDoneMs: summarize(toGpu),
    eventToNextFrameMs: summarize(toFrame),
    rustOnlyMs: rust.length ? summarize(rust) : null,
  };
}

function handTest(backend) {
  let down = false;
  surface.addEventListener("pointerdown", (event) => {
    down = true;
    surface.setPointerCapture(event.pointerId);
  });
  surface.addEventListener("pointerup", () => {
    down = false;
    void backend.penUp();
  });
  surface.addEventListener("pointermove", (event) => {
    penStats.events++;
    penStats.pointerTypes[event.pointerType] = true;
    penStats.maxPressure = Math.max(penStats.maxPressure, event.pressure);
    if (!down) return;
    const batch = event.getCoalescedEvents?.() ?? [event];
    penStats.coalesced = Math.max(penStats.coalesced, batch.length);
    for (const e of batch.length ? batch : [event]) {
      void backend.sample(
        (e.clientX / innerWidth) * 2 - 1,
        1 - (e.clientY / innerHeight) * 2,
        e.pointerType === "mouse" ? 1 : e.pressure,
      );
    }
    log(`Hand test (${config.mode})\npen types: ${Object.keys(penStats.pointerTypes)}
max pressure: ${penStats.maxPressure.toFixed(3)}\nmost coalesced: ${penStats.coalesced}`);
  });
}

async function main() {
  const backend =
    config.mode === "native" ? await nativeBackend() : await webBackend();
  if (!config.scripted) {
    await backend.background(1000);
    log(
      `${config.mode}: ${JSON.stringify(backend.info)}\nDraw with a pen or mouse.`,
    );
    handTest(backend);
    return;
  }
  const ipc = await measureIpc();
  const runs = [];
  for (const count of LOADS) runs.push(await measureLoad(backend, count));
  await invoke("write_results", {
    results: {
      mode: config.mode,
      platform: navigator.userAgent,
      devicePixelRatio,
      info: backend.info,
      ipcRoundTripMs: ipc,
      runs,
    },
  });
}

main().catch(async (error) => {
  log(`Failed: ${error}`);
  if (config.scripted) {
    await invoke("write_results", {
      results: {
        mode: config.mode,
        platform: navigator.userAgent,
        hasWebGpu: "gpu" in navigator,
        error: String(error),
      },
    });
  }
});
