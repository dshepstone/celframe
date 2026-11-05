// @vitest-environment happy-dom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend, type DocumentView } from "./backend";
import { useApp } from "./store";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    undo: vi.fn(),
    redo: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const view = (overrides: Partial<DocumentView> = {}): DocumentView => ({
  project: {
    id: "p",
    name: "Bouncing ball",
    scenes: [{ id: "s", name: "Shot 1" }],
  },
  canUndo: false,
  canRedo: false,
  dirty: false,
  path: null,
  ...overrides,
});

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
        project: { id: "p", name: "Earlier", scenes: [] },
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

  it("collapses the side panel", async () => {
    render(<App />);
    await screen.findByText("Shot 1");
    await userEvent.click(
      screen.getByRole("button", { name: "Collapse Scene panel" }),
    );
    expect(screen.queryByText("Shot 1")).toBeNull();
  });

  it("switches and applies the theme", async () => {
    render(<App />);
    await screen.findByText("Bouncing ball");
    await userEvent.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(screen.getByRole("button", { name: "Light theme" })).toBeTruthy();
  });
});
