/** The arrangement of panel tabs in dock areas. See docs/specs/docking.md. */

export const PANELS = [
  "tool",
  "colour",
  "layers",
  "timeline",
  "xsheet",
  "camera",
] as const;
export type PanelId = (typeof PANELS)[number];

export type AreaId = "left" | "right" | "bottom";
export const AREAS: readonly AreaId[] = ["left", "right", "bottom"];

export interface Stack {
  id: string;
  tabs: PanelId[];
  active: PanelId;
  /** Share of the area's length, relative to the other stacks. */
  weight: number;
}

export interface DockLayout {
  /** Raised when default places change, so stored layouts can move panels. */
  version?: number;
  areas: Record<AreaId, Stack[]>;
  /** Side areas' widths and the bottom area's height, in CSS pixels. */
  sizes: Record<AreaId, number>;
}

export const AREA_LIMITS: Record<AreaId, [number, number]> = {
  left: [200, 520],
  right: [200, 520],
  bottom: [100, 600],
};

/** Where a dragged tab lands. */
export type DropTarget =
  /** Into a stack's tab strip, before the tab at `index`. */
  | { kind: "tabs"; area: AreaId; stack: string; index: number }
  /** A new stack beside an existing one. */
  | { kind: "split"; area: AreaId; stack: string; after: boolean }
  /** A new stack in an empty area. */
  | { kind: "area"; area: AreaId };

let counter = 0;
/** A stack ID unique within this session. Stack IDs aren't meaningful beyond that. */
export function newStackId(): string {
  counter += 1;
  return `stack-${Date.now().toString(36)}-${counter}`;
}

function stack(id: string, tabs: PanelId[], weight = 1): Stack {
  return { id, tabs, active: tabs[0] as PanelId, weight };
}

/** The current layout version. Version 2 moved the exposure sheet to the right panels. */
export const LAYOUT_VERSION = 2;

/** Panels whose default place changed in each version, to move in older stored layouts. */
const MOVED: Record<number, PanelId[]> = { 2: ["xsheet"] };

export function defaultLayout(): DockLayout {
  return {
    version: LAYOUT_VERSION,
    areas: {
      left: [],
      right: [
        stack("right-tool", ["tool", "camera"], 1.4),
        stack("right-colour", ["colour"], 0.6),
        stack("right-layers", ["layers", "xsheet"], 1),
      ],
      bottom: [stack("bottom-timeline", ["timeline"])],
    },
    sizes: { left: 260, right: 272, bottom: 160 },
  };
}

export function clampSize(area: AreaId, size: number): number {
  const [min, max] = AREA_LIMITS[area];
  return Math.round(Math.min(max, Math.max(min, size)));
}

/** Where a panel is, if it's in the layout. */
export function findPanel(
  layout: DockLayout,
  panel: PanelId,
): { area: AreaId; stack: Stack } | null {
  for (const area of AREAS) {
    const found = layout.areas[area].find((s) => s.tabs.includes(panel));
    if (found) return { area, stack: found };
  }
  return null;
}

/** Removes empty stacks and makes every active tab one of its stack's tabs. */
function tidy(layout: DockLayout): DockLayout {
  const areas = { ...layout.areas };
  for (const area of AREAS) {
    areas[area] = areas[area]
      .filter((s) => s.tabs.length > 0)
      .map((s) =>
        s.tabs.includes(s.active) ? s : { ...s, active: s.tabs[0] as PanelId },
      );
  }
  return { ...layout, areas };
}

/** Takes a panel out of whichever stack holds it. */
function without(layout: DockLayout, panel: PanelId): DockLayout {
  const areas = { ...layout.areas };
  for (const area of AREAS) {
    areas[area] = areas[area].map((s) =>
      s.tabs.includes(panel)
        ? { ...s, tabs: s.tabs.filter((t) => t !== panel) }
        : s,
    );
  }
  return { ...layout, areas };
}

/** Moves a panel's tab to a drop target and makes it active there. */
export function moveTab(
  layout: DockLayout,
  panel: PanelId,
  target: DropTarget,
): DockLayout {
  const from = findPanel(layout, panel);
  // Dropping a stack's only tab beside or into that same stack changes nothing.
  if (
    from &&
    from.stack.tabs.length === 1 &&
    target.kind !== "area" &&
    target.stack === from.stack.id
  ) {
    return layout;
  }
  let index = target.kind === "tabs" ? target.index : 0;
  if (
    from &&
    target.kind === "tabs" &&
    target.stack === from.stack.id &&
    from.stack.tabs.indexOf(panel) < index
  ) {
    // Taking the tab out first shifts the later positions back by one.
    index -= 1;
  }
  const next = without(layout, panel);
  const areas = { ...next.areas };
  const list = [...areas[target.area]];
  if (target.kind === "tabs") {
    const at = list.findIndex((s) => s.id === target.stack);
    const into = list[at];
    if (!into) return layout;
    const tabs = [...into.tabs];
    tabs.splice(Math.min(Math.max(index, 0), tabs.length), 0, panel);
    list[at] = { ...into, tabs, active: panel };
  } else if (target.kind === "split") {
    const at = list.findIndex((s) => s.id === target.stack);
    const beside = list[at];
    if (!beside) return layout;
    // The new stack takes half of the one it splits.
    const weight = beside.weight / 2;
    list[at] = { ...beside, weight };
    list.splice(
      target.after ? at + 1 : at,
      0,
      stack(newStackId(), [panel], weight),
    );
  } else {
    list.push(stack(newStackId(), [panel]));
  }
  areas[target.area] = list;
  return tidy({ ...next, areas });
}

