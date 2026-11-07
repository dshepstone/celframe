import { Button } from "@celframe/ui";
import { BRUSH_WIDTH_MAX, BRUSH_WIDTH_MIN, useApp } from "../store";
import { rgbaToHex } from "./colour";

/** The bar above the canvas: the current tool's settings, plus undo and redo. */
export function ToolOptions() {
  const {
    document: doc,
    brushWidth,
    brushColor,
    setBrushWidth,
    art,
    setArt,
    undo,
    redo,
  } = useApp();

  return (
    <div
      role="toolbar"
      aria-label="Tool options"
      className="flex h-10 items-center gap-4 border-b border-border bg-surface-raised px-3 text-sm"
    >
      <span className="font-medium">Pen</span>
      <label className="flex items-center gap-2">
        <span className="text-text-muted">Size</span>
        <input
          type="range"
          min={BRUSH_WIDTH_MIN}
          max={60}
          step={0.5}
          value={Math.min(brushWidth, 60)}
          onChange={(e) => setBrushWidth(Number(e.target.value))}
          className="w-36 accent-[var(--color-accent)]"
          aria-label="Brush size slider"
        />
        <input
          type="number"
          min={BRUSH_WIDTH_MIN}
          max={BRUSH_WIDTH_MAX}
          step={0.5}
          value={brushWidth}
          onChange={(e) => {
            const value = Number(e.target.value);
            if (Number.isFinite(value) && value > 0) setBrushWidth(value);
          }}
          className="w-16 rounded border border-control-border bg-surface px-1.5 py-0.5 text-text"
          aria-label="Brush size"
        />
      </label>
      <span className="flex items-center gap-2">
        <span className="text-text-muted">Colour</span>
        <span
          className="h-5 w-5 rounded border border-control-border"
          style={{ background: rgbaToHex(brushColor) }}
          aria-label={`Current colour ${rgbaToHex(brushColor)}`}
          role="img"
        />
      </span>
      <span
        role="radiogroup"
        aria-label="Draw on"
        className="flex items-center rounded border border-control-border"
        title="Switch with L"
      >
        {(["line", "colour"] as const).map((part) => (
          <button
            key={part}
            type="button"
            role="radio"
            aria-checked={art === part}
            onClick={() => setArt(part)}
            className={`px-2 py-0.5 outline-none focus-visible:outline-2 focus-visible:outline-focus-ring ${
              art === part
                ? "bg-accent text-text-on-accent"
                : "text-text hover:bg-surface-sunken"
            }`}
          >
            {part === "line" ? "Line art" : "Colour art"}
          </button>
        ))}
      </span>
      <div className="flex-1" />
      <Button disabled={!doc?.canUndo} onClick={() => void undo()}>
        Undo
      </Button>
      <Button disabled={!doc?.canRedo} onClick={() => void redo()}>
        Redo
      </Button>
    </div>
  );
}
