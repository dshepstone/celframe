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
import { backend, type DocumentView } from "./backend";
import { useApp } from "./store";
import { documentView, layer, scene } from "./testing/fixtures";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    setCanvasLayout: vi.fn(() => Promise.resolve(false)),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    undo: vi.fn(),
    redo: vi.fn(),
    executeCommand: vi.fn(),
    addLayer: vi.fn(),
    selectAll: vi.fn(),
    clearSelection: vi.fn(),
    deleteSelection: vi.fn(),
    nudgeSelection: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const view = (overrides: Partial<DocumentView> = {}): DocumentView =>
  documentView(overrides);

beforeEach(() => {
  vi.mocked(backend.getDocument).mockResolvedValue(view());
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

describe("App shell", () => {
  it("shows the document the core sends", async () => {
    vi.mocked(backend.getDocument).mockResolvedValue(view({ dirty: true }));
    render(<App />);
    expect(await screen.findByText("Bouncing ball")).toBeTruthy();
    expect(screen.getByText("(unsaved)")).toBeTruthy();
    expect(screen.getByText("Shot 1")).toBeTruthy();
  });

  it("has only Draw mode active", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    const radio = (name: string) =>
      screen.getByRole("radio", { name }) as HTMLButtonElement;
    expect(radio("Draw").getAttribute("aria-checked")).toBe("true");
    for (const mode of ["Animate", "Rig", "Composite"]) {
      expect(radio(mode).disabled).toBe(true);
    }
  });

  it("disables undo and redo until the core allows them", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    const undo = screen.getByRole("button", { name: "Undo" });
    expect((undo as HTMLButtonElement).disabled).toBe(true);
  });

  it("renders the state undo returns", async () => {
    vi.mocked(backend.getDocument).mockResolvedValue(view({ canUndo: true }));
    vi.mocked(backend.undo).mockResolvedValue(
      view({
        project: { id: "p", name: "Earlier", scenes: [scene()] },
      }),
    );
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(await screen.findByText("Earlier")).toBeTruthy();
  });

  it("shows core errors without losing the document", async () => {
    vi.mocked(backend.getDocument).mockResolvedValue(view({ canUndo: true }));
    vi.mocked(backend.undo).mockRejectedValue("the item no longer exists");
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.click(screen.getByRole("button", { name: "Undo" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "no longer exists",
      ),
    );
    expect(screen.getByText("Bouncing ball")).toBeTruthy();
  });

  it("hides the side panels from the View menu", async () => {
    render(<App />);
    await screen.findByRole("listbox", { name: "Layer list" });
    await userEvent.click(screen.getByRole("button", { name: "View" }));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: /Hide side panels/ }),
    );
    expect(screen.queryByRole("listbox", { name: "Layer list" })).toBeNull();
  });

  it("switches and applies the theme from the View menu", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.click(screen.getByRole("button", { name: "View" }));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: /Dark theme/ }),
    );
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("has a tool rail where later tools are marked as coming", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    const pen = screen.getByRole("button", { name: "Pen" });
    expect(pen.getAttribute("aria-pressed")).toBe("true");
    const fill = screen.getByRole("button", { name: /Fill/ });
    expect((fill as HTMLButtonElement).disabled).toBe(true);
  });

  it("drags out shapes with the pen's size and colour", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("u");
    expect(useApp.getState().penOptions()).toMatchObject({
      width: 10,
      tool: { type: "shape", shape: { type: "rectangle" } },
    });
    await userEvent.click(screen.getByRole("radio", { name: "Polygon" }));
    fireEvent.change(screen.getByRole("spinbutton", { name: "Sides" }), {
      target: { value: "40" },
    });
    expect(useApp.getState().penOptions().tool).toEqual({
      type: "shape",
      shape: { type: "polygon", sides: 12 },
    });
    await userEvent.click(screen.getByRole("radio", { name: "Line" }));
    expect(screen.queryByRole("spinbutton", { name: "Sides" })).toBeNull();
  });

  it("selects, nudges and deletes through the core", async () => {
    const selected = view({ selection: ["k1"] });
    vi.mocked(backend.selectAll).mockResolvedValue(selected);
    vi.mocked(backend.nudgeSelection).mockResolvedValue(selected);
    vi.mocked(backend.deleteSelection).mockResolvedValue(view());
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("v");
    expect(useApp.getState().penOptions().tool).toEqual({ type: "select" });

    await userEvent.keyboard("{Control>}a{/Control}");
    expect(backend.selectAll).toHaveBeenCalledWith({
      layer: null,
      art: "line",
    });
    await screen.findByText("1 selected");
    await userEvent.keyboard("{Shift>}{ArrowUp}{/Shift}");
    expect(backend.nudgeSelection).toHaveBeenCalledWith(0, 10);
    await userEvent.keyboard("{ArrowLeft}");
    expect(backend.nudgeSelection).toHaveBeenCalledWith(-1, 0);
    await userEvent.keyboard("{Delete}");
    expect(backend.deleteSelection).toHaveBeenCalledOnce();
    await waitFor(() => expect(screen.queryByText("1 selected")).toBeNull());
  });

  it("erases with the eraser's own size and mode", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("e");
    expect(useApp.getState().tool).toBe("eraser");
    const size = screen.getByRole("spinbutton", { name: "Eraser size" });
    fireEvent.change(size, { target: { value: "35" } });
    await userEvent.click(
      screen.getByRole("checkbox", { name: "Whole strokes" }),
    );
    expect(useApp.getState().penOptions()).toMatchObject({
      width: 35,
      tool: { type: "erase", wholeStrokes: true },
    });
    await userEvent.click(screen.getByRole("button", { name: "Pen" }));
    expect(useApp.getState().penOptions()).toMatchObject({
      width: 10,
      tool: { type: "draw" },
    });
  });

  it("toggles a layer's visibility through the core", async () => {
    vi.mocked(backend.executeCommand).mockResolvedValue(view());
    render(<App />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Hide Layer 1" }),
    );
    expect(backend.executeCommand).toHaveBeenCalledWith({
      type: "setLayerVisible",
      scene: "s",
      layer: "l1",
      visible: false,
    });
  });

  it("adds a layer and makes it the one to draw on", async () => {
    const two = scene({
      layers: [
        layer("l1", { name: "Layer 1" }),
        layer("l2", { name: "Layer 2" }),
      ],
    });
    vi.mocked(backend.addLayer).mockResolvedValue(
      view({ project: { id: "p", name: "Bouncing ball", scenes: [two] } }),
    );
    render(<App />);
    await userEvent.click(
      await screen.findByRole("button", { name: "Add layer" }),
    );
    await screen.findByText("Drawing on: Layer 2");
    const selected = screen.getByRole("option", { selected: true });
    expect(selected.textContent).toContain("Layer 2");
  });

  it("renames a layer with a double-click", async () => {
    vi.mocked(backend.executeCommand).mockResolvedValue(view());
    render(<App />);
    const row = await screen.findByRole("option", { name: /Layer 1/ });
    await userEvent.dblClick(row);
    const input = screen.getByRole("textbox", { name: "Rename Layer 1" });
    await userEvent.clear(input);
    await userEvent.type(input, "Ink{Enter}");
    expect(backend.executeCommand).toHaveBeenCalledWith({
      type: "renameLayer",
      scene: "s",
      layer: "l1",
      name: "Ink",
    });
  });

  it("switches between line art and colour art with the bar or L", async () => {
    render(<App />);
    await screen.findByText("Line art", { selector: "footer span" });
    await userEvent.click(screen.getByRole("radio", { name: "Colour art" }));
    expect(useApp.getState().penOptions().art).toBe("colour");
    await screen.findByText("Colour art", { selector: "footer span" });
    await userEvent.keyboard("l");
    expect(useApp.getState().art).toBe("line");
  });

  it("picks a pen colour from the swatches", async () => {
    render(<App />);
    await userEvent.click(
      await screen.findByRole("radio", { name: "Line red" }),
    );
    expect(useApp.getState().brushColor).toEqual([200, 50, 42, 255]);
    expect(
      screen.getByRole("img", { name: "Current colour #c8322a" }),
    ).toBeTruthy();
  });

  it("clamps the brush size", async () => {
    render(<App />);
    const size = await screen.findByRole("spinbutton", { name: "Brush size" });
    await userEvent.clear(size);
    await userEvent.type(size, "900");
    expect(useApp.getState().brushWidth).toBe(200);
  });

  it("shows each exposure on the timeline", async () => {
    const withExposure = scene({
      layers: [
        layer("l1", {
          name: "Layer 1",
          drawings: [{ id: "d1", strokes: [] }],
          exposures: [{ startFrame: 0, length: 3, drawing: "d1" }],
        }),
      ],
    });
    vi.mocked(backend.getDocument).mockResolvedValue(
      view({
        project: { id: "p", name: "Bouncing ball", scenes: [withExposure] },
      }),
    );
    render(<App />);
    expect(
      await screen.findByRole("cell", { name: "Drawing 1, frames 1 to 3" }),
    ).toBeTruthy();
  });
});
