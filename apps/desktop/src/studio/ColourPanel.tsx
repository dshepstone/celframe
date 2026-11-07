import { useApp } from "../store";
import { Panel } from "./Panel";
import { SWATCHES, hexToRgbaColour, rgbaToHex } from "./colour";

/** Swatches and a custom picker for the pen colour. */
export function ColourPanel() {
  const { brushColor, setBrushColor } = useApp();
  const current = rgbaToHex(brushColor);

  return (
    <Panel id="colour" title="Colour" detail={current}>
      <div className="px-2.5 pt-2 pb-2.5">
        <div
          role="radiogroup"
          aria-label="Swatches"
          className="grid grid-cols-6 gap-1"
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
              className="h-5 rounded-sm border border-black/30 outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus-ring aria-checked:ring-2 aria-checked:ring-accent aria-checked:ring-offset-1 aria-checked:ring-offset-surface"
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
            className="h-6 w-10 rounded border border-control-border bg-surface-sunken"
            aria-label="Custom colour"
          />
          <span className="font-mono text-xs text-text-muted">{current}</span>
        </label>
      </div>
    </Panel>
  );
}
