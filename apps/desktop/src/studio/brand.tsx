// Celframe's logo: two overlapping frames, like cels on a peg bar. Kept in step with the
// files in docs/brand/.
import { useId } from "react";

const FRONT = "#f24b36";
const BACK = "#e38a9c";
const TILE = "#0e0e10";

/** The two frames on a 64-unit grid. Each use needs its own mask ID. */
function Frames({ mask }: { mask: string }) {
  return (
    <g fill="none" strokeWidth="6.5">
      <mask id={mask}>
        <rect width="64" height="64" fill="#fff" />
        <rect x="5.5" y="17.5" width="40" height="40" rx="8" fill="#000" />
      </mask>
      <rect
        x="17.5"
        y="5.5"
        width="40"
        height="40"
        rx="8"
        stroke={BACK}
        mask={`url(#${mask})`}
      />
      <path
        d="M45.5 41.5V49.5A8 8 0 0 1 37.5 57.5H13.5A8 8 0 0 1 5.5 49.5V25.5A8 8 0 0 1 13.5 17.5H37.5A8 8 0 0 1 45.5 25.5V30"
        stroke={FRONT}
      />
    </g>
  );
}

/** A mask ID that's valid inside `url(#...)`. */
function useMaskId(): string {
  return `logo-${useId().replace(/[^a-zA-Z0-9_-]/g, "")}`;
}

/** The small mark: the frames on a dark rounded tile, as the app icon shows it. */
export function LogoTile({ size = 20 }: { size?: number }) {
  const mask = useMaskId();
  return (
    <svg
      viewBox="0 0 64 64"
      width={size}
      height={size}
      role="img"
      aria-label="Celframe"
    >
      <rect x="0" y="0" width="64" height="64" rx="14" fill={TILE} />
      <g transform="translate(12.5 12.5) scale(0.61)">
        <Frames mask={mask} />
      </g>
    </svg>
  );
}

/** The full logo: the frames and the name, for larger places such as About. */
export function Logo({ height = 40 }: { height?: number }) {
  const mask = useMaskId();
  return (
    <svg
      viewBox="0 0 360 80"
      height={height}
      width={(height * 360) / 80}
      role="img"
      aria-label="Celframe"
    >
      <g transform="translate(4 8)">
        <Frames mask={mask} />
      </g>
      <text
        x="80"
        y="60"
        fill="currentColor"
        fontSize="56"
        fontWeight="800"
        letterSpacing="-1.5"
        fontFamily="'Segoe UI Variable Display', 'Segoe UI', 'SF Pro Display', 'Helvetica Neue', Arial, sans-serif"
      >
        celframe
      </text>
    </svg>
  );
}
