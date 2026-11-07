import type { ReactNode } from "react";
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
  icon: ReactNode;
  /** Why the tool isn't available yet, or undefined when it is. */
  later?: string;
}

export const TOOLS: Tool[] = [
  { id: "pen", label: "Pen", icon: <PenIcon /> },
  { id: "eraser", label: "Eraser", icon: <EraserIcon />, later: "Phase 2" },
  { id: "fill", label: "Fill", icon: <FillIcon />, later: "Phase 2" },
  { id: "select", label: "Select", icon: <SelectIcon />, later: "Phase 2" },
  { id: "hand", label: "Pan", icon: <HandIcon />, later: "issue #22" },
  { id: "zoom", label: "Zoom", icon: <ZoomIcon />, later: "issue #22" },
];

/** The vertical tool strip beside the canvas. Only the pen works in Phase 1. */
export function ToolRail() {
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
          aria-pressed={tool.id === "pen"}
          disabled={tool.later !== undefined}
          title={
            tool.later ? `${tool.label}: coming in ${tool.later}` : tool.label
          }
          className="flex h-8 w-8 items-center justify-center rounded-md text-text-muted outline-none hover:bg-surface-sunken hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:cursor-not-allowed disabled:opacity-40 aria-pressed:bg-accent aria-pressed:text-text-on-accent"
        >
          {tool.icon}
        </button>
      ))}
    </nav>
  );
}
