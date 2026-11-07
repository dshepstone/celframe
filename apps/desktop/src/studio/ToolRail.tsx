import type { ReactNode } from "react";
import { useApp, type Tool as ToolId } from "../store";
import {
  EraserIcon,
  FillIcon,
  HandIcon,
  PenIcon,
  SelectIcon,
  ZoomIcon,
} from "./icons";

interface Tool {
  id: string;
  label: string;
  shortcut?: string;
  icon: ReactNode;
  /** Why the tool isn't available yet, or undefined when it is. */
  later?: string;
}

export const TOOLS: Tool[] = [
  { id: "pen", label: "Pen", shortcut: "P", icon: <PenIcon /> },
  { id: "eraser", label: "Eraser", shortcut: "E", icon: <EraserIcon /> },
  { id: "fill", label: "Fill", icon: <FillIcon />, later: "Phase 2" },
  { id: "select", label: "Select", icon: <SelectIcon />, later: "Phase 2" },
  { id: "hand", label: "Pan", shortcut: "H", icon: <HandIcon /> },
  { id: "zoom", label: "Zoom", shortcut: "Z", icon: <ZoomIcon /> },
];

/** The vertical tool strip beside the canvas. */
export function ToolRail() {
  const { tool: current, setTool } = useApp();
  return (
    <nav
      aria-label="Tools"
      className="flex w-11 flex-col items-center gap-1 border-r border-border bg-surface py-2"
    >
      {TOOLS.map((tool) => (
        <button
          key={tool.id}
          type="button"
          aria-label={
            tool.later ? `${tool.label} (coming in ${tool.later})` : tool.label
          }
          aria-pressed={tool.id === current}
          onClick={() => setTool(tool.id as ToolId)}
          disabled={tool.later !== undefined}
          title={
            tool.later
              ? `${tool.label}: coming in ${tool.later}`
              : `${tool.label} (${tool.shortcut})`
          }
          className="flex h-8 w-8 items-center justify-center rounded-md text-text-muted outline-none hover:bg-surface-sunken hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:cursor-not-allowed disabled:opacity-40 aria-pressed:bg-accent aria-pressed:text-text-on-accent"
        >
          {tool.icon}
        </button>
      ))}
    </nav>
  );
}
