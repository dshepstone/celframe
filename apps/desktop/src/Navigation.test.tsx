// @vitest-environment happy-dom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend } from "./backend";
import { useApp } from "./store";
import { documentView } from "./testing/fixtures";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    setCanvasLayout: vi.fn(),
    changeView: vi.fn(),
    penDown: vi.fn(),
    penMove: vi.fn(),
    penUp: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

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

async function renderWithCanvas() {
  render(<App />);
  await screen.findByText("Bouncing ball");
  await waitFor(() =>
    expect(canvas().getAttribute("data-native-canvas")).toBe("true"),
  );
}

beforeEach(() => {
  vi.mocked(backend.getDocument).mockResolvedValue(documentView());
  vi.mocked(backend.setCanvasLayout).mockResolvedValue(true);
  vi.mocked(backend.changeView).mockResolvedValue({
    zoomPercent: 150,
    rotationDegrees: 15,
    fitted: false,
  });
  vi.mocked(backend.penDown).mockResolvedValue();
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

describe("canvas navigation", () => {
  it("zooms with the wheel, anchored at the pointer", async () => {
    await renderWithCanvas();
    // The test DOM doesn't give wheel events a position, so build one that has it.
    const wheel = new MouseEvent("wheel", {
      clientX: 30,
      clientY: 40,
      bubbles: true,
      cancelable: true,
    });
    Object.assign(wheel, { deltaY: -100, deltaMode: 0 });
    canvas().dispatchEvent(wheel);
    expect(backend.changeView).toHaveBeenCalledWith({
      type: "zoom",
      factor: expect.closeTo(1.15, 6),
      anchor: { x: 30, y: 40 },
    });
    expect(wheel.defaultPrevented).toBe(true);
  });

  it("pans with the Pan tool instead of drawing", async () => {
    await renderWithCanvas();
    await userEvent.click(screen.getByRole("button", { name: "Pan" }));
    pointer("pointerDown", { clientX: 10, clientY: 10 });
    pointer("pointerMove", { clientX: 25, clientY: 4 });
    pointer("pointerUp", { clientX: 25, clientY: 4 });
    expect(backend.changeView).toHaveBeenCalledWith({
      type: "pan",
      dx: 15,
      dy: -6,
    });
    expect(backend.penDown).not.toHaveBeenCalled();
  });

  it("pans with the middle button while the pen is chosen", async () => {
    await renderWithCanvas();
    pointer("pointerDown", { button: 1, clientX: 0, clientY: 0 });
    pointer("pointerMove", { clientX: 5, clientY: 5 });
    pointer("pointerUp", { clientX: 5, clientY: 5 });
    expect(backend.changeView).toHaveBeenCalledWith({
      type: "pan",
      dx: 5,
      dy: 5,
    });
    expect(backend.penDown).not.toHaveBeenCalled();
  });

  it("pans while Space is held, and draws again after", async () => {
    await renderWithCanvas();
    fireEvent.keyDown(window, { key: " " });
    pointer("pointerDown", { clientX: 0, clientY: 0 });
    pointer("pointerMove", { clientX: 3, clientY: 0 });
    pointer("pointerUp", { clientX: 3, clientY: 0 });
    expect(backend.penDown).not.toHaveBeenCalled();
    expect(backend.changeView).toHaveBeenCalledWith({
      type: "pan",
      dx: 3,
      dy: 0,
    });
    fireEvent.keyUp(window, { key: " " });
    await waitFor(() => expect(useApp.getState().spaceHeld).toBe(false));
    pointer("pointerDown", { pointerId: 2, clientX: 1, clientY: 1 });
    expect(backend.penDown).toHaveBeenCalledOnce();
  });

  it("zooms in on a click with the Zoom tool, and out with Alt", async () => {
    await renderWithCanvas();
    fireEvent.keyDown(window, { key: "z" });
    pointer("pointerDown", { clientX: 7, clientY: 8 });
    expect(backend.changeView).toHaveBeenLastCalledWith({
      type: "zoom",
      factor: 2,
      anchor: { x: 7, y: 8 },
    });
    pointer("pointerDown", {
      pointerId: 2,
      altKey: true,
      clientX: 7,
      clientY: 8,
    });
    expect(backend.changeView).toHaveBeenLastCalledWith({
      type: "zoom",
      factor: 0.5,
      anchor: { x: 7, y: 8 },
    });
    expect(backend.penDown).not.toHaveBeenCalled();
  });

  it("has keyboard shortcuts for zoom, fit and rotation", async () => {
    await renderWithCanvas();
    for (const key of ["+", "-", "0", ",", "."]) {
      fireEvent.keyDown(window, { key });
    }
    expect(vi.mocked(backend.changeView).mock.calls.map(([c]) => c)).toEqual([
      { type: "zoom", factor: 1.25 },
      { type: "zoom", factor: 0.8 },
      { type: "fit" },
      { type: "rotate", degrees: 15 },
      { type: "rotate", degrees: -15 },
    ]);
  });

  it("ignores single-key shortcuts while typing", async () => {
    await renderWithCanvas();
    const size = screen.getByRole("spinbutton", { name: "Brush size" });
    fireEvent.keyDown(size, { key: "0" });
    fireEvent.keyDown(size, { key: "h" });
    expect(backend.changeView).not.toHaveBeenCalled();
    expect(useApp.getState().tool).toBe("pen");
  });

  it("shows the zoom and rotation the core reports", async () => {
    await renderWithCanvas();
    await userEvent.click(screen.getByRole("button", { name: "Zoom in" }));
    expect(screen.getByLabelText("Zoom level").textContent).toBe("150%");
    expect(screen.getByLabelText("Rotation angle").textContent).toBe("15°");
    expect(
      (screen.getByRole("button", { name: "Fit" }) as HTMLButtonElement)
        .disabled,
    ).toBe(false);
  });
});
