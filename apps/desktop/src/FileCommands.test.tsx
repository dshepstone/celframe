// @vitest-environment happy-dom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend, type DocumentView } from "./backend";
import { useApp } from "./store";
import { scene } from "./testing/fixtures";

let closeRequested: () => void = () => undefined;

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    setCanvasLayout: vi.fn(() => Promise.resolve(false)),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    undo: vi.fn(),
    redo: vi.fn(),
    newDocument: vi.fn(),
    openDocument: vi.fn(),
    saveDocument: vi.fn(),
    saveDocumentAs: vi.fn(),
    closeWindow: vi.fn(),
    onCloseRequested: vi.fn((handler: () => void) => {
      closeRequested = handler;
      return Promise.resolve(() => undefined);
    }),
  },
}));

const view = (name: string, dirty = false): DocumentView => ({
  project: { id: name, name, scenes: [scene()] },
  canUndo: dirty,
  canRedo: false,
  dirty,
  path: null,
  selection: [],
  selectionBounds: null,
  currentFrame: 0,
  sceneLength: 1,
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(backend.getDocument).mockResolvedValue(view("Ball"));
  vi.mocked(backend.newDocument).mockResolvedValue(view("Untitled"));
  vi.mocked(backend.openDocument).mockResolvedValue(view("Walk cycle"));
  vi.mocked(backend.saveDocument).mockResolvedValue(view("Ball"));
  vi.mocked(backend.saveDocumentAs).mockResolvedValue(view("Ball"));
  vi.mocked(backend.closeWindow).mockResolvedValue();
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

async function renderApp(dirty = false) {
  vi.mocked(backend.getDocument).mockResolvedValue(view("Ball", dirty));
  render(<App />);
  await screen.findByText("Ball");
}

const prompt = () => screen.findByText(/Save changes to “Ball” before/);

describe("file commands", () => {
  it("runs New, Open, Save and Save As from their shortcuts", async () => {
    await renderApp();
    await userEvent.keyboard("{Control>}s{/Control}");
    expect(backend.saveDocument).toHaveBeenCalledOnce();
    await userEvent.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    expect(backend.saveDocumentAs).toHaveBeenCalledOnce();
    await userEvent.keyboard("{Control>}o{/Control}");
    expect(await screen.findByText("Walk cycle")).toBeTruthy();
    await userEvent.keyboard("{Meta>}n{/Meta}");
    expect(await screen.findByText("Untitled")).toBeTruthy();
  });

  it("runs file commands from the palette", async () => {
    await renderApp();
    await userEvent.keyboard("{Control>}k{/Control}");
    await userEvent.keyboard("save as{Enter}");
    expect(backend.saveDocumentAs).toHaveBeenCalledOnce();
  });

  it("keeps the document when the file picker is cancelled", async () => {
    vi.mocked(backend.openDocument).mockResolvedValue(null);
    await renderApp();
    await userEvent.keyboard("{Control>}o{/Control}");
    await waitFor(() => expect(backend.openDocument).toHaveBeenCalled());
    expect(screen.getByText("Ball")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("shows why a file couldn't be opened and keeps the document", async () => {
    vi.mocked(backend.openDocument).mockRejectedValue(
      "Couldn't open broken.celf: this isn't a valid .celf file",
    );
    await renderApp();
    await userEvent.keyboard("{Control>}o{/Control}");
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("broken.celf"),
    );
    expect(screen.getByText("Ball")).toBeTruthy();
  });
});

describe("unsaved changes", () => {
  it("doesn't ask when there is nothing to lose", async () => {
    await renderApp();
    await userEvent.keyboard("{Control>}n{/Control}");
    expect(await screen.findByText("Untitled")).toBeTruthy();
    expect(screen.queryByText(/Save changes/)).toBeNull();
  });

  it("cancels with Escape and does nothing", async () => {
    await renderApp(true);
    await userEvent.keyboard("{Control>}n{/Control}");
    await prompt();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByText(/Save changes/)).toBeNull();
    expect(backend.newDocument).not.toHaveBeenCalled();
    expect(backend.saveDocument).not.toHaveBeenCalled();
  });

  it("discards the changes with Don't save", async () => {
    await renderApp(true);
    await userEvent.keyboard("{Control>}o{/Control}");
    await prompt();
    await userEvent.click(screen.getByRole("button", { name: "Don’t save" }));
    expect(await screen.findByText("Walk cycle")).toBeTruthy();
    expect(backend.saveDocument).not.toHaveBeenCalled();
  });

  it("saves first, then continues", async () => {
    await renderApp(true);
    await userEvent.keyboard("{Control>}n{/Control}");
    await prompt();
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByText("Untitled")).toBeTruthy();
    expect(backend.saveDocument).toHaveBeenCalledOnce();
  });

  it("stops if saving is cancelled or fails", async () => {
    vi.mocked(backend.saveDocument).mockResolvedValueOnce(null);
    await renderApp(true);
    await userEvent.keyboard("{Control>}n{/Control}");
    await prompt();
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(backend.saveDocument).toHaveBeenCalled());

    vi.mocked(backend.saveDocument).mockRejectedValueOnce("Couldn't save");
    await userEvent.keyboard("{Control>}n{/Control}");
    await prompt();
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(screen.getByRole("alert")).toBeTruthy());

    expect(backend.newDocument).not.toHaveBeenCalled();
    expect(screen.getByText("Ball")).toBeTruthy();
  });

  it("asks when the core reports a close request, then closes", async () => {
    await renderApp(true);
    closeRequested();
    expect(
      await screen.findByText(/Save changes to “Ball” before closing/),
    ).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Don’t save" }));
    expect(backend.closeWindow).toHaveBeenCalledOnce();
  });
});
