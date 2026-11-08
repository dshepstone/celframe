// @vitest-environment happy-dom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend, type DocumentView } from "./backend";
import { useApp } from "./store";
import { scene } from "./testing/fixtures";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    setCanvasLayout: vi.fn(() => Promise.resolve(false)),
    getRecovery: vi.fn(),
    restoreRecovery: vi.fn(),
    discardRecovery: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const view = (name: string, dirty = false): DocumentView => ({
  project: { id: name, name, scenes: [scene()] },
  canUndo: false,
  canRedo: false,
  dirty,
  path: null,
  selection: [],
  selectionBounds: null,
  currentFrame: 0,
  sceneLength: 1,
});

const question = /Celframe closed before your changes were saved/;

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(backend.getDocument).mockResolvedValue(view("Untitled"));
  vi.mocked(backend.getRecovery).mockResolvedValue({
    originalPath: "ball.celf",
    savedAtMs: Date.UTC(2026, 10, 5, 12, 0),
  });
  vi.mocked(backend.restoreRecovery).mockResolvedValue(view("Ball", true));
  vi.mocked(backend.discardRecovery).mockResolvedValue();
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

describe("recovery at launch", () => {
  it("doesn't ask when there is nothing to recover", async () => {
    vi.mocked(backend.getRecovery).mockResolvedValue(null);
    render(<App />);
    await screen.findByText("Untitled");
    await waitFor(() => expect(backend.getRecovery).toHaveBeenCalled());
    expect(screen.queryByText(question)).toBeNull();
  });

  it("restores the work as unsaved changes", async () => {
    render(<App />);
    expect(await screen.findByText(question)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Restore" }));
    expect(await screen.findByText("Ball")).toBeTruthy();
    expect(screen.getByText("(unsaved)")).toBeTruthy();
    expect(screen.queryByText(question)).toBeNull();
  });

  it("discards the work", async () => {
    render(<App />);
    await screen.findByText(question);
    await userEvent.click(screen.getByRole("button", { name: "Discard" }));
    await waitFor(() => expect(screen.queryByText(question)).toBeNull());
    expect(backend.discardRecovery).toHaveBeenCalledOnce();
    expect(screen.getByText("Untitled")).toBeTruthy();
  });

  it("can't be dismissed with Escape", async () => {
    render(<App />);
    await screen.findByText(question);
    await userEvent.keyboard("{Escape}");
    expect(screen.getByText(question)).toBeTruthy();
  });

  it("keeps asking, and explains, when the copy can't be restored", async () => {
    vi.mocked(backend.restoreRecovery).mockRejectedValue(
      "Couldn't restore your unsaved work: this .celf file is damaged",
    );
    render(<App />);
    await screen.findByText(question);
    await userEvent.click(screen.getByRole("button", { name: "Restore" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("damaged"),
    );
    expect(screen.getByText(question)).toBeTruthy();
  });
});
