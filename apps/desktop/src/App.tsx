import { Menu, SegmentedControl } from "@celframe/ui";
import { useEffect } from "react";
import { backend } from "./backend";
import { CommandPalette } from "./CommandPalette";
import { RecoveryDialog } from "./RecoveryDialog";
import { useApp } from "./store";
import { ColourPanel } from "./studio/ColourPanel";
import { RedoIcon, SearchIcon, UndoIcon } from "./studio/icons";
import { SidePanelResizer } from "./studio/SidePanelResizer";
import { LayersPanel } from "./studio/LayersPanel";
import { ShelfBar } from "./studio/ShelfBar";
import { ToolProperties } from "./studio/ToolProperties";
import { StatusBar } from "./studio/StatusBar";
import { Timeline } from "./studio/Timeline";
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

const headerIcon =
  "flex h-6 w-6 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 disabled:hover:bg-transparent";

export function App() {
  const {
    document: doc,
    error,
    theme,
    sidePanelOpen,
    sidePanelWidth,
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
    selectAll,
    clearSelection,
    deleteSelection,
    nudgeSelection,
    editSelection,
  } = useApp();

  const [canvasRef, nativeCanvas] = useNativeCanvas<HTMLElement>(theme);
  usePenInput(
    canvasRef,
    nativeCanvas &&
      (tool === "pen" ||
        tool === "eraser" ||
        tool === "shape" ||
        tool === "select") &&
      !spaceHeld,
  );
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
      // Scene units per arrow press. +Y is up in the scene.
      const step = event.shiftKey ? 10 : 1;
      const lower = event.key.toLowerCase();
      if (event.shiftKey && (lower === "h" || lower === "v")) {
        event.preventDefault();
        void editSelection({ type: "flip", horizontal: lower === "h" });
        return;
      }
      const action: Record<string, () => void> = {
        p: () => setTool("pen"),
        e: () => setTool("eraser"),
        u: () => setTool("shape"),
        v: () => setTool("select"),
        escape: () => void clearSelection(),
        delete: () => void deleteSelection(),
        backspace: () => void deleteSelection(),
        arrowup: () => void nudgeSelection(0, step),
        arrowdown: () => void nudgeSelection(0, -step),
        arrowleft: () => void nudgeSelection(-step, 0),
        arrowright: () => void nudgeSelection(step, 0),
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
  }, [
    setTool,
    setSpaceHeld,
    changeView,
    toggleArt,
    clearSelection,
    deleteSelection,
    nudgeSelection,
    editSelection,
  ]);

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
      } else if (
        ["[", "]", "{", "}"].includes(key) &&
        !isTextField(event.target)
      ) {
        // Shift turns [ and ] into { and } on many layouts.
        event.preventDefault();
        const up = key === "]" || key === "}";
        const all = event.shiftKey;
        void editSelection({
          type: "arrange",
          to: up ? (all ? "front" : "forward") : all ? "back" : "backward",
        });
      } else if (key === "a" && !isTextField(event.target)) {
        event.preventDefault();
        void selectAll();
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
  }, [
    setPaletteOpen,
    undo,
    redo,
    save,
    saveAs,
    requestFileAction,
    selectAll,
    editSelection,
  ]);

  return (
    // No background here: the canvas area must stay see-through for the native canvas.
    <div className="flex h-screen flex-col bg-surface text-text">
      <header className="flex h-8 shrink-0 items-center gap-0.5 border-b border-border bg-surface-raised px-1.5">
        <span
          aria-hidden="true"
          className="mr-1.5 ml-0.5 flex h-[18px] w-[18px] items-center justify-center rounded-[4px] bg-accent text-[10px] font-bold text-text-on-accent"
        >
          C
        </span>
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
                label: "Select all",
                shortcut: "Ctrl+A",
                onSelect: () => void selectAll(),
              },
              {
                label: "Deselect",
                shortcut: "Esc",
                disabled: !doc?.selection.length,
                onSelect: () => void clearSelection(),
              },
              {
                label: "Delete",
                shortcut: "Del",
                disabled: !doc?.selection.length,
                onSelect: () => void deleteSelection(),
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
        <div className="mx-2 h-4 w-px bg-border" aria-hidden="true" />
        <h1 className="truncate text-sm text-text-muted">
          <span className="font-medium text-text">
            {doc?.project.name ?? "Celframe"}
          </span>
          {doc?.dirty ? <span> (unsaved)</span> : null}
        </h1>
        <div className="flex-1" />
        <div className="h-8">
          <SegmentedControl
            label="Mode"
            options={MODES}
            value="draw"
            onChange={() => undefined}
          />
        </div>
        <div className="flex-1" />
        <div role="group" aria-label="History" className="mr-2 flex gap-0.5">
          <button
            type="button"
            aria-label="Undo"
            title="Undo (Ctrl+Z)"
            disabled={!doc?.canUndo}
            onClick={() => void undo()}
            className={headerIcon}
          >
            <UndoIcon />
          </button>
          <button
            type="button"
            aria-label="Redo"
            title="Redo (Ctrl+Shift+Z)"
            disabled={!doc?.canRedo}
            onClick={() => void redo()}
            className={headerIcon}
          >
            <RedoIcon />
          </button>
        </div>
        <button
          type="button"
          onClick={() => setPaletteOpen(true)}
          className="flex h-6 w-56 items-center gap-1.5 rounded border border-border bg-surface-sunken px-2 text-sm text-text-muted outline-none hover:border-control-border focus-visible:outline-2 focus-visible:outline-focus-ring"
        >
          <SearchIcon />
          <span className="flex-1 text-left">Search commands</span>
          <kbd className="font-sans text-xs">Ctrl+K</kbd>
        </button>
      </header>

      <ShelfBar />

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
                      : tool === "eraser"
                        ? "cursor-cell"
                        : "cursor-crosshair"
                }`
              : "bg-canvas-backdrop"
          }`}
        />
        {sidePanelOpen ? (
          <aside
            aria-label="Side panels"
            style={{ width: sidePanelWidth }}
            className="relative flex shrink-0 flex-col border-l border-border bg-surface"
          >
            <SidePanelResizer />
            <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
              <ToolProperties />
              <ColourPanel />
              <LayersPanel />
            </div>
          </aside>
        ) : null}
      </div>

      {timelineOpen ? (
        <div className="h-40 shrink-0 border-t border-border">
          <Timeline />
        </div>
      ) : null}

      {error ? (
        <div
          role="alert"
          className="border-t border-border bg-surface-raised px-3 py-1 text-sm text-danger"
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
