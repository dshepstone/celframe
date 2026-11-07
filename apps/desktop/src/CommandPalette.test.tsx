// @vitest-environment happy-dom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { backend, type DocumentView } from "./backend";
import { matches } from "./CommandPalette";
import { useApp } from "./store";
import { scene } from "./testing/fixtures";

vi.mock("./backend", () => ({
  backend: {
    getDocument: vi.fn(),
    setCanvasLayout: vi.fn(() => Promise.resolve(false)),
    getRecovery: vi.fn(() => Promise.resolve(null)),
    undo: vi.fn(),
    redo: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => undefined)),
  },
}));

const view = (canUndo: boolean): DocumentView => ({
  project: { id: "p", name: "Ball", scenes: [scene()] },
  canUndo,
  canRedo: false,
  dirty: false,
  path: null,
});

beforeEach(() => {
  vi.mocked(backend.getDocument).mockResolvedValue(view(true));
  vi.mocked(backend.undo).mockResolvedValue(view(false));
  vi.mocked(backend.redo).mockResolvedValue(view(true));
  useApp.setState(useApp.getInitialState());
});

afterEach(cleanup);

async function openPalette() {
  render(<App />);
  await screen.findByText("Ball");
  await userEvent.keyboard("{Control>}k{/Control}");
  return screen.findByRole("combobox", { name: "Search commands" });
}

describe("matches", () => {
  it("finds every word, in any order and case", () => {
    expect(matches("Use dark theme", "THEME dark")).toBe(true);
    expect(matches("Use dark theme", "light")).toBe(false);
    expect(matches("Undo", "")).toBe(true);
  });
});

describe("command palette", () => {
  it("opens with Ctrl+K, focuses the search and closes with Escape", async () => {
    const input = await openPalette();
    expect(document.activeElement).toBe(input);
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("combobox")).toBeNull();
  });

  it("opens with Cmd+K", async () => {
    render(<App />);
    await screen.findByText("Ball");
    await userEvent.keyboard("{Meta>}k{/Meta}");
    expect(await screen.findByRole("combobox")).toBeTruthy();
  });

  it("filters, then runs the selected command with Enter", async () => {
    await openPalette();
    await userEvent.keyboard("undo{Enter}");
    expect(backend.undo).toHaveBeenCalledOnce();
    expect(screen.queryByRole("combobox")).toBeNull();
  });

  it("moves through results with the arrow keys, and wraps", async () => {
    await openPalette();
    const selected = () =>
      screen.getByRole("option", { selected: true }).textContent;
    expect(selected()).toContain("New project");
    await userEvent.keyboard("{ArrowDown}");
    expect(selected()).toContain("Open project");
    await userEvent.keyboard("{ArrowUp}{ArrowUp}");
    expect(selected()).toContain("Collapse side panel");
  });

  it("leaves out commands that can't run now", async () => {
    await openPalette();
    expect(screen.queryByRole("option", { name: /Redo/ })).toBeNull();
  });

  it("says when nothing matches, and Enter does nothing", async () => {
    await openPalette();
    await userEvent.keyboard("zzz{Enter}");
    expect(screen.getByText("No commands")).toBeTruthy();
    expect(screen.getByRole("combobox")).toBeTruthy();
  });
});

describe("keyboard shortcuts", () => {
  it("undoes with Ctrl+Z and redoes with Ctrl+Shift+Z", async () => {
    render(<App />);
    await screen.findByText("Ball");
    await userEvent.keyboard("{Control>}z{/Control}");
    expect(backend.undo).toHaveBeenCalledOnce();
    await userEvent.keyboard("{Control>}{Shift>}z{/Shift}{/Control}");
    expect(backend.redo).toHaveBeenCalledOnce();
  });

  it("leaves Ctrl+Z to the search box while typing", async () => {
    await openPalette();
    await userEvent.keyboard("{Control>}z{/Control}");
    expect(backend.undo).not.toHaveBeenCalled();
  });
});
