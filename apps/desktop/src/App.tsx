import { Menu, SegmentedControl } from "@celframe/ui";
import { useEffect } from "react";
import { backend } from "./backend";
import { CommandPalette } from "./CommandPalette";
import { RecoveryDialog } from "./RecoveryDialog";
import { useApp } from "./store";
import { ColourPanel } from "./studio/ColourPanel";
import { LayersPanel } from "./studio/LayersPanel";
import { StatusBar } from "./studio/StatusBar";
import { Timeline } from "./studio/Timeline";
import { ToolOptions } from "./studio/ToolOptions";
import { ToolRail } from "./studio/ToolRail";
import { UnsavedChangesDialog } from "./UnsavedChangesDialog";
import { useNativeCanvas } from "./useNativeCanvas";
import { useCanvasNavigation } from "./useCanvasNavigation";
import { usePenInput } from "./usePenInput";

const LATER = "Coming in a later phase";

export const MODES = [
  { value: "draw", label: "Draw" },
  { value: "animate", label: "Animate", disabled: true, hint: LATER },
  { value: "rig", label: "Rig", disabled: true, hint: LATER },
  { value: "composite", label: "Composite", disabled: true, hint: LATER },
];

function isTextField(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && target.matches("input, textarea");
}

export function App() {
  const {
    document: doc,
    error,
    theme,
    sidePanelOpen,
    timelineOpen,
    load,
    undo,
    redo,
    requestFileAction,
    save,
    saveAs,
    toggleTheme,
    setSidePanelOpen,
    setTimelineOpen,
    setPaletteOpen,
    addLayer,
    tool,
    spaceHeld,
    setTool,
    setSpaceHeld,
    changeView,
    toggleArt,
  } = useApp();

  const [canvasRef, nativeCanvas] = useNativeCanvas<HTMLElement>(theme);
  usePenInput(canvasRef, nativeCanvas && tool === "pen" && !spaceHeld);
  useCanvasNavigation(canvasRef, nativeCanvas);

  // Single-key shortcuts for tools and the view. Ignored while typing.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      if (isTextField(event.target)) return;
      if (event.key === " ") {
        event.preventDefault();
        if (!event.repeat) setSpaceHeld(true);
        return;
      }
      const action: Record<string, () => void> = {
        p: () => setTool("pen"),
        h: () => setTool("hand"),
        z: () => setTool("zoom"),
        l: () => toggleArt(),
        "=": () => void changeView({ type: "zoom", factor: 1.25 }),
        "+": () => void changeView({ type: "zoom", factor: 1.25 }),
        "-": () => void changeView({ type: "zoom", factor: 0.8 }),
        "0": () => void changeView({ type: "fit" }),
        ",": () => void changeView({ type: "rotate", degrees: 15 }),
        ".": () => void changeView({ type: "rotate", degrees: -15 }),
      };
      const run = action[event.key.toLowerCase()];
      if (run) {
        event.preventDefault();
        run();
      }
    };
    const onKeyUp = (event: KeyboardEvent) => {
      if (event.key === " ") setSpaceHeld(false);
    };
    // Releasing Space in another window must not leave panning stuck on.
    const onBlur = () => setSpaceHeld(false);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("blur", onBlur);
    };
  }, [setTool, setSpaceHeld, changeView, toggleArt]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const unlisten = backend.onCloseRequested(
      () => void requestFileAction("close"),
    );
    return () => void unlisten.then((stop) => stop());
  }, [requestFileAction]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!event.ctrlKey && !event.metaKey) return;
      const key = event.key.toLowerCase();
      if (key === "k") {
        event.preventDefault();
        setPaletteOpen(true);
      } else if (key === "z" && !isTextField(event.target)) {
        event.preventDefault();
        void (event.shiftKey ? redo() : undo());
      } else if (key === "s") {
        event.preventDefault();
        void (event.shiftKey ? saveAs() : save());
      } else if ((key === "n" || key === "o") && !event.shiftKey) {
        event.preventDefault();
        void requestFileAction(key === "n" ? "new" : "open");
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setPaletteOpen, undo, redo, save, saveAs, requestFileAction]);

  return (
    // No background here: the canvas area must stay see-through for the native canvas.
    <div className="flex h-screen flex-col text-text">
      <header className="flex h-10 items-center gap-1 border-b border-border bg-surface px-2">
        <nav aria-label="Menu" className="flex items-center">
          <Menu
            label="File"
            items={[
              {
                label: "New",
                shortcut: "Ctrl+N",
                onSelect: () => void requestFileAction("new"),
              },
              {
                label: "Open…",
                shortcut: "Ctrl+O",
                onSelect: () => void requestFileAction("open"),
              },
              "separator",
              {
                label: "Save",
                shortcut: "Ctrl+S",
                onSelect: () => void save(),
              },
              {
                label: "Save As…",
                shortcut: "Ctrl+Shift+S",
                onSelect: () => void saveAs(),
              },
            ]}
          />
          <Menu
            label="Edit"
            items={[
              {
                label: "Undo",
                shortcut: "Ctrl+Z",
                disabled: !doc?.canUndo,
                onSelect: () => void undo(),
              },
              {
                label: "Redo",
                shortcut: "Ctrl+Shift+Z",
                disabled: !doc?.canRedo,
                onSelect: () => void redo(),
              },
              "separator",
              {
                label: "Command palette",
                shortcut: "Ctrl+K",
                onSelect: () => setPaletteOpen(true),
              },
            ]}
          />
          <Menu
            label="Layer"
            items={[{ label: "New layer", onSelect: () => void addLayer() }]}
          />
          <Menu
            label="View"
            items={[
              {
                label: "Zoom in",
                shortcut: "+",
                onSelect: () => void changeView({ type: "zoom", factor: 1.25 }),
              },
              {
                label: "Zoom out",
                shortcut: "-",
                onSelect: () => void changeView({ type: "zoom", factor: 0.8 }),
              },
              {
                label: "Fit scene",
                shortcut: "0",
                onSelect: () => void changeView({ type: "fit" }),
              },
              {
                label: "Rotate left",
                shortcut: ",",
                onSelect: () =>
                  void changeView({ type: "rotate", degrees: 15 }),
              },
              {
                label: "Rotate right",
                shortcut: ".",
                onSelect: () =>
                  void changeView({ type: "rotate", degrees: -15 }),
              },
              "separator",
              {
                label: sidePanelOpen ? "Hide side panels" : "Show side panels",
                onSelect: () => setSidePanelOpen(!sidePanelOpen),
              },
              {
                label: timelineOpen ? "Hide timeline" : "Show timeline",
                onSelect: () => setTimelineOpen(!timelineOpen),
              },
              "separator",
              {
                label: theme === "dark" ? "Light theme" : "Dark theme",
                onSelect: toggleTheme,
              },
            ]}
          />
        </nav>
        <h1 className="ml-3 truncate text-sm font-semibold tracking-tight">
          {doc?.project.name ?? "Celframe"}
          {doc?.dirty ? (
            <span className="text-text-muted"> (unsaved)</span>
          ) : null}
        </h1>
        <div className="flex-1" />
        <SegmentedControl
          label="Mode"
          options={MODES}
          value="draw"
          onChange={() => undefined}
        />
        <div className="flex-1" />
        <button
          type="button"
          onClick={() => setPaletteOpen(true)}
          className="rounded-md border border-control-border px-2.5 py-1 text-sm text-text-muted outline-none hover:bg-surface-sunken focus-visible:outline-2 focus-visible:outline-focus-ring"
        >
          Search commands <kbd className="ml-1 font-sans">Ctrl+K</kbd>
        </button>
      </header>

      <ToolOptions />

      <div className="flex min-h-0 flex-1">
        <ToolRail />
        <main
          ref={canvasRef}
          aria-label="Canvas"
          data-native-canvas={nativeCanvas}
          className={`flex flex-1 touch-none items-center justify-center ${
            nativeCanvas
              ? `bg-transparent ${
                  spaceHeld || tool === "hand"
                    ? "cursor-grab"
                    : tool === "zoom"
                      ? "cursor-zoom-in"
                      : "cursor-crosshair"
                }`
              : "bg-canvas-backdrop"
          }`}
        />
        {sidePanelOpen ? (
          <aside
            aria-label="Side panels"
            className="flex w-64 flex-col border-l border-border bg-surface-raised"
          >
            <div className="min-h-0 flex-1 overflow-auto">
              <LayersPanel />
            </div>
            <ColourPanel />
          </aside>
        ) : null}
      </div>

      {timelineOpen ? (
        <div className="h-44 border-t border-border">
          <Timeline />
        </div>
      ) : null}

      {error ? (
        <div
          role="alert"
          className="bg-surface-raised px-4 py-1.5 text-sm text-danger"
        >
          {error}
        </div>
      ) : null}
      <StatusBar />
      <CommandPalette />
      <UnsavedChangesDialog />
      <RecoveryDialog />
    </div>
  );
}
