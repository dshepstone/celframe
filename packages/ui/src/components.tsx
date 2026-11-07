import * as Collapsible from "@radix-ui/react-collapsible";
import * as Dialog from "@radix-ui/react-dialog";
import * as ToggleGroup from "@radix-ui/react-toggle-group";
import type { ButtonHTMLAttributes, ReactNode } from "react";

const focusRing =
  "outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus-ring";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "quiet";
}

export function Button({
  variant = "quiet",
  className = "",
  type = "button",
  ...props
}: ButtonProps) {
  const look =
    variant === "primary"
      ? "bg-accent text-text-on-accent hover:bg-accent-hover border-transparent"
      : "bg-surface-raised text-text hover:bg-surface-hover border-control-border";
  return (
    <button
      type={type}
      className={`inline-flex h-6 items-center gap-1.5 rounded border px-2.5 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50 ${look} ${focusRing} ${className}`}
      {...props}
    />
  );
}

export interface SegmentOption {
  value: string;
  label: string;
  disabled?: boolean;
  /** Explains why an option is unavailable. */
  hint?: string;
}

export interface SegmentedControlProps {
  label: string;
  options: readonly SegmentOption[];
  value: string;
  onChange: (value: string) => void;
}

/** Picks one option. Arrow keys move between options. */
export function SegmentedControl({
  label,
  options,
  value,
  onChange,
}: SegmentedControlProps) {
  return (
    <ToggleGroup.Root
      type="single"
      aria-label={label}
      value={value}
      // Radix reports an empty value when the active option is pressed again.
      onValueChange={(next) => {
        if (next) onChange(next);
      }}
      className="inline-flex h-full items-stretch gap-3"
    >
      {options.map((option) => (
        <ToggleGroup.Item
          key={option.value}
          value={option.value}
          disabled={option.disabled ?? false}
          title={option.hint}
          className={`border-b-2 border-transparent px-0.5 text-sm font-medium text-text-muted hover:text-text data-[state=on]:border-accent data-[state=on]:text-text disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:text-text-muted ${focusRing}`}
        >
          {option.label}
        </ToggleGroup.Item>
      ))}
    </ToggleGroup.Root>
  );
}

export interface SidePanelProps {
  title: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
}

/** A panel whose contents collapse away, leaving its toggle reachable. */
export function SidePanel({
  title,
  open,
  onOpenChange,
  children,
}: SidePanelProps) {
  return (
    <Collapsible.Root
      open={open}
      onOpenChange={onOpenChange}
      className="flex h-full flex-col border-l border-border bg-surface-raised"
    >
      <Collapsible.Trigger
        aria-label={`${open ? "Collapse" : "Expand"} ${title} panel`}
        className={`m-2 rounded border border-control-border px-2 py-1 text-sm text-text hover:bg-surface-hover ${focusRing}`}
      >
        {open ? `${title} ›` : "‹"}
      </Collapsible.Trigger>
      <Collapsible.Content className="w-64 flex-1 overflow-auto p-3 text-sm text-text">
        {children}
      </Collapsible.Content>
    </Collapsible.Root>
  );
}

export interface ModalProps {
  title: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
}

/** A modal that traps focus, closes on Escape and returns focus when it closes. */
export function Modal({ title, open, onOpenChange, children }: ModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed top-[15%] left-1/2 w-[min(34rem,calc(100vw-2rem))] -translate-x-1/2 rounded-lg border border-border bg-surface-raised p-2 text-text shadow-2xl"
        >
          <Dialog.Title className="sr-only">{title}</Dialog.Title>
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
