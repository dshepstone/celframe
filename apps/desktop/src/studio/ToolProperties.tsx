import { useEffect, useState, type ReactNode } from "react";
import type { Arrange, SelectionEdit } from "../backend";
import { useApp } from "../store";
import { rgbaToHex } from "./colour";
import {
  BackwardIcon,
  FlipHorizontalIcon,
  FlipVerticalIcon,
  ForwardIcon,
  LockIcon,
  RotateLeftIcon,
  RotateRightIcon,
  ToBackIcon,
  ToFrontIcon,
} from "./icons";
import {
  ArtSwitch,
  Group,
  Hint,
  ShapePicker,
  SizeControl,
  heading,
} from "./toolControls";

const iconButton =
  "flex h-7 w-7 items-center justify-center rounded border border-control-border text-text outline-none hover:bg-surface-sunken focus-visible:outline-2 focus-visible:outline-focus-ring disabled:cursor-not-allowed disabled:opacity-40 aria-pressed:bg-accent aria-pressed:text-text-on-accent";

/** Rounds for display without showing float noise such as 0.30000000000000004. */
function format(value: number): string {
  return String(Math.round(value * 1000) / 1000);
}

/**
 * A number field that applies its value on Enter or when it loses focus. Anything that isn't
 * a valid number puts the current value back.
 */
function NumberField({
  label,
  short,
  value,
  disabled,
  onCommit,
  valid = Number.isFinite,
}: {
  label: string;
  short: string;
  value: number | null;
  disabled?: boolean;
  onCommit: (value: number) => void;
  valid?: (value: number) => boolean;
}) {
  const shown = value === null ? "" : format(value);
  const [draft, setDraft] = useState(shown);
  useEffect(() => setDraft(shown), [shown]);

  const commit = () => {
    const next = Number(draft);
    if (draft.trim() !== "" && valid(next) && draft !== shown) onCommit(next);
    else setDraft(shown);
  };
  return (
    <label className="flex items-center gap-1.5">
      <span className="w-3 text-xs text-text-muted" aria-hidden="true">
        {short}
      </span>
      <input
        type="text"
        inputMode="decimal"
        aria-label={label}
        value={draft}
        disabled={disabled}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") setDraft(shown);
        }}
        className="w-full min-w-0 rounded border border-control-border bg-surface px-1.5 py-0.5 text-sm text-text tabular-nums disabled:opacity-50"
      />
    </label>
  );
}

function IconButton({
  label,
  shortcut,
  onClick,
  disabled,
  children,
}: {
  label: string;
  shortcut?: string;
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={shortcut ? `${label} (${shortcut})` : label}
      onClick={onClick}
      disabled={disabled}
      className={iconButton}
    >
      {children}
    </button>
  );
}

const ARRANGE: {
  to: Arrange;
  label: string;
  shortcut: string;
  icon: ReactNode;
}[] = [
  {
    to: "front",
    label: "Bring to front",
    shortcut: "Ctrl+Shift+]",
    icon: <ToFrontIcon />,
  },
  {
    to: "forward",
    label: "Bring forward",
    shortcut: "Ctrl+]",
    icon: <ForwardIcon />,
  },
  {
    to: "backward",
    label: "Send backward",
    shortcut: "Ctrl+[",
    icon: <BackwardIcon />,
  },
  {
    to: "back",
    label: "Send to back",
    shortcut: "Ctrl+Shift+[",
    icon: <ToBackIcon />,
  },
];

