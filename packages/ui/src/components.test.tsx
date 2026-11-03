// @vitest-environment happy-dom
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Button, Modal, SegmentedControl, SidePanel } from "./components";

afterEach(cleanup);

describe("Button", () => {
  it("defaults to type button and responds to the keyboard", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Save</Button>);
    const button = screen.getByRole("button", { name: "Save" });
    expect(button.getAttribute("type")).toBe("button");
    button.focus();
    await userEvent.keyboard("{Enter}");
    expect(onClick).toHaveBeenCalledOnce();
  });
});

describe("SegmentedControl", () => {
  const options = [
    { value: "draw", label: "Draw" },
    { value: "rig", label: "Rig", disabled: true, hint: "Coming later" },
    { value: "animate", label: "Animate" },
  ];

  function Example({ onChange }: { onChange: (value: string) => void }) {
    const [value, setValue] = useState("draw");
    return (
      <SegmentedControl
        label="Mode"
        options={options}
        value={value}
        onChange={(next) => {
          setValue(next);
          onChange(next);
        }}
      />
    );
  }

  it("marks the active option and blocks disabled ones", async () => {
    const onChange = vi.fn();
    render(<Example onChange={onChange} />);
    const draw = screen.getByRole("radio", { name: "Draw" });
    expect(draw.getAttribute("aria-checked")).toBe("true");
    await userEvent.click(screen.getByRole("radio", { name: "Rig" }));
    expect(onChange).not.toHaveBeenCalled();
  });

  it("keeps a selection when the active option is pressed again", async () => {
    const onChange = vi.fn();
    render(<Example onChange={onChange} />);
    await userEvent.click(screen.getByRole("radio", { name: "Draw" }));
    expect(onChange).not.toHaveBeenCalled();
  });
});

describe("SidePanel", () => {
  function Example() {
    const [open, setOpen] = useState(true);
    return (
      <SidePanel title="Layers" open={open} onOpenChange={setOpen}>
        <p>Layer list</p>
      </SidePanel>
    );
  }

  it("collapses and expands from its toggle", async () => {
    render(<Example />);
    expect(screen.queryByText("Layer list")).not.toBeNull();
    await userEvent.click(
      screen.getByRole("button", { name: "Collapse Layers panel" }),
    );
    expect(screen.queryByText("Layer list")).toBeNull();
    await userEvent.click(
      screen.getByRole("button", { name: "Expand Layers panel" }),
    );
    expect(screen.queryByText("Layer list")).not.toBeNull();
  });
});

describe("Modal", () => {
  it("is labelled by its title and closes on Escape", async () => {
    const onOpenChange = vi.fn();
    render(
      <Modal title="Commands" open onOpenChange={onOpenChange}>
        <input aria-label="Search" />
      </Modal>,
    );
    expect(screen.getByRole("dialog", { name: "Commands" })).toBeTruthy();
    await userEvent.keyboard("{Escape}");
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});
