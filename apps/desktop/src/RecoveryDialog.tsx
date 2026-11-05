import { Button, Modal } from "@celframe/ui";
import { useApp } from "./store";

/** Offers unsaved work from a run that didn't exit normally. It can only be answered. */
export function RecoveryDialog() {
  const { recovery, error, restoreRecovery, discardRecovery } = useApp();
  return (
    <Modal
      title="Restore unsaved work"
      open={recovery !== null}
      // Escape and outside clicks can't dismiss it: autosave waits for an answer.
      onOpenChange={() => undefined}
    >
      {recovery ? (
        <>
          <p className="px-1 py-2 text-sm">
            Celframe closed before your changes were saved. Restore your unsaved
            work from {new Date(recovery.savedAtMs).toLocaleString()}?
          </p>
          {/* The app's own alert area is hidden behind this modal. */}
          {error ? (
            <p role="alert" className="px-1 pb-2 text-sm text-danger">
              {error}
            </p>
          ) : null}
          <div className="flex justify-end gap-2 pt-2">
            <Button onClick={() => void discardRecovery()}>Discard</Button>
            <Button variant="primary" onClick={() => void restoreRecovery()}>
              Restore
            </Button>
          </div>
        </>
      ) : null}
    </Modal>
  );
}
