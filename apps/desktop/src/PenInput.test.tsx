// @vitest-environment happy-dom
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend, type DocumentView } from "./backend";
import { useApp } from "./store";
import { scene } from "./testing/fixtures";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    setCanvasLayout: vi.fn(),
    penDown: vi.fn(),
    penMove: vi.fn(),
    penUp: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const view = (dirty: boolean): DocumentView => ({
  project: { id: "p", name: "Ball", scenes: [scene()] },
  canUndo: dirty,
  canRedo: false,
  dirty,
  path: null,
});

const canvas = () => screen.getByRole("main", { name: "Canvas" });

const pointer = (
  type: "pointerDown" | "pointerMove" | "pointerUp",
  init: Partial<PointerEventInit>,
) =>
  fireEvent[type](canvas(), {
    pointerId: 1,
    pointerType: "mouse",
    button: 0,
    ...init,
  });

async function renderWithCanvas(native = true) {
  vi.mocked(backend.setCanvasLayout).mockResolvedValue(native);
  render(<App />);
  await screen.findByText("Ball");
  await waitFor(() =>
    expect(canvas().getAttribute("data-native-canvas")).toBe(String(native)),
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(backend.getDocument).mockResolvedValue(view(false));
  vi.mocked(backend.penDown).mockResolvedValue();
  vi.mocked(backend.penMove).mockResolvedValue();
  vi.mocked(backend.penUp).mockResolvedValue(view(true));
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

describe("pen input", () => {
  it("sends a stroke to the core, batching moves, and shows the result", async () => {
    await renderWithCanvas();
    pointer("pointerDown", { clientX: 10, clientY: 20 });
    expect(backend.penDown).toHaveBeenCalledWith(
      { x: 10, y: 20, pressure: 1, shift: false, alt: false },
      {
        width: 10,
        color: [0, 0, 0, 255],
        layer: null,
        art: "line",
        tool: { type: "draw" },
      },
    );

    pointer("pointerMove", { clientX: 11, clientY: 21 });
    pointer("pointerMove", { clientX: 12, clientY: 22 });
    await waitFor(() => expect(backend.penMove).toHaveBeenCalled());
    // Both moves arrive in one batch, in order.
    expect(backend.penMove).toHaveBeenCalledTimes(1);
    expect(vi.mocked(backend.penMove).mock.calls[0]?.[0]).toEqual([
      { x: 11, y: 21, pressure: 1, shift: false, alt: false },
      { x: 12, y: 22, pressure: 1, shift: false, alt: false },
    ]);

    pointer("pointerUp", { clientX: 12, clientY: 22 });
    expect(backend.penUp).toHaveBeenCalledOnce();
    expect(await screen.findByText("(unsaved)")).toBeTruthy();
  });

  it("sends samples still waiting for a frame before ending the stroke", async () => {
    await renderWithCanvas();
    pointer("pointerDown", { clientX: 0, clientY: 0 });
    pointer("pointerMove", { clientX: 5, clientY: 5 });
    pointer("pointerUp", {});
    expect(backend.penMove).toHaveBeenCalledWith([
      { x: 5, y: 5, pressure: 1, shift: false, alt: false },
    ]);
    const [moveOrder = 0] = vi.mocked(backend.penMove).mock.invocationCallOrder;
    const [upOrder = 0] = vi.mocked(backend.penUp).mock.invocationCallOrder;
    expect(moveOrder).toBeGreaterThan(0);
    expect(moveOrder).toBeLessThan(upOrder);
  });

  it("passes pen pressure through", async () => {
    await renderWithCanvas();
    pointer("pointerDown", {
      pointerType: "pen",
      pressure: 0.35,
      clientX: 1,
      clientY: 2,
    });
    expect(backend.penDown).toHaveBeenCalledWith(
      { x: 1, y: 2, pressure: 0.35, shift: false, alt: false },
      expect.anything(),
    );
  });

  it("draws with the chosen size, colour and layer", async () => {
    await renderWithCanvas();
    act(() => {
      useApp.getState().setBrushWidth(4);
      useApp.getState().setBrushColor([200, 10, 20, 255]);
      useApp.getState().setActiveLayer("l1");
    });
    pointer("pointerDown", { clientX: 1, clientY: 2 });
    expect(backend.penDown).toHaveBeenCalledWith(expect.anything(), {
      width: 4,
      color: [200, 10, 20, 255],
      layer: "l1",
      art: "line",
      tool: { type: "draw" },
    });
  });

  it("ignores other mouse buttons and a second pointer", async () => {
    await renderWithCanvas();
    pointer("pointerDown", { button: 2 });
    expect(backend.penDown).not.toHaveBeenCalled();
    pointer("pointerDown", {});
    pointer("pointerDown", { pointerId: 2 });
    expect(backend.penDown).toHaveBeenCalledOnce();
  });

  it("doesn't draw when there is no native canvas", async () => {
    await renderWithCanvas(false);
    pointer("pointerDown", {});
    expect(backend.penDown).not.toHaveBeenCalled();
  });

  it("shows why a stroke couldn't start", async () => {
    vi.mocked(backend.penDown).mockRejectedValue(
      "Show or unlock a layer to draw on it.",
    );
    await renderWithCanvas();
    pointer("pointerDown", {});
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("unlock a layer"),
    );
  });
});
