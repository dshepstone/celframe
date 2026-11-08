import type { ReactNode } from "react";
import { ColourPanel } from "../studio/ColourPanel";
import { LayersPanel } from "../studio/LayersPanel";
import { Timeline } from "../studio/Timeline";
import { ToolProperties } from "../studio/ToolProperties";
import type { PanelId } from "./layout";

/** Every panel that can sit in a dock tab, with its tab name. */
export const PANEL_INFO: Record<
  PanelId,
  { title: string; render: () => ReactNode }
> = {
  tool: { title: "Tool properties", render: () => <ToolProperties /> },
  colour: { title: "Colour", render: () => <ColourPanel /> },
  layers: { title: "Layers", render: () => <LayersPanel /> },
  timeline: { title: "Timeline", render: () => <Timeline /> },
};
