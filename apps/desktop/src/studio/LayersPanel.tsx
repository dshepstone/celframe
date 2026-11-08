import { useState } from "react";
import { targetLayer, useApp } from "../store";
import { OpacityControl } from "./OpacityControl";
import { Panel, panelIconButton } from "./Panel";
import {
  EyeIcon,
  EyeOffIcon,
  LockIcon,
  PlusIcon,
  TrashIcon,
  UnlockIcon,
} from "./icons";

const iconButton =
  "flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring aria-[pressed=false]:opacity-60";

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
    <Panel
      id="layers"
      title="Layers"
      grow
      actions={
        <>
          <button
            type="button"
            className={panelIconButton}
            aria-label="Add layer"
            title="Add layer"
            onClick={() => void addLayer()}
          >
            <PlusIcon />
          </button>
          <button
            type="button"
            className={panelIconButton}
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
        </>
      }
    >
      {target ? <OpacityControl scene={scene.id} layer={target} /> : null}
      <ul role="listbox" aria-label="Layer list" className="py-1">
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
              className={`group relative flex h-[26px] items-center gap-0.5 px-1.5 text-sm outline-none focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus-ring ${
                selected
                  ? "bg-accent-muted text-text before:absolute before:inset-y-0 before:left-0 before:w-0.5 before:bg-accent"
                  : "text-text hover:bg-surface-hover"
              }`}
            >
              <button
                type="button"
                className={iconButton}
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
                className={iconButton}
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
                  className="ml-1 h-5 min-w-0 flex-1 rounded border border-accent bg-surface-sunken px-1 text-text outline-none"
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
                <span className="ml-1 min-w-0 flex-1 truncate">
                  {layer.name}
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}
