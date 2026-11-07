// @vitest-environment happy-dom
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend } from "./backend";
import { useApp } from "./store";
import { scene } from "./testing/fixtures";
import { hexToRgba } from "./useNativeCanvas";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(() =>
      Promise.resolve({
        project: { id: "p", name: "Ball", scenes: [scene()] },
        canUndo: false,
        canRedo: false,
        dirty: false,
        path: null,
      }),
    ),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    setCanvasLayout: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const canvas = () => screen.getByRole("main", { name: "Canvas" });

beforeEach(() => {
  vi.clearAllMocks();
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

describe("native canvas", () => {
  it("converts hex colours to RGBA bytes", () => {
    expect(hexToRgba("#d9d4c8")).toEqual([217, 212, 200, 255]);
    expect(hexToRgba("#0f1012")).toEqual([15, 16, 18, 255]);
  });

  it("sends the canvas area and theme backdrop, then lets the canvas show through", async () => {
    vi.mocked(backend.setCanvasLayout).mockResolvedValue(true);
    render(<App />);
    await waitFor(() =>
      expect(canvas().getAttribute("data-native-canvas")).toBe("true"),
    );
    expect(canvas().className).toContain("bg-transparent");
    expect(backend.setCanvasLayout).toHaveBeenCalledWith(
      expect.objectContaining({
        scale: window.devicePixelRatio,
        backdrop: [217, 212, 200, 255],
      }),
    );
  });

  it("stays opaque when there is no native canvas", async () => {
    vi.mocked(backend.setCanvasLayout).mockResolvedValue(false);
    render(<App />);
    await waitFor(() => expect(backend.setCanvasLayout).toHaveBeenCalled());
    expect(canvas().className).toContain("bg-canvas-backdrop");
  });

  it("stays opaque if the call fails, such as outside the desktop app", async () => {
    vi.mocked(backend.setCanvasLayout).mockRejectedValue(new Error("no IPC"));
    render(<App />);
    await waitFor(() => expect(backend.setCanvasLayout).toHaveBeenCalled());
    expect(canvas().className).toContain("bg-canvas-backdrop");
  });

  it("sends the new backdrop when the theme changes", async () => {
    vi.mocked(backend.setCanvasLayout).mockResolvedValue(true);
    render(<App />);
    await screen.findByText("Ball");
    act(() => useApp.getState().setTheme("dark"));
    await waitFor(() =>
      expect(backend.setCanvasLayout).toHaveBeenLastCalledWith(
        expect.objectContaining({ backdrop: [15, 16, 18, 255] }),
      ),
    );
  });
});
