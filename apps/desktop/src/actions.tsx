import type { ReactNode } from "react";
import type { ShapeKind } from "./backend";
import { useApp, type Tool } from "./store";
import {
  BackwardIcon,
  EraserIcon,
  FlipHorizontalIcon,
  FlipVerticalIcon,
  ForwardIcon,
  HandIcon,
  PenIcon,
  RedoIcon,
  RotateLeftIcon,
  RotateRightIcon,
  SelectIcon,
  ShapeIcon,
  ToBackIcon,
  ToFrontIcon,
  UndoIcon,
  ZoomIcon,
} from "./studio/icons";

export type ActionGroup =
  "Tools" | "Shapes" | "Edit" | "Selection" | "Art" | "View" | "Layer" | "File";

/** Something the user can do, from the command palette or a shelf. See docs/specs/shelves.md. */
export interface Action {
  id: string;
  label: string;
  group: ActionGroup;
  shortcut?: string;
  icon?: ReactNode;
  /** False when the action can't run now. */
  enabled: boolean;
  run: () => void;
}

/** Every action, in the order the command palette lists them. */
export function useActions(): Action[] {
  const state = useApp();
  const {
    document: doc,
    theme,
    sidePanelOpen,
    timelineOpen,
    undo,
    redo,
    requestFileAction,
    save,
    saveAs,
    toggleTheme,
    setSidePanelOpen,
    setTimelineOpen,
    setTool,
    setShapeKind,
    setArt,
    toggleArt,
    selectAll,
    clearSelection,
    deleteSelection,
    editSelection,
    changeView,
    addLayer,
  } = state;
  const hasDoc = doc !== null;
  const selected = (doc?.selection.length ?? 0) > 0;

  const tool = (
    id: Tool,
    label: string,
    shortcut: string,
    icon: ReactNode,
  ) => ({
    id: `tool.${id}`,
    label: `${label} tool`,
    group: "Tools" as const,
    shortcut,
    icon,
    enabled: true,
    run: () => setTool(id),
  });
  const shape = (kind: ShapeKind, label: string) => ({
    id: `shape.${kind}`,
    label,
    group: "Shapes" as const,
    enabled: true,
    run: () => {
      setShapeKind(kind);
      setTool("shape");
    },
  });

  return [
    {
      id: "new",
      label: "New project",
      group: "File",
      shortcut: "Ctrl/Cmd+N",
      enabled: true,
      run: () => void requestFileAction("new"),
    },
    {
      id: "open",
      label: "Open project",
      group: "File",
      shortcut: "Ctrl/Cmd+O",
      enabled: true,
      run: () => void requestFileAction("open"),
    },
    {
      id: "save",
      label: "Save",
      group: "File",
      shortcut: "Ctrl/Cmd+S",
      enabled: hasDoc,
      run: () => void save(),
    },
    {
      id: "save-as",
      label: "Save as",
      group: "File",
      shortcut: "Ctrl/Cmd+Shift+S",
      enabled: hasDoc,
      run: () => void saveAs(),
    },
    {
      id: "undo",
      label: "Undo",
      group: "Edit",
      shortcut: "Ctrl/Cmd+Z",
      icon: <UndoIcon />,
      enabled: doc?.canUndo ?? false,
      run: () => void undo(),
    },
    {
      id: "redo",
      label: "Redo",
      group: "Edit",
      shortcut: "Ctrl/Cmd+Shift+Z",
      icon: <RedoIcon />,
      enabled: doc?.canRedo ?? false,
      run: () => void redo(),
    },
    {
      id: "select-all",
      label: "Select all",
      group: "Edit",
      shortcut: "Ctrl/Cmd+A",
      enabled: hasDoc,
      run: () => void selectAll(),
    },
    {
      id: "deselect",
      label: "Deselect",
      group: "Edit",
      shortcut: "Esc",
      enabled: selected,
      run: () => void clearSelection(),
    },
    {
      id: "delete",
      label: "Delete selection",
      group: "Edit",
      shortcut: "Del",
      enabled: selected,
      run: () => void deleteSelection(),
    },
    tool("pen", "Pen", "P", <PenIcon />),
    tool("eraser", "Eraser", "E", <EraserIcon />),
    tool("shape", "Shape", "U", <ShapeIcon />),
    tool("select", "Select", "V", <SelectIcon />),
    tool("hand", "Pan", "H", <HandIcon />),
    tool("zoom", "Zoom", "Z", <ZoomIcon />),
    shape("line", "Line"),
    shape("rectangle", "Rectangle"),
    shape("ellipse", "Ellipse"),
    shape("polygon", "Polygon"),
    {
      id: "flip-horizontal",
      label: "Flip horizontally",
      group: "Selection",
      shortcut: "Shift+H",
      icon: <FlipHorizontalIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "flip", horizontal: true }),
    },
    {
      id: "flip-vertical",
      label: "Flip vertically",
      group: "Selection",
      shortcut: "Shift+V",
      icon: <FlipVerticalIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "flip", horizontal: false }),
    },
    {
      id: "rotate-left",
      label: "Rotate 90° left",
      group: "Selection",
      icon: <RotateLeftIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "rotate", degrees: 90 }),
    },
    {
      id: "rotate-right",
      label: "Rotate 90° right",
      group: "Selection",
      icon: <RotateRightIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "rotate", degrees: -90 }),
    },
    {
      id: "to-front",
      label: "Bring to front",
      group: "Selection",
      shortcut: "Ctrl/Cmd+Shift+]",
      icon: <ToFrontIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "arrange", to: "front" }),
    },
    {
      id: "forward",
      label: "Bring forward",
      group: "Selection",
      shortcut: "Ctrl/Cmd+]",
      icon: <ForwardIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "arrange", to: "forward" }),
    },
    {
      id: "backward",
      label: "Send backward",
      group: "Selection",
      shortcut: "Ctrl/Cmd+[",
      icon: <BackwardIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "arrange", to: "backward" }),
    },
    {
      id: "to-back",
      label: "Send to back",
      group: "Selection",
      shortcut: "Ctrl/Cmd+Shift+[",
      icon: <ToBackIcon />,
      enabled: selected,
      run: () => void editSelection({ type: "arrange", to: "back" }),
    },
    {
      id: "art.line",
      label: "Work on line art",
      group: "Art",
      enabled: true,
      run: () => setArt("line"),
    },
    {
      id: "art.colour",
      label: "Work on colour art",
      group: "Art",
      enabled: true,
      run: () => setArt("colour"),
    },
    {
      id: "art.toggle",
      label: "Switch line and colour art",
      group: "Art",
      shortcut: "L",
      enabled: true,
      run: toggleArt,
    },
    {
      id: "zoom-in",
      label: "Zoom in",
      group: "View",
      shortcut: "+",
      enabled: true,
      run: () => void changeView({ type: "zoom", factor: 1.25 }),
    },
    {
      id: "zoom-out",
      label: "Zoom out",
      group: "View",
      shortcut: "-",
      enabled: true,
      run: () => void changeView({ type: "zoom", factor: 0.8 }),
    },
    {
      id: "fit",
      label: "Fit scene in view",
      group: "View",
      shortcut: "0",
      enabled: true,
      run: () => void changeView({ type: "fit" }),
    },
    {
      id: "view-rotate-left",
      label: "Rotate view left",
      group: "View",
      shortcut: ",",
      enabled: true,
      run: () => void changeView({ type: "rotate", degrees: 15 }),
    },
    {
      id: "view-rotate-right",
      label: "Rotate view right",
      group: "View",
      shortcut: ".",
      enabled: true,
      run: () => void changeView({ type: "rotate", degrees: -15 }),
    },
    {
      id: "timeline",
      label: timelineOpen ? "Hide timeline" : "Show timeline",
      group: "View",
      enabled: true,
      run: () => setTimelineOpen(!timelineOpen),
    },
    {
      id: "theme",
      label: theme === "dark" ? "Use light theme" : "Use dark theme",
      group: "View",
      enabled: true,
      run: toggleTheme,
    },
    {
      id: "new-layer",
      label: "New layer",
      group: "Layer",
      enabled: hasDoc,
      run: () => void addLayer(),
    },
    {
      id: "panel",
      label: sidePanelOpen ? "Collapse side panel" : "Expand side panel",
      group: "View",
      enabled: true,
      run: () => setSidePanelOpen(!sidePanelOpen),
    },
  ];
}