/** Transform, flip and arrange controls for the selection. */
function SelectProperties() {
  const {
    document: doc,
    editSelection,
    lockProportions,
    setLockProportions,
    scaleStrokeWidths,
    setScaleStrokeWidths,
  } = useApp();
  const [rotation, setRotation] = useState("15");
  const bounds = doc?.selectionBounds ?? null;
  const none = !doc || doc.selection.length === 0 || bounds === null;

  const [x0, y0, x1, y1] = bounds ?? [0, 0, 0, 0];
  const box = { x: (x0 + x1) / 2, y: (y0 + y1) / 2, w: x1 - x0, h: y1 - y0 };
  const setBounds = (change: Partial<typeof box>) => {
    let { w, h } = { ...box, ...change };
    // Keep proportions when one side changes, unless the other side is flat.
    if (lockProportions && change.w !== undefined && box.w > 0 && box.h > 0) {
      h = (box.h * change.w) / box.w;
    }
    if (lockProportions && change.h !== undefined && box.w > 0 && box.h > 0) {
      w = (box.w * change.h) / box.h;
    }
    const next = { ...box, ...change };
    void editSelection({
      type: "setBounds",
      x: next.x,
      y: next.y,
      // A flat side keeps a placeholder size; the core leaves that axis alone.
      width: box.w > 0 ? w : 1,
      height: box.h > 0 ? h : 1,
      scaleWidths: scaleStrokeWidths,
    });
  };
  const edit = (change: SelectionEdit) => void editSelection(change);
  const positive = (v: number) => Number.isFinite(v) && v > 0;
  const rotate = () => {
    const degrees = Number(rotation);
    if (Number.isFinite(degrees) && degrees !== 0)
      edit({ type: "rotate", degrees });
  };

  return (
    <>
      <div role="group" aria-label="Transform" className="mb-3">
        <h3 className={heading}>Transform</h3>
        <div className="grid grid-cols-[1fr_1fr_auto] items-center gap-x-2 gap-y-1.5">
          <NumberField
            label="X"
            short="X"
            value={none ? null : box.x}
            disabled={none}
            onCommit={(x) => setBounds({ x })}
          />
          <NumberField
            label="Y"
            short="Y"
            value={none ? null : box.y}
            disabled={none}
            onCommit={(y) => setBounds({ y })}
          />
          <span />
          <NumberField
            label="Width"
            short="W"
            value={none ? null : box.w}
            disabled={none || box.w === 0}
            valid={positive}
            onCommit={(w) => setBounds({ w })}
          />
          <NumberField
            label="Height"
            short="H"
            value={none ? null : box.h}
            disabled={none || box.h === 0}
            valid={positive}
            onCommit={(h) => setBounds({ h })}
          />
          <button
            type="button"
            aria-label="Lock proportions"
            title="Lock proportions"
            aria-pressed={lockProportions}
            onClick={() => setLockProportions(!lockProportions)}
            className={iconButton}
          >
            <LockIcon />
          </button>
        </div>
        <div className="mt-1.5 flex items-center gap-2">
          <label className="flex min-w-0 flex-1 items-center gap-1.5">
            <span className="text-xs text-text-muted">Rotate</span>
            <input
              type="text"
              inputMode="decimal"
              aria-label="Rotation angle in degrees"
              value={rotation}
              disabled={none}
              onChange={(e) => setRotation(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") rotate();
              }}
              className="w-full min-w-0 rounded border border-control-border bg-surface px-1.5 py-0.5 text-sm text-text tabular-nums disabled:opacity-50"
            />
            <span className="text-xs text-text-muted">°</span>
          </label>
          <button
            type="button"
            disabled={none}
            onClick={rotate}
            className="rounded border border-control-border px-2 py-0.5 text-sm text-text hover:bg-surface-sunken disabled:opacity-40"
          >
            Apply
          </button>
        </div>
        <label className="mt-2 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={scaleStrokeWidths}
            onChange={(e) => setScaleStrokeWidths(e.target.checked)}
            className="accent-[var(--color-accent)]"
          />
          <span>Scale stroke widths</span>
        </label>
      </div>

      <div role="group" aria-label="Flip and rotate" className="mb-3">
        <h3 className={heading}>Flip and rotate</h3>
        <div className="flex gap-1.5">
          <IconButton
            label="Flip horizontally"
            shortcut="Shift+H"
            disabled={none}
            onClick={() => edit({ type: "flip", horizontal: true })}
          >
            <FlipHorizontalIcon />
          </IconButton>
          <IconButton
            label="Flip vertically"
            shortcut="Shift+V"
            disabled={none}
            onClick={() => edit({ type: "flip", horizontal: false })}
          >
            <FlipVerticalIcon />
          </IconButton>
          <IconButton
            label="Rotate 90° left"
            disabled={none}
            onClick={() => edit({ type: "rotate", degrees: 90 })}
          >
            <RotateLeftIcon />
          </IconButton>
          <IconButton
            label="Rotate 90° right"
            disabled={none}
            onClick={() => edit({ type: "rotate", degrees: -90 })}
          >
            <RotateRightIcon />
          </IconButton>
        </div>
      </div>

      <div role="group" aria-label="Arrange">
        <h3 className={heading}>Arrange</h3>
        <div className="flex gap-1.5">
          {ARRANGE.map(({ to, label, shortcut, icon }) => (
            <IconButton
              key={to}
              label={label}
              shortcut={shortcut}
              disabled={none}
              onClick={() => edit({ type: "arrange", to })}
            >
              {icon}
            </IconButton>
          ))}
        </div>
      </div>
    </>
  );
}

const TOOL_NAMES: Record<string, string> = {
  pen: "Pen",
  eraser: "Eraser",
  shape: "Shape",
  select: "Select",
  hand: "Pan",
  zoom: "Zoom",
};

