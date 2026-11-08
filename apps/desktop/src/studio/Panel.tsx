import type { ReactNode } from "react";

/**
 * A panel's content inside its dock tab. The tab shows the title; a small toolbar row above
 * the content shows any detail and actions. See docs/specs/docking.md.
 */
export function Panel({
  title,
  detail,
  actions,
  children,
}: {
  /** Kept for callers; the dock tab names the panel. */
  id?: string;
  title: string;
  /** Short text such as the current tool's name. */
  detail?: ReactNode;
  /** Small buttons for the panel, such as Add layer. */
  actions?: ReactNode;
  /** Kept for callers; every panel fills its dock stack. */
  grow?: boolean;
  children: ReactNode;
}) {
  return (
    <section aria-label={title} className="flex min-h-full flex-col">
      {detail || actions ? (
        <div className="flex h-7 shrink-0 items-center gap-1 border-b border-border px-2.5">
          {detail ? (
            <span className="truncate text-xs text-text-muted">{detail}</span>
          ) : null}
          <span className="flex-1" />
          {actions}
        </div>
      ) : null}
      {children}
    </section>
  );
}

/** A 22 px icon button for a panel's toolbar. */
export const panelIconButton =
  "flex h-[22px] w-[22px] items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 disabled:hover:bg-transparent";
