import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { Button, Modal, SegmentedControl, SidePanel } from "./components";

const meta: Meta = { title: "Components" };
export default meta;

type Story = StoryObj;

export const Buttons: Story = {
  render: () => (
    <div className="flex gap-2">
      <Button variant="primary">Save</Button>
      <Button>Cancel</Button>
      <Button disabled>Unavailable</Button>
    </div>
  ),
};

export const Segmented: Story = {
  render: function Render() {
    const [value, setValue] = useState("draw");
    return (
      <SegmentedControl
        label="Mode"
        value={value}
        onChange={setValue}
        options={[
          { value: "draw", label: "Draw" },
          { value: "animate", label: "Animate" },
          { value: "rig", label: "Rig", disabled: true, hint: "Coming later" },
        ]}
      />
    );
  },
};

export const Panel: Story = {
  render: function Render() {
    const [open, setOpen] = useState(true);
    return (
      <div className="flex h-64 justify-end">
        <SidePanel title="Layers" open={open} onOpenChange={setOpen}>
          Layer list
        </SidePanel>
      </div>
    );
  },
};

export const Dialog: Story = {
  render: function Render() {
    const [open, setOpen] = useState(false);
    return (
      <>
        <Button onClick={() => setOpen(true)}>Open</Button>
        <Modal title="Example" open={open} onOpenChange={setOpen}>
          Modal contents
        </Modal>
      </>
    );
  },
};
