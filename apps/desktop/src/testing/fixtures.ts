import type { DocumentView, Layer, Scene } from "../backend";

export function layer(id: string, overrides: Partial<Layer> = {}): Layer {
  return {
    id,
    name: id,
    visible: true,
    locked: false,
    drawings: [],
    exposures: [],
    ...overrides,
  };
}

export function scene(overrides: Partial<Scene> = {}): Scene {
  return {
    id: "s",
    name: "Shot 1",
    width: 1920,
    height: 1080,
    frameRate: 24,
    layers: [layer("l1", { name: "Layer 1" })],
    ...overrides,
  };
}

export function documentView(
  overrides: Partial<DocumentView> = {},
): DocumentView {
  return {
    project: { id: "p", name: "Bouncing ball", scenes: [scene()] },
    canUndo: false,
    canRedo: false,
    dirty: false,
    path: null,
    selection: [],
    selectionBounds: null,
    ...overrides,
  };
}
