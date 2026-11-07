import { Button } from "@celframe/ui";
import type { ShapeKind } from "../backend";
import {
  BRUSH_WIDTH_MAX,
  BRUSH_WIDTH_MIN,
  POLYGON_SIDES_MAX,
  POLYGON_SIDES_MIN,
  useApp,
} from "../store";
import { rgbaToHex } from "./colour";

const field =
  "w-16 rounded border border-control-border bg-surface px-1.5 py-0.5 text-text";

/** A size slider and number field, in scene units. */
function SizeControl({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="flex items-center gap-2">
      <span className="text-text-muted">Size</span>
      <input
        type="range"
        min={BRUSH_WIDTH_MIN}
        max={60}
        step={0.5}
        value={Math.min(value, 60)}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-36 accent-[var(--color-accent)]"
        aria-label={`${label} slider`}
      />
      <input
        type="number"
        min={BRUSH_WIDTH_MIN}
        max={BRUSH_WIDTH_MAX}
        step={0.5}
        value={value}
        onChange={(e) => {
          const next = Number(e.target.value);
          if (Number.isFinite(next) && next > 0) onChange(next);
        }}
        className={field}
        aria-label={label}
      />
    </label>
  );
}

const SHAPES: { kind: ShapeKind; label: string }[] = [
  { kind: "line", label: "Line" },
  { kind: "rectangle", label: "Rectangle" },
  { kind: "ellipse", label: "Ellipse" },
  { kind: "polygon", label: "Polygon" },
];

const segment = (on: boolean) =>
  `px-2 py-0.5 outline-none focus-visible:outline-2 focus-visible:outline-focus-ring ${
    on ? "bg-accent text-text-on-accent" : "text-text hover:bg-surface-sunken"
  }`;

/** Picks the shape the Shape tool draws, and a polygon's sides. */
function ShapePicker() {
  const { shapeKind, setShapeKind, polygonSides, setPolygonSides } = useApp();
  return (
    <>
      <span
        role="radiogroup"
        aria-label="Shape"
        className="flex items-center rounded border border-control-border"
      >
        {SHAPES.map(({ kind, label }) => (
          <button
            key={kind}
            type="button"
            role="radio"
            aria-checked={shapeKind === kind}
            onClick={() => setShapeKind(kind)}
            className={segment(shapeKind === kind)}
          >
            {label}
          </button>
        ))}
      </span>
      {shapeKind === "polygon" ? (
        <label className="flex items-center gap-2">
          <span className="text-text-muted">Sides</span>
          <input
            type="number"
            min={POLYGON_SIDES_MIN}
            max={POLYGON_SIDES_MAX}
            step={1}
            value={polygonSides}
            onChange={(e) => {
              const next = Number(e.target.value);
              if (Number.isFinite(next) && next > 0) setPolygonSides(next);
            }}
            className={field}
            aria-label="Sides"
          />
        </label>
      ) : null}
    </>
  );
}

/** Chooses whether tools work on line art or colour art. */
function ArtSwitch() {
  const { art, setArt } = useApp();
  return (
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
          className={segment(art === part)}
        >
          {part === "line" ? "Line art" : "Colour art"}
        </button>
      ))}
    </span>
  );
}

/** The bar above the canvas: the current tool's settings, plus undo and redo. */
export function ToolOptions() {
  const {
    document: doc,
    tool,
    brushWidth,
    brushColor,
    setBrushWidth,
    eraserWidth,
    setEraserWidth,
    eraseWholeStrokes,
    setEraseWholeStrokes,
    undo,
    redo,
  } = useApp();
  const erasing = tool === "eraser";

  return (
    <div
      role="toolbar"
      aria-label="Tool options"
      className="flex h-10 items-center gap-4 border-b border-border bg-surface-raised px-3 text-sm"
    >
      {tool === "select" ? (
        <>
          <span className="font-medium">Select</span>
          <span className="text-text-muted">
            Drag to move. Shift-click to add. Arrow keys nudge.
          </span>
        </>
      ) : erasing ? (
        <>
          <span className="font-medium">Eraser</span>
          <SizeControl
            label="Eraser size"
            value={eraserWidth}
            onChange={setEraserWidth}
          />
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={eraseWholeStrokes}
              onChange={(e) => setEraseWholeStrokes(e.target.checked)}
              className="accent-[var(--color-accent)]"
            />
            <span>Whole strokes</span>
          </label>
        </>
      ) : (
        <>
          <span className="font-medium">
            {tool === "shape" ? "Shape" : "Pen"}
          </span>
          {tool === "shape" ? <ShapePicker /> : null}
          <SizeControl
            label="Brush size"
            value={brushWidth}
            onChange={setBrushWidth}
          />
          <span className="flex items-center gap-2">
            <span className="text-text-muted">Colour</span>
            <span
              className="h-5 w-5 rounded border border-control-border"
              style={{ background: rgbaToHex(brushColor) }}
              aria-label={`Current colour ${rgbaToHex(brushColor)}`}
              role="img"
            />
          </span>
        </>
      )}
      <ArtSwitch />
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
