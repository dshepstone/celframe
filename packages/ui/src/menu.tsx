import * as Dropdown from "@radix-ui/react-dropdown-menu";

export interface MenuItem {
  label: string;
  shortcut?: string;
  disabled?: boolean;
  onSelect: () => void;
}

/** A menu-bar entry. Arrow keys move through items, Enter runs one, Escape closes. */
export function Menu({
  label,
  items,
}: {
  label: string;
  items: ReadonlyArray<MenuItem | "separator">;
}) {
  return (
    <Dropdown.Root modal={false}>
      <Dropdown.Trigger className="h-6 rounded px-2 text-sm text-text hover:bg-surface-hover outline-none focus-visible:outline-2 focus-visible:outline-focus-ring data-[state=open]:bg-surface-hover">
        {label}
      </Dropdown.Trigger>
      <Dropdown.Portal>
        <Dropdown.Content
          align="start"
          sideOffset={2}
          className="z-50 min-w-52 rounded-lg border border-border bg-surface-raised py-1 text-sm text-text shadow-2xl"
        >
          {items.map((item, index) =>
            item === "separator" ? (
              <Dropdown.Separator
                key={`separator-${index}`}
                className="mx-2 my-1 h-px bg-border"
              />
            ) : (
              <Dropdown.Item
                key={item.label}
                disabled={item.disabled ?? false}
                onSelect={item.onSelect}
                className="mx-1 flex h-6 cursor-default items-center justify-between gap-8 rounded px-2 outline-none data-[disabled]:opacity-45 data-[highlighted]:bg-accent-muted"
              >
                <span>{item.label}</span>
                {item.shortcut ? (
                  <span className="text-xs text-text-muted">
                    {item.shortcut}
                  </span>
                ) : null}
              </Dropdown.Item>
            ),
          )}
        </Dropdown.Content>
      </Dropdown.Portal>
    </Dropdown.Root>
  );
}
