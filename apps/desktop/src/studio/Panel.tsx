import type { ReactNode } from "react";
import { useApp } from "../store";
import { ChevronIcon } from "./icons";

/**
 * A titled side-panel section that collapses to its header. Collapsed panels are remembered.
 * See docs/specs/visual-design.md.
 */
export function Panel({
  id,
  title,
  detail,
  actions,
  grow = false,
  children,
}: {
  /** Remembers whether this panel is collapsed. */
  id: string;
  title: string;
  /** Short text beside the title, such as the current tool's name. */
  detail?: ReactNode;
  /** Small buttons at the right of the header. */
  actions?: ReactNode;
  /** Takes the remaining height and scrolls. */
  grow?: boolean;
  children: ReactNode;
}) {
  const { collapsedPanels, togglePanel } = useApp();
  const open = !collapsedPanels.includes(id);
  return (
    <section
      aria-label={title}
      className={`flex flex-col border-b border-border ${
        grow && open ? "min-h-40 flex-1" : "shrink-0"
      }`}
    >
      <header className="flex h-[26px] shrink-0 items-center gap-1 bg-surface-raised pr-1 pl-1">
        <button
          type="button"
          aria-expanded={open}
          aria-label={`${open ? "Collapse" : "Expand"} ${title}`}
          onClick={() => togglePanel(id)}
          className="flex h-5 min-w-0 flex-1 items-center gap-1 rounded px-1 text-left outline-none hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring"
        >
          <span
            className={`text-text-muted transition-transform duration-75 ${open ? "rotate-90" : ""}`}
          >
            <ChevronIcon />
          </span>
          <h2 className="truncate text-2xs font-semibold tracking-[0.06em] text-text-muted uppercase">
            {title}
          </h2>
          {detail ? (
            <span className="ml-auto truncate pl-2 text-2xs text-text-muted">
              {detail}
            </span>
          ) : null}
        </button>
        {actions && open ? (
          <span className="flex shrink-0 items-center gap-0.5">{actions}</span>
        ) : null}
      </header>
      {open ? (
        <div className={grow ? "min-h-0 flex-1 overflow-auto" : ""}>
          {children}
        </div>
      ) : null}
    </section>
  );
}

/** A 22 px icon button for panel headers. */
export const panelIconButton =
  "flex h-[22px] w-[22px] items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 disabled:hover:bg-transparent";
