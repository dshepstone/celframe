import { Modal } from "@celframe/ui";
import { useEffect, useMemo, useState, type KeyboardEvent } from "react";
import { useApp } from "./store";

export interface PaletteCommand {
  id: string;
  label: string;
  shortcut?: string;
  enabled: boolean;
  run: () => void;
}

/** Matches when every typed word appears in the label. */
export function matches(label: string, query: string): boolean {
  const text = label.toLowerCase();
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((word) => text.includes(word));
}

export function useCommands(): PaletteCommand[] {
  const {
    document: doc,
    theme,
    sidePanelOpen,
    undo,
    redo,
    requestFileAction,
    save,
    saveAs,
    toggleTheme,
    setSidePanelOpen,
  } = useApp();
  return [
    {
      id: "new",
      label: "New project",
      shortcut: "Ctrl/Cmd+N",
      enabled: true,
      run: () => void requestFileAction("new"),
    },
    {
      id: "open",
      label: "Open project",
      shortcut: "Ctrl/Cmd+O",
      enabled: true,
      run: () => void requestFileAction("open"),
    },
    {
      id: "save",
      label: "Save",
      shortcut: "Ctrl/Cmd+S",
      enabled: doc !== null,
      run: () => void save(),
    },
    {
      id: "save-as",
      label: "Save as",
      shortcut: "Ctrl/Cmd+Shift+S",
      enabled: doc !== null,
      run: () => void saveAs(),
    },
    {
      id: "undo",
      label: "Undo",
      shortcut: "Ctrl/Cmd+Z",
      enabled: doc?.canUndo ?? false,
      run: () => void undo(),
    },
    {
      id: "redo",
      label: "Redo",
      shortcut: "Ctrl/Cmd+Shift+Z",
      enabled: doc?.canRedo ?? false,
      run: () => void redo(),
    },
    {
      id: "theme",
      label: theme === "dark" ? "Use light theme" : "Use dark theme",
      enabled: true,
      run: toggleTheme,
    },
    {
      id: "panel",
      label: sidePanelOpen ? "Collapse side panel" : "Expand side panel",
      enabled: true,
      run: () => setSidePanelOpen(!sidePanelOpen),
    },
  ];
}

export function CommandPalette() {
  const { paletteOpen, setPaletteOpen } = useApp();
  const commands = useCommands();
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);

  const results = useMemo(
    () => commands.filter((c) => c.enabled && matches(c.label, query)),
    [commands, query],
  );

  useEffect(() => {
    if (paletteOpen) {
      setQuery("");
      setActive(0);
    }
  }, [paletteOpen]);

  const runCommand = (command: PaletteCommand | undefined) => {
    if (!command) return;
    setPaletteOpen(false);
    command.run();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (results.length === 0) return;
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActive((i) => (i + 1) % results.length);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive((i) => (i - 1 + results.length) % results.length);
    } else if (event.key === "Enter") {
      event.preventDefault();
      runCommand(results[active]);
    }
  };

  return (
    <Modal title="Commands" open={paletteOpen} onOpenChange={setPaletteOpen}>
      <input
        role="combobox"
        aria-label="Search commands"
        aria-expanded="true"
        aria-controls="palette-results"
        aria-activedescendant={
          results[active] ? `palette-${results[active].id}` : undefined
        }
        placeholder="Type a command"
        value={query}
        onChange={(event) => {
          setQuery(event.target.value);
          setActive(0);
        }}
        onKeyDown={onKeyDown}
        className="w-full rounded-md border border-control-border bg-surface px-3 py-2 text-sm text-text outline-none placeholder:text-text-muted focus-visible:outline-2 focus-visible:outline-focus-ring"
      />
      <ul id="palette-results" role="listbox" className="mt-2">
        {results.map((command, index) => (
          <li
            key={command.id}
            id={`palette-${command.id}`}
            role="option"
            aria-selected={index === active}
            onClick={() => runCommand(command)}
            onMouseEnter={() => setActive(index)}
            className={`flex cursor-pointer justify-between rounded-md px-3 py-1.5 text-sm ${
              index === active ? "bg-accent text-text-on-accent" : "text-text"
            }`}
          >
            <span>{command.label}</span>
            {command.shortcut ? <span>{command.shortcut}</span> : null}
          </li>
        ))}
        {results.length === 0 ? (
          <li className="px-3 py-1.5 text-sm text-text-muted">No commands</li>
        ) : null}
      </ul>
    </Modal>
  );
}
