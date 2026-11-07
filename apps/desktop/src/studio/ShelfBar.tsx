import { useEffect, useMemo, useRef, useState } from "react";
import { matches } from "../CommandPalette";
import { useActions, type Action } from "../actions";
import { useApp } from "../store";
import { SlidersIcon } from "./icons";

const control =
  "h-6 rounded px-2 text-sm outline-none focus-visible:outline-2 focus-visible:outline-focus-ring";
const shelfButton =
  "flex h-7 min-w-7 items-center justify-center rounded px-1.5 text-sm text-text outline-none hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:opacity-35 disabled:hover:bg-transparent";
const editButton =
  "flex h-3.5 w-4 items-center justify-center rounded text-[10px] leading-none text-text-muted hover:bg-surface-hover hover:text-text disabled:opacity-30";

/** A shelf's tab, renamed in place with a double-click or F2. */
function ShelfTab({
  id,
  name,
  active,
}: {
  id: string;
  name: string;
  active: boolean;
}) {
  const { setActiveShelf, renameShelf } = useApp();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(name);

  if (editing) {
    const finish = (keep: boolean) => {
      if (keep) renameShelf(id, draft);
      setEditing(false);
    };
    return (
      <input
        autoFocus
        aria-label={`Rename ${name}`}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={() => finish(true)}
        onKeyDown={(e) => {
          if (e.key === "Enter") finish(true);
          if (e.key === "Escape") finish(false);
        }}
        className="my-auto h-6 w-24 rounded border border-control-border bg-surface-sunken px-1.5 text-sm text-text"
      />
    );
  }
  return (
    <button
      type="button"
      role="tab"
      aria-selected={active}
      onClick={() => setActiveShelf(id)}
      onDoubleClick={() => {
        setDraft(name);
        setEditing(true);
      }}
      onKeyDown={(e) => {
        if (e.key === "F2") {
          setDraft(name);
          setEditing(true);
        }
      }}
      className={`border-b-2 px-1.5 text-sm outline-none focus-visible:outline-2 focus-visible:outline-focus-ring ${
        active
          ? "border-accent text-text"
          : "border-transparent text-text-muted hover:text-text"
      }`}
    >
      {name}
    </button>
  );
}

/** A searchable list of every action, to add one to the shelf. */
function AddAction({
  actions,
  onPick,
}: {
  actions: Action[];
  onPick: (id: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const box = useRef<HTMLDivElement>(null);
  const results = useMemo(
    () => actions.filter((a) => matches(`${a.group} ${a.label}`, query)),
    [actions, query],
  );

  useEffect(() => {
    if (!open) return;
    const close = (event: PointerEvent) => {
      if (!box.current?.contains(event.target as Node)) setOpen(false);
    };
    window.addEventListener("pointerdown", close);
    return () => window.removeEventListener("pointerdown", close);
  }, [open]);

  return (
    <div ref={box} className="relative">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => {
          setQuery("");
          setOpen(!open);
        }}
        className={`${control} border border-control-border bg-surface-raised text-text hover:bg-surface-hover`}
      >
        Add action
      </button>
      {open ? (
        <div className="absolute top-full right-0 z-20 mt-1 w-72 rounded-lg border border-border bg-surface-raised p-1.5 shadow-2xl">
          <input
            autoFocus
            aria-label="Find an action"
            placeholder="Find an action"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") setOpen(false);
              if (e.key === "Enter" && results[0]) {
                onPick(results[0].id);
                setOpen(false);
              }
            }}
            className="h-6 w-full rounded border border-control-border bg-surface-sunken px-2 text-sm text-text"
          />
          <ul
            role="listbox"
            aria-label="Actions"
            className="mt-1 max-h-72 overflow-auto"
          >
            {results.map((action) => (
              <li
                key={action.id}
                role="option"
                aria-selected={false}
                onClick={() => {
                  onPick(action.id);
                  setOpen(false);
                }}
                className="flex h-6 cursor-default items-center gap-2 rounded px-2 text-sm text-text hover:bg-accent-muted"
              >
                <span className="w-16 shrink-0 text-xs text-text-muted">
                  {action.group}
                </span>
                <span className="flex-1">{action.label}</span>
              </li>
            ))}
            {results.length === 0 ? (
              <li className="px-2 py-1 text-sm text-text-muted">No actions</li>
            ) : null}
          </ul>
        </div>
      ) : null}
    </div>
  );
}

