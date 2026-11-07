import type { ReactNode } from "react";
import { useApp, type Tool as ToolId } from "../store";
import {
  EraserIcon,
  FillIcon,
  HandIcon,
  PenIcon,
  SelectIcon,
  ShapeIcon,
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
  { id: "select", label: "Select", shortcut: "V", icon: <SelectIcon /> },
  { id: "pen", label: "Pen", shortcut: "P", icon: <PenIcon /> },
  { id: "eraser", label: "Eraser", shortcut: "E", icon: <EraserIcon /> },
  { id: "shape", label: "Shape", shortcut: "U", icon: <ShapeIcon /> },
  { id: "fill", label: "Fill", icon: <FillIcon />, later: "Phase 2" },
  { id: "hand", label: "Pan", shortcut: "H", icon: <HandIcon /> },
  { id: "zoom", label: "Zoom", shortcut: "Z", icon: <ZoomIcon /> },
];

/** Where the rail draws a divider: after Select and after the drawing tools. */
const GROUP_ENDS = new Set(["fill", "select"]);

/** The vertical tool strip beside the canvas. See docs/specs/visual-design.md. */
export function ToolRail() {
  const { tool: current, setTool } = useApp();
  return (
    <nav
      aria-label="Tools"
      className="flex w-10 shrink-0 flex-col items-center gap-0.5 border-r border-border bg-surface-sunken py-1.5"
    >
      {TOOLS.map((tool) => (
        <div key={tool.id} className="contents">
          <button
            type="button"
            aria-label={
              tool.later
                ? `${tool.label} (coming in ${tool.later})`
                : tool.label
            }
            aria-pressed={tool.id === current}
            onClick={() => setTool(tool.id as ToolId)}
            disabled={tool.later !== undefined}
            title={
              tool.later
                ? `${tool.label}: coming in ${tool.later}`
                : `${tool.label} (${tool.shortcut})`
            }
            className="relative flex h-8 w-8 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:cursor-not-allowed disabled:opacity-35 disabled:hover:bg-transparent aria-pressed:bg-accent-muted aria-pressed:text-text aria-pressed:before:absolute aria-pressed:before:top-1.5 aria-pressed:before:bottom-1.5 aria-pressed:before:-left-1 aria-pressed:before:w-0.5 aria-pressed:before:rounded-full aria-pressed:before:bg-accent"
          >
            {tool.icon}
          </button>
          {GROUP_ENDS.has(tool.id) ? (
            <span aria-hidden="true" className="my-1 h-px w-5 bg-border" />
          ) : null}
        </div>
      ))}
    </nav>
  );
}
