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
      <Dropdown.Trigger className="rounded px-2.5 py-1 text-sm text-text hover:bg-surface-sunken outline-none focus-visible:outline-2 focus-visible:outline-focus-ring data-[state=open]:bg-surface-sunken">
        {label}
      </Dropdown.Trigger>
      <Dropdown.Portal>
        <Dropdown.Content
          align="start"
          sideOffset={4}
          className="z-50 min-w-56 rounded-md border border-border bg-surface-raised p-1 text-sm text-text shadow-lg"
        >
          {items.map((item, index) =>
            item === "separator" ? (
              <Dropdown.Separator
                key={`separator-${index}`}
                className="my-1 h-px bg-border"
              />
            ) : (
              <Dropdown.Item
                key={item.label}
                disabled={item.disabled ?? false}
                onSelect={item.onSelect}
                className="flex cursor-default justify-between gap-6 rounded px-2 py-1.5 outline-none data-[disabled]:opacity-50 data-[highlighted]:bg-accent data-[highlighted]:text-text-on-accent"
              >
                <span>{item.label}</span>
                {item.shortcut ? (
                  <span className="opacity-70">{item.shortcut}</span>
                ) : null}
              </Dropdown.Item>
            ),
          )}
        </Dropdown.Content>
      </Dropdown.Portal>
    </Dropdown.Root>
  );
}
