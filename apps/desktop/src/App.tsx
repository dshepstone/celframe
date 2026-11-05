import { Button, SegmentedControl, SidePanel } from "@celframe/ui";
import { useEffect } from "react";
import { backend } from "./backend";
import { CommandPalette } from "./CommandPalette";
import { RecoveryDialog } from "./RecoveryDialog";
import { useApp } from "./store";
import { UnsavedChangesDialog } from "./UnsavedChangesDialog";

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
    load,
    undo,
    redo,
    requestFileAction,
    save,
    saveAs,
    toggleTheme,
    setSidePanelOpen,
    setPaletteOpen,
  } = useApp();

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
    <div className="flex h-screen flex-col bg-surface text-text">
      <header className="flex items-center gap-4 border-b border-border px-4 py-2">
        <h1 className="text-base font-semibold tracking-tight">
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
        <Button onClick={() => setPaletteOpen(true)}>Commands</Button>
        <Button onClick={toggleTheme}>
          {theme === "dark" ? "Light theme" : "Dark theme"}
        </Button>
      </header>

      <div
        role="toolbar"
        aria-label="Tool options"
        className="flex items-center gap-2 border-b border-border bg-surface-raised px-4 py-1.5 text-sm"
      >
        <Button disabled={!doc?.canUndo} onClick={() => void undo()}>
          Undo
        </Button>
        <Button disabled={!doc?.canRedo} onClick={() => void redo()}>
          Redo
        </Button>
        <span className="text-text-muted">Pen options appear here.</span>
      </div>

      <div className="flex min-h-0 flex-1">
        <main
          aria-label="Canvas"
          className="flex flex-1 items-center justify-center bg-canvas-backdrop"
        />
        <SidePanel
          title="Scene"
          open={sidePanelOpen}
          onOpenChange={setSidePanelOpen}
        >
          <ul>
            {doc?.project.scenes.map((scene) => (
              <li key={scene.id}>{scene.name}</li>
            ))}
          </ul>
        </SidePanel>
      </div>

      {error ? (
        <div role="alert" className="bg-surface-raised px-4 py-2 text-danger">
          {error}
        </div>
      ) : null}
      <CommandPalette />
      <UnsavedChangesDialog />
      <RecoveryDialog />
    </div>
  );
}