export function activate(layout: DockLayout, panel: PanelId): DockLayout {
  const areas = { ...layout.areas };
  for (const area of AREAS) {
    areas[area] = areas[area].map((s) =>
      s.tabs.includes(panel) ? { ...s, active: panel } : s,
    );
  }
  return { ...layout, areas };
}

/** Shows a panel: activates it, or opens it in the right area if it isn't in the layout. */
export function showPanel(layout: DockLayout, panel: PanelId): DockLayout {
  if (findPanel(layout, panel)) return activate(layout, panel);
  return moveTab(layout, panel, { kind: "area", area: "right" });
}

/** Moves the divider after stack `index` in an area, by `delta` pixels of `length`. */
export function resizeStacks(
  layout: DockLayout,
  area: AreaId,
  index: number,
  delta: number,
  length: number,
  minimum = 80,
): DockLayout {
  const list = layout.areas[area];
  const a = list[index];
  const b = list[index + 1];
  if (!a || !b || length <= 0) return layout;
  const total = list.reduce((sum, s) => sum + s.weight, 0);
  const perPixel = total / length;
  const pair = a.weight + b.weight;
  const least = Math.min(minimum * perPixel, pair / 2);
  const first = Math.min(
    pair - least,
    Math.max(least, a.weight + delta * perPixel),
  );
  const areas = { ...layout.areas };
  areas[area] = list.map((s, i) =>
    i === index
      ? { ...s, weight: first }
      : i === index + 1
        ? { ...s, weight: pair - first }
        : s,
  );
  return { ...layout, areas };
}

/**
 * Adds panels a stored layout doesn't know about, such as ones added in a newer version, next
 * to a panel they share a stack with by default, or in their default area.
 */
function withNewPanels(
  layout: DockLayout,
  present: ReadonlySet<PanelId>,
): DockLayout {
  let result = layout;
  const defaults = defaultLayout();
  for (const panel of PANELS) {
    if (present.has(panel)) continue;
    const home = findPanel(defaults, panel);
    if (!home) continue;
    const neighbour = home.stack.tabs.find(
      (t) => t !== panel && findPanel(result, t),
    );
    const there = neighbour ? findPanel(result, neighbour) : null;
    const areas = { ...result.areas };
    if (there) {
      areas[there.area] = areas[there.area].map((s) =>
        s.id === there.stack.id ? { ...s, tabs: [...s.tabs, panel] } : s,
      );
    } else {
      areas[home.area] = [...areas[home.area], stack(newStackId(), [panel])];
    }
    result = { ...result, areas };
  }
  return result;
}

/** Reads a stored layout, falling back to the default when it isn't usable. */
export function parseLayout(stored: string | null): DockLayout {
  const fallback = defaultLayout();
  if (stored === null) return fallback;
  try {
    const value = JSON.parse(stored) as Partial<DockLayout>;
    const seen = new Set<PanelId>();
    // Panels that moved since this layout was stored go to their new default places.
    const storedVersion = Number(value.version) || 1;
    const moved = new Set<PanelId>(
      Object.entries(MOVED)
        .filter(([version]) => Number(version) > storedVersion)
        .flatMap(([, panels]) => panels),
    );
    const areas = {} as Record<AreaId, Stack[]>;
    for (const area of AREAS) {
      const raw: unknown = value.areas?.[area];
      areas[area] = (Array.isArray(raw) ? raw : []).flatMap((item): Stack[] => {
        if (typeof item !== "object" || item === null) return [];
        const s = item as Partial<Stack>;
        // Each panel appears once in the whole layout.
        const tabs: PanelId[] = [];
        for (const t of Array.isArray(s.tabs) ? s.tabs : []) {
          if (
            (PANELS as readonly string[]).includes(t) &&
            !seen.has(t) &&
            !moved.has(t)
          ) {
            seen.add(t);
            tabs.push(t);
          }
        }
        if (tabs.length === 0 || typeof s.id !== "string") return [];
        const weight = Number(s.weight);
        return [
          {
            id: s.id,
            tabs,
            active: tabs.includes(s.active as PanelId)
              ? (s.active as PanelId)
              : (tabs[0] as PanelId),
            weight: Number.isFinite(weight) && weight > 0 ? weight : 1,
          },
        ];
      });
    }
    if (seen.size === 0) return fallback;
    const sizes = { ...fallback.sizes };
    for (const area of AREAS) {
      const size = Number(value.sizes?.[area]);
      if (Number.isFinite(size)) sizes[area] = clampSize(area, size);
    }
    return withNewPanels({ version: LAYOUT_VERSION, areas, sizes }, seen);
  } catch {
    return fallback;
  }
}
