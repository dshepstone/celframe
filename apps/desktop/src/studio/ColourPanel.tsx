import { useApp } from "../store";
import { SWATCHES, hexToRgbaColour, rgbaToHex } from "./colour";

/** Swatches and a custom picker for the pen colour. */
export function ColourPanel() {
  const { brushColor, setBrushColor } = useApp();
  const current = rgbaToHex(brushColor);

  return (
    <section aria-label="Colour" className="border-t border-border px-3 py-2">
      <h2 className="mb-2 text-xs font-semibold tracking-wide text-text-muted uppercase">
        Colour
      </h2>
      <div
        role="radiogroup"
        aria-label="Swatches"
        className="grid grid-cols-6 gap-1.5"
      >
        {SWATCHES.map((swatch) => (
          <button
            key={swatch.hex}
            type="button"
            role="radio"
            aria-checked={current === swatch.hex}
            aria-label={swatch.name}
            title={swatch.name}
            onClick={() => setBrushColor(hexToRgbaColour(swatch.hex))}
            className="h-7 rounded border border-control-border outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus-ring aria-checked:ring-2 aria-checked:ring-focus-ring aria-checked:ring-offset-1 aria-checked:ring-offset-surface-raised"
            style={{ background: swatch.hex }}
          />
        ))}
      </div>
      <label className="mt-2 flex items-center gap-2 text-sm">
        <span className="text-text-muted">Custom</span>
        <input
          type="color"
          value={current}
          onChange={(e) => setBrushColor(hexToRgbaColour(e.target.value))}
          className="h-7 w-12 cursor-pointer rounded border border-control-border bg-surface"
          aria-label="Custom colour"
        />
        <span className="font-mono text-xs text-text-muted">{current}</span>
      </label>
    </section>
  );
}
