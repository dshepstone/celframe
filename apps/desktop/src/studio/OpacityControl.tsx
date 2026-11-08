import { useEffect, useState } from "react";
import { valueAt } from "../animated";
import type { Interpolation, Layer } from "../backend";
import { useApp } from "../store";

const keyButton =
  "flex h-5 w-5 items-center justify-center rounded text-sm leading-none outline-none hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-focus-ring";

/**
 * The current layer's opacity at the current frame, with buttons to key it. With keys, a new
 * value sets a key here; without, it changes the base value. See docs/specs/keyframes.md.
 */
export function OpacityControl({
  scene,
  layer,
}: {
  scene: string;
  layer: Layer;
}) {
  const { document: doc, runCommand } = useApp();
  const frame = doc?.currentFrame ?? 0;
  const shown = Math.round(valueAt(layer.opacity, frame) * 100);
  const [draft, setDraft] = useState(shown);
  useEffect(() => setDraft(shown), [shown]);
  const key = layer.opacity.keys.find((k) => k.frame === frame);
  const keyed = layer.opacity.keys.length > 0;

  const setKey = (percent: number, interpolation: Interpolation) =>
    void runCommand({
      type: "setKey",
      scene,
      layer: layer.id,
      property: "opacity",
      key: { frame, value: percent / 100, interpolation },
    });
  /** Applies a new value: once, when a drag or typing ends. */
  const commit = (percent: number) => {
    const value = Math.min(100, Math.max(0, Math.round(percent)));
    if (value === shown) return;
    if (keyed) setKey(value, key?.interpolation ?? "ease");
    else
      void runCommand({
        type: "setBaseValue",
        scene,
        layer: layer.id,
        property: "opacity",
        value: value / 100,
      });
  };

  return (
    <div
      role="group"
      aria-label={`${layer.name} opacity`}
      className="flex items-center gap-1.5 border-b border-border px-2.5 py-1.5 text-xs text-text-muted"
    >
      <span className="w-12 shrink-0">Opacity</span>
      <input
        type="range"
        aria-label="Opacity"
        min={0}
        max={100}
        value={draft}
        onChange={(e) => setDraft(Number(e.target.value))}
        onPointerUp={() => commit(draft)}
        onKeyUp={() => commit(draft)}
        className="min-w-0 flex-1 accent-[var(--color-accent)]"
      />
      <input
        type="number"
        aria-label="Opacity percent"
        min={0}
        max={100}
        value={draft}
        onChange={(e) => setDraft(Number(e.target.value))}
        onBlur={() => commit(draft)}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit(draft);
        }}
        className="h-5 w-10 rounded border border-control-border bg-surface-sunken px-1 text-right text-xs text-text tabular-nums outline-none focus:border-accent"
      />
      <span className="text-text-muted">%</span>
      {key ? (
        <>
          <button
            type="button"
            aria-label="Remove opacity key"
            title="Remove the key at this frame"
            onClick={() =>
              void runCommand({
                type: "removeKey",
                scene,
                layer: layer.id,
                property: "opacity",
                frame,
              })
            }
            className={`${keyButton} text-accent`}
          >
            ◆
          </button>
          <select
            aria-label="Key interpolation"
            value={key.interpolation}
            onChange={(e) => setKey(shown, e.target.value as Interpolation)}
            className="h-5 rounded border border-control-border bg-surface-sunken px-0.5 text-xs text-text"
          >
            <option value="linear">Linear</option>
            <option value="ease">Ease</option>
            <option value="step">Step</option>
          </select>
        </>
      ) : (
        <button
          type="button"
          aria-label="Set opacity key"
          title="Key the opacity at this frame"
          onClick={() => setKey(shown, "ease")}
          className={`${keyButton} text-text-muted hover:text-text`}
        >
          ◇
        </button>
      )}
    </div>
  );
}
