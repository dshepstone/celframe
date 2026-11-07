// Original line icons, drawn on a 20 × 20 grid.
import type { ReactNode } from "react";

function Icon({ children }: { children: ReactNode }) {
  return (
    <svg
      viewBox="0 0 20 20"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export const PenIcon = () => (
  <Icon>
    <path d="M4 16l1-4 8-8 3 3-8 8-4 1z" />
    <path d="M11 6l3 3" />
  </Icon>
);
export const FlipHorizontalIcon = () => (
  <Icon>
    <path d="M10 3v14" strokeDasharray="1.5 2" />
    <path d="M8 6L3 14h5z" />
    <path d="M12 6l5 8h-5z" />
  </Icon>
);
export const FlipVerticalIcon = () => (
  <Icon>
    <path d="M3 10h14" strokeDasharray="1.5 2" />
    <path d="M6 8l8-5v5z" />
    <path d="M6 12l8 5v-5z" />
  </Icon>
);
export const RotateLeftIcon = () => (
  <Icon>
    <path d="M5 9a5.5 5.5 0 1 1 1.6 5.4" />
    <path d="M5 4.5V9h4.5" />
  </Icon>
);
export const RotateRightIcon = () => (
  <Icon>
    <path d="M15 9a5.5 5.5 0 1 0-1.6 5.4" />
    <path d="M15 4.5V9h-4.5" />
  </Icon>
);
export const ToFrontIcon = () => (
  <Icon>
    <rect x="3" y="9" width="8" height="8" rx="1" strokeDasharray="1.5 2" />
    <rect x="8" y="3" width="9" height="9" rx="1" />
  </Icon>
);
export const ForwardIcon = () => (
  <Icon>
    <rect x="4" y="8" width="9" height="9" rx="1" strokeDasharray="1.5 2" />
    <path d="M14 10V3M11 6l3-3 3 3" />
  </Icon>
);
export const BackwardIcon = () => (
  <Icon>
    <rect x="4" y="3" width="9" height="9" rx="1" strokeDasharray="1.5 2" />
    <path d="M14 10v7M11 14l3 3 3-3" />
  </Icon>
);
export const ToBackIcon = () => (
  <Icon>
    <rect x="3" y="3" width="9" height="9" rx="1" />
    <rect x="8" y="8" width="9" height="9" rx="1" strokeDasharray="1.5 2" />
  </Icon>
);
export const ShapeIcon = () => (
  <Icon>
    <rect x="3" y="8" width="8" height="8" rx="0.5" />
    <circle cx="13" cy="7" r="4" />
  </Icon>
);
export const EraserIcon = () => (
  <Icon>
    <path d="M3 13l7-7 5 5-5 5H6z" />
    <path d="M9 16h8" />
  </Icon>
);
export const FillIcon = () => (
  <Icon>
    <path d="M4 9l5-5 6 6-5 5z" />
    <path d="M16 13c0 1.5 1 2.2 1 3a1 1 0 01-2 0c0-.8 1-1.5 1-3z" />
  </Icon>
);
export const SelectIcon = () => (
  <Icon>
    <path d="M5 3l10 6-4.5 1.2L13 16l-2 1-2.4-5.6L5 15z" />
  </Icon>
);
export const HandIcon = () => (
  <Icon>
    <path d="M6 11V6a1 1 0 012 0v4M8 9V4a1 1 0 012 0v5M10 9V5a1 1 0 012 0v5M12 10V7a1 1 0 012 0v5c0 3-2 5-5 5-2 0-3-1-4-3l-2-3a1 1 0 011.6-1.2L6 11" />
  </Icon>
);
export const ZoomIcon = () => (
  <Icon>
    <circle cx="8.5" cy="8.5" r="5" />
    <path d="M12.5 12.5L17 17M6.5 8.5h4M8.5 6.5v4" />
  </Icon>
);
export const EyeIcon = () => (
  <Icon>
    <path d="M2 10s3-5 8-5 8 5 8 5-3 5-8 5-8-5-8-5z" />
    <circle cx="10" cy="10" r="2.2" />
  </Icon>
);
export const EyeOffIcon = () => (
  <Icon>
    <path d="M3 3l14 14M7.5 5.6A8 8 0 0110 5c5 0 8 5 8 5a13 13 0 01-2.6 3M12.5 14.5A8 8 0 0110 15c-5 0-8-5-8-5a13 13 0 013-3.3" />
  </Icon>
);
export const LockIcon = () => (
  <Icon>
    <rect x="4.5" y="9" width="11" height="8" rx="1.5" />
    <path d="M7 9V6.5a3 3 0 016 0V9" />
  </Icon>
);
export const UnlockIcon = () => (
  <Icon>
    <rect x="4.5" y="9" width="11" height="8" rx="1.5" />
    <path d="M7 9V6.5a3 3 0 015.6-1.5" />
  </Icon>
);
export const PlusIcon = () => (
  <Icon>
    <path d="M10 4v12M4 10h12" />
  </Icon>
);
export const TrashIcon = () => (
  <Icon>
    <path d="M4 6h12M8 6V4h4v2M6 6l1 11h6l1-11" />
  </Icon>
);
