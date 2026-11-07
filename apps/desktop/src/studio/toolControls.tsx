import type { ReactNode } from "react";
import type { ShapeKind } from "../backend";
import {
  BRUSH_WIDTH_MAX,
  BRUSH_WIDTH_MIN,
  POLYGON_SIDES_MAX,
  POLYGON_SIDES_MIN,
  useApp,
} from "../store";

export const heading = "mb-1 text-xs text-text-muted";

const field =
  "h-6 w-14 rounded border border-control-border bg-surface-sunken px-1.5 text-right text-sm text-text tabular-nums outline-none focus:border-accent";

const segment = (on: boolean) =>
  `h-6 flex-1 px-2 text-sm outline-none focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus-ring ${
    on
      ? "bg-accent-muted text-text shadow-[inset_0_-2px_0_var(--color-accent)]"
      : "text-text-muted hover:bg-surface-hover hover:text-text"
  }`;

/** A titled group of controls in the Tool properties panel. */
export function Group({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div role="group" aria-label={label} className="mb-2.5 last:mb-0">
      <h3 className={heading}>{label}</h3>
      {children}
    </div>
  );
}

/** A size slider and number field, in scene units. */
export function SizeControl({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <input
        type="range"
        min={BRUSH_WIDTH_MIN}
        max={60}
        step={0.5}
        value={Math.min(value, 60)}
        onChange={(e) => onChange(Number(e.target.value))}
        className="min-w-0 flex-1 accent-[var(--color-accent)]"
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
    </div>
  );
}

/** Two or more mutually exclusive choices filling the panel's width. */
function Segments<T extends string>({
  label,
  title,
  options,
  value,
  onChange,
}: {
  label: string;
  title?: string;
  options: { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      title={title}
      className="flex overflow-hidden rounded border border-control-border bg-surface-sunken"
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          role="radio"
          aria-checked={value === option.value}
          onClick={() => onChange(option.value)}
          className={segment(value === option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

/** Chooses whether a tool works on line art or colour art. */
export function ArtSwitch({ label }: { label: string }) {
  const { art, setArt } = useApp();
  return (
    <Segments
      label={label}
      title="Switch with L"
      value={art}
      onChange={setArt}
      options={[
        { value: "line", label: "Line art" },
        { value: "colour", label: "Colour art" },
      ]}
    />
  );
}

/** Chooses whether dragging from empty space draws a box or a lasso. */
export function SelectModeSwitch() {
  const { selectMode, setSelectMode } = useApp();
  return (
    <Segments
      label="Drag selects with"
      value={selectMode}
      onChange={setSelectMode}
      options={[
        { value: "box", label: "Box" },
        { value: "lasso", label: "Lasso" },
      ]}
    />
  );
}

const SHAPES: { value: ShapeKind; label: string }[] = [
  { value: "line", label: "Line" },
  { value: "rectangle", label: "Rectangle" },
  { value: "ellipse", label: "Ellipse" },
  { value: "polygon", label: "Polygon" },
];

/** Picks the shape the Shape tool draws, and a polygon's sides. */
export function ShapePicker() {
  const { shapeKind, setShapeKind, polygonSides, setPolygonSides } = useApp();
  return (
    <>
      <div
        role="radiogroup"
        aria-label="Shape"
        className="grid grid-cols-2 overflow-hidden rounded border border-control-border bg-surface-sunken"
      >
        {SHAPES.map(({ value, label }) => (
          <button
            key={value}
            type="button"
            role="radio"
            aria-checked={shapeKind === value}
            onClick={() => setShapeKind(value)}
            className={segment(shapeKind === value)}
          >
            {label}
          </button>
        ))}
      </div>
      {shapeKind === "polygon" ? (
        <label className="mt-2 flex items-center gap-2 text-sm">
          <span className="flex-1 text-text-muted">Sides</span>
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

/** A short line of help under a tool's settings. */
export function Hint({ children }: { children: ReactNode }) {
  return (
    <p className="mt-1 border-t border-border pt-2 text-xs text-text-muted">
      {children}
    </p>
  );
}
