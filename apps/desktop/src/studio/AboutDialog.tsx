import { Button, Modal } from "@celframe/ui";
import { Logo } from "./brand";

/** Help › About Celframe: the full logo and what Celframe is. */
export function AboutDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Modal title="About Celframe" open={open} onOpenChange={onOpenChange}>
      <div className="flex flex-col items-start gap-3 p-4">
        <Logo height={44} />
        <p className="text-base text-text">
          Free, open-source 2D animation studio.
        </p>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
          <dt className="text-text-muted">Build</dt>
          <dd className="text-text">Early development</dd>
          <dt className="text-text-muted">Licence</dt>
          <dd className="text-text">GPL-3.0-or-later</dd>
          <dt className="text-text-muted">Source</dt>
          <dd className="text-text">github.com/dshepstone/celframe</dd>
        </dl>
        <div className="flex w-full justify-end">
          <Button variant="primary" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </div>
      </div>
    </Modal>
  );
}
