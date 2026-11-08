// @vitest-environment happy-dom
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
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
    editSelection: vi.fn(),
    setHandlesVisible: vi.fn(() => Promise.resolve()),
    setViewMode: vi.fn(() => Promise.resolve(null)),
    setCurrentFrame: vi.fn(),
    editTiming: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

/** The Undo and Redo buttons in the menu row, not the ones on a shelf. */
const history = () => within(screen.getByRole("group", { name: "History" }));
const panel = () =>
  within(screen.getByRole("region", { name: "Tool properties" }));

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
    const undo = history().getByRole("button", { name: "Undo" });
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
    await userEvent.click(history().getByRole("button", { name: "Undo" }));
    expect(await screen.findByText("Earlier")).toBeTruthy();
  });

  it("shows core errors without losing the document", async () => {
    vi.mocked(backend.getDocument).mockResolvedValue(view({ canUndo: true }));
    vi.mocked(backend.undo).mockRejectedValue("the item no longer exists");
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.click(history().getByRole("button", { name: "Undo" }));
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
    expect(useApp.getState().penOptions().tool).toEqual({
      type: "select",
      lasso: false,
      scaleWidths: false,
    });

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

  it("shows panels in dock tabs that can be dragged, resized and reset", async () => {
    localStorage.removeItem("celframe.dock");
    render(<App />);
    await screen.findByText("Bouncing ball");
    const right = () =>
      within(screen.getByRole("region", { name: "right panels" }));
    expect(right().getByRole("tab", { name: "Tool properties" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Timeline" })).toBeTruthy();

    // Drag the Colour tab into the Layers stack's tab strip.
    const colour = right().getByRole("tab", { name: "Colour" });
    const layersStrip = right()
      .getByRole("tab", { name: "Layers" })
      .closest("[data-dock-tabs]");
    fireEvent.pointerDown(colour, { button: 0, clientX: 10, clientY: 10 });
    act(() => {
      window.dispatchEvent(
        new MouseEvent("pointermove", { clientX: 40, clientY: 40 }),
      );
    });
    expect(useApp.getState().dockDrag?.panel).toBe("colour");
    document.elementsFromPoint = () => (layersStrip ? [layersStrip] : []);
    act(() => {
      window.dispatchEvent(
        new MouseEvent("pointermove", { clientX: 500, clientY: 300 }),
      );
    });
    act(() => {
      window.dispatchEvent(new MouseEvent("pointerup"));
    });
    expect(useApp.getState().dockDrag).toBeNull();
    const stacks = useApp.getState().dock.areas.right.map((s) => s.tabs);
    expect(stacks).toEqual([["tool"], ["layers", "colour"]]);
    expect(
      right().getByRole("tab", { name: "Colour", selected: true }),
    ).toBeTruthy();

    // Resize the right area from the keyboard.
    const edge = screen.getByRole("separator", {
      name: "Resize the right area",
    });
    const before = Number(edge.getAttribute("aria-valuenow"));
    edge.focus();
    await userEvent.keyboard("{ArrowLeft}");
    expect(Number(edge.getAttribute("aria-valuenow"))).toBe(before + 16);

    const stored = JSON.parse(localStorage.getItem("celframe.dock") ?? "{}");
    expect(stored.sizes.right).toBe(before + 16);

    await userEvent.click(screen.getByRole("button", { name: "Window" }));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: "Reset layout" }),
    );
    expect(useApp.getState().dock.areas.right.map((s) => s.tabs)).toEqual([
      ["tool"],
      ["colour"],
      ["layers"],
    ]);
  });

  it("switches between the Drawing and Camera views", async () => {
    vi.mocked(backend.setViewMode).mockResolvedValue({
      zoomPercent: 42,
      rotationDegrees: 0,
      fitted: true,
    });
    render(<App />);
    await screen.findByText("Bouncing ball");
    const views = within(screen.getByRole("tablist", { name: "View" }));
    expect(
      views.getByRole("tab", { name: "Camera", selected: true }),
    ).toBeTruthy();

    await userEvent.click(views.getByRole("tab", { name: "Drawing" }));
    expect(backend.setViewMode).toHaveBeenLastCalledWith("drawing", null);
    expect(
      views.getByRole("tab", { name: "Drawing", selected: true }),
    ).toBeTruthy();
    await waitFor(() => expect(useApp.getState().view?.zoomPercent).toBe(42));

    // The Drawing view follows the chosen layer.
    act(() => useApp.getState().setActiveLayer("l1"));
    expect(backend.setViewMode).toHaveBeenLastCalledWith("drawing", "l1");

    await userEvent.keyboard("{Control>}2{/Control}");
    expect(backend.setViewMode).toHaveBeenLastCalledWith("camera", "l1");
  });

  it("moves through frames and edits timing from the timeline and keyboard", async () => {
    const at = (currentFrame: number) =>
      view({ currentFrame, sceneLength: 12 });
    vi.mocked(backend.getDocument).mockResolvedValue(at(0));
    vi.mocked(backend.setCurrentFrame).mockImplementation((frame) =>
      Promise.resolve(at(frame)),
    );
    vi.mocked(backend.editTiming).mockResolvedValue(at(0));
    render(<App />);
    await screen.findByText("Bouncing ball");
    const timeline = within(screen.getByRole("region", { name: "Timeline" }));

    await userEvent.click(
      timeline.getByRole("button", { name: "Go to frame 5" }),
    );
    expect(backend.setCurrentFrame).toHaveBeenLastCalledWith(4);
    await waitFor(() =>
      expect(
        timeline.getByRole("status", { name: "Current frame" }).textContent,
      ).toContain("Frame 5"),
    );
    await userEvent.keyboard("{Alt>}{ArrowRight}{/Alt}");
    expect(backend.setCurrentFrame).toHaveBeenLastCalledWith(5);
    await userEvent.keyboard("{End}");
    expect(backend.setCurrentFrame).toHaveBeenLastCalledWith(11);
    await userEvent.keyboard("{Home}");
    expect(backend.setCurrentFrame).toHaveBeenLastCalledWith(0);

    await userEvent.keyboard("{Alt>}n{/Alt}");
    expect(backend.editTiming).toHaveBeenLastCalledWith(
      { type: "newDrawing" },
      null,
    );
    await userEvent.click(
      timeline.getByRole("button", { name: "Hold longer" }),
    );
    expect(backend.editTiming).toHaveBeenLastCalledWith(
      { type: "hold", by: 1 },
      null,
    );
  });

  it("plays at the frame rate, looping, and stops on an edit", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const at = (currentFrame: number) => view({ currentFrame, sceneLength: 3 });
    vi.mocked(backend.getDocument).mockResolvedValue(at(1));
    vi.mocked(backend.setCurrentFrame).mockImplementation((frame) =>
      Promise.resolve(at(frame)),
    );
    vi.mocked(backend.editTiming).mockResolvedValue(at(0));
    render(<App />);
    await screen.findByText("Bouncing ball");
    act(() => useApp.getState().togglePlayback());
    expect(useApp.getState().playing).toBe(true);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2 * (1000 / 24) + 1);
    });
    // From frame 2 of 3: on to frame 3, then wrap to frame 1.
    const frames = vi
      .mocked(backend.setCurrentFrame)
      .mock.calls.map(([f]) => f);
    expect(frames.slice(0, 2)).toEqual([2, 0]);
    await act(async () => {
      await useApp.getState().editTiming({ type: "insertFrame" });
    });
    expect(useApp.getState().playing).toBe(false);
    vi.useRealTimers();
  });

  it("cancels a tab drag with Escape", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    act(() =>
      useApp
        .getState()
        .setDockDrag({ panel: "layers", x: 0, y: 0, target: null }),
    );
    await userEvent.keyboard("{Escape}");
    expect(useApp.getState().dockDrag).toBeNull();
    expect(useApp.getState().dock.areas.right.at(-1)?.tabs).toEqual(["layers"]);
  });

  it("runs, customises and remembers shelves", async () => {
    localStorage.removeItem("celframe.shelves");
    vi.mocked(backend.getDocument).mockResolvedValue(view({ canUndo: true }));
    vi.mocked(backend.undo).mockResolvedValue(view());
    render(<App />);
    await screen.findByText("Bouncing ball");
    const bar = () => within(screen.getByRole("toolbar", { name: "Shelves" }));

    expect(
      bar().getByRole("tab", { name: "Quick", selected: true }),
    ).toBeTruthy();
    await userEvent.click(bar().getByRole("button", { name: "Undo" }));
    expect(backend.undo).toHaveBeenCalledOnce();
    const flip = bar().getByRole("button", { name: "Flip horizontally" });
    expect((flip as HTMLButtonElement).disabled).toBe(true);

    await userEvent.click(bar().getByRole("tab", { name: "Shapes" }));
    await userEvent.click(bar().getByRole("button", { name: "Ellipse" }));
    expect(useApp.getState().tool).toBe("shape");
    expect(useApp.getState().shapeKind).toBe("ellipse");

    await userEvent.click(bar().getByRole("button", { name: "New shelf" }));
    expect(
      bar().getByRole("tab", { name: "Shelf 3", selected: true }),
    ).toBeTruthy();
    await userEvent.dblClick(bar().getByRole("tab", { name: "Shelf 3" }));
    await userEvent.type(
      bar().getByRole("textbox", { name: "Rename Shelf 3" }),
      "{Control>}a{/Control}Cleanup{Enter}",
    );
    await userEvent.click(bar().getByRole("button", { name: "Customise" }));
    await userEvent.click(bar().getByRole("button", { name: "Add action" }));
    await userEvent.type(
      bar().getByRole("textbox", { name: "Find an action" }),
      "fit{Enter}",
    );
    await userEvent.click(bar().getByRole("button", { name: "Done" }));
    expect(
      bar().getByRole("button", { name: "Fit scene in view" }),
    ).toBeTruthy();

    const stored = JSON.parse(localStorage.getItem("celframe.shelves") ?? "[]");
    expect(stored.at(-1)).toMatchObject({ name: "Cleanup", actions: ["fit"] });
  });

  it("shows each tool's settings in the Tool properties panel", async () => {
    vi.mocked(backend.undo).mockResolvedValue(view());
    vi.mocked(backend.getDocument).mockResolvedValue(view({ canUndo: true }));
    render(<App />);
    await screen.findByText("Bouncing ball");
    const panel = screen.getByRole("region", { name: "Tool properties" });
    expect(panel.textContent).toContain("Pen");
    expect(screen.getByRole("spinbutton", { name: "Brush size" })).toBeTruthy();
    expect(screen.queryByRole("toolbar", { name: "Tool options" })).toBeNull();

    await userEvent.keyboard("u");
    expect(screen.getByRole("radiogroup", { name: "Shape" })).toBeTruthy();
    await userEvent.keyboard("h");
    expect(within(panel).getByRole("button", { name: "Fit" })).toBeTruthy();

    await userEvent.click(history().getByRole("button", { name: "Undo" }));
    expect(backend.undo).toHaveBeenCalledOnce();
  });

  it("transforms and arranges the selection from the tool properties", async () => {
    const selected = view({
      selection: ["k1"],
      selectionBounds: [0, 0, 100, 50],
    });
    vi.mocked(backend.getDocument).mockResolvedValue(selected);
    vi.mocked(backend.editSelection).mockResolvedValue(selected);
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("v");

    const width = screen.getByRole("textbox", { name: "Width" });
    expect((width as HTMLInputElement).value).toBe("100");
    await userEvent.clear(width);
    await userEvent.type(width, "200{Enter}");
    expect(backend.editSelection).toHaveBeenLastCalledWith({
      type: "setBounds",
      x: 50,
      y: 25,
      width: 200,
      height: 100,
      scaleWidths: false,
    });

    await userEvent.click(
      panel().getByRole("button", { name: "Flip vertically" }),
    );
    expect(backend.editSelection).toHaveBeenLastCalledWith({
      type: "flip",
      horizontal: false,
    });
    await userEvent.click(
      panel().getByRole("button", { name: "Send to back" }),
    );
    expect(backend.editSelection).toHaveBeenLastCalledWith({
      type: "arrange",
      to: "back",
    });
    await userEvent.keyboard("{Control>}]{/Control}");
    expect(backend.editSelection).toHaveBeenLastCalledWith({
      type: "arrange",
      to: "forward",
    });
    await userEvent.keyboard("{Shift>}h{/Shift}");
    expect(backend.editSelection).toHaveBeenLastCalledWith({
      type: "flip",
      horizontal: true,
    });
    expect(useApp.getState().tool).toBe("select");
  });

  it("shows handles only with the Select tool, and sends the lasso mode", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("v");
    await waitFor(() =>
      expect(backend.setHandlesVisible).toHaveBeenLastCalledWith(true),
    );
    await userEvent.click(screen.getByRole("radio", { name: "Lasso" }));
    expect(useApp.getState().penOptions().tool).toMatchObject({ lasso: true });
    await userEvent.keyboard("p");
    await waitFor(() =>
      expect(backend.setHandlesVisible).toHaveBeenLastCalledWith(false),
    );
  });

  it("disables the transform controls with nothing selected", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.keyboard("v");
    expect(
      (
        panel().getByRole("button", {
          name: "Flip horizontally",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
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
