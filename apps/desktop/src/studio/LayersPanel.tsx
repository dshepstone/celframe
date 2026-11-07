import { useState } from "react";
import { targetLayer, useApp } from "../store";
import {
  EyeIcon,
  EyeOffIcon,
  LockIcon,
  PlusIcon,
  TrashIcon,
  UnlockIcon,
} from "./icons";

const iconButton =
  "flex h-7 w-7 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-sunken hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-40";

/** The first scene's layers, top layer first. Click a layer to draw on it. */
export function LayersPanel() {
  const {
    document: doc,
    activeLayer,
    setActiveLayer,
    runCommand,
    addLayer,
  } = useApp();
  const [renaming, setRenaming] = useState<string | null>(null);
  const scene = doc?.project.scenes[0];
  if (!scene) return null;
  const layers = [...scene.layers].reverse();
  const target = targetLayer(scene.layers, activeLayer);

  return (
    <section aria-label="Layers" className="flex min-h-0 flex-col">
      <header className="flex items-center justify-between px-3 py-2">
        <h2 className="text-xs font-semibold tracking-wide text-text-muted uppercase">
          Layers
        </h2>
        <span className="flex gap-0.5">
          <button
            type="button"
            className={iconButton}
            aria-label="Add layer"
            title="Add layer"
            onClick={() => void addLayer()}
          >
            <PlusIcon />
          </button>
          <button
            type="button"
            className={iconButton}
            aria-label="Delete layer"
            title="Delete layer"
            disabled={!target || scene.layers.length <= 1}
            onClick={() =>
              target &&
              void runCommand({
                type: "removeLayer",
                scene: scene.id,
                layer: target.id,
              })
            }
          >
            <TrashIcon />
          </button>
        </span>
      </header>
      <ul
        role="listbox"
        aria-label="Layer list"
        className="overflow-auto px-1.5 pb-2"
      >
        {layers.map((layer) => {
          const selected = layer.id === target?.id;
          return (
            <li
              key={layer.id}
              role="option"
              aria-selected={selected}
              tabIndex={0}
              onClick={() => setActiveLayer(layer.id)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  setActiveLayer(layer.id);
                } else if (e.key === "F2") {
                  setRenaming(layer.id);
                }
              }}
              onDoubleClick={() => setRenaming(layer.id)}
              className={`group flex items-center gap-1 rounded-md px-1 py-0.5 text-sm outline-none focus-visible:outline-2 focus-visible:outline-focus-ring ${
                selected
                  ? "bg-accent text-text-on-accent"
                  : "hover:bg-surface-sunken"
              }`}
            >
              <button
                type="button"
                className={`${iconButton} ${selected ? "text-text-on-accent" : ""}`}
                aria-label={`${layer.visible ? "Hide" : "Show"} ${layer.name}`}
                aria-pressed={layer.visible}
                onClick={(e) => {
                  e.stopPropagation();
                  void runCommand({
                    type: "setLayerVisible",
                    scene: scene.id,
                    layer: layer.id,
                    visible: !layer.visible,
                  });
                }}
              >
                {layer.visible ? <EyeIcon /> : <EyeOffIcon />}
              </button>
              <button
                type="button"
                className={`${iconButton} ${selected ? "text-text-on-accent" : ""}`}
                aria-label={`${layer.locked ? "Unlock" : "Lock"} ${layer.name}`}
                aria-pressed={layer.locked}
                onClick={(e) => {
                  e.stopPropagation();
                  void runCommand({
                    type: "setLayerLocked",
                    scene: scene.id,
                    layer: layer.id,
                    locked: !layer.locked,
                  });
                }}
              >
                {layer.locked ? <LockIcon /> : <UnlockIcon />}
              </button>
              {renaming === layer.id ? (
                <input
                  autoFocus
                  defaultValue={layer.name}
                  aria-label={`Rename ${layer.name}`}
                  className="min-w-0 flex-1 rounded border border-control-border bg-surface px-1 text-text"
                  onClick={(e) => e.stopPropagation()}
                  onBlur={() => setRenaming(null)}
                  onKeyDown={(e) => {
                    e.stopPropagation();
                    if (e.key === "Escape") setRenaming(null);
                    if (e.key !== "Enter") return;
                    const name = e.currentTarget.value.trim();
                    setRenaming(null);
                    if (name && name !== layer.name) {
                      void runCommand({
                        type: "renameLayer",
                        scene: scene.id,
                        layer: layer.id,
                        name,
                      });
                    }
                  }}
                />
              ) : (
                <span className="min-w-0 flex-1 truncate">{layer.name}</span>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
