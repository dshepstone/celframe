import { Button, Modal } from "@celframe/ui";
import { useApp, type FileAction } from "./store";

const BEFORE: Record<FileAction, string> = {
  new: "starting a new project",
  open: "opening another project",
  close: "closing",
};

/** Asks whether to save before an action discards unsaved changes. Escape cancels. */
export function UnsavedChangesDialog() {
  const { document: doc, pendingAction, resolvePending } = useApp();
  return (
    <Modal
      title="Unsaved changes"
      open={pendingAction !== null}
      onOpenChange={(open) => {
        if (!open) void resolvePending("cancel");
      }}
    >
      {pendingAction ? (
        <>
          <p className="px-1 py-2 text-sm">
            Save changes to &ldquo;{doc?.project.name}&rdquo; before{" "}
            {BEFORE[pendingAction]}?
          </p>
          <div className="flex justify-end gap-2 pt-2">
            <Button onClick={() => void resolvePending("cancel")}>
              Cancel
            </Button>
            <Button onClick={() => void resolvePending("discard")}>
              Don&rsquo;t save
            </Button>
            <Button
              variant="primary"
              onClick={() => void resolvePending("save")}
            >
              Save
            </Button>
          </div>
        </>
      ) : null}
    </Modal>
  );
}
