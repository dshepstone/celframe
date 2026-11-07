/** Shelves of action buttons under the menus. See docs/specs/shelves.md. */

export interface Shelf {
  id: string;
  name: string;
  /** Action IDs, in button order. */
  actions: string[];
}

export const SHELVES_KEY = "celframe.shelves";

export const DEFAULT_SHELVES: Shelf[] = [
  {
    id: "quick",
    name: "Quick",
    actions: [
      "undo",
      "redo",
      "flip-horizontal",
      "flip-vertical",
      "rotate-left",
      "rotate-right",
      "to-front",
      "to-back",
    ],
  },
  {
    id: "shapes",
    name: "Shapes",
    actions: [
      "shape.line",
      "shape.rectangle",
      "shape.ellipse",
      "shape.polygon",
    ],
  },
];

/**
 * Reads stored shelves, keeping only well-formed ones and actions in `known`. Falls back to
 * the defaults when nothing usable is stored.
 */
export function parseShelves(
  stored: string | null,
  known: ReadonlySet<string>,
): Shelf[] {
  if (stored === null) return DEFAULT_SHELVES;
  try {
    const value: unknown = JSON.parse(stored);
    if (!Array.isArray(value)) return DEFAULT_SHELVES;
    const shelves = value.flatMap((item: unknown): Shelf[] => {
      if (typeof item !== "object" || item === null) return [];
      const { id, name, actions } = item as Record<string, unknown>;
      if (typeof id !== "string" || typeof name !== "string") return [];
      if (!Array.isArray(actions)) return [];
      const kept = actions.filter(
        (a): a is string => typeof a === "string" && known.has(a),
      );
      return [{ id, name, actions: [...new Set(kept)] }];
    });
    return shelves.length > 0 ? shelves : DEFAULT_SHELVES;
  } catch {
    return DEFAULT_SHELVES;
  }
}

export function loadShelves(known: ReadonlySet<string>): Shelf[] {
  try {
    return parseShelves(localStorage.getItem(SHELVES_KEY), known);
  } catch {
    return DEFAULT_SHELVES;
  }
}

export function storeShelves(shelves: Shelf[]): void {
  try {
    localStorage.setItem(SHELVES_KEY, JSON.stringify(shelves));
  } catch {
    // Storage can be unavailable, for example in a private window. Shelves still work.
  }
}

/** A name like "Shelf 3" that no shelf uses yet. */
export function nextShelfName(shelves: Shelf[]): string {
  const names = new Set(shelves.map((s) => s.name));
  let n = shelves.length + 1;
  while (names.has(`Shelf ${n}`)) n += 1;
  return `Shelf ${n}`;
}

/** Moves the button at `index` by `by` places, within the shelf. */
export function moveAction(
  actions: string[],
  index: number,
  by: number,
): string[] {
  const to = index + by;
  if (index < 0 || index >= actions.length || to < 0 || to >= actions.length) {
    return actions;
  }
  const next = [...actions];
  const [moved] = next.splice(index, 1);
  if (moved === undefined) return actions;
  next.splice(to, 0, moved);
  return next;
}