/** The bar under the menus: the user's shelves of actions. See docs/specs/shelves.md. */
export function ShelfBar() {
  const {
    shelves,
    activeShelf,
    initShelves,
    addShelf,
    removeShelf,
    addToShelf,
    removeFromShelf,
    moveOnShelf,
  } = useApp();
  const actions = useActions();
  const byId = useMemo(() => new Map(actions.map((a) => [a.id, a])), [actions]);
  const [customising, setCustomising] = useState(false);

  useEffect(() => {
    initShelves(new Set(actions.map((a) => a.id)));
    // Loading needs the action IDs once; later renders change only labels.
  }, [initShelves]);

  if (!shelves) return null;
  const shelf = shelves.find((s) => s.id === activeShelf) ?? shelves[0];
  if (!shelf) return null;
  const buttons = shelf.actions.flatMap((id) => {
    const action = byId.get(id);
    return action ? [action] : [];
  });

  return (
    <div
      role="toolbar"
      aria-label="Shelves"
      className="flex h-[34px] shrink-0 items-center gap-2 border-b border-border bg-surface px-2"
    >
      <div
        role="tablist"
        aria-label="Shelf"
        className="flex h-full items-stretch gap-1"
      >
        {shelves.map((s) => (
          <ShelfTab
            key={s.id}
            id={s.id}
            name={s.name}
            active={s.id === shelf.id}
          />
        ))}
        <button
          type="button"
          aria-label="New shelf"
          title="New shelf"
          onClick={addShelf}
          className="my-auto flex h-6 w-6 items-center justify-center rounded text-base text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring"
        >
          +
        </button>
      </div>
      <div className="h-5 w-px bg-border" aria-hidden="true" />
      <div
        role="group"
        aria-label={`${shelf.name} shelf`}
        className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto"
      >
        {buttons.map((action, index) => (
          <div key={action.id} className="flex items-center">
            <button
              type="button"
              aria-label={action.label}
              title={
                action.shortcut
                  ? `${action.label} (${action.shortcut})`
                  : action.label
              }
              disabled={!action.enabled || customising}
              onClick={action.run}
              className={shelfButton}
            >
              {action.icon ?? (
                <span className="whitespace-nowrap">{action.label}</span>
              )}
            </button>
            {customising ? (
              <span className="flex flex-col">
                <button
                  type="button"
                  aria-label={`Move ${action.label} left`}
                  disabled={index === 0}
                  onClick={() => moveOnShelf(shelf.id, index, -1)}
                  className={editButton}
                >
                  ‹
                </button>
                <button
                  type="button"
                  aria-label={`Move ${action.label} right`}
                  disabled={index === buttons.length - 1}
                  onClick={() => moveOnShelf(shelf.id, index, 1)}
                  className={editButton}
                >
                  ›
                </button>
              </span>
            ) : null}
            {customising ? (
              <button
                type="button"
                aria-label={`Remove ${action.label}`}
                onClick={() => removeFromShelf(shelf.id, index)}
                className={editButton}
              >
                ×
              </button>
            ) : null}
          </div>
        ))}
        {buttons.length === 0 && !customising ? (
          <span className="text-sm text-text-muted">
            Empty shelf. Choose Customise to add actions.
          </span>
        ) : null}
      </div>
      {customising ? (
        <>
          <AddAction
            actions={actions}
            onPick={(id) => addToShelf(shelf.id, id)}
          />
          <button
            type="button"
            disabled={shelves.length <= 1}
            onClick={() => removeShelf(shelf.id)}
            className={`${control} text-text-muted hover:text-text disabled:opacity-40`}
          >
            Delete shelf
          </button>
        </>
      ) : null}
      {customising ? (
        <button
          type="button"
          onClick={() => setCustomising(false)}
          className={`${control} bg-accent font-medium text-text-on-accent hover:bg-accent-hover`}
        >
          Done
        </button>
      ) : (
        <button
          type="button"
          aria-label="Customise"
          title="Customise this shelf"
          onClick={() => setCustomising(true)}
          className="flex h-6 w-6 items-center justify-center rounded text-text-muted outline-none hover:bg-surface-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring"
        >
          <SlidersIcon />
        </button>
      )}
    </div>
  );
}