/** The pen colour, chosen in the Colour panel. */
function ColourPreview() {
  const { brushColor } = useApp();
  const hex = rgbaToHex(brushColor);
  return (
    <div className="flex items-center gap-2 text-sm">
      <span
        className="h-6 w-10 rounded border border-control-border"
        style={{ background: hex }}
        aria-label={`Current colour ${hex}`}
        role="img"
      />
      <span className="font-mono text-xs text-text-muted">{hex}</span>
    </div>
  );
}

function PenProperties() {
  const { brushWidth, setBrushWidth } = useApp();
  return (
    <>
      <Group label="Brush">
        <SizeControl
          label="Brush size"
          value={brushWidth}
          onChange={setBrushWidth}
        />
      </Group>
      <Group label="Colour">
        <ColourPreview />
      </Group>
      <Group label="Draw on">
        <ArtSwitch label="Draw on" />
      </Group>
      <Hint>
        Pressure changes the width. Size is the width at full pressure.
      </Hint>
    </>
  );
}

function EraserProperties() {
  const {
    eraserWidth,
    setEraserWidth,
    eraseWholeStrokes,
    setEraseWholeStrokes,
  } = useApp();
  return (
    <>
      <Group label="Eraser">
        <SizeControl
          label="Eraser size"
          value={eraserWidth}
          onChange={setEraserWidth}
        />
        <label className="mt-2 flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={eraseWholeStrokes}
            onChange={(e) => setEraseWholeStrokes(e.target.checked)}
            className="accent-[var(--color-accent)]"
          />
          <span>Whole strokes</span>
        </label>
      </Group>
      <Group label="Erase from">
        <ArtSwitch label="Erase from" />
      </Group>
      <Hint>
        Cuts strokes where you drag. Whole strokes removes any stroke it
        touches.
      </Hint>
    </>
  );
}

function ShapeProperties() {
  const { brushWidth, setBrushWidth } = useApp();
  return (
    <>
      <Group label="Shape">
        <ShapePicker />
      </Group>
      <Group label="Line">
        <SizeControl
          label="Brush size"
          value={brushWidth}
          onChange={setBrushWidth}
        />
        <div className="mt-2">
          <ColourPreview />
        </div>
      </Group>
      <Group label="Draw on">
        <ArtSwitch label="Draw on" />
      </Group>
      <Hint>
        Shift keeps proportions or snaps angles. Alt draws from the centre.
      </Hint>
    </>
  );
}

function ViewProperties({ zoomTool }: { zoomTool: boolean }) {
  const { changeView, view } = useApp();
  const button =
    "flex-1 rounded border border-control-border px-2 py-1 text-sm text-text hover:bg-surface-sunken";
  return (
    <>
      <Group label="View">
        <div className="flex gap-1.5">
          <button
            type="button"
            className={button}
            onClick={() => void changeView({ type: "zoom", factor: 0.8 })}
          >
            Zoom out
          </button>
          <button
            type="button"
            className={button}
            onClick={() => void changeView({ type: "zoom", factor: 1.25 })}
          >
            Zoom in
          </button>
          <button
            type="button"
            className={button}
            onClick={() => void changeView({ type: "fit" })}
          >
            Fit
          </button>
        </div>
        {view ? (
          <p className="mt-2 text-xs text-text-muted tabular-nums">
            {Math.round(view.zoomPercent)}% · {Math.round(view.rotationDegrees)}
            °
          </p>
        ) : null}
      </Group>
      <Hint>
        {zoomTool
          ? "Click to zoom in at the pointer. Alt-click zooms out."
          : "Drag to pan. Hold Space with any tool to pan too."}
      </Hint>
    </>
  );
}

/** The current tool's full settings, at the top of the side panel. */
export function ToolProperties() {
  const { tool } = useApp();
  return (
    <section
      aria-label="Tool properties"
      className="border-b border-border px-3 py-2"
    >
      <h2 className="mb-2 flex items-baseline justify-between text-xs font-semibold tracking-wide text-text-muted uppercase">
        <span>Tool properties</span>
        <span className="normal-case">{TOOL_NAMES[tool]}</span>
      </h2>
      {tool === "select" ? (
        <>
          <Group label="Select from">
            <ArtSwitch label="Select from" />
          </Group>
          <SelectProperties />
        </>
      ) : tool === "pen" ? (
        <PenProperties />
      ) : tool === "eraser" ? (
        <EraserProperties />
      ) : tool === "shape" ? (
        <ShapeProperties />
      ) : (
        <ViewProperties zoomTool={tool === "zoom"} />
      )}
    </section>
  );
}
